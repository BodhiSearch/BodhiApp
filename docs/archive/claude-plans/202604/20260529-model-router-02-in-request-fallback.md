# Phase 2 — Model-Router In-Request Fallback

## Context

Phase 1 (commit `52753d0d`) landed the model-router foundation: a `ModelRouterAlias` (CRUD, persistence, validation, listing) that resolves a chat-completion request to its **first enabled target** and forwards it, returning the upstream response verbatim with observability headers. There is no failover — if the first enabled target returns `429`/`503`/etc., that error reaches the client even when a healthy secondary target exists.

Phase 2 makes the router **resilient within a single request**. When a target fails with a **retryable** error, routing falls through to the next enabled target until one succeeds or the chain is exhausted. **Terminal** (client-caused) errors stop immediately and return verbatim. There is still **no cross-request memory** — every request re-evaluates the chain from the top (cooldown/health is Phase 3).

The intended outcome: a router with a deliberately-broken primary and a working secondary serves a chat completion via the secondary, and the user sees a normal reply — without any client-side change.

### Design decisions (confirmed)
- **Status-only classification.** The fall-through decision uses the HTTP status alone, which `forward_to_upstream` (`crates/services/src/ai_apis/provider_shared.rs:155`) makes available before the body stream is consumed. We never buffer a failed attempt's body. The committed response (success / terminal / last-attempt) streams through unbuffered — preserving the "decide before first byte" rule and BodhiApp's opaque-proxy convention.
- **Exhaustion returns the last attempt's `Response` verbatim** (status + body stream intact), with the attempts header set to the number of enabled targets tried. If the only failure mode on the last attempt was a transport error (no `Response` in hand), surface the typed router error instead.
- **`max_attempts` is wired up in Phase 2; `honor_retry_after` and `cooldown_secs` are deferred to Phase 3.** The phase docs are explicit: Phase 3 says max_attempts is *"Honored since Phase 2; its UI lands here."* Conversely, `honor_retry_after`/`cooldown_secs` are cross-request cooldown concepts (proposal §6 ties `honor_retry_after` to `cooldown_for`), which require the Phase 3 health registry. Phase 2 reads `max_attempts` from `FallbackConfig` and leaves the other two persisted-but-unused (as today).

## Key facts grounding the implementation

- **Two failure shapes from `forward_one`:**
  1. An upstream HTTP error returns `Ok(Response)` with a non-2xx status embedded (`forward_to_upstream` never classifies — it copies status + all headers and streams the body). So `200`…`599` all arrive as `Ok(resp)`.
  2. A transport error (connection refused, timeout, DNS) returns `Err(ModelRouterError::Forward(AiApiClientFactoryError::Reqwest(_)))`.
  3. A structural problem (dangling/missing alias, nested router, unsupported format) returns a typed `Err(ModelRouterError::{ReferencedAliasNotFound, NestedRouterNotAllowed, TargetFormatUnsupported, ...})`.
- All of (2) and (3), plus retryable statuses from (1), must **fall through** to the next target. This makes "one broken target never kills the chain" fall out naturally.
- `RouterContext` and `forward_one` (`crates/services/src/models/router/strategy.rs:34`) need **no signature change** — Phase 2 logic lives entirely in `FallbackConfig::execute`.
- Existing test patterns to mirror: `crates/services/src/models/router/test_fallback_strategy.rs` (unit, `MockAiApiClient` + `MockDataService`), `crates/server_app/tests/test_live_model_router.rs` (live, mockito upstreams), and the Phase-1 E2E in `crates/lib_bodhiserver/tests-js/`.

## Implementation

### 1. Failure classification (`crates/services/src/models/router/`)

