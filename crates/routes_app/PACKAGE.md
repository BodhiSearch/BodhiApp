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

Transparent HTTP reverse proxy. Single `mcp_proxy_handler` Axum handler used by both `/bodhi/v1/mcps/{id}/mcp` and `/bodhi/v1/apps/mcps/{id}/mcp`.

**Handler flow:**
1. `AuthScope` + `Path(id)` extractors
2. `auth_scope.mcps().get(&id)` -- resolve MCP instance + server (`McpWithServerEntity`)
3. Check `server_enabled` and `enabled` flags (`McpRouteError::McpServerDisabled` / `McpInstanceDisabled`)
4. `auth_scope.mcps().resolve_auth_params(&id)` -- `Option<McpAuthParams>` (headers + query params, includes OAuth Bearer token with auto-refresh)
5. Build upstream URL, append auth query params
6. Forward headers (`content-type`, `accept`, `mcp-session-id`, `mcp-protocol-version`, `last-event-id`), inject auth headers
7. Send via shared `reqwest::Client` (static `Lazy`, connection-pooled, no request timeout for SSE), stream response body back via `bytes_stream()` + `Body::from_stream()`

`tools_filter` is NOT enforced at proxy level -- transparent pass-through.

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
