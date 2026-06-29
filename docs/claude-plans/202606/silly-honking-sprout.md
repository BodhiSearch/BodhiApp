# API Token Enhancement — Per-Resource Grants (App Tokens deferred)

## Context

BodhiApp **API tokens** (`api_tokens` table) are opaque `bodhiapp_<rand>.<client_id>` tokens a user mints for programmatic access. Today they carry only a `scopes` string (`scope_token_user` / `scope_token_power_user`) and a status — **no per-model or per-MCP access, and no delete endpoint**. The list page is already on Screen-V2.

This plan brings API tokens up to a richer permission model: each token carries `(list_models toggle, model inference grant, list_mcps toggle, mcp connect grant, scope)`, those grants are **hard-enforced** on every inference/listing surface, and tokens gain a hard **Delete**. The token-creation and list/detail UX follow the designs (`design/Tokens-API.html`, `design/Tokens-New-API.html`; sources `design/tokens/api-tokens-app.jsx`, `design/tokens/new-app-token-app.jsx`).

**Scope boundary — App Tokens are explicitly OUT of this plan.** The external 3rd-party app flow (`app_access_requests`, `ApprovedResources`/`RequestedResources`, the consent/review screen, OAuth token-exchange, upgrade/supersede) is **untouched** here and will be done as a **follow-up** (the API-token envelope below is intentionally built as its own standalone type so the App side can mirror it later — see decision 4). The existing `ExternalApp` MCP enforcement in `mcps_index` stays as-is.

Not in production → **no backwards-compatibility or data-migration requirement**. The developer stays in the loop: one phase at a time, check in at each phase's gate/commit, then review → verify → push before circling back to App Tokens.

## Locked decisions

1. **Flat model grant** (`All | Specific[ids]` + separate `list_models` toggle). Per-category model *slots* deferred.
2. **Hard-enforce** grants in the request path for `ApiToken` (filtered listings; 403 on a disallowed model in chat/anthropic/gemini/router entry, or disallowed MCP connect). `ExternalApp` path unchanged.
3. **API-token grants are immutable after creation.** `PUT` edits name/status only; changing grants = delete + re-mint. Add a hard **Delete** endpoint.
4. **Standalone, self-contained envelope type** (`TokenGrants`). Even though App Tokens will later get a structurally-similar envelope, we **do not** share code between them — they may diverge. This plan builds only the API-token type.
5. **Listing filter (revised):** for an `ApiToken`, `list_*` OFF ⇒ listings return **only the granted resources** (empty when grant is `Specific{[]}`/`None`; everything when grant is `All`); `list_*` ON ⇒ **all** resources (granted + not). Session/ExternalApp listings are unfiltered. Listing is separate from inference.
6. **"All" means all-including-future** (wildcard), for both models and MCPs.
7. **No `access` field on listings.** Instead, the existing `/bodhi/v1/user` reflection endpoint is extended so a caller (API token) sees its effective grants: `{ scope, models: {list, access_all} | {list, access_all:false, ids:[...]}, mcps: {…} }` (`access_all:true` ⇒ no `ids`). No new endpoint.
8. **`/v1/models*` stays OpenAI-shaped, filtered only** (no `access` field). If convenient we may replace `async_openai::Model` with our own equivalently-shaped struct, but no shape change is required for filtering.

## Envelope design — `crates/services/src/tokens/token_objs.rs`

Versioned with the existing `#[serde(tag = "version")] "1"` hand-written-`Deserialize` style used by `ApprovedResources` (keep that convention). Standalone — not shared with the App flow.

```rust
#[serde(tag = "version")] enum TokenGrants { #[serde(rename="1")] V1(TokenGrantsV1) }
TokenGrantsV1 {
  list_models: bool,
  models:      ModelGrant,   // All | Specific{ ids: Vec<String> }
  list_mcps:   bool,
  mcps:        McpGrant,     // All | None | Specific{ ids: Vec<String> }   // ids = user's own instance ids
}
#[serde(tag="type", rename_all="snake_case")] enum ModelGrant { #[default] All, Specific { ids: Vec<String> } }
#[serde(tag="type", rename_all="snake_case")] enum McpGrant   { #[default] All, None, Specific { ids: Vec<String> } }
```

---

## Phases (each = a vertical, test-gated, independently-committable slice; commit only after its gate is green)

