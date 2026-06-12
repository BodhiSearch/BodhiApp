# Plan: LLM Liberty OAuth — Review-Driven Fixes

## Context

Commit `f2892bcb` shipped LLM Liberty OAuth (Anthropic v1) — a paste-an-envelope flow that lets users bring their own OAuth credentials acquired via the sibling `llm-liberty` CLI. The five-agent review (`ai-docs/claude-plans/202604/reviews/`) surfaced 1 Critical and 27 Important findings spanning the migration, services repository, refresh path, routes_app proxy, UI validator, and E2E tests.

This plan addresses the Important+Critical findings the user has accepted, plus three new requirements clarified during planning:

1. **Disable format switching on edit entirely** (UI + Rust). Investigation confirmed `LlmLibertyOauth` is the only `ApiFormat` variant with a sibling table (`api_model_oauth_credentials`); switching out of it orphans the encrypted-credentials row, and switching into it has its own silent-no-op bug. The simplest, safest fix is to forbid the toggle in edit mode. Create-mode keeps the selector.
2. **401 retry with force-refresh** on the upstream forward / fetch_models / test_prompt paths. Anthropic may invalidate access tokens before `expires_at` (third-party-usage flags). On 401, force-refresh tokens (bypassing skew check) and retry the upstream call once.
3. **Zod-migrate the envelope validator** (UI), validating only the truly-required leaves the user enumerated: `auth.{in,key,scheme}`, `oauth.{token_url,client_id}`. Skip `authorize_url`, `api.*_url`, and `expires_at` type-as-integer.

