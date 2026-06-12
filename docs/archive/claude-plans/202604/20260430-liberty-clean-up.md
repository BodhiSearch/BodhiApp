# Plan: Clean up the LLM Liberty OAuth landing commit

## Context

Commit `c74c1002 Implement LLM Liberty OAuth support in API model configuration` landed an end-to-end implementation of `ApiFormat::LlmLibertyOauth`: new sibling table, repository, envelope DTOs, refresh logic, route wiring, frontend form, and a local-only Playwright smoke test. It works end-to-end (a smoke run reaches Anthropic Messages from the chat UI through the new envelope), so the commit is preserved as the working snapshot per the user's directive: **clean up will be amended onto this commit before push**, not committed separately.

The implementation was layered hastily across two paired plans (`20260429-liberty-intro.md`, `20260429-liberty-anthropic.md`). Reviewing the diff reveals concrete duplication, dead code, a real correctness regression vs. the design, and missing test coverage — all of which the plans called out as required and the commit did not actually deliver. This plan tightens the implementation to production quality without changing observable behaviour.

The cleanup falls into four buckets:

1. **Dead code and a correctness bug**: a parallel provider client and a parallel refresh function that the live code paths bypass.
2. **Mechanical duplication**: the same envelope→`(api_key, base_url, extra_headers, extra_body)` shape is reconstructed by hand in 6 sites; envelope JSON validation is duplicated 3× in TS and 1× in Rust.
3. **Schema/structure cleanups** in the frontend (two near-identical Zod schemas, two near-identical converters).
4. **Test gap**: the plans listed ~12 unit tests across services + routes_app + frontend; only one trivial sanity check made it in. Live-flow E2E remains intentionally local-only (3rd-party OAuth, ~8h token TTL, automation blockers).

## Cleanup tasks

### 1. Delete dead code: `LlmLibertyAnthropicClient`

`crates/services/src/ai_apis/llm_liberty/anthropic.rs` (181 lines) defines a fully-implemented client (`new`, `apply_auth`, `base_url`, `models`, `test_connection`, `forward`) that is never called. The actual dispatch in `crates/services/src/ai_apis/ai_api_service.rs:141,195,256` reuses `AnthropicOAuthProviderClient` directly. Confirmed via `grep -rn "LlmLibertyAnthropicClient"` — only the definition and the `pub use` in `mod.rs`.

**Actions:**
- Delete `crates/services/src/ai_apis/llm_liberty/anthropic.rs`.
- Drop `pub mod anthropic;` and `pub use anthropic::LlmLibertyAnthropicClient;` from `crates/services/src/ai_apis/llm_liberty/mod.rs:1,4`.

### 2. Wire the per-alias mutex back in: collapse two refresh implementations into one

The intro plan specified a per-alias `tokio::sync::Mutex` to serialize concurrent refresh attempts on the same alias (single-node correctness, multi-node deferred). The committed code has the mutex in `crates/services/src/ai_apis/llm_liberty/refresh.rs::ensure_fresh_credentials` — but **nothing calls it**. The three live call sites (anthropic proxy, `api_models_test`, `api_models_fetch_models`) all go through `crates/routes_app/src/providers/mod.rs:33-141::resolve_llm_liberty_credentials`, a 109-line copy of the same logic without a mutex. Result: two parallel inbound chat requests on the same alias near token expiry will both refresh, both succeed, and only the second `update_llm_liberty_tokens` write wins. This is the exact scenario the mutex was supposed to prevent on a single node.

**Actions:**
- Make `services::ai_apis::llm_liberty::ensure_fresh_credentials` the single source of truth. It already takes a `&dyn LlmLibertyCredentialsRepository`; verify its signature is callable from `routes_app` (the `db: &R` generic constraint may need to be `&dyn LlmLibertyCredentialsRepository` or accept `Arc<dyn DbService>` — adapt as needed since `DbService` already has `LlmLibertyCredentialsRepository` as a supertrait per `crates/services/src/db/service.rs:19,39`).
- Have it return a typed error (`AiApiServiceError` or a dedicated `LlmLibertyRefreshError`) instead of `DbError::ValidationError("LLM Liberty token refresh failed: ...")`, which is misleading.
- Replace `routes_app/src/providers/mod.rs::resolve_llm_liberty_credentials` (delete entirely) with calls to `services::ai_apis::llm_liberty::ensure_fresh_credentials(db, http, tenant, user, alias)` from the three call sites:
  - `crates/routes_app/src/anthropic/routes_anthropic.rs:79`
  - `crates/routes_app/src/models/api/routes_api_models.rs:184`
  - `crates/routes_app/src/models/api/routes_api_models.rs:347`
