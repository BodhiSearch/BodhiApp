# API & App Token Enhancement — Permission Parity

## Context

Today BodhiApp has two token-ish concepts that are wired very differently:

- **API tokens** (`api_tokens` table): opaque `bodhiapp_<rand>.<client_id>` tokens a user mints for programmatic access. They carry a single `scopes` string (`scope_token_user` / `scope_token_power_user`), a status, and **nothing else** — no per-model or per-MCP access, no delete endpoint. The list page is already on Screen-V2.
- **App / external-app tokens**: 3rd-party apps obtain access through an OAuth2 token-exchange flow (Keycloak) gated by an approval workflow (`app_access_requests` table, `ApprovedResources::V1` versioned envelope, `approved_role`). The approved envelope carries **MCPs only**, and only `mcps_index` enforces it.

We want to **bring API tokens on par with App tokens** by giving both a permission model of the same *shape* — `(list_models toggle, model inference grant, list_mcps toggle, mcp connect grant, scope)` — and to **enforce those grants** on every inference/listing surface for both token kinds. The designs are the source of truth for UX: `design/Tokens-API.html`, `Tokens-App.html`, `Tokens-New-API.html`, `App-Access-Review.html` (and the `design/tokens/*.jsx` / `design/users/access-request-app.jsx` sources).

Not in production → **no backwards-compatibility or data-migration requirement**; we change schemas directly.

## Locked decisions

1. **Flat model grant now** (`All | Specific[ids]` + separate `list_models` toggle). Per-category model *slots* (Main Reasoning / Visual / Embedding) are **deferred** to a later effort.
2. **Full App Access Review (consent) rebuild**, including the **upgrade / token-exchange diff mode** (previously-granted vs new) and the **supersede chain** (re-approval mints a new token, marks the old `exchanged`).
3. **Hard-enforce** grants in the request path **now**, for **both** `ApiToken` and `ExternalApp`: filtered listings, 403 on a disallowed model (chat/anthropic/gemini/router entry) or MCP connect.
4. **Similar but duplicated — two stores, no shared code.** API tokens and app access requests get **parallel** versioned envelope types that *look* alike but are separate. They may diverge; do not factor a single shared grant type. (Explicit override of the usual DRY instinct.)
5. **API-token grants are immutable after creation** (parallel to App-token immutability). `PUT` edits name/status only; changing grants = delete + re-mint. API tokens gain a hard **Delete**; App tokens gain **exchange/supersede**.
6. **Listing toggle off ⇒ empty list** (discovery hidden), *not* 403 — inference on an individually-granted model still succeeds. Listing is separate from inference/connect.
7. **"All" means all-including-future** (wildcard), for both models and MCPs.

## Envelope design

Both envelopes are versioned with the existing `#[serde(tag = "version")] "1"` hand-written-`Deserialize` style already used by `ApprovedResources` (keep that convention; do **not** switch to `{v1:{…}}`).

### App access-request side (3rd-party flow) — `crates/services/src/app_access_requests/access_request_objs.rs`

The app declares **generic intent**; the owner grants concretely at consent. A requested item the owner leaves unchecked = declined.

```rust
// REQUESTED — what the app asks for (generic, not concrete model ids)
RequestedResourcesV1 {
  list_models:  bool,                      // app wants GET /v1/models → owner sees a grant toggle only if true
  list_mcps:    bool,                      // app wants GET /v1/mcps
  models:       bool,                      // generic "needs model access" flag → owner picks All|Specific at consent
  mcp_servers:  Vec<RequestedMcpServer>,   // specific MCPs requested by url (existing)
  mcps_open:    bool,                      // open MCP ask → consent shows a multiselect; owner grants any instances
}

// APPROVED — what the owner grants (concrete)
ApprovedResourcesV1 {
  list_models:  bool,                      // grantable only if requested.list_models
  list_mcps:    bool,
  models:       ModelApproval,             // All | Specific(Vec<ModelGrantItem{ alias_id, status }>)
  mcps:         McpApprovalSet,            // All | Specific(Vec<McpApproval{ url, status, instance? }>)  (instance binds proxy path)
}
```

