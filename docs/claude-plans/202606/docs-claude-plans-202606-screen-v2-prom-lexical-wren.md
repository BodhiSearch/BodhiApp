# App Token Enhancement — Per-Resource Grants (parity with API Tokens, lighter convergence)

## Context

API Tokens were recently given a full per-resource permission model (`feat(tokens): Phase 1..6b`): a versioned grant envelope (`TokenGrantsV1`) carrying listing toggles + model/MCP grants, hard-enforced on every request surface through one `AccessPolicy` abstraction, introspectable via `/bodhi/v1/user`. App Tokens — the external 3rd-party-app flow (`AuthContext::ExternalApp`, Keycloak token-exchange, `app_access_requests`) — predate that effort and only carry **MCP** grants (by url, instance-bound). They get **automatic access to all models** with no way to restrict.

This effort brings App Tokens to the same model: an app **requests** capability flags, the **owner decides** the actual grant at consent (list-all toggles + model All/Specific/none + MCP All/Specific incl. owner-granted extras), grants are **hard-enforced** identically to API tokens, **introspectable**, and the consent + management UX matches the designs.

**Locked decisions (from interview):**
1. **Token-exchange / upgrade-supersede is OUT.** App tokens stay immutable; consent is single "new request" mode (no upgrade-diff, no `exchanged` status, no supersede chain). To change grants: revoke + re-request.
2. **Lighter convergence.** Keep `ApprovedResources` and `TokenGrants` as **parallel** envelope types; unify *behavior* behind a shared `ResourceGrants` predicate trait. Reuse the existing `ModelGrant` / `McpGrant` enums as shared primitives. The app-specific **lifecycle** (Keycloak exchange, access-request approval record, `access_request_auth_middleware`) stays separate — that's legitimately divergent storage/lifecycle.
3. **App requests models too — as UI-driver flags only.** The requested envelope gains `models_list`, `models_access`, `mcps_list`, `mcps_access` (booleans that tell the consent screen which controls to render). Existing `mcp_servers` (url-requested MCPs) stays unchanged. The **owner** picks the actual grant (All / Specific / empty=none). Flat model list — per-category slots are OUT.
4. **Frontend: consent/review rebuild + standalone App-Tokens management screen.**

**Build directly on the just-landed API-Token frontend redesign** (`feat(tokens): Phase 0..4`, `f1265209..HEAD`; plan `docs/claude-plans/202606/the-api-token-screen-mellow-porcupine.md`). That work deliberately created shared building blocks for *this* effort:
- **`McpGrant::None` is gone** — `McpGrant` now mirrors `ModelGrant`: `All | Specific { ids }`, empty `ids` ⇒ no access. `ResourceAccess::None` was removed too. The plan below uses the post-removal shapes everywhere.
- **Shared, reusable picker components** at `crates/bodhi/src/components/access-picker/`: `AccessPicker` (All/Specific radio + selected-rows + slide-in panel), `AccessPickerPanel` (search/type-filter/grouped list on shadcn `Sheet`), `ListingToggle` (the "List all …" `/v1/models` permission row). Generic over a resource via `AccessItem { id, label, type?, meta? }` + `mode: 'all'|'specific'` + `selectedIds`/`onToggle`. The consent rebuild and the App-Tokens screen **reuse these as-is** — do not fork.
- **Catalog-table + URL-driven detail-rail list pattern** (redesigned `tokens/index.tsx`: semantic table, ↑/↓ keyboard nav via `useListKeyNav`, auto-open rail on select, browser back/forward) — the App-Tokens management list follows the same pattern. New-form theme: 4 titled sections + sticky footer + role cards + scoped CSS. E2E page-object/spec patterns: `tests-js/pages/TokensPage.mjs`, `specs/tokens/api-tokens.spec.mjs`.

## Design

### Shared predicate trait (the unification seam)
The model/MCP listing+inference logic on `TokenGrantsV1` (`services/src/tokens/token_objs.rs:70-99`) is lifted into a trait so both principals answer the same questions:

