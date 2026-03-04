# routes_app -- CLAUDE.md

**Companion docs** (load as needed):
- `PACKAGE.md` -- Implementation details, file index, error enum reference
- `TESTING.md` -- Test patterns, mocking strategy, router construction
- `TECHDEBT.md` -- Known issues and planned refactors

## Purpose

API orchestration layer: HTTP endpoint handlers for all BodhiApp application routes. Defines `ApiError`/`OpenAIApiError`/`ErrorBody` in `shared/` (moved from `services`). Consumes `AuthContext` from `auth_middleware` via the `AuthScope` extractor.

## Architecture Position

```
services + auth_middleware + server_core
                  |
             routes_app        <-- this crate
            /     |      \
    server_app  lib_bodhiserver  bodhi/src-tauri
```

State type: `Arc<dyn AppService>` (not `RouterState` -- that was removed).

## AuthScope Extractor (Critical Pattern)

All route handlers use `AuthScope` (`src/shared/auth_scope_extractor.rs`), a newtype around `AuthScopedAppService`. Replaces the old `Extension<AuthContext>` + `State(state)` dual-extractor.

**Handler signature**: see `src/shared/auth_scope_extractor.rs:19` for the type definition.

Key methods on `AuthScope` (via `Deref` to `AuthScopedAppService`):
- `auth_context()` -- raw `AuthContext` enum
- `require_user_id()` / `require_client_id()` / `require_tenant_id()` -- return `Result<&str, AuthContextError>`
- `tokens()`, `mcps()`, `tools()`, `users()` -- auth-scoped sub-services
- `inference()` -- `Arc<dyn InferenceService>`
- `data_service()`, `setting_service()` -- passthrough accessors (no auth scoping)

Falls back to `AuthContext::Anonymous { client_id: None, tenant_id: None }` when no auth middleware has populated the extension.

**AuthContext variants** (defined in `services::auth::auth_context`):
- `Anonymous { client_id: Option<String>, tenant_id: Option<String> }`
- `Session { client_id, tenant_id, user_id, username, role: Option<ResourceRole>, token }`
- `ApiToken { client_id, tenant_id, user_id, role: TokenScope, token }`
- `ExternalApp { client_id, tenant_id, user_id, role: Option<UserScope>, token, external_app_token, app_client_id, access_request_id: Option<String> }`

All non-Anonymous variants have `client_id: String` and `tenant_id: String` (multi-tenant support).

## Domain Module Structure

Flat naming (no `routes_` prefix in module names). Each module has:
- `error.rs` -- single `<Domain>RouteError` enum with `#[error_meta(trait_to_impl = AppError)]`
- `<domain>_api_schemas.rs` -- request/response types
- `routes_<domain>.rs` -- handler functions
- `mod.rs` -- declarations and re-exports only

| Module | Error Enum | Purpose |
|--------|------------|---------|
| `auth/` | `AuthRouteError` | OAuth2 initiate/callback/logout |
| `users/` | `UsersRouteError` | User mgmt, access requests |
| `apps/` | `AppsRouteError` | App access request workflow |
| `tokens/` | `TokenRouteError` | API token CRUD |
| `models/` | `ModelRouteError` | Model aliases, metadata, pull |
| `api_models/` | `ApiModelsRouteError` | Remote API model config |
| `settings/` | `SettingsRouteError` | Settings CRUD |
| `setup/` | `SetupRouteError` | App setup/init |
| `toolsets/` | `ToolsetRouteError` | Toolset CRUD + execution |
| `mcps/` | `McpRouteError` | MCP CRUD, tools, servers, OAuth |
| `oai/` | `OAIRouteError` | OpenAI-compatible endpoints |
| `ollama/` | `OllamaRouteError` | Ollama-compatible endpoints |

Standalone files: `routes_ping.rs`, `routes_dev.rs`, `routes_proxy.rs`

## Handler Naming Convention

Rails-style, no `_handler` suffix:
- `<domain>_index` (list), `<domain>_show` (get), `<domain>_create`, `<domain>_update`, `<domain>_destroy`
- Non-CRUD: descriptive names (`toolsets_execute`, `auth_initiate`, `auth_callback`)

## Error Handling Chain

Service error -> domain `<X>RouteError` (this crate) -> `ApiError` (`shared/api_error.rs`) -> OpenAI-compatible JSON.

`ApiError`, `OpenAIApiError`, `ErrorBody` are in `routes_app::shared` (import as `use crate::ApiError`, NOT `use services::ApiError`).

Domain error enums wrap service errors via `#[error(transparent)]` + `#[from]`. Error codes auto-generated: `model_route_error-alias_not_found`.

## ValidatedJson Extractor

`ValidatedJson<T>` (`src/shared/validated_json.rs`) combines JSON deserialization with `validator::Validate`. Use instead of manual `WithRejection<Json<T>>` + `validate()`.

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

**Time handling**: Always use `app_service.time_service().utc_now()`, never `Utc::now()`.

**Session clearing on role change**: When a user's role changes, all sessions must be cleared via `session_service`. The handler logs but does not fail if clearing errors.

**Settings allowlist**: Only `BODHI_EXEC_VARIANT` and `BODHI_KEEP_ALIVE_SECS` are editable via API. `BODHI_HOME` only via env var. Others return `SettingsRouteError::Unsupported`.

**Network host detection**: Setup/login flows extract `Host` header for callback URLs when `BODHI_PUBLIC_HOST` is not configured.

**MCP OAuth CSRF**: Token exchange validates `state` parameter from session.

## Commands

- `cargo test -p routes_app` -- all tests
- `cargo test -p routes_app -- <module>` -- specific module (e.g., `mcps`, `tokens`)
- `cargo test -p routes_app -- openapi` -- verify OpenAPI spec matches