### Phase 1 — Schema + envelope type (services only; zero behavior change)
- **Files:** new migration `crates/services/src/db/sea_migrations/m20250101_0000NN_token_grants.rs` (register in migrator list); `tokens/api_token_entity.rs` (+`grants: String` JSON, +`last_used_at: Option<DateTime<Utc>>`); `tokens/token_objs.rs` (new `TokenGrants` family).
- **Migration:** `grants TEXT NOT NULL DEFAULT '{"version":"1","list_models":true,"models":{"type":"all"},"list_mcps":true,"mcps":{"type":"all"}}'`; `last_used_at` nullable. Mirror the Postgres RLS block from `m20250101_000005_api_tokens.rs`.
- **Tests:** services inline `mod tests` — round-trip every `TokenGrants` variant; missing-field defaults; unknown-version → typed error (mirror existing `ApprovedResources` version test). Extend the dual `#[values("sqlite","postgres")]` migration test for new columns + default backfill.
- **Gate:** `cargo test -p services --lib` green; `cargo run --package xtask openapi` clean. **Chrome:** none.

### Phase 2 — Token CRUD: create-with-grants, Delete, response shape
- **Files:** `token_objs.rs` (`CreateTokenRequest{ name, scope, grants }`; `TokenDetail` +`grants` +`last_used_at`; `UpdateTokenRequest` stays name/status); `token_repository.rs` (+`delete_api_token`; set `grants` explicitly on create); `token_service.rs` (persist grants; `delete_token`); `tokens/auth_scoped.rs` (+`delete_token`); `routes_app/src/tokens/routes_tokens.rs` (grants on create; new `DELETE /bodhi/v1/tokens/{id}`); register route + OpenAPI.
- **Tests:** services `test_token_service.rs` (create persists grants; delete removes row; privilege rules unchanged); `test_token_repository_isolation.rs` (tenant-B cannot get/delete tenant-A token); routes_app `test_tokens_crud.rs` oneshot (POST grants → 201 echoes; DELETE → 204 then GET 404, `.code()`); `test_tokens_auth.rs` (DELETE session-only; bearer → 403).
- **Gate:** `cargo test -p services -p routes_app` green; `make build.ts-client` regenerates types. **Chrome:** optional curl.

### Phase 3 — Plumb grant into `AuthContext` (no enforcement yet)
- **Files:** `auth/auth_context.rs` (`AuthContext::ApiToken` +`grants: TokenGrants`; update all match arms + `test_utils/auth_context.rs::test_api_token()`); `middleware/token_service/token_service.rs` (parse `api_token.grants` → set on `ApiToken`; malformed JSON → typed `InvalidToken`). `last_used_at`: best-effort/throttled write off the hot path; display-only acceptable.
- **Tests:** services `auth_context` inline; routes_app `test_token_service.rs` (opaque path yields parsed grants; malformed → `.code()`; ExternalApp unchanged).
- **Gate:** `cargo check -p services` first (enum-field fan-out), then `cargo test -p services -p routes_app`. **Chrome:** none.

### Phase 4a — Reflection on `/bodhi/v1/user`
- **Files:** `users/users_api_schemas.rs` — extend `TokenInfo` with `models`/`mcps: ResourceAccess { list, access_all, ids? }`; `users/routes_users_info.rs` — build it from `AuthContext::ApiToken.grants`; register `ResourceAccess` in `openapi.rs`; regenerate ts-client.
- **Tests:** routes_app `test_user_info.rs` (api token reflects All → `access_all:true,no ids`; Specific → ids; `None` mcps → empty ids; list flags). Reads grants only, no enforcement.
- **Gate:** `cargo test -p routes_app` + ts-client build green.

### Phase 4 — Hard enforcement: model listing filter + inference (API tokens)
Encapsulated, no per-handler grant logic:
- **Domain (`services/tokens/token_objs.rs`):** `TokenGrantsV1::{allows_model_inference, model_listable, allows_mcp_connect, mcp_listable}` — `ModelGrant`/`McpGrant` matching lives here only.
- **HTTP policy (`routes_app/shared/token_grants.rs`):** `AccessPolicy { Unrestricted | Token(&TokenGrantsV1) }` resolved via `AuthScope::access_policy()`; ops `model_listable(id)`, `ensure_model_inference(id) -> Result<(),TokenGrantError>`, mcp equivalents. `Unrestricted` (session/external-app) passes everything. `TokenGrantError::{ModelForbidden,McpForbidden}` (Forbidden) → 403 in any envelope via `From<AppError>`.
- **Handlers (uniform calls):** `oai/routes_oai_models.rs` (list `retain(model_listable)`; single GET 404 if `!model_listable`), `oai/routes_oai_chat.rs` (`ensure_model_inference?` — entry alias for router), `models/routes_models.rs` (`models_index`: filter inner `ApiModel`s, drop aliases with none left), `anthropic/*` + `gemini` (`ensure_model_inference?`). May swap `async_openai::Model` for our own struct if convenient (not required).
- **Tests:** unit on `AccessPolicy` (Unrestricted passes; Token all/specific/none × list on/off); thin per-surface oneshot (granted 200 / denied 403·404; list filter; Session + ExternalApp unaffected). server_app `#[serial(live)]`: one real-HTTP 403 for a scoped api token.
- **Gate:** `cargo test -p routes_app -p server_app` green. **Chrome:** `make app.run.live` — filtered `/v1/models`; disallowed model → 403.

