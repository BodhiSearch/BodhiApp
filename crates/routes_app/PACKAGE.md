# PACKAGE.md -- routes_app

Implementation details and file index. For architecture and design, see `CLAUDE.md`.

## Module Structure

Entry point: `src/lib.rs` -- re-exports all public modules with conditional `test_utils` compilation.

### Domain Route Modules

| Module | Error Enum | Purpose |
|--------|------------|---------|
| `auth/` | `AuthRouteError` | OAuth2 initiate/callback/logout |
| `users/` | `UsersRouteError` | User mgmt, access requests |
| `apps/` | `AppsRouteError` | App access request workflow |
| `tokens/` | `TokenRouteError` | API token CRUD |
| `models/` | `ModelRouteError` | Model alias CRUD, metadata, pull, API models, local files |
| `settings/` | `SettingsRouteError` | Settings CRUD |
| `setup/` | `SetupRouteError` | App setup/init |
| `mcps/` | `McpRouteError` | MCP CRUD, tools, servers, OAuth, MCP proxy |
| `oai/` | `OAIRouteError` | OpenAI-compatible endpoints |
| `ollama/` | `OllamaRouteError` | Ollama-compatible endpoints |
| `tenants/` | `DashboardAuthRouteError` | Dashboard auth, tenant CRUD, multi-tenant management |

`models/` sub-modules: `alias/` (user-created aliases), `api/` (remote API model configs), `files/` (local model files + downloads).

Standalone: `routes_ping.rs`, `routes_dev.rs`, `routes_proxy.rs`, `spa_router.rs`

### MCP Proxy (`src/mcps/mcp_proxy.rs`)

Transparent HTTP reverse proxy. Single `mcp_proxy_handler` Axum handler mounted at `/bodhi/v1/apps/mcps/{id}/mcp` (OAuth token-authenticated only).

**Handler flow:**
1. `AuthScope` + `Path(id)` extractors
2. `auth_scope.mcps().get(&id)` -- resolve MCP instance + server (`McpWithServerEntity`)
3. Check `server_enabled` and `enabled` flags (`McpRouteError::McpServerDisabled` / `McpInstanceDisabled`)
4. `auth_scope.mcps().resolve_auth_params(&id)` -- `Option<McpAuthParams>` (headers + query params, includes OAuth Bearer token with auto-refresh)
5. Build upstream URL, append auth query params
6. Forward headers (`content-type`, `accept`, `mcp-session-id`, `mcp-protocol-version`, `last-event-id`), inject auth headers
7. Send via shared `reqwest::Client` (static `Lazy`, connection-pooled, no request timeout for SSE), stream response body back via `bytes_stream()` + `Body::from_stream()`

The proxy forwards all operations transparently to upstream (tools_filter was removed from the data model).

### Middleware (`src/middleware/`)

Authentication, authorization, and request processing middleware. Merged from the former `auth_middleware` crate. See `src/middleware/CLAUDE.md` and `src/middleware/PACKAGE.md` for details.

| Module | Purpose |
|--------|---------|
| `auth/` | `auth_middleware`, `optional_auth_middleware`, `AuthError` |
| `apis/` | `api_auth_middleware`, `ApiAuthError` |
| `access_requests/` | `access_request_auth_middleware`, entity-level access control |
| `token_service/` | `DefaultTokenService`, `CachedExchangeResult` |
| `redirects/` | `canonical_url_middleware` |
| `error.rs` | `MiddlewareError` |
| `utils.rs` | `app_status_or_default`, `generate_random_string` |

### Shared Infrastructure (`src/shared/`)

| File | Purpose |
|------|---------|
| `api_error.rs` | `ApiError` with blanket `From<T: AppError>` conversion |
| `error_oai.rs` | `OpenAIApiError`, `ErrorBody` |
| `error_wrappers.rs` | Framework error type wrappers |
| `auth_scope_extractor.rs` | `AuthScope` Axum extractor |
| `pagination.rs` | Pagination params, paginated response |
| `constants.rs` | API tag constants, endpoint paths |
| `openapi.rs` | `BodhiOpenAPIDoc`, `OpenAPIEnvModifier`, `GlobalErrorResponses` |
| `common.rs` | `RedirectResponse` DTO |
| `utils.rs` | `extract_request_host` |

### Route Composition

`src/routes.rs` -- `build_routes(app_service: Arc<dyn AppService>, static_router: Option<Router>) -> Router`

Composes all domain routes with auth middleware layers. State is `Arc<dyn AppService>`.

### Test Utilities (`test-utils` feature)

| File | Purpose |
|------|---------|
| `src/test_utils/router.rs` | `build_test_router()`, `create_authenticated_session()`, `build_live_test_router()` |
| `src/test_utils/assertions.rs` | Assertion helpers |
| `src/test_utils/mcp.rs` | MCP test setup helpers |

## Domain Error Enums

All enums use `#[error_meta(trait_to_impl = AppError)]` from `errmeta_derive`.

| Error Enum | Module | Key Variants |
|------------|--------|-------------|
| `AuthRouteError` | `auth/error.rs` | OAuth flow failures |
| `UsersRouteError` | `users/error.rs` | Admin user operations, access requests |
| `AppsRouteError` | `apps/error.rs` | App access request workflow |
| `TokenRouteError` | `tokens/error.rs` | Token lifecycle, privilege escalation |
| `ModelRouteError` | `models/error.rs` | Model metadata, alias operations |
| `ApiModelsRouteError` | `api_models/error.rs` | API model config errors |
| `SettingsRouteError` | `settings/error.rs` | Settings management |
| `SetupRouteError` | `setup/error.rs` | Setup flow errors |
| `McpRouteError` | `mcps/error.rs` | MCP CRUD, OAuth validation |
| `OAIRouteError` | `oai/error.rs` | OpenAI endpoint errors |
| `OllamaRouteError` | `ollama/error.rs` | Ollama endpoint errors |