- "List all…" toggles render in the consent UI **only when the app requested that listing** (per your call). Owner-added/open MCPs and the model grant are the owner's choice (additive); each requested resource is a checkbox the owner can grant or decline (mirrors the slot-based MCP interaction).
- `All` = wildcard incl. future resources.

### API-token side (user-minted) — `crates/services/src/tokens/token_objs.rs`

No "requested" half (the user mints directly). **Separate, duplicated** type:

```rust
#[serde(tag = "version")] enum TokenGrants { #[serde(rename="1")] V1(TokenGrantsV1) }
TokenGrantsV1 {
  list_models: bool,
  models:      ModelGrant,   // All | Specific{ ids: Vec<String> }
  list_mcps:   bool,
  mcps:        McpGrant,     // All | None | Specific{ ids: Vec<String> }   // MCP ids = user's own instance ids
}
```

API-token MCP grant has an explicit `None` and no instance-binding (the user already owns the instances), so it deliberately diverges from the app side — duplication is intended.

---

## Phases (each = a vertical, test-gated, independently-committable slice; commit only after its gate is green)

### Phase 1 — Schema + parallel envelope types (services only; zero behavior change)
- **Files:** new migration `crates/services/src/db/sea_migrations/m20250101_0000NN_token_grants.rs` (register in migrator list); `tokens/api_token_entity.rs` (+`grants: String` JSON, +`last_used_at: Option<DateTime<Utc>>`); `tokens/token_objs.rs` (new `TokenGrants` family); `app_access_requests/access_request_objs.rs` (evolve `RequestedResourcesV1`/`ApprovedResourcesV1` as above — duplicated, not shared with `TokenGrants`).
- **Migration:** `grants TEXT NOT NULL DEFAULT '{"version":"1","list_models":true,"models":{"type":"all"},"list_mcps":true,"mcps":{"type":"all"}}'`; `last_used_at` nullable. Mirror the Postgres RLS block from `m20250101_000005_api_tokens.rs`.
- **Tests:** services inline `mod tests` — round-trip every `TokenGrants` / `ApprovedResources` variant; missing-field defaults; unknown-version → typed error (mirror existing `ApprovedResources` version test); old MCP-only approved JSON still deserializes. Extend the dual `#[values("sqlite","postgres")]` migration test to assert new columns + default backfill.
- **Gate:** `cargo test -p services --lib` green; `cargo run --package xtask openapi` clean. **Chrome:** none.

### Phase 2 — Token CRUD: create-with-grants, Delete, response shape
- **Files:** `token_objs.rs` (`CreateTokenRequest{ name, scope, grants }`; `TokenDetail` +`grants` +`last_used_at`; `UpdateTokenRequest` unchanged — name/status only, grants immutable); `token_repository.rs` (+`delete_api_token`; set `grants` explicitly on create); `token_service.rs` (persist grants on create; `delete_token`); `tokens/auth_scoped.rs` (+`delete_token`); `routes_app/src/tokens/routes_tokens.rs` (accept grants on create; new `DELETE /bodhi/v1/tokens/{id}` `tokens_delete`); register route + OpenAPI in `routes.rs` / `shared/openapi.rs`.
- **Tests:** services `test_token_service.rs` (create persists grants; delete removes row; privilege rules unchanged); `test_token_repository_isolation.rs` (tenant-B cannot get/delete tenant-A token); routes_app `test_tokens_crud.rs` oneshot (POST w/ grants → 201 echoes; DELETE → 204 then GET 404, `.code()` assertions); `test_tokens_auth.rs` (DELETE stays session-only; bearer → 403).
- **Gate:** `cargo test -p services -p routes_app` green; `make build.ts-client` regenerates `TokenGrants`/`CreateTokenRequest`. **Chrome:** optional curl via `make app.run.live`.

### Phase 3 — Plumb grant into `AuthContext` (no enforcement yet)
- **Files:** `crates/services/src/auth/auth_context.rs` (`AuthContext::ApiToken` +`grants: TokenGrants`; update all match arms + `test_utils/auth_context.rs::test_api_token()`); `middleware/token_service/token_service.rs` (parse `api_token.grants` → set on `ApiToken`; malformed JSON → typed `InvalidToken`).
- **`last_used_at`:** best-effort/throttled write on validate — keep off the hot path; acceptable to land display-only ("Never"/timestamp) and refine later.
- **Tests:** services `auth_context` inline (constructor w/ grants; `app_role()` unaffected); routes_app `test_token_service.rs` (opaque path yields parsed grants; malformed → `.code()`; ExternalApp path unchanged).
- **Gate:** `cargo check -p services` first (enum-field fan-out), then `cargo test -p services -p routes_app`. **Chrome:** none.