```rust
// services — new shared module (e.g. services/src/grants/grant_objs.rs), re-exported
pub trait ResourceGrants {
  fn allows_model_inference(&self, model_id: &str) -> bool;
  fn model_listable(&self, model_id: &str) -> bool;
  fn allows_mcp_connect(&self, mcp_id: &str) -> bool;
  fn mcp_listable(&self, mcp_id: &str) -> bool;
}
```
`ModelGrant` and `McpGrant` are now **symmetric** (post `f1265209`): both `All | Specific { ids }`, empty `ids` ⇒ no access. Move them here as shared primitives (re-exported from `tokens` so existing imports keep working). `TokenGrantsV1` impls the trait (logic unchanged). `ApprovedResourcesV1` impls it too.

### Envelope extensions (parallel types, `#[serde(default)]` on all new fields)
`services/src/app_access_requests/access_request_objs.rs`:

```rust
RequestedResourcesV1 {        // app-sent UI-driver flags + existing url requests
  models_list:  bool,         // render "list all models" checkbox
  models_access: bool,        // render model All/Specific selector
  mcps_list:    bool,         // render "list all MCPs" checkbox
  mcps_access:  bool,         // render MCP All/Specific (owner-extra) selector
  mcp_servers:  Vec<RequestedMcpServer>,  // UNCHANGED — url-requested MCPs
}

ApprovedResourcesV1 {         // owner's decision
  list_models: bool,
  models:      ModelGrant,    // All | Specific{ids}  (empty Specific = no models)
  list_mcps:   bool,
  mcps:        Vec<McpApproval>,  // UNCHANGED — url-tied instance approvals
  mcps_extra:  McpGrant,      // All | Specific{ids} — owner-granted beyond requested urls (empty = none)
}
```
`ResourceGrants for ApprovedResourcesV1`: model side mirrors `TokenGrantsV1`; `allows_mcp_connect(id)` = any url-tied approved `McpApproval.instance.id == id` **OR** `mcps_extra` allows it; `mcp_listable` = `list_mcps || allows_mcp_connect`.

### Plumbing grants into the request path (the minimum to enforce models everywhere)
This is required: model enforcement at OAI/Anthropic/Gemini/models-list goes through `AccessPolicy`, which is sync and has no DB — so the approved grant must ride on the `AuthContext`.

- Add `grants: ApprovedResourcesV1` to `AuthContext::ExternalApp` (`services/src/auth/auth_context.rs`) + `test_external_app` factory.
- Load it at token-exchange (`routes_app/src/middleware/token_service/token_service.rs` `handle_external_client_token`) from the approved access-request row; cache it in `CachedExchangeResult`. Empty/default when no access request bound.
- Refactor `AccessPolicy` (`routes_app/src/shared/token_grants.rs`) to a 2-arm enum `Unrestricted | Grants(&dyn ResourceGrants)`; `of()` returns `Grants(...)` for **both** `ApiToken` and `ExternalApp`. All existing call-sites (OAI chat `:134`, OAI models `:90`, models list `:146`, anthropic `:148`, gemini `:213`, mcp_proxy `:70`, mcps list `:66`) then enforce app grants with **zero new call-sites**.
- Delete the bespoke ExternalApp MCP re-fetch in `mcps_index` (`routes_app/src/mcps/routes_mcps.rs:32-63`) — now redundant (AccessPolicy.mcp_listable covers it). `access_request_auth_middleware` keeps its **lifecycle** validation (status=Approved, azp/sub match); its **resource** validation is superseded by `ensure_mcp_connect` at `mcp_proxy.rs:70`.

### Reflection
Extend the `ExternalApp` arm of `/bodhi/v1/user` (`routes_app/src/users/routes_users_info.rs:134`) to report approved model/MCP access using the existing `ResourceAccess` shape (`users/users_api_schemas.rs` — now just `All { list } | Specific { list, ids }`), built from the resolved app grant. `ResourceAccess::models/mcps` currently take `&TokenGrantsV1`; add app-grant constructors (or generalize over `ResourceGrants`).

### Management lifecycle
Add a `Revoked` status (`AppAccessRequestStatus`) and `revoke_request` (Approved→Revoked) in `access_request_service`. Token-exchange already requires status=Approved, so revoke stops the token. Best-effort Keycloak consent revocation (note if upstream API unavailable).

## Phases (vertical slices, upstream→downstream, commit per phase once gates green)