- Each site currently calls `.await` and matches `Some(c)` / `None`; the new function returns `Result<ResolvedLlmLibertyCredentials, _>`. Map errors to the existing `BodhiErrorResponse::from(...)` paths (or `OAIRouteError`/`AnthropicApiError` shells already used).
- Drop the bespoke `REFRESH_SKEW_SECS` constant in `routes_app/src/providers/mod.rs:6` — the services-layer copy is now canonical.
- The `SafeReqwest` HTTP client should come from `auth_scope.ai_api()` if exposed, or be constructed once and shared. Avoid building one per call as the deleted route version did.

### 3. Collapse identical match arms in `ai_api_service.rs`

`crates/services/src/ai_apis/ai_api_service.rs:141-152, 195-206, 256-266` — three `LlmLibertyOauth` arms whose bodies are byte-for-byte identical to the preceding `AnthropicOAuth` arms. Comments explain "Route handler resolves credentials and passes access_token as api_key…", which is exactly the contract `AnthropicOAuth` already follows.

**Actions:**
- Replace each pair with `ApiFormat::AnthropicOAuth | ApiFormat::LlmLibertyOauth => { … }`. Saves ~36 lines and removes the implicit invariant that the two arms must drift in lockstep. The explanatory comment moves above the merged arm.

### 4. Centralize the envelope-to-request-parts conversion

The shape `(Option<String> /* api_key */, String /* base_url */, Option<Value> /* extra_headers */, Option<Value> /* extra_body */)` is built by hand from either a `ResolvedLlmLibertyCredentials` or an `LlmLibertyEnvelope` in 6 places, all containing the same `if .is_null() { None } else { Some(...) }` ladder and the same `derive_base_url(&chat_url)` call:

- `crates/services/src/models/api_model_service.rs:255-277` (sync_models — from creds)
- `crates/services/src/models/api_model_service.rs:401-411` (create_llm_liberty — from envelope)
- `crates/services/src/models/api_model_service.rs:537-575` (update_llm_liberty — both branches)
- `crates/routes_app/src/models/api/routes_api_models.rs:184-244` (api_models_test — from creds + from envelope)
- `crates/routes_app/src/models/api/routes_api_models.rs:346-398` (api_models_fetch_models — from creds + from envelope)
- `crates/routes_app/src/anthropic/routes_anthropic.rs:78-87` (resolve_anthropic_alias — from creds)

**Actions:**
- Add to `crates/services/src/models/llm_liberty_envelope.rs`:

  ```rust
  pub struct LlmLibertyRequestParts {
    pub access_token: Option<String>,
    pub base_url: String,
    pub extra_headers: Option<serde_json::Value>,
    pub extra_body: Option<serde_json::Value>,
  }

  impl ResolvedLlmLibertyCredentials {
    pub fn into_request_parts(self) -> LlmLibertyRequestParts { … }
  }

  impl LlmLibertyEnvelope {
    pub fn to_request_parts(&self) -> LlmLibertyRequestParts { … }
  }
  ```

  Both rely on the same private `value_to_opt(v: serde_json::Value) -> Option<serde_json::Value>` helper that returns `None` for `Value::Null`.
- Replace the 6 call sites with one-liners. `routes_anthropic.rs::resolve_anthropic_alias` becomes `let parts = c.into_request_parts(); api_alias.base_url = parts.base_url; api_alias.extra_headers = parts.extra_headers; api_alias.extra_body = parts.extra_body; (api_alias, parts.access_token)`.
- Once `into_request_parts()` is the only consumer of `derive_base_url`, drop `derive_base_url` (`crates/services/src/ai_apis/llm_liberty/mod.rs:9-15`) entirely and use `envelope.api.base_url` / `creds.api_base_url` directly. The envelope schema already has both `base_url` and `chat_url` (`crates/services/src/models/llm_liberty_envelope.rs:27-31`); recomputing from `chat_url` was an oversight.

