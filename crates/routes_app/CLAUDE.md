# routes_app -- CLAUDE.md

**Companion docs** (load as needed):
- `PACKAGE.md` -- Implementation details, file index, error enum reference
- `src/middleware/CLAUDE.md` -- Middleware-specific documentation (auth, authorization, token service)
- `src/middleware/PACKAGE.md` -- Middleware module index and error enum reference
- `TESTING.md` -- Test patterns, mocking strategy, router construction
- `TECHDEBT.md` -- Known issues and planned refactors

## Purpose

API orchestration layer: HTTP endpoint handlers for all BodhiApp application routes. Defines `ApiError`/`OpenAIApiError`/`ErrorBody` in `shared/` (moved from `services`). Includes authentication/authorization middleware (merged from former `auth_middleware` crate) in `src/middleware/`. Consumes `AuthContext` via the `AuthScope` extractor.

## Architecture Position

```
services + server_core
         |
    routes_app        <-- this crate (includes middleware module)
    /     |      \
server_app  lib_bodhiserver  bodhi/src-tauri
```

State type: `Arc<dyn AppService>` (not `RouterState` -- that was removed).

## Route Group Architecture

Routes are organized into groups with different auth requirements and CORS policies.
Defined in `src/routes.rs`. Two CORS tiers: **restrictive** (blocks all cross-origin, for session-only APIs) and **permissive** (allows any origin, for external tools/apps).

### Permissive CORS (public + API-protected)

| Group | Auth Middleware | Role/Scope | Purpose |
|-------|---------------|------------|---------|
| `public_apis` | none | -- | `/ping`, `/health`, `/setup`, `/logout`, app access request create/status |
| `optional_auth` | `optional_auth_middleware` | -- | `/info`, `/user`, auth initiate/callback, dashboard auth, tenants, dev-only routes |
| `user_apis` | `api_auth_middleware` | User / TokenScope::User / UserScope::User | OpenAI/Ollama compat, model listing, model files |
| `power_user_apis` | `api_auth_middleware` | PowerUser / TokenScope::PowerUser / UserScope::PowerUser | Model alias CRUD, file pull/downloads |
| `apps_apis` | `api_auth_middleware` + `access_request_auth_middleware` | User / UserScope::User | External app endpoints under `/bodhi/v1/apps/...` (OAuth tokens), includes MCP transparent proxy |

### Restrictive CORS (session-protected)

| Group | Auth Middleware | Role | Purpose |
|-------|---------------|------|---------|
| `guest_endpoints` | `api_auth_middleware` | Guest | `users_request_access`, `users_request_status` |
| `user_session_apis` | `api_auth_middleware` | User | MCP CRUD, MCP auth configs, MCP OAuth, MCP servers (read), app access reviews, API model management |
| `power_user_session_apis` | `api_auth_middleware` | PowerUser | Token CRUD, metadata refresh, queue status |
| `admin_session_apis` | `api_auth_middleware` | Admin | Settings CRUD, MCP server create/update |
| `manager_session_apis` | `api_auth_middleware` | Manager | User access request approval/rejection, user listing, role changes, user deletion |

All session-protected groups share a base `auth_middleware` layer and restrictive CORS layer.

### UI Serving

`spa_router.rs` serves embedded UI assets under `/ui` prefix with SPA-aware fallback (non-extension paths return `index.html`). `routes_proxy.rs` proxies to Vite dev server (`localhost:3000`) for HMR when `BODHI_DEV_PROXY_UI=true`. Root `/` redirects to `/ui/`.

## AuthScope Extractor (Critical Pattern)

All route handlers use `AuthScope` (`src/shared/auth_scope_extractor.rs`), a newtype around `AuthScopedAppService`. Replaces the old `Extension<AuthContext>` + `State(state)` dual-extractor.

**Handler signature**: see `src/shared/auth_scope_extractor.rs:19` for the type definition.

Key methods on `AuthScope` (via `Deref` to `AuthScopedAppService`):
- `auth_context()` -- raw `AuthContext` enum
- `require_user_id()` / `require_client_id()` / `require_tenant_id()` -- return `Result<&str, AuthContextError>`
- `tokens()`, `mcps()`, `users()` -- auth-scoped sub-services
- `inference()` -- `Arc<dyn InferenceService>`
- `data_service()`, `setting_service()` -- passthrough accessors (no auth scoping)

Falls back to `AuthContext::Anonymous { deployment: DeploymentMode::Standalone }` when no auth middleware has populated the extension.

**AuthContext**: 5 variants — `Anonymous`, `Session`, `MultiTenantSession`, `ApiToken`, `ExternalApp`. `Session.role` and `MultiTenantSession.role` are `ResourceRole` (not `Option`). Full variant details in `crates/services/CLAUDE.md`.

## Domain Module Structure

Flat naming (no `routes_` prefix in module names). Each module has: `error.rs` (single `<Domain>RouteError`), `<domain>_api_schemas.rs` (request/response types), `routes_<domain>.rs` (handlers), `mod.rs` (declarations only). Full module index in `PACKAGE.md`.

The `models/` module has three sub-modules: `alias/`, `api/`, `files/`. Standalone files: `routes_ping.rs`, `routes_dev.rs`, `routes_proxy.rs`, `spa_router.rs`.

## Handler Naming Convention