**Phase 1 — Shared primitives + envelope extension (services).**
Extract `ModelGrant`/`McpGrant` + `ResourceGrants` trait to shared module; impl for `TokenGrantsV1` (logic unchanged). Extend `RequestedResourcesV1` + `ApprovedResourcesV1`; impl trait for the latter.
*Tests:* services unit — predicate matrix for `ApprovedResourcesV1` (model All/Specific/empty; url-tied + extra MCP union; list toggles), serde versioned round-trip, old-JSON-still-deserializes via defaults.
*Gate:* `cargo test -p services`.

**Phase 2 — AuthContext plumbing + AccessPolicy unification (services + routes_app).**
Add `grants` to `AuthContext::ExternalApp` + factory. Load+cache at token-exchange. Refactor `AccessPolicy` to `Grants(&dyn ResourceGrants)`; `of()` covers both token kinds.
*Tests:* routes_app unit — `AccessPolicy` over both `TokenGrantsV1` and `ApprovedResourcesV1`; token_service exchange test populating grants from an approved request.
*Gate:* `cargo test -p services -p routes_app`.

**Phase 3 — Enforce model + MCP grants for ExternalApp (routes_app).**
Delete bespoke `mcps_index` re-fetch; trim resource-validation from `access_request_auth_middleware` (keep lifecycle).
*Tests:* routes_app oneshot — ExternalApp: granted model infers / disallowed model 403 (OAI + anthropic + gemini); model-list filter honors `list_models`; mcp-list filter honors `list_mcps`; disallowed mcp connect 403; owner-extra mcp connects.
*Gate:* `cargo test -p routes_app`.

**Phase 4 — Reflection (routes_app).**
Extend ExternalApp `/bodhi/v1/user` arm with `ResourceAccess` model/mcp report.
*Tests:* oneshot reflection assertions.
*Gate:* `cargo test -p routes_app`; regenerate OpenAPI + ts-client.