### 5. Centralize envelope JSON validation in the frontend

Three near-identical implementations check `version === '1.0.0'` and `provider === 'anthropic'`:

- `crates/bodhi/src/components/api-models/form/LlmLibertyEnvelopeInput.tsx:26-61` — `parseEnvelope` (returns summary or error)
- `crates/bodhi/src/schemas/apiModel.ts:111-144` — `createApiModelSchema.superRefine` (envelope branch)
- `crates/bodhi/src/schemas/apiModel.ts:221-247` — `updateApiModelSchema.superRefine` (envelope branch, allows empty)

**Actions:**
- Extract one helper, e.g. `crates/bodhi/src/schemas/llmLibertyEnvelope.ts` (new file co-located with `apiModel.ts`):

  ```ts
  export type EnvelopeValidation =
    | { ok: true; envelope: LlmLibertyEnvelope; summary: { provider: string; expiresAt: Date; hasRefreshToken: boolean } }
    | { ok: false; error: string };

  export function validateLlmLibertyEnvelope(text: string): EnvelopeValidation { … }
  ```

  It owns the version/provider/required-field checks. The existing `LlmLibertyEnvelopeInput.parseEnvelope` becomes a thin wrapper or is replaced inline. Both Zod schemas call it inside `superRefine` and convert `{ ok: false, error }` into `ctx.addIssue`.

### 6. Collapse the two Zod schemas

`createApiModelSchema` and `updateApiModelSchema` (`crates/bodhi/src/schemas/apiModel.ts:96-203, 206-296`) duplicate ~60% of their `superRefine` body. The differences are exactly two:

- envelope is **required** in create / **optional** in update,
- api_key min-length is enforced on create / not on update (so an empty string in update means "keep").

**Actions:**
- Factor a single `buildApiModelSchema({ requireEnvelope: boolean; requireApiKey: boolean })` and call it twice with the two configs. The body of the `superRefine` (forward_all_with_prefix, models length, base_url, extra_headers/body) becomes a single block.

### 7. Collapse the two API-to-form converters

`convertApiToForm` and `convertApiToUpdateForm` (`crates/bodhi/src/schemas/apiModel.ts:393-419`) have identical bodies; only the inferred return type differs and that's structural. Drop one and have the other be the single source.

### 8. Tighten `validate_supported` to a typed error

`LlmLibertyEnvelope::validate_supported` (`crates/services/src/models/llm_liberty_envelope.rs:64-93`) returns `Result<(), String>`. Callers wrap it as `ObjValidationError::LlmLibertyEnvelopeInvalid(String)`. Make the function return `Result<(), ObjValidationError>` directly (define a `LlmLibertyEnvelopeInvalid { reason }` variant if not already there) so the four call sites (`api_model_service.rs:397,539`, `routes_api_models.rs:216,375`) drop the `.map_err(|e| …)` adapter line.

### 9. Drop redundant `llm_liberty_oauth` baseUrl from `API_FORMAT_PRESETS`

`crates/bodhi/src/schemas/apiModel.ts:45-49` defines a baseUrl for `llm_liberty_oauth`, but the form never shows a base-URL field for that format and the value is overwritten from the envelope on submit. Either delete the entry or add a note. Recommend delete.

### 10. Remove the `pub` on `derive_base_url`'s last consumers

After tasks 4 + 9, `derive_base_url` should have no callers. Confirm with a final `grep -rn derive_base_url` and remove. Module `mod.rs` becomes 5 lines.

## Test coverage to add (services layer)

The intro plan listed these as required; none made it into the commit. With the refresh path now correctly going through `ensure_fresh_credentials`, the mutex test in particular gates real correctness.

Create `crates/services/src/ai_apis/llm_liberty/test_refresh.rs` (sibling-`test_*.rs` pattern per `crates/CLAUDE.md`):