Rails-style, no `_handler` suffix:
- `<domain>_index` (list), `<domain>_show` (get), `<domain>_create`, `<domain>_update`, `<domain>_destroy`
- Non-CRUD: descriptive names (`auth_initiate`, `auth_callback`)

## JSON Extraction Convention

All handlers accepting JSON bodies with Validate-deriving types use `ValidatedJson<DomainRequest>`:
see `src/shared/validated_json.rs` for implementation. `ValidatedJson` deserializes JSON and calls `form.validate()` automatically. Validation errors return 400 with structured error body. Services assume input is already validated.

**Entity->Response conversion**: Auth-scoped services return Entity types. Route handlers convert to Response via `.into()` before returning (e.g., `let mcp: Mcp = entity.into();`).

**Two-layer authorization model**: Middleware checks endpoint access, route handler checks operation-specific params (e.g., token scope privileges, role hierarchy for approval).

**Auth-scoped services only**: Route handlers MUST use `auth_scope.tokens()`, `auth_scope.mcps()`, etc. -- never call domain services directly.

## Error Handling Chain

Service error -> domain `<X>RouteError` (this crate) -> `ApiError` (`shared/api_error.rs`) -> OpenAI-compatible JSON.

`ApiError`, `OpenAIApiError`, `ErrorBody` are in `routes_app::shared` (import as `use crate::ApiError`, NOT `use services::ApiError`).

Domain error enums wrap service errors via `#[error(transparent)]` + `#[from]`. Error codes auto-generated: `model_route_error-alias_not_found`.

## OpenAPI Registration Checklist

Every new route must:
1. Add `#[utoipa::path(...)]` to handler (generates `__path_<handler>` symbol)
2. Add `API_TAG_<DOMAIN>` constant to `src/shared/constants.rs` (if new domain)
3. Register in `src/shared/openapi.rs`: imports, tags, schemas, paths
4. Regenerate: `cargo run --package xtask openapi`
5. Build TS client: `make build.ts-client`
6. Import from `@bodhiapp/ts-client` in frontend (not hand-rolled types)
7. Verify: `cargo test -p routes_app -- openapi` and `cd crates/bodhi && npm test`

## Key Workflow Gotchas

**Session clearing on role change**: When a user's role changes, all sessions must be cleared via `session_service`. The handler logs but does not fail if clearing errors.

**Settings allowlist**: Only `BODHI_EXEC_VARIANT` and `BODHI_KEEP_ALIVE_SECS` are editable via API. `BODHI_HOME` only via env var. Others return `SettingsRouteError::Unsupported`.

**Network host detection**: Setup/login flows extract `Host` header for callback URLs when `BODHI_PUBLIC_HOST` is not configured.

**MCP OAuth CSRF**: Token exchange validates `state` parameter from session.

**Access request role validation**: `users_access_request_approve` rejects `Anonymous`/`Guest` as role assignment targets (`!request.role.has_access_to(&ResourceRole::User)`). Approvers can only assign roles at or below their own level.

**Multi-tenant endpoints**: Dashboard auth (`/auth/dashboard/initiate`, `/auth/dashboard/callback`) and tenant management (`/tenants`, `/tenants/{client_id}/activate`) in `tenants/` module. Dashboard tokens stored under `dashboard:*` session keys. `/info` returns `deployment` and `client_id`. `/user/info` returns `dashboard: Option<DashboardUser>` with user details from the dashboard JWT.

**AppStatus values**: `Setup` (default), `Ready`, `ResourceAdmin`. `TenantSelection` was removed -- Anonymous{MultiTenant} and MultiTenantSession{client_id: None} with memberships now return `Ready`.

**Apps API thin wrappers**: `apps_mcps_index`, etc. in the `apps_apis` group are thin wrappers that delegate to the same auth-scoped services but are mounted under `/bodhi/v1/apps/...` with permissive CORS for external OAuth app access.

## Responses API (Pass-Through Proxy)

5 endpoints under `/v1/responses` in the `user_apis` route group:
- `POST /v1/responses` — create response (body forwarded to remote)
- `GET /v1/responses/{response_id}` — get response
- `POST /v1/responses/{response_id}/cancel` — cancel response
- `GET /v1/responses/{response_id}/input_items` — list input items
- `DELETE /v1/responses/{response_id}` — delete response

`response_id` path parameter validated: alphanumeric, underscore, hyphen only. GET/DELETE/cancel/input_items require `model` query parameter for multi-provider routing (not part of upstream OpenAI API). `resolve_api_key_for_alias` shared helper in `oai` module handles API key resolution.

## MCP Proxy Endpoint

**Endpoint**: `/bodhi/v1/apps/mcps/{id}/mcp` -- transparent HTTP reverse proxy to upstream MCP server (OAuth token-authenticated only). Accepts `any()` HTTP method (POST, GET, DELETE forwarded to upstream). Auth headers/query params injected from MCP instance config (header auth + OAuth with token refresh). SSE responses streamed without buffering. The proxy forwards all operations transparently to upstream (tools_filter was removed from the data model). See `PACKAGE.md` for handler flow details.

## Commands

- `cargo test -p routes_app` -- all tests
- `cargo test -p routes_app -- <module>` -- specific module (e.g., `mcps`, `tokens`)
- `cargo test -p routes_app -- openapi` -- verify OpenAPI spec matches
