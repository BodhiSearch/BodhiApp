# PACKAGE.md -- routes_app

Implementation details and file index. For architecture and design, see `CLAUDE.md`.

## Module Structure

Entry point: `src/lib.rs` -- re-exports all public modules with conditional `test_utils` compilation.

### Domain Route Modules

| Module | Directory | Key Files |
|--------|-----------|-----------|
| `auth` | `src/auth/` | `routes_auth.rs` (initiate/callback/logout) |
| `users` | `src/users/` | `routes_users.rs` (admin mgmt), `routes_users_access_request.rs`, `routes_users_info.rs` |
| `apps` | `src/apps/` | `routes_apps.rs` (app access request workflow) |
| `tokens` | `src/tokens/` | `routes_tokens.rs` (CRUD with privilege escalation prevention) |
| `models` | `src/models/` | aliases, metadata refresh, model pull |
| `api_models` | `src/api_models/` | Remote API model config CRUD |
| `settings` | `src/settings/` | Settings list, update, reset |
| `setup` | `src/setup/` | App setup, health, app info |
| `toolsets` | `src/toolsets/` | Toolset CRUD, execution, type management |
| `mcps` | `src/mcps/` | MCP CRUD, tools, servers, auth configs, OAuth |
| `oai` | `src/oai/` | OpenAI-compatible chat completions, models, embeddings |
| `ollama` | `src/ollama/` | Ollama-compatible models, show, chat |

Standalone: `routes_ping.rs`, `routes_dev.rs`, `routes_proxy.rs`

### Shared Infrastructure (`src/shared/`)

| File | Purpose |
|------|---------|
| `api_error.rs` | `ApiError` with blanket `From<T: AppError>` conversion |
| `error_oai.rs` | `OpenAIApiError`, `ErrorBody` |
| `error_wrappers.rs` | Framework error type wrappers |
| `auth_scope_extractor.rs` | `AuthScope` Axum extractor |
| `validated_json.rs` | `ValidatedJson<T>` extractor |
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
| `ToolsetRouteError` | `toolsets/error.rs` | Toolset validation |
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

`system`, `setup`, `auth`, `api-keys`, `api-models`, `models`, `settings`, `toolsets`, `mcps`, `openai`, `ollama`

## Commands

- `cargo test -p routes_app` -- all tests
- `cargo test -p routes_app -- <module>` -- specific module
- `cargo test -p routes_app -- openapi` -- verify spec