- `refresh_returns_creds_when_token_within_skew_window` — frozen time, expires_at > now+60s; assert no HTTP call to `oauth.token_url` (mockito with `expect(0)`).
- `refresh_persists_rotated_tokens_when_expired` — frozen time, expires_at < now+60s; mockito returns `{access_token, refresh_token, expires_in}`; assert `update_llm_liberty_tokens` was called with rotated values.
- `refresh_uses_old_refresh_token_when_response_omits_one` — defends against the `unwrap_or(&creds.refresh_token)` branch (`refresh.rs:107-109`).
- `refresh_concurrent_requests_serialize` — spawn two tasks calling `ensure_fresh_credentials` for the same alias, expired token, mockito with `expect_at_most(1)` on the token endpoint; both tasks return; only one HTTP refresh observed.
- `refresh_propagates_5xx_as_error` — mockito returns 503; assert `Err(...)` (codifies fail-fast vs. retry — the plan flagged this as needing a decision; pick fail-fast and lock it in).

Create `crates/services/src/models/test_llm_liberty_credentials_repository.rs`:

- `create_then_get_round_trip_decrypts` — encrypted at rest, plaintext on read.
- `update_credentials_replaces_atomically` — second envelope wins; old refresh_token gone.
- `update_tokens_only_changes_token_columns` — `auth_in/auth_key/oauth_token_url` etc. unchanged after token-only update.
- `get_returns_none_for_other_tenant` — RLS/scope check (mirror `crates/services/src/mcps/test_mcp_repository_isolation.rs`).
- `get_returns_none_for_other_user_same_tenant`.
- `delete_cascades_on_alias_delete` — set up an alias + creds, delete the alias, verify creds row is gone via FK CASCADE.

Run with `#[values("sqlite", "postgres")]` per `crates/CLAUDE.md` Multi-Tenant Isolation Test Pattern.

## Test coverage to add (routes_app layer)

Create `crates/routes_app/src/models/api/test_api_models_llm_liberty.rs`:

- `create_201_with_valid_envelope` — POST `/bodhi/v1/api-models` with `api_format: "llm_liberty_oauth"`, full envelope; mockito on Anthropic `/models` for the fetch step; assert response `llm_liberty: { provider: "anthropic", expires_at: …, has_refresh_token: true }`.
- `create_400_when_envelope_version_unsupported` — version `"2.0.0"`; assert `LlmLibertyEnvelopeInvalid` code.
- `create_400_when_envelope_provider_unsupported` — provider `"openai-codex"`.
- `update_replaces_credentials_when_envelope_set` — second PUT with new envelope; assert `update_llm_liberty_credentials` path.
- `update_keeps_credentials_when_envelope_keep` — PUT with `{action: "keep"}`; alias-scope fields update; envelope row untouched (verify via repo read).
- `test_endpoint_400_when_id_and_envelope_both_present` — exercises the `(Some, Some)` guard at `routes_api_models.rs:180`.
- `test_endpoint_400_when_neither_id_nor_envelope` — exercises `(None, None)` guard at the same line.

Extend `crates/routes_app/src/anthropic/test_anthropic_oauth_routing.rs` with one `LlmLibertyOauth` row in its parameterized table — exercising the new branch in `resolve_anthropic_alias` after task 4 collapses the credential-shaping. Exercise once with provider=`"anthropic"` (works) and once with provider not present (the existing fallback returns the alias with `None` api_key — confirm or reject this fallback in code review).

## Test coverage to add (frontend)

Create `crates/bodhi/src/components/api-models/form/LlmLibertyEnvelopeInput.test.tsx`:

- `parses_valid_envelope_and_renders_summary`.
- `rejects_malformed_json_with_error_message`.
- `rejects_unsupported_version`.
- `rejects_unsupported_provider`.
- `clears_summary_when_input_emptied`.
- `shows_keep_existing_hint_when_edit_mode_with_stored_credentials`.

Extend `crates/bodhi/src/components/api-models/ApiModelForm.extras.test.tsx` (or co-locate a `ApiModelForm.llm_liberty.test.tsx`):

- `llm_liberty_oauth_format_hides_api_key_input_and_extras_section`.
- `llm_liberty_oauth_format_hides_base_url_input`.
- `anthropic_oauth_format_still_shows_legacy_extras_section` — regression guard for task 3 (collapsed match arm must not change observable form behaviour).