User-rejected findings (will NOT be addressed): the `LOGIN_COMMAND` `anthropic` argument (#19), and excluding the local spec from the `multi_tenant` Playwright project (#25).

The intended outcome is a v1-ready LLM Liberty OAuth feature with: no orphan rows, hard-fail on missing/cross-tenant updates, robust refresh-on-401, validated envelope inputs at both layers, and the test coverage the plan originally promised.

## Implementation Phases

Each phase ends with `cargo test -p <crate>` (or the layer-appropriate gate) and a commit per `feedback_layered_refactors.md`. No commit skipping (`feedback_run_all_gate_checks.md`).

---

### Phase 1 — Migration & entity hardening
Edit migration in place (per CLAUDE.md "no backwards compat" rule).

- `crates/services/src/db/sea_migrations/m20250101_000021_api_model_oauth_credentials.rs`
  - Add `idx_api_model_oauth_credentials_expires_at` on `expires_at` (matches `mcps/m20250101_000012` pattern).
  - Switch `headers_json`, `body_json`, `extra_json` from `json(...)` to `.json_binary()` (matches `m20250101_000020_api_alias_extra_fields.rs`).
  - Add `.default(Expr::cust("'{}'::jsonb"))` to `headers_json` and `body_json`.
- `crates/services/src/models/llm_liberty_credentials_entity.rs`
  - Define `Relation::ApiModelAlias` (belongs_to `api_model_aliases`, FK `api_alias_id` → `id`, `on_delete = "Cascade"`, `on_update = "Cascade"`).
  - `impl Related<api_model_alias_entity::Entity> for Entity`.

Verify: `cargo test -p services -- --test-threads=1` (DB-touching tests).

---

### Phase 2 — Repository correctness & isolation
- `crates/services/src/models/llm_liberty_credentials_repository.rs`
  - `update_llm_liberty_credentials`: replace silent `return Ok(())` on missing/mismatched row with `Err(DbError::ItemNotFound { id, item_type: "llm_liberty_credentials" })` (the canonical variant per `db/error.rs:52`).
  - `delete_llm_liberty_credentials`: same — return `ItemNotFound` (caller handles idempotency via match on error variant if needed).
  - `update_llm_liberty_tokens`: add explicit `WHERE tenant_id = ?` filter (defends SQLite where RLS doesn't fire). Document with a one-liner: `// SQLite has no RLS; filter explicitly.`
- `crates/services/src/models/llm_liberty_envelope.rs`
  - `LlmLibertyEnvelope.headers` / `body`: replace `#[serde(default)]` with `#[serde(default = "default_empty_obj")]` returning `serde_json::json!({})`. Avoids `Value::Null` writes to PG `JSON NOT NULL` and matches the schema DEFAULT.
- `crates/services/src/models/test_llm_liberty_credentials_repository.rs`
  - Add `test_update_credentials_returns_not_found_when_row_missing`.
  - Add `test_update_credentials_returns_not_found_for_cross_tenant_attempt` (mirrors `mcps/test_mcp_repository_isolation.rs`).
  - Add `test_delete_credentials_returns_not_found_for_cross_tenant_attempt`.
  - Add `assert_eq!(expires_at_secs, summary.expires_at)` to existing summary test.

Verify: `cargo test -p services llm_liberty`.

---

### Phase 3 — Envelope validation (server-side)
- `crates/services/src/models/llm_liberty_envelope.rs::validate_supported`
  - Add validation: `auth.location == "header"`, `auth.key == "Authorization"`, `auth.scheme == "Bearer"`. v1 hardcodes Bearer in the provider client; reject anything else loudly so future provider variations surface as a clear error rather than silent auth failure.
  - Existing checks (`version == "1.0.0"`, `provider == "anthropic"`, non-empty tokens, non-empty `oauth.token_url` / `oauth.client_id` / `api.chat_url`) stay.

Verify: `cargo test -p services llm_liberty_envelope`.

---

### Phase 4 — Refresh path: client timeout, no-default `expires_in`, 401 retry
- `crates/services/src/ai_apis/llm_liberty/refresh.rs`
  - Add explicit timeout to the OAuth refresh `Client::builder().timeout(Duration::from_secs(15))` (or use the shared `SafeReqwest` if it already enforces one — use the existing `http: &SafeReqwest` parameter and document the assumption).
  - Remove `unwrap_or(28800)` on `expires_in`. If response omits the field, return `LlmLibertyRefreshError::MissingExpiresIn`.
  - Add `pub async fn force_refresh_credentials(...)` — same as `ensure_fresh_credentials` but bypasses the skew check and always calls `do_refresh`. Reuses the per-alias mutex.
- `crates/services/src/ai_apis/provider_anthropic_oauth.rs` (the client used by `LlmLibertyOauth`'s dispatch arm)
  - In `test_connection`, `models`, `forward`: on receiving HTTP 401, call back into `routes_app::providers::resolve_llm_liberty_credentials_with_force_refresh` (new helper — see Phase 7) and retry once with the new access token. If the second attempt also returns 401, surface the upstream 401 to the caller untouched.
  - Implementation note: the cleanest seam is a small `with_retry_on_401` helper in `provider_shared.rs` that takes a `request_builder: impl Fn(&str) -> RequestBuilder` and a `refresh: impl FnOnce() -> Future<Output = Result<String>>`. The non-OAuth providers pass a no-op refresh closure; LlmLibertyOauth wires it through.
- `crates/services/src/ai_apis/test_ai_api_provider_matrix.rs`
  - Add `LlmLibertyOauth` to the `#[case]` parameterization for `test_prompt`, `models`, `forward` matrices.
  - Add new test `test_forward_retries_once_after_401_with_refresh` exercising the new behavior (mockito).

Verify: `cargo test -p services ai_apis`.

---

### Phase 5 — Format-switch lockdown (Critical)
Disable changing `api_format` on existing aliases. Only LlmLibertyOauth has a sibling table today, but locking the contract project-wide eliminates a whole class of orphan/validation bugs and matches the user's directive.

- `crates/services/src/models/api_model_service.rs::update`
  - Reject `form.api_format != existing.api_format` with `ApiModelServiceError::Validation(ObjValidationError::ApiFormatImmutableOnEdit)`.
  - Remove the now-dead "format-changed-with-Keep-key" branch in `update_default` (its responsibility moves to the upstream guard).
- `crates/services/src/lib.rs` (or wherever `ObjValidationError` lives, see `crates/objs`)
  - Add `ApiFormatImmutableOnEdit` variant with code `obj_validation_error_api_format_immutable_on_edit`. (Note: `ObjValidationError` lives in `objs` crate per the dependency chain — change there if needed.)
- `crates/services/src/models/test_api_model_service.rs`
  - Update the existing `test_update_rejects_api_format_change_with_keep_key` to match the new shape (rejects unconditionally, regardless of api_key).
  - Add `test_update_rejects_api_format_change_to_llm_liberty`.
  - Add `test_update_rejects_api_format_change_from_llm_liberty`.
  - Defense-in-depth: even with format-switch forbidden, keep `update_llm_liberty_credentials` returning `ItemNotFound` from Phase 2 — guards against races (alias deleted concurrently).

Verify: `cargo test -p services models::api_model_service`.

---

### Phase 6 — routes_app: provider check, request XOR, OpenAPI, integration tests
- `crates/services/src/models/llm_liberty_envelope.rs::ResolvedLlmLibertyCredentials`
  - Add `pub provider: String`. Populate in `llm_liberty_credentials_repository::get_llm_liberty_credentials` from the row.
- `crates/routes_app/src/anthropic/routes_anthropic.rs`
  - In the `LlmLibertyOauth` match arm of the proxy resolver, after `resolve_llm_liberty_credentials`, assert `creds.provider == "anthropic"` — otherwise return `ApiError` with status 400 and code `llm_liberty_unsupported_provider`. v1 routes only Anthropic envelopes through this proxy; future providers will get their own routes.
- `crates/routes_app/src/providers/mod.rs`
  - Replace the per-call `SafeReqwest::default()` (or however it's constructed) with a clone of an `Arc<SafeReqwest>` injected via the existing service container. Mirror the pattern already used in `ai_api_service.rs:82`.
  - Add `resolve_llm_liberty_credentials_with_force_refresh(...)` for the 401-retry path (Phase 4).
- `crates/services/src/models/model_objs.rs`
  - `LlmLibertyTestPromptRequest` and `LlmLibertyFetchModelsRequest`: replace the current "id and envelope optional" shape with a tagged enum: `enum Source { Saved { id: String }, Inline { envelope: LlmLibertyEnvelope } }`. Update the OpenAPI schema accordingly so the discriminator is explicit.
  - Confirm existing test cases in `test_api_models_llm_liberty.rs` still pass after the type change.
- `crates/routes_app/src/shared/openapi.rs`
  - Update the `api_models_formats` description and example to enumerate all 6 formats (including `llm_liberty_oauth`).
- `crates/routes_app/src/anthropic/test_anthropic_oauth_routing.rs` and `test_anthropic_messages.rs`
  - Add `LlmLibertyOauth` rows to the parameterized matrix (mirror existing `AnthropicOAuth` cases).
  - Add `test_anthropic_proxy_rejects_non_anthropic_envelope` (would only fire if a future provider lands; assert the v1 guard works).
- `crates/routes_app/src/models/api/test_api_models_llm_liberty.rs`
  - Add `400_on_missing_required_envelope_field` cases (one per leaf: `auth.in`, `auth.key`, `auth.scheme`, `oauth.token_url`, `oauth.client_id`, `api.chat_url`).

Verify: `cargo test -p routes_app`.

Run cumulative gate: `make test.backend`.

---

### Phase 7 — ts-client regen
```
cargo run --package xtask openapi
cd ts-client && npm run generate
make build.ts-client
```
Capture the diff in the same commit as the type-consuming UI changes (Phase 8).

---

### Phase 8 — UI: zod validator, textarea attrs, format selector lock, edit-mode tests
- `crates/bodhi/src/schemas/llmLibertyEnvelope.ts`
  - Replace the hand-rolled `validateLlmLibertyEnvelope` with a zod schema. Required leaves: `version` (literal `'1.0.0'`), `provider` (literal `'anthropic'`), `access_token` (non-empty), `refresh_token` (non-empty), `auth.in`, `auth.key`, `auth.scheme`, `oauth.token_url`, `oauth.client_id`. NOT validated: `oauth.authorize_url`, `api.base_url`, `api.chat_url`, `api.models_url`, `expires_at` (per user direction — server is source of truth for those).
  - Keep the function's existing return contract (`{ ok, envelope?, summary?, error? }`) so `apiModel.ts::buildApiModelSchema` doesn't need changes.
- `crates/bodhi/src/components/api-models/form/LlmLibertyEnvelopeInput.tsx`
  - Add to the `<Textarea>`: `autoComplete="off"`, `spellCheck={false}`, `autoCorrect="off"`, `data-1p-ignore` (matches the existing `ApiKeyInput` pattern — verify by grep when implementing).
- `crates/bodhi/src/components/api-models/ApiModelForm.tsx`
  - Pass `disabled={isEditMode}` to `<ApiFormatSelector>`.
- `crates/bodhi/src/components/api-models/form/ApiFormatSelector.tsx`
  - Honor existing `disabled` prop; add a tooltip / helper text "Format cannot be changed after creation. Delete and recreate the alias to use a different format."
- `crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts`
  - In `handleApiFormatChange`: noop (or guard against being called) when in edit mode. Belt-and-suspenders against the form being mutated programmatically.
- Tests:
  - `crates/bodhi/src/components/api-models/ApiModelForm.llm_liberty.test.tsx`: add cases for edit mode — `envelope: { action: 'keep' }` when textarea empty; `envelope: { action: 'set', value: <parsed> }` when textarea has new JSON; format selector is disabled.
  - `crates/bodhi/src/components/api-models/form/LlmLibertyEnvelopeInput.test.tsx`: add cases that the new zod validator rejects each missing required leaf.

Verify: `cd crates/bodhi && npm test`.

---

### Phase 9 — E2E: documented offline-only skip
- `crates/lib_bodhiserver/tests-js/specs/api-models/api-llm-liberty-anthropic.spec.mjs`
  - Keep the `test.skip(!process.env.BODHI_E2E_LOCAL, ...)` guard — this is the documented exception per user direction.
  - Add a top-of-describe comment block:
    ```js
    // OFFLINE-ONLY EXCEPTION TO feedback_no_skip_for_missing_env.md:
    // This spec drives a live OAuth flow against Anthropic. Tokens are short-lived
    // (~8h) and the refresh token may be revoked when flagged as third-party usage,
    // so we cannot store a long-lived envelope in CI. Local-only by design.
    // Run with: BODHI_E2E_LOCAL=1 npm run test:playwright -- <filter>
    ```
  - Update the existing `test.skip` reason string to mention the comment block.

Verify: spec is parsed (no syntax break) — the test itself only runs locally.

---

### Phase 10 — Documentation
- `crates/services/CLAUDE.md` — note the `LlmLibertyOauth` sibling-table pattern and the format-immutable-on-edit invariant.
- `crates/routes_app/CLAUDE.md` — note the `provider == "anthropic"` proxy guard and the request-XOR contract.
- `crates/bodhi/src/CLAUDE.md` — note that `api_format` is read-only in edit mode.

---

### Phase 11 — Manual verification (per `feedback_functional_testing.md`)
1. `make app.run.live`
2. Browser:
   - Create flow: paste envelope → fetch models → test connection → save → verify alias appears.
   - Edit flow: open saved alias → verify format selector is disabled with tooltip → change prefix only → save → verify update.
   - Edit flow: re-paste a fresh envelope → save → verify credentials replaced atomically.
   - Negative: malformed envelope (missing `auth.scheme`) → UI rejects with zod error before submit.
   - Run a chat against the alias via `/anthropic/v1/messages`; verify response.
3. 401-retry verification (manual, requires triggering refresh):
   - Wait until token is near `expires_at`, observe `expires_in` honored, no `28800` fallback in logs.
   - Force-stale a token in DB (set `expires_at` to past); next chat triggers refresh-then-retry; verify exactly one refresh per concurrent burst (per-alias mutex).
   - Force-revoke (mock 401 from Anthropic): chat triggers force-refresh + retry; verify success.
4. Inspect DB:
   - `SELECT * FROM api_model_oauth_credentials` — verify `headers_json` / `body_json` are JSONB with `'{}'` defaults, `expires_at` index exists.
5. Confirm the `Anthropic Setup Token` card label still renders (regression).

---

## Critical files (modify)

### Rust — services
- `crates/services/src/db/sea_migrations/m20250101_000021_api_model_oauth_credentials.rs`
- `crates/services/src/models/llm_liberty_credentials_entity.rs`
- `crates/services/src/models/llm_liberty_credentials_repository.rs`
- `crates/services/src/models/llm_liberty_envelope.rs`
- `crates/services/src/models/api_model_service.rs`
- `crates/services/src/models/model_objs.rs`
- `crates/services/src/ai_apis/llm_liberty/refresh.rs`
- `crates/services/src/ai_apis/provider_anthropic_oauth.rs`
- `crates/services/src/ai_apis/provider_shared.rs` (new `with_retry_on_401` helper)
- `crates/services/src/ai_apis/test_ai_api_provider_matrix.rs`
- `crates/services/src/models/test_llm_liberty_credentials_repository.rs`
- `crates/services/src/models/test_api_model_service.rs`

### Rust — objs (for new error variant)
- `crates/objs/src/error/...` (whichever file defines `ObjValidationError`)

### Rust — routes_app
- `crates/routes_app/src/providers/mod.rs`
- `crates/routes_app/src/anthropic/routes_anthropic.rs`
- `crates/routes_app/src/anthropic/test_anthropic_oauth_routing.rs`
- `crates/routes_app/src/anthropic/test_anthropic_messages.rs`
- `crates/routes_app/src/models/api/test_api_models_llm_liberty.rs`
- `crates/routes_app/src/shared/openapi.rs`

### TS — generated
- `ts-client/src/types/types.gen.ts` (regenerated)
- `ts-client/src/openapi-typescript/openapi-schema.ts` (regenerated)
- `openapi.json` (regenerated)

### Frontend
- `crates/bodhi/src/schemas/llmLibertyEnvelope.ts`
- `crates/bodhi/src/components/api-models/form/LlmLibertyEnvelopeInput.tsx`
- `crates/bodhi/src/components/api-models/form/ApiFormatSelector.tsx`
- `crates/bodhi/src/components/api-models/ApiModelForm.tsx`
- `crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts`
- `crates/bodhi/src/components/api-models/ApiModelForm.llm_liberty.test.tsx`
- `crates/bodhi/src/components/api-models/form/LlmLibertyEnvelopeInput.test.tsx`

### E2E
- `crates/lib_bodhiserver/tests-js/specs/api-models/api-llm-liberty-anthropic.spec.mjs`

### Docs
- `crates/services/CLAUDE.md`
- `crates/routes_app/CLAUDE.md`
- `crates/bodhi/src/CLAUDE.md`

---

## Existing utilities to reuse (do NOT reinvent)

- `DbError::ItemNotFound` — `crates/services/src/db/error.rs:52` (canonical "row missing" variant for all repository methods).
- `with_tenant_txn` — already wraps repo calls; PG RLS handled here.
- `mcps/test_mcp_repository_isolation.rs` — canonical cross-tenant isolation test pattern.
- `mcps/mcp_oauth_token_entity.rs` — relation pattern for the entity FK fix.
- `db/sea_migrations/m20250101_000020_api_alias_extra_fields.rs` — `.json_binary()` pattern for the migration fix.
- `routes_app/src/anthropic/routes_anthropic.rs` `into_request_parts` — the conversion shape stays; only the upstream call learns 401 retry.
- `crates/bodhi/src/schemas/apiModel.ts::buildApiModelSchema` — already wires the validator into form-level zod via `.superRefine`; keep contract intact when migrating the validator internals.
- Existing `<ApiKeyInput>` textarea security attrs — copy them verbatim to `LlmLibertyEnvelopeInput`.

---

## Test plan summary

| Layer | New / changed tests |
|---|---|
| services repo | `test_update_credentials_returns_not_found_when_row_missing`, cross-tenant update/delete tests, summary-expires_at value assertion |
| services envelope | new `auth.{in,key,scheme}` validation cases |
| services refresh | matrix extended for `LlmLibertyOauth`, `test_forward_retries_once_after_401_with_refresh`, `test_refresh_rejects_missing_expires_in` |
| services api_model_service | format-immutable test (replaces existing keep-key test), `to/from LlmLiberty` rejection tests |
| routes_app | proxy `LlmLibertyOauth` rows in matrix, `non_anthropic_envelope_rejected`, six `400_on_missing_envelope_field` cases |
| ts-client | regen + diff verification (no manual tests) |
| UI | zod validator rejection-per-leaf, edit-mode action:keep vs action:set, format selector disabled when editing |
| E2E | (no new spec; comment-only) |
| Manual | full live round-trip including 401-induced force-refresh |

---

## Out of scope (intentionally deferred)

Per user directives:
- Rename of `LOGIN_COMMAND` to include `anthropic` argument (review Finding 19).
- Excluding the local spec from `multi_tenant` Playwright project (review Finding 25).

Already deferred by plan:
- Other llm-liberty providers (`openai-codex`, `google-gemini`, `google-antigravity`, `github-copilot`).
- Multi-node refresh coordination, background pre-emptive scheduler, OAuth revoke on alias delete (per original `20260429-liberty-intro.md`).
- The ~10 Nice-to-have findings across all reviews (review reports remain on disk for future reference).