### Phase 4 — Hard enforcement: model listing + inference (both token kinds)
- **Files:** new `routes_app/src/.../grant_filter.rs` with **two duplicated resolvers** (ApiToken reads `grants` inline; ExternalApp re-fetches approved resources, mirroring `mcps_index:32-63`) — no shared type. Apply in `oai/routes_oai_models.rs` (filter `list_aliases` by grant; empty when `list_models` off; single-model → 404/403), `oai/routes_oai_chat.rs` (403 `OaiApiError` after `find_alias` if entry alias not granted), `models/routes_models.rs` (`models_index` filter), `anthropic/*` + `gemini` handlers (provider-shaped 403 at alias resolution). Enforce at the **entry alias only** for model-router (targets are internal).
- **Tests:** routes_app oneshot, parameterized over api-token + external-app: `Specific` returns only granted; `list_models:false` → empty; granted alias chat → 200, ungranted → 403 `.code()`; anthropic/gemini/router-entry equivalents; Session unfiltered (no regression). server_app `#[serial(live)]`: one real-HTTP 403 for a scoped api token (security boundary).
- **Gate:** `cargo test -p routes_app -p server_app` green. **Chrome:** `make app.run.live` — `/v1/models` with a scoped token shows filtered list; disallowed model → 403.

### Phase 5 — Hard enforcement: MCP listing + connect/invoke (both token kinds)
- **Files:** `mcps/routes_mcps.rs` `mcps_index` (+`ApiToken{grants}` arm filtering the user's instances by `McpGrant` + `list_mcps` toggle, parallel to the ExternalApp arm); MCP connect/invoke routes under `crates/routes_app/src/mcps/` (403 if target instance not granted, for both kinds). Enumerate all MCP action endpoints so none is unguarded.
- **Tests:** routes_app oneshot (api token `Specific` filters; `None` → empty; connect to ungranted → 403 `.code()`; ExternalApp parity retained); assert cross-tenant/user instance ids never leak via a grant.
- **Gate:** `cargo test -p routes_app` green. **Chrome:** `make app.run.live` — list MCPs with scoped token; disallowed connect → 403.

### Phase 6 — Frontend: API-token grant UI + Delete (Screen-V2)
- **Files:** `routes/tokens/new/index.tsx` + its `TokenForm`/`TokenDialog` (model `Specific` via reused `components/ModelSelector.tsx`; MCP multiselect via `hooks/mcps/useMcpInstances`; all/specific radios + `list_*` `Switch` toggles, per `design/tokens/new-app-token-app.jsx`); `routes/tokens/index.tsx` (`TokenDetailPanel` grant summary + Delete in row/rail with confirm, per `api-tokens-app.jsx`); `hooks/tokens/` (+`useDeleteToken`); `src/schemas/` (zod + `convertFormToApi`/`convertApiToForm` for grants); `test-utils/msw-v2/handlers/tokens.ts` (+`mockDeleteToken`, grant fixtures; sub-path-before-wildcard order).
- **Tests:** Vitest+MSW (form submits correct `TokenGrants` JSON for all vs specific + toggles; detail renders grants; delete invalidates list; `data-pagestatus`); grow the single E2E `specs/tokens/api-tokens.spec.mjs` with `test.step`s (create w/ specific-model grant → verify → delete), update `pages/TokensPage.mjs` + `tokenFixtures.mjs`. Black-box only; `reducedMotion:'reduce'` for the rail.
- **Gate:** `cd crates/bodhi && npm test` + E2E green. **Chrome:** `make app.run.live` create→view→delete with grants.

### Phase 7 — Consent / App Access Review rebuild (models + listing flags)
- **Files:** backend mostly from Phase 1 — verify `apps_approve_access_request` / `approve_request` pass through the evolved `ApprovedResourcesV1`; extend the MCP `instance required` validation loop (`routes_apps.rs:~307`) with model-approval validation. Frontend: rebuild the consent screen (locate the `request-access`/`review` route under `crates/bodhi/src/routes/`) per `design/users/access-request-app.jsx` + `App-Access-Review.html` — model multiselect, MCP list (specific + open-request multiselect), `list_*` toggles shown only when requested, role selector (`UserScope`); each requested resource is a grant/decline checkbox. Fixtures in `src/test-fixtures/` (access-requests, apps).
- **Tests:** routes_app oneshot (approve w/ models+listing persists evolved JSON; version-mismatch + privilege-escalation guards hold, `.code()`); Vitest+MSW (renders requested resources; approve posts correct `ApprovedResources` V1; deny path; old approved request still renders); E2E approve flow granting a model + MCP.
- **Gate:** `cargo test -p routes_app` + `npm test` + E2E green. **Chrome:** `make app.run.live` consent walkthrough.

### Phase 8 — Upgrade / token-exchange diff mode + supersede chain
- **Files:** migration — `app_access_requests` +`supersedes_id` (nullable) + `AppAccessRequestStatus::Exchanged`; `access_request_service.rs` (re-approval creates a new row referencing the prior approved request and marks the old `Exchanged`, transactionally — today only `Draft→Approved`); `app_access_request_entity.rs` + repository; `routes_apps.rs` (review returns prior-approved snapshot for the diff; approve handles supersede); `token_service.rs` exchange path — **a superseded request's token is invalidated** (reject on validate; accept up-to-300s cache lag). Frontend consent diff mode (previously-granted vs new), per upgrade mode in `access-request-app.jsx`.
- **Tests:** services (approve→re-approve creates supersede chain; old row `Exchanged`; isolation; `FrozenTimeService`); routes_app oneshot (review returns prior grant; approve sets `supersedes_id`; superseded token validation rejected, `.code()`); Vitest+E2E (diff rendering; approve-upgrade `test.step`).
- **Gate:** full `make test.backend` + `npm test` + E2E green. **Chrome:** upgrade-consent walkthrough.

## Cross-cutting gates (every phase)
Upstream→downstream order; `cargo test -p <crate>` as you go then `make test.backend` after Rust; `cargo run --package xtask openapi` + `make build.ts-client` after any API change; a multi-tenant isolation test for every new tenant-scoped column (pattern: `services/.../test_mcp_repository_isolation.rs`); never `Utc::now()` (use `TimeService`/`FrozenTimeService`); E2E black-box only; commit per phase after gate is green.

## Verification (end-to-end)
1. `make app.run.live`, mint an API token granting one specific model + one MCP with listing off → confirm `/v1/models` hides others, chat to the granted model works, chat to a disallowed model is 403, MCP connect to a disallowed instance is 403.
2. Delete the token from the rail → it disappears and its bearer use 401s.
3. Drive the 3rd-party consent flow (new request) granting a model + MCP, then re-approve to exercise the exchange/supersede diff; confirm the old token stops working and the new one carries the grants.
4. `make test.backend`, `cd crates/bodhi && npm test`, and the grown token + consent E2E specs all green.

## Critical files
- `crates/services/src/tokens/{token_objs.rs, api_token_entity.rs, token_repository.rs}` + `db/sea_migrations/`
- `crates/services/src/app_access_requests/access_request_objs.rs` + `access_request_service.rs`
- `crates/services/src/auth/auth_context.rs`
- `crates/routes_app/src/middleware/token_service/token_service.rs`, `middleware/apis/api_middleware.rs`
- `crates/routes_app/src/{tokens/routes_tokens.rs, oai/routes_oai_models.rs, oai/routes_oai_chat.rs, models/routes_models.rs, mcps/routes_mcps.rs, apps/routes_apps.rs}`
- `crates/bodhi/src/routes/tokens/{index.tsx, new/index.tsx}`, `hooks/tokens/`, the consent/`request-access` route, `src/schemas/`, `test-utils/msw-v2/handlers/tokens.ts`
- `crates/lib_bodhiserver/tests-js/{pages/TokensPage.mjs, specs/tokens/api-tokens.spec.mjs, fixtures}`