These component tests are MSW-driven and don't need real OAuth.

## Test coverage that remains intentionally manual

The live OAuth flow against Anthropic's real authorize URL cannot be CI-tested:

- Tokens are ~8h-lived; refresh tokens get server-revoked when flagged as third-party usage.
- The `npx @bodhiapp/llm-liberty@latest login anthropic` step requires a real browser-based login at console.anthropic.com with bot-protection.
- `crates/lib_bodhiserver/tests-js/specs/api-models/api-llm-liberty-anthropic.spec.mjs` is correctly opt-in via `BODHI_E2E_LOCAL=1` and reads `tests-js/data/local/anthropic.json` (gitignored). It throws (rather than silently skips) when the file is malformed/expired — already aligned with `feedback_no_skip_for_missing_env.md`.

Manual verification gate (mandatory before amending the commit):

1. `make app.run.live` (or `ports kill 1135 && make app.run.live`).
2. Browser → API Models → New → "LLM Liberty OAuth"; paste a fresh envelope from `npx @bodhiapp/llm-liberty@latest login anthropic`; Fetch Models → pick Haiku; Test Connection → success; Save.
3. Open the saved alias → Edit → change prefix only → save → confirm prefix updated and credentials untouched.
4. Edit again → paste a new envelope → confirm credentials replaced (DB inspection or refresh log line).
5. Chat UI → select the alias → send "what day comes after monday, answer in one word"; assert request hits `/anthropic/v1/messages` (DevTools Network) and reply contains "tuesday".
6. Trip refresh manually: directly UPDATE the row's `expires_at` to `now() - interval '5 minutes'` in the DB; send another chat message; confirm exactly one `POST oauth.token_url` in the logs (per-alias mutex working) and the chat reply succeeds.
7. Regression: `Anthropic Setup Token` provider card still works end-to-end with a `claude setup-token` token; `OpenAI` provider unaffected.

Black-box-only per `feedback_blackbox_e2e.md` — no `page.evaluate`/`page.context` fetches.

## Critical files to modify

### Rust — services
| File | Change |
|---|---|
| `crates/services/src/ai_apis/llm_liberty/anthropic.rs` | **delete** (dead code, task 1) |
| `crates/services/src/ai_apis/llm_liberty/mod.rs` | drop `pub mod anthropic`, drop `pub use anthropic::…`, drop `derive_base_url` (task 1, 4, 10) |
| `crates/services/src/ai_apis/llm_liberty/refresh.rs` | adapt signature to be callable from routes_app (`&dyn DbService`), return typed error (task 2) |
| `crates/services/src/ai_apis/llm_liberty/test_refresh.rs` | **new** — mutex + skew + rotation tests (test coverage) |
| `crates/services/src/ai_apis/ai_api_service.rs:141,195,256` | merge `AnthropicOAuth` and `LlmLibertyOauth` arms (task 3) |
| `crates/services/src/models/llm_liberty_envelope.rs` | add `LlmLibertyRequestParts`, `into_request_parts`/`to_request_parts`, `value_to_opt`; tighten `validate_supported` to typed error (task 4, 8) |
| `crates/services/src/models/api_model_service.rs:255-277,401-411,537-575` | use new `to_request_parts`/`into_request_parts` (task 4) |
| `crates/services/src/models/test_llm_liberty_credentials_repository.rs` | **new** — repo CRUD + RLS isolation (test coverage) |
| `crates/services/src/shared_objs/error_wrappers.rs` (if `LlmLibertyEnvelopeInvalid` lives there) | optional shape change for typed error (task 8) |

### Rust — routes_app
| File | Change |
|---|---|
| `crates/routes_app/src/providers/mod.rs` | **delete** `resolve_llm_liberty_credentials` (task 2); keep `resolve_api_key_for_alias` |
| `crates/routes_app/src/anthropic/routes_anthropic.rs:77-87` | call `services::ai_apis::llm_liberty::ensure_fresh_credentials`; use `into_request_parts()` (task 2, 4) |
| `crates/routes_app/src/models/api/routes_api_models.rs:184,347` | same — and dedup the test/fetch_models LlmLiberty branches against the default branch (task 2, 4) |
| `crates/routes_app/src/anthropic/test_anthropic_oauth_routing.rs` | extend with `LlmLibertyOauth` parameterized row (test coverage) |
| `crates/routes_app/src/models/api/test_api_models_llm_liberty.rs` | **new** (test coverage) |