**Phase 5 — Consent backend: review + approve (routes_app + services).**
Review GET returns the requested flags + candidate models (for the owner's Specific picker) alongside existing `mcps_info`. Approve handler accepts extended `ApprovedResources` and validates: model ids exist/owned, `mcps_extra` instances owned+enabled (mirror existing url-tied check `routes_apps.rs:306-335`), role caps unchanged.
*Tests:* oneshot — approve with model Specific + list toggles + owner-extra MCP; validation rejects unowned model/mcp.
*Gate:* `cargo test -p routes_app`; OpenAPI + ts-client regen.

**Phase 6 — Management backend (routes_app + services).**
`GET /bodhi/v1/apps` (session) — list caller's approved app requests with grant summary. Revoke endpoint (Approved→Revoked) + `revoke_request` service method + `Revoked` status.
*Tests:* services unit + multi-tenant isolation (reference `mcps/test_mcp_repository_isolation.rs`); oneshot list + revoke + post-revoke token rejected.
*Gate:* `make test.backend`; OpenAPI + ts-client regen.

**Phase 7 — Frontend: consent/review rebuild (bodhi).**
Rebuild review screen (`routes/apps/access-requests/review/`) per `design/users/access-request-app.jsx` (new-request mode). **Reuse the shared `access-picker` components** (`crates/bodhi/src/components/access-picker/`): conditionally render `ListingToggle` for list-models (`models_list`), `AccessPicker` for model All/Specific (`models_access`, feed `AccessItem[]` from the owner's models), `ListingToggle` for list-mcps (`mcps_list`), `AccessPicker` for owner-extra MCP All/Specific (`mcps_access`) — plus the existing url-MCP approval (`McpServerCard`). Map owner selections → extended `ApprovedResources` approve payload. Consume generated ts-client types only; reuse the `tokens/new` 4-section + sticky-footer + role-card theme.
*Tests:* vitest + MSW for each conditional control (driven by requested flags) + approve payload shape. Chrome verify against `make app.run.live` (rebuilt dev-server, per `feedback_gateb_rebuild_binary_for_backend_batches`).
*Gate:* `cd crates/bodhi && npm test`.

**Phase 8 — Frontend: App-Tokens management screen (bodhi).**
New list/detail screen per `design/Tokens-App.html` + `design/tokens/app-tokens-app.jsx`, following the **redesigned API-token list pattern** (`tokens/index.tsx`): semantic catalog table + URL-driven detail rail, `useListKeyNav` ↑/↓ nav, auto-open rail, `EmptyState`. Rail grant-summary reuses `ListingToggle`/`AccessPicker` in a read-only (`disabled`) presentation for the "List all / Inference / Connect" sections; revoke with confirm. New hooks under `src/hooks/apps/` (list approved apps, revoke) + fixtures in `src/test-fixtures/apps.ts`.
*Tests:* vitest + MSW (list, summary render, revoke). Chrome verify.
*Gate:* `cd crates/bodhi && npm test`.

**Phase 9 — E2E (lib_bodhiserver/tests-js).**
Grow ONE black-box Playwright spec (many `test.step`), reusing the `TokensPage.mjs` / `api-tokens.spec.mjs` page-object + access-picker selector patterns: app requests access with model+mcp flags → owner reviews, grants one specific model + one specific MCP + list toggles → app token: granted model infers, other model 403; granted MCP connects, other 403; `/v1/models` & `/v1/mcps` reflect toggles → management screen shows the grant → revoke → token stops working. UI-only interactions (no `page.evaluate`/context fetch); `reducedMotion:'reduce'`; throw in `beforeAll` on missing env.
*Gate:* `make test.e2e`.

## Critical files
- `crates/services/src/tokens/token_objs.rs` — source of `ModelGrant`/`McpGrant`/predicates to extract.
- `crates/services/src/grants/` *(new)* — shared `ResourceGrants` trait + grant primitives.
- `crates/services/src/app_access_requests/access_request_objs.rs` — envelope extensions + trait impl.
- `crates/services/src/app_access_requests/access_request_service.rs` — `revoke_request`, list-for-user.
- `crates/services/src/auth/auth_context.rs` — `ExternalApp.grants`.
- `crates/routes_app/src/shared/token_grants.rs` — `AccessPolicy` trait-object refactor.
- `crates/routes_app/src/middleware/token_service/token_service.rs` — load+cache grants at exchange.
- `crates/routes_app/src/mcps/routes_mcps.rs` + `mcps/mcp_proxy.rs` + `middleware/access_requests/*` — drop bespoke MCP filter, keep lifecycle.
- `crates/routes_app/src/apps/routes_apps.rs` — review/approve extensions, list + revoke endpoints.
- `crates/routes_app/src/users/routes_users_info.rs` + `users/users_api_schemas.rs` — ExternalApp reflection.
- `crates/bodhi/src/components/access-picker/` — **reuse** `AccessPicker`/`AccessPickerPanel`/`ListingToggle` (built for this).
- `crates/bodhi/src/routes/apps/access-requests/review/` — consent rebuild.
- `crates/bodhi/src/routes/.../app-tokens` *(new)* — management screen (mirror `tokens/index.tsx` catalog+rail).
- `crates/bodhi/src/hooks/apps/` + `test-fixtures/apps.ts` — list/revoke hooks + fixtures.
- `crates/lib_bodhiserver/tests-js/pages/TokensPage.mjs`, `specs/tokens/api-tokens.spec.mjs` — E2E patterns to mirror.

## Verification (end-to-end)
1. Backend gates per phase (`cargo test -p ...`, `make test.backend` after Phase 6).
2. After Phase 4/5/6: `cargo run --package xtask openapi && make build.ts-client` — confirm clean diff, frontend builds.
3. Manual (Chrome via `make app.run.live`, per `feedback_gateb_rebuild_binary_for_backend_batches` — ensure dev-server rebuilt so new query params/flags are live): drive an external app request → review screen renders only requested controls → grant a specific model + specific MCP with list-off → confirm `/v1/models` & `/v1/mcps` hide non-granted, inference on disallowed model returns 403, granted MCP connects → management screen shows summary → revoke → token 401s.
4. Phase 9 Playwright spec green via `make test.e2e`.

## Retro (deliver at end)
Short note on what was unified (the `ResourceGrants` trait + `AccessPolicy` over both principals + shared grant enums) vs. kept separate (envelope types, Keycloak exchange/approval lifecycle, `access_request_auth_middleware`) and why.