### Phase 5 — Hard enforcement: MCP listing + connect/invoke (API tokens)
- **Files:** `mcps/routes_mcps.rs` `mcps_index` (+`ApiToken{grants}` arm: `list_mcps` ON → all instances, OFF → only granted by `McpGrant`; `None` → empty; parallel to the existing ExternalApp arm); MCP connect/invoke routes (403 if target not granted). Enumerate all MCP action endpoints so none is unguarded. No `access` field — reflection already covers introspection (Phase 4a).
- **Tests:** routes_app oneshot (api token `Specific` filters; `None` → empty; connect ungranted → 403 `.code()`; ExternalApp parity retained); assert cross-tenant/user instance ids never leak via a grant.
- **Gate:** `cargo test -p routes_app` green. **Chrome:** `make app.run.live` MCP list + disallowed connect → 403.

### Phase 6 — Frontend: API-token grant UI + Delete (Screen-V2)
- **Files:** `routes/tokens/new/index.tsx` + `TokenForm`/`TokenDialog` (model Specific via reused `components/ModelSelector.tsx`; MCP multiselect via `hooks/mcps/useMcpInstances`; all/specific radios + `list_*` Switches, per `design/tokens/new-app-token-app.jsx`); `routes/tokens/index.tsx` (`TokenDetailPanel` grant summary + Delete with confirm, per `api-tokens-app.jsx`); `hooks/tokens/` (+`useDeleteToken`); `src/schemas/` (zod + `convertFormToApi`/`convertApiToForm`); `test-utils/msw-v2/handlers/tokens.ts` (+`mockDeleteToken`, grant fixtures; sub-path-before-wildcard order).
- **Tests:** Vitest+MSW (form submits correct `TokenGrants`; detail renders grants; delete invalidates list; `data-pagestatus`); grow single E2E `specs/tokens/api-tokens.spec.mjs` with `test.step`s (create w/ specific-model grant → verify → delete); update `pages/TokensPage.mjs` + `tokenFixtures.mjs`. Black-box; `reducedMotion:'reduce'`.
- **Gate:** `cd crates/bodhi && npm test` + E2E green. **Chrome:** `make app.run.live` create→view→delete with grants.

## Cross-cutting gates (every phase)
Upstream→downstream order; `cargo test -p <crate>` then `make test.backend` after Rust; `cargo run --package xtask openapi` + `make build.ts-client` after any API change; a multi-tenant isolation test for the new tenant-scoped column; never `Utc::now()` (use `TimeService`/`FrozenTimeService`); E2E black-box only; commit per phase after gate is green.

## Verification (end-to-end)
1. `make app.run.live`, mint an API token granting one specific model + one MCP with listing off → `/v1/models` hides others, chat to granted model works, chat to disallowed model is 403, MCP connect to disallowed instance is 403.
2. Delete the token from the rail → it disappears and its bearer use 401s.
3. `make test.backend`, `cd crates/bodhi && npm test`, and the grown token E2E spec all green.

## Follow-up (separate plan, after this ships)
App Tokens: evolve `ApprovedResources`/`RequestedResources` (mirroring `TokenGrants`, duplicated), rebuild the consent/review screen with model + listing grants, and add the upgrade/token-exchange supersede flow + `ExternalApp` model enforcement.

## Critical files
- `crates/services/src/tokens/{token_objs.rs, api_token_entity.rs, token_repository.rs}` + `db/sea_migrations/`
- `crates/services/src/auth/auth_context.rs`
- `crates/routes_app/src/middleware/token_service/token_service.rs`
- `crates/routes_app/src/{tokens/routes_tokens.rs, oai/routes_oai_models.rs, oai/routes_oai_chat.rs, models/routes_models.rs, mcps/routes_mcps.rs}`
- `crates/bodhi/src/routes/tokens/{index.tsx, new/index.tsx}`, `hooks/tokens/`, `src/schemas/`, `test-utils/msw-v2/handlers/tokens.ts`
- `crates/lib_bodhiserver/tests-js/{pages/TokensPage.mjs, specs/tokens/api-tokens.spec.mjs, fixtures}`