Add a small classification helper — a new `classify.rs` module (kept separate so future strategies reuse it, per the proposal's "shared primitives" design).

```rust
// classify.rs
pub enum Disposition { Success, Retryable, Terminal }

/// Status-only classification (proposal §4 / Phase-2 doc).
pub fn classify_status(status: StatusCode) -> Disposition {
  match status.as_u16() {
    200..=299 => Disposition::Success,
    // Terminal — the request itself is the problem; another vendor won't help.
    400 | 422 => Disposition::Terminal,
    // Retryable — transport-ish + the free-tier stacking rule (401/403/404 fall through).
    _ => Disposition::Retryable, // 408/429/5xx + 401/403/404 + any other non-terminal
  }
}
```

Notes:
- The retryable set in the doc is `408/429/500/502/503/504` **and** `401/403/404`. Treating "everything not 2xx and not 400/422" as retryable is the simplest faithful encoding and matches the doc's intent (content-policy / context-window violations surface as `400/422` from providers). If we later find a provider returns context-window as a different status, that's a Phase-3+ refinement.
- Content-policy / context-window are not distinct HTTP statuses we can detect status-only; they arrive as `400/422` and are correctly treated terminal. This is consistent with the "status-only" decision — no body inspection.

### 2. Fallback loop (`crates/services/src/models/router/fallback.rs`)

Replace the single-target `execute` with the fall-through loop:

```rust
async fn execute(&self, targets: &[RouterTarget], ctx: &RouterContext)
  -> Result<ModelRouterError> /* Result<Response, _> */
{
  let enabled: Vec<&RouterTarget> = targets.iter().filter(|t| t.enabled).collect();
  if enabled.is_empty() { return Err(ModelRouterError::EmptyChain); }

  // max_attempts: 0 = whole chain; otherwise cap.
  let cap = if self.max_attempts == 0 { enabled.len() }
            else { (self.max_attempts as usize).min(enabled.len()) };

  let mut attempts: u32 = 0;
  let mut last_resp: Option<Response> = None;       // last upstream Response (for verbatim exhaustion)
  let mut last_err: Option<ModelRouterError> = None; // last transport/structural error

  for target in enabled.into_iter().take(cap) {
    attempts += 1;
    match ctx.forward_one(target).await {
      Ok(resp) => match classify_status(resp.status()) {
        Disposition::Success | Disposition::Terminal =>
          return Ok(with_obs_headers(resp, target, STRATEGY_NAME, attempts)),
        Disposition::Retryable => {
          last_resp = Some(resp);  // hold for possible exhaustion; body NOT buffered/consumed
          last_err = None;
        }
      },
      // transport error OR structural skip (dangling alias, nested router, unsupported format)
      Err(e) => { last_err = Some(e); }
    }
  }

  // Exhausted. Prefer returning the last real upstream response verbatim.
  match (last_resp, last_err) {
    (Some(resp), _) => {
      // re-borrow the target that produced it; track it alongside last_resp instead.
      Ok(with_obs_headers(resp, /* last served target */, STRATEGY_NAME, attempts))
    }
    (None, Some(e)) => Err(e),       // every attempt was transport/structural — surface typed error
    (None, None)    => Err(ModelRouterError::EmptyChain), // unreachable (cap>=1)
  }
}
```

Refinements during implementation:
- Track the **target that produced `last_resp`** alongside the response (e.g. `last: Option<(&RouterTarget, Response)>`) so the exhaustion header reports the correct served target.
- `with_obs_headers` already stamps `x-bodhi-router-attempts`; pass the real `attempts` count instead of the hardcoded `1`. The success path now reports the attempt number it succeeded on (e.g. served by target #3 → `attempts = 3`), satisfying the observability requirement.
- Disabled targets are filtered out before the loop, so they are **never counted as attempts** (acceptance gate).
- A structural error (dangling alias) is captured as `last_err` and the loop **continues** to the next target — this is the "one broken target never kills the chain" behavior. Only if *every* attempt is structural/transport (no `Response` ever obtained) does the typed error surface.

### 3. `max_attempts` honoring
Already covered by `cap` above. `FallbackConfig.max_attempts` is read directly (it's `self.max_attempts` inside the trait impl). No new field, no migration. `honor_retry_after`/`cooldown_secs` remain untouched (Phase 3).

### 4. No changes needed to
- `RouterContext` / `forward_one` (strategy.rs) — signatures unchanged.
- The chat handler (`routes_app/src/oai/routes_oai_chat.rs`) — it already delegates to `route_chat_completion` and returns the Response.
- DB schema, entity, OpenAPI, TS client — no API surface change (config fields already exist).

## Tests (all layers — mirror existing patterns)

### Unit — `crates/services/src/models/router/test_fallback_strategy.rs`
Extend `ctx_with` (and add a multi-status variant) so different targets can return different statuses. Add cases:
- **Fall-through success:** primary `503`, secondary `200` → client gets `200`; `x-bodhi-routed-alias` = secondary; `x-bodhi-router-attempts` = `2`.
- **Parameterized retryable set** (`#[case]` over `408,429,500,502,503,504,401,403,404` + a transport-error case): each causes fall-through to a `200` secondary.
- **Parameterized terminal set** (`#[case]` over `400,422`): first such response returned verbatim, secondary **never** forwarded (assert via `MockAiApiClient` call count / `.times(1)`).
- **All retryable → last verbatim:** three targets all `503` → returns `503`, `attempts = 3`, served-alias = third target.
- **Dangling reference skipped:** first target's alias absent from `list_aliases`, second present and `200` → served by second, request succeeds.
- **Transport-error-only exhaustion:** single target whose client returns `Err(Reqwest)` → typed `ModelRouterError::Forward` surfaces.
- **Disabled not counted:** `[disabled, enabled→503, enabled→200]` → `attempts = 2`.
- **max_attempts cap:** chain of 3 all `503`, `max_attempts = 2` → stops after 2, returns 2nd response, `attempts = 2`, 3rd target never forwarded.

Add a `classify.rs` unit test table mapping representative statuses → `Disposition`.

### Server-app integration (live, mockito) — `crates/server_app/tests/test_live_model_router.rs`
Add `#[serial_test::serial(live)]` cases with two mockito upstreams:
- Primary mock returns `503`, secondary returns `200` → real HTTP request to the router alias returns `200`; assert observability headers (served = secondary, attempts = 2).
- Terminal: primary returns `400` → client gets `400`, secondary mock asserts **zero** hits.
- Streaming: primary returns `503`, secondary returns a streamed SSE `200` → assert the stream body arrives intact and is served by the secondary (status known before first byte).
- Streaming happy path: first enabled target streams `200` unchanged, attempts = 1.

### E2E (Playwright, UI-only) — `crates/lib_bodhiserver/tests-js/`
Extend the Phase-1 model-router E2E: create a router whose primary points at an unreachable/erroring provider and whose secondary is a working alias; send a chat message through the UI and assert a normal reply renders. Black-box (UI interactions only — no `page.evaluate`/context fetch), and throw in `beforeAll` if required env/creds are missing (no `test.skip`).

## Verification

1. **Upstream-to-downstream Rust gate:**
   - `cargo test -p services --lib 2>&1 | grep -E "test result|FAILED|failures:"` (classification + fallback unit tests).
   - `cargo test -p server_app 2>&1 | grep -E "test result|FAILED|failures:"` (live fallback/streaming).
   - `make test.backend` once after Rust changes settle (tee to a file per the capture-long-commands rule).
2. **No API/codegen drift:** config fields already exist; still run `cargo run --package xtask openapi && make ci.ts-client-check` to confirm the spec is unchanged.
3. **E2E:** `make build.dev-server` then `make test.e2e` from `crates/lib_bodhiserver/tests-js`.
4. **Manual demo (per phase doc):** `make app.run`, create alias A (bad endpoint) → target B (working) router; send a chat message → reply from B; swap order → served by primary; configure both to fail → real upstream error surfaced verbatim. Confirm `x-bodhi-router-attempts` / `x-bodhi-routed-alias` headers via browser network panel or `curl -i`.
5. **Docs:** update `crates/services/CLAUDE.md` model-router lines (fallback now does in-request failover with status-only classification + max_attempts) and the Phase-2 doc's status if tracked.

## Out of scope (Phase 3+)
- Cross-request cooldown / skipping / recovery; `cooldown_secs` + `honor_retry_after` activation; health registry keyed by `(tenant, referenced_alias_id, model)`.
- Config-knob UI for max_attempts/cooldown/retry-after (Phase 3).
- Router test/probe capability (Phase 4).
- New strategies.