## API Token Privilege Matrix

`TokenScope` has two variants (`User`, `PowerUser`); match is exhaustive:

| User Role | Allowed Scopes |
|-----------|---------------|
| `User` | `scope_token_user` only |
| `PowerUser`+ | `scope_token_user`, `scope_token_power_user` |

## OpenAPI Tags

Defined in `src/shared/constants.rs`, registered in `src/shared/openapi.rs`:

`system`, `setup`, `auth`, `api-keys`, `api-models`, `models`, `settings`, `mcps`, `openai`, `ollama`

## Commands

- `cargo test -p routes_app` -- all tests
- `cargo test -p routes_app -- <module>` -- specific module
- `cargo test -p routes_app -- openapi` -- verify spec

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

## UI Serving

`spa_router.rs` serves embedded UI assets under `/ui` prefix with SPA-aware fallback (non-extension paths return `index.html`). `routes_proxy.rs` proxies to Vite dev server (`localhost:3000`) for HMR when `BODHI_DEV_PROXY_UI=true`. Root `/` redirects to `/ui/`.

## Domain Module Structure

Flat naming (no `routes_` prefix in module names). Each module has: `error.rs` (single `<Domain>RouteError`), `<domain>_api_schemas.rs` (request/response types), `routes_<domain>.rs` (handlers), `mod.rs` (declarations only). Full module index above.

The `models/` module has three sub-modules: `alias/`, `api/`, `files/`. The `anthropic/` module handles Anthropic-specific API routes and error schemas (moved from `oai/` and `shared/`). The `providers/` module has shared utilities for multi-provider routing (e.g., `resolve_api_key_for_alias`). Standalone files: `routes_ping.rs`, `routes_dev.rs`, `routes_proxy.rs`, `spa_router.rs`.

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

## Responses API (Pass-Through Proxy)

5 endpoints under `/v1/responses` in the `user_apis` route group:
- `POST /v1/responses` — create response (body forwarded to remote)
- `GET /v1/responses/{response_id}` — get response
- `POST /v1/responses/{response_id}/cancel` — cancel response
- `GET /v1/responses/{response_id}/input_items` — list input items
- `DELETE /v1/responses/{response_id}` — delete response

`response_id` path parameter validated: alphanumeric, underscore, hyphen only. GET/DELETE/cancel/input_items require `model` query parameter for multi-provider routing (not part of upstream OpenAI API). `resolve_api_key_for_alias` shared helper in `providers/` module handles API key resolution (moved from `oai/`).

## Anthropic API (Pass-Through Proxy)

Routes in `src/anthropic/`: `routes_anthropic.rs` (handlers), `anthropic_api_schemas.rs` (AnthropicApiError types, moved from `shared/anthropic_error.rs`). `anthropic_models_list_handler` returns full Anthropic metadata (display_name, created_at, capabilities) from stored `ApiModel::Anthropic` data. Error types follow Anthropic's error format (distinct from OpenAI's `ErrorBody`).

Anthropic endpoints use a pre-built `openapi-anthropic.json` spec (in `resources/` and `ts-client/`) synced from the official Anthropic API to maintain format consistency. These routes intentionally do NOT use `#[utoipa::path]` annotations — the Anthropic API spec is kept separate from the Bodhi management API OpenAPI spec to avoid overloading it. Do not manually edit `openapi-anthropic.json`; re-sync from Anthropic when the upstream spec changes.

## MCP Proxy Endpoint

**Endpoint**: `/bodhi/v1/apps/mcps/{id}/mcp` -- transparent HTTP reverse proxy to upstream MCP server (OAuth token-authenticated only). Accepts `any()` HTTP method (POST, GET, DELETE forwarded to upstream). Auth headers/query params injected from MCP instance config (header auth + OAuth with token refresh). SSE responses streamed without buffering. The proxy forwards all operations transparently to upstream (tools_filter was removed from the data model). See MCP Proxy handler flow above.

## Key Workflow Gotchas

**Settings allowlist**: Only `BODHI_EXEC_VARIANT` and `BODHI_KEEP_ALIVE_SECS` are editable via API. `BODHI_HOME` only via env var. Others return `SettingsRouteError::Unsupported`.

**Network host detection**: Setup/login flows extract `Host` header for callback URLs when `BODHI_PUBLIC_HOST` is not configured.

**MCP OAuth CSRF**: Token exchange validates `state` parameter from session.

**Access request role validation**: `users_access_request_approve` rejects `Anonymous`/`Guest` as role assignment targets (`!request.role.has_access_to(&ResourceRole::User)`). Approvers can only assign roles at or below their own level.

**Multi-tenant endpoints**: Dashboard auth (`/auth/dashboard/initiate`, `/auth/dashboard/callback`) and tenant management (`/tenants`, `/tenants/{client_id}/activate`) in `tenants/` module. Dashboard tokens stored under `dashboard:*` session keys. `/info` returns `deployment` and `client_id`. `/user/info` returns `dashboard: Option<DashboardUser>` with user details from the dashboard JWT.

**Apps API thin wrappers**: `apps_mcps_index`, etc. in the `apps_apis` group are thin wrappers that delegate to the same auth-scoped services but are mounted under `/bodhi/v1/apps/...` with permissive CORS for external OAuth app access.