### Frontend
| File | Change |
|---|---|
| `crates/bodhi/src/schemas/llmLibertyEnvelope.ts` | **new** — single envelope validator (task 5) |
| `crates/bodhi/src/schemas/apiModel.ts:45-49,96-296,393-419` | call new validator; collapse two schemas via builder; collapse two converters (task 5, 6, 7, 9) |
| `crates/bodhi/src/components/api-models/form/LlmLibertyEnvelopeInput.tsx:26-61` | replace `parseEnvelope` body with the new shared validator (task 5) |
| `crates/bodhi/src/components/api-models/form/LlmLibertyEnvelopeInput.test.tsx` | **new** (test coverage) |
| `crates/bodhi/src/components/api-models/ApiModelForm.llm_liberty.test.tsx` | **new** (test coverage) |

## Reusable building blocks already in place

Use these — don't reinvent:

- `crates/services/src/db/encryption.rs::{encrypt_api_key, decrypt_api_key}` — already used by the new repo, fine.
- `crates/services/src/mcps/test_mcp_repository_isolation.rs` — canonical RLS multi-tenant test pattern; mirror it for the new credentials repo.
- `crates/services/src/ai_apis/provider_shared.rs::{merge_extra_body, forward_to_upstream}` — already what `AnthropicOAuthProviderClient` uses; the merged match arm in task 3 keeps this as the single forward path.
- `crates/services/src/test_utils/db.rs::{TEST_TENANT_ID, TEST_TENANT_B_ID, TEST_USER_ID}` — for isolation tests.
- `mockito` is already used in `test_ai_api_anthropic.rs`/`test_ai_api_anthropic_oauth.rs` — refresh tests should follow the same setup.
- `pretty_assertions::assert_eq` per `crates/CLAUDE.md` testing conventions.

## Layered execution order

Per `CLAUDE.md` upstream-first methodology and `feedback_layered_refactors.md` (refactors defer commits until amend):

1. **services/** (tasks 1, 2 signature, 3, 4 helpers, 8): code change. Run `cargo test -p services`.
2. **services/** test additions (refresh tests, repo tests). Run `cargo test -p services`.
3. **routes_app/** (tasks 2 wiring, 4 call sites, plus new tests). Run `cargo test -p routes_app`.
4. **Full backend gate**: `make test.backend` (per `feedback_run_all_gate_checks.md`; per `feedback_capture_long_commands.md`, tee output the first time).
5. **Frontend** (tasks 5, 6, 7, 9 + new component tests). Run `cd crates/bodhi && npm run test`.
6. **Manual verification gate** (steps 1–7 above). Required before amend.
7. **Amend**: `git commit --amend --no-edit` (then `git status` to confirm clean tree).

Single amend at the end; no intermediate commits, per the user directive that this preserves a working snapshot until cleanup is verified.

## Verification

End-to-end correctness:

```bash
make test.backend                                      # Rust gate
cd crates/bodhi && npm run test                        # Frontend gate
make app.run.live                                      # for manual flow
# then BODHI_E2E_LOCAL=1 npm run test:playwright -- specs/api-models/api-llm-liberty-anthropic.spec.mjs
# (only if the local envelope file is fresh)
```

Specifically validate:

- `grep -rn "LlmLibertyAnthropicClient\|derive_base_url\|resolve_llm_liberty_credentials\|REFRESH_SKEW_SECS" crates/` returns zero results after task 1+2+4+10.
- `cargo test -p services --lib refresh_concurrent_requests_serialize` passes (mutex correctness).
- Manual step 6 (manual `expires_at` rewrite + concurrent chat messages) shows exactly one `oauth.token_url` POST in logs.
- `Anthropic Setup Token` provider card still saves and chats successfully (no regression from task 3 arm collapse).
