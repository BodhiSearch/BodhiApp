# Plan: Sentinel API-key + sentinel-stripping middleware for chat UI auth

## Context

pi-ai's SDK wrappers (`@anthropic-ai/sdk`, OpenAI SDK, etc.) always send an auth header derived from the `apiKey` argument passed at construction. BodhiApp's chat UI runs in-browser against BodhiApp's own proxy endpoints (`/anthropic/v1/messages`, `/v1/chat/completions`, `/v1/responses`) which authenticate via session cookies. We currently pass a hardcoded dummy (`bodhi-proxy`) as the apiKey, which pi-ai faithfully forwards as either `x-api-key: bodhi-proxy` (Anthropic) or `Authorization: Bearer bodhi-proxy` (OpenAI). That value then trips the token validator, producing 401 "malformed token format".

pi-ai does not expose a way to disable SDK auth header injection (confirmed by exploration of `packages/ai/src/providers/anthropic.ts` and the registry). Custom providers would require duplicating ~700 lines. **We will not take that route.**

Instead: use a recognizable sentinel value and strip it server-side.

**Sentinel**: `bodhiapp_sentinel_api_key_ignored`

When the UI has no user-configured BodhiApp API token, it passes the sentinel to pi-ai. The SDKs send it as auth header. Middleware on BodhiApp's proxy routes detects the sentinel and strips the header, letting session-cookie auth take over. When the user HAS set an API token in chat settings, the token overrides the sentinel (via `model.headers`) and authenticates normally.

**Revert**: commit `fcc295a2` (both the frontend `x-api-key: null` hack and the middleware `bodhi-proxy` special-case) is superseded by this plan.

---

## Phase 1 — Revert `fcc295a2`

Revert only the `bodhi-proxy` specifics. We will layer the sentinel approach on top immediately after.

**Files** (changes reverted):
- `crates/bodhi/src/stores/agentStore.ts` — restore the simpler header override (only Authorization null when no token)
- `crates/routes_app/src/middleware/anthropic_auth_middleware.rs` — remove the `"bodhi-proxy"` and empty-string special-cases and their tests

Do NOT revert the frontend test changes in `agentStore.test.ts` — they still pass with the new sentinel.

---

## Phase 2 — Introduce sentinel constant (frontend)

**File**: `crates/bodhi/src/stores/agentStore.ts`

- Rename the constant: `DUMMY_API_KEY` → `SENTINEL_API_KEY`
- Value: `'bodhiapp_sentinel_api_key_ignored'`
- Update `getOrCreateAgent()` to pass the new constant via `getApiKey: () => SENTINEL_API_KEY`
- Update `createBodhiStreamFn` to override auth headers ONLY when the user has set a BodhiApp API token:
  ```ts
  const headers = apiToken
    ? { ...model.headers, Authorization: `Bearer ${apiToken}`, 'x-api-key': apiToken }
    : model.headers;
  ```
  When `apiToken` is undefined, let the sentinel flow through — middleware strips it.

Export `SENTINEL_API_KEY` so frontend tests can import it.

---

## Phase 3 — Update `anthropic_auth_middleware` for sentinel

**File**: `crates/routes_app/src/middleware/anthropic_auth_middleware.rs`

Current logic (from `49038f69`):
- `x-api-key` present → strip + rewrite to `Authorization: Bearer <value>`
- `Authorization` already present → just strip `x-api-key`

Extend:
- If stripped `x-api-key` value equals the sentinel, DO NOT rewrite it. Just drop it.
- Same for `Authorization: Bearer <sentinel>`: detect and drop the Authorization header, too.

Implementation sketch:
```rust
const SENTINEL: &str = "bodhiapp_sentinel_api_key_ignored";

if let Some(key) = req.headers().get("x-api-key").cloned() {
  req.headers_mut().remove("x-api-key");
  if let Ok(key_str) = key.to_str() {
    if key_str != SENTINEL && req.headers().get("authorization").is_none() {
      // existing rewrite logic
    }
  }
}
if let Some(auth) = req.headers().get("authorization").cloned() {
  if let Ok(s) = auth.to_str() {
    if s == format!("Bearer {}", SENTINEL) {
      req.headers_mut().remove("authorization");
    }
  }
}
```

Tests:
- `x-api-key: bodhiapp_sentinel_... → no Authorization` → stripped, no rewrite
- `Authorization: Bearer bodhiapp_sentinel_...` → Authorization stripped
- Keep existing tests passing (real tokens still rewritten)

---

## Phase 4 — Create `openai_auth_middleware`

**New file**: `crates/routes_app/src/middleware/openai_auth_middleware.rs`

- Path-scoped: activate on `/v1/*` paths (OpenAI-compat endpoints).
- Logic: if `Authorization: Bearer bodhiapp_sentinel_api_key_ignored`, strip the header so downstream `auth_middleware` can fall through to session cookie auth.
- All other auth values pass through unchanged.

```rust
pub async fn openai_auth_middleware(mut req: Request, next: Next) -> Response {
  let path = req.uri().path();
  if path.starts_with("/v1/") {
    if let Some(auth) = req.headers().get("authorization") {
      if let Ok(s) = auth.to_str() {
        if s == format!("Bearer {}", SENTINEL_API_KEY) {
          req.headers_mut().remove("authorization");
        }
      }
    }
  }
  next.run(req).await
}
```

Shared `SENTINEL_API_KEY` constant: put in `crates/routes_app/src/middleware/mod.rs` or a shared `constants.rs`, export `pub const`.

Tests (mirroring anthropic middleware test file):
- `Authorization: Bearer sentinel` on `/v1/chat/completions` → stripped
- `Authorization: Bearer sentinel` on `/v1/responses` → stripped
- `Authorization: Bearer sentinel` on `/v1/models` → stripped
- `Authorization: Bearer <real-token>` → passes through unchanged
- `Authorization: Bearer sentinel` on `/anthropic/v1/messages` → passes through (not scope of openai middleware)
- No Authorization header → passes through unchanged

---

## Phase 5 — Register middleware in routes

**File**: `crates/routes_app/src/routes.rs`

The OpenAI-compat endpoints (`/v1/chat/completions`, `/v1/responses`, `/v1/models`, etc.) already live in the `user_apis` route group. Add `openai_auth_middleware` as a layer **before** `api_auth_middleware` so the sentinel strip happens first, then auth runs.

Pattern mirrors how `anthropic_auth_middleware` is already stacked (see the `api_protected` layer).

Module declaration: add `pub mod openai_auth_middleware;` in `middleware/mod.rs`, re-export the handler.

---

## Phase 6 — Rebuild + browser verification

1. `cargo check -p routes_app --lib`
2. `cargo test -p routes_app --lib middleware 2>&1 | grep -E "test result|FAILED"`
3. `cd crates/bodhi && npm test -- --run src/stores/agentStore.test.ts`
4. `make build.ui-rebuild`
5. Restart local server (`make app.run` in background)
6. Using `claude-in-chrome`: login to `/ui/`, navigate to `/ui/models/api/new/`, select `Anthropic (Claude Code OAuth)`, paste the OAuth token, create prefix `ant-auth/`, fetch & select a model, create the alias.
7. Navigate to `/ui/chat/`, select `ant-auth/claude-sonnet-4-6`, send a message. Verify:
   - Request goes to `/anthropic/v1/messages`
   - Response streams correctly (200, text chunks visible)
   - No `x-api-key: bodhiapp_sentinel_...` header leaks upstream (check backend log or add assertion)
8. Repeat for an `openai` alias: create with API key → chat UI sends to `/v1/chat/completions` → streams correctly.

---

## Phase 7 — Commit + TECHDEBT update

- One commit per phase where practical (revert, frontend, anthropic middleware, openai middleware, routes wiring)
- Or a single squashed commit if the sub-agent chooses — each commit must leave the repo in a compiling state with passing tests.
- Update `ai-docs/claude-plans/202604/20260408-anthropic/TECHDEBT.md` if items are now resolved (e.g., removing the `bodhi-proxy` concern).

---

## Key Files

| File | Change |
|------|--------|
| `crates/bodhi/src/stores/agentStore.ts` | Rename DUMMY_API_KEY → SENTINEL_API_KEY; revert x-api-key:null hack |
| `crates/bodhi/src/stores/agentStore.test.ts` | Add/adjust assertion for sentinel value |
| `crates/routes_app/src/middleware/anthropic_auth_middleware.rs` | Sentinel detection + Authorization: Bearer sentinel stripping |
| `crates/routes_app/src/middleware/openai_auth_middleware.rs` | NEW — strip sentinel on `/v1/*` |
| `crates/routes_app/src/middleware/mod.rs` | Declare + re-export new middleware |
| `crates/routes_app/src/routes.rs` | Layer `openai_auth_middleware` onto user_apis group |

## Verification

- All unit tests for both middlewares pass (including new sentinel cases)
- Frontend agentStore tests pass
- Live browser test: anthropic_oauth alias streams correctly in `/ui/chat/` without user configuring a BodhiApp API token
- `anthropic` alias continues to work
- OpenAI alias continues to work
- Network tab shows NO `bodhiapp_sentinel_...` reaching Anthropic/OpenAI upstream (only internal to BodhiApp)
