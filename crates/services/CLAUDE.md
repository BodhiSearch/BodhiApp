# services — CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` — Implementation details, file index, error types table
- `src/test_utils/CLAUDE.md` — Test utility infrastructure
- `src/test_utils/PACKAGE.md` — Test utility implementation details

## Purpose

Domain types + business logic hub. All domain objects live here co-located with services that use them. Re-exports `errmeta` types so downstream crates import from `services::` only.

## Architecture Position

```
errmeta / errmeta_derive / llama_server_proc / mcp_client
                         ↓
                    [services]  ← YOU ARE HERE
                    ↓        ↓
            server_core   auth_middleware
                    ↓        ↓
                  routes_app
```

**Re-exports**: `AppError`, `ErrorType`, `IoError`, `EntityError`, `RwLockReadError`, `impl_error_from!` from errmeta. Also `pub use db::*` in lib.rs — use `services::DbService` not `services::db::DbService`.

## Critical Rules

### Time: Never Use `Utc::now()`
All timestamps must go through `TimeService`. Tests use `FrozenTimeService` (defaults to 2025-01-01T00:00:00Z). See `src/db/time_service.rs`.

### Multi-Tenant Transactions
All mutating DbService operations on tenant-scoped rows use `begin_tenant_txn(tenant_id)` from `DbCore` trait (`src/db/db_core.rs`). On PostgreSQL this sets RLS via `SET LOCAL app.current_tenant_id`. On SQLite returns plain transaction. Settings are global (no tenant_id) — use `DefaultDbService` directly.

### API Token Format
`bodhiapp_<base64url_random>.<client_id>` — prefix lookup is cross-tenant by design; tenant resolved from `client_id` suffix after hash verification.

### Error Layer Separation
- **Services layer**: Domain errors (`TokenServiceError`, `McpError`, `ToolsetError`, etc.) — all implement `AppError` via `errmeta_derive`
- **Auth context errors**: `AuthContextError` in `src/auth/auth_context.rs`
- **HTTP layer**: `ApiError` / `OpenAIApiError` / `ErrorBody` live in `routes_app::shared` (NOT here)

### ApiError Is NOT in Services
`ApiError`, `OpenAIApiError`, `ErrorBody` moved to `routes_app::shared`. Do not add them back here.

## AppService Trait

Central service registry with 18 service accessors. Defined in `src/app_service/app_service.rs`. Key services:
- `tenant_service()`, `inference_service()`, `token_service()` — newer additions not in older docs
- All services are `Arc<dyn Trait>` with `#[mockall::automock]`

## AuthScopedAppService

Wraps `Arc<dyn AppService>` + `AuthContext`. Defined in `src/app_service/auth_scoped.rs`.

**Auth-aware sub-services** (inject user context):
- `tokens()` → `AuthScopedTokenService` (in `auth_scoped_tokens.rs`)
- `mcps()` → `AuthScopedMcpService` (in `auth_scoped_mcps.rs`)
- `tools()` → `AuthScopedToolService` (in `auth_scoped_tools.rs`)
- `users()` → `AuthScopedUserService` (in `auth_scoped_users.rs`)
- `data()` → `AuthScopedDataService` (in `auth_scoped_data.rs`)

**Short-name passthrough accessors** (D1-D10): `settings()`, `tenant()`, `auth_flow()`, `network()`, `sessions()`, `db()`, `hub()`, `ai_api()`, `time()`, `inference()`.

**Legacy passthrough accessors**: `data_service()`, `hub_service()`, etc. — kept for backward compatibility.

**Architecture rule**: Route handlers use `AuthScopedAppService`. Infrastructure (bootstrap, middleware) uses `AppService` directly.

## Domain Module Layout

Each domain module follows `*_objs.rs` pattern for types and `error.rs` for errors:
- `auth/auth_objs.rs` — `ResourceRole`, `TokenScope`, `UserScope`, `AppRole`, `UserInfo`
- `auth/auth_context.rs` — `AuthContext` enum, `AuthContextError`
- `tokens/token_objs.rs` — `TokenStatus`, `ApiTokenRow`
- `models/model_objs.rs` — `Repo`, `HubFile`, `Alias` (User/Model/Api), `OAIRequestParams`, `JsonVec`, `DownloadStatus`
- `settings/setting_objs.rs` — `Setting`, `EnvType`, `AppType`, `LogLevel`
- `tenants/tenant_objs.rs` — `AppStatus`, tenant types
- `mcps/mcp_objs.rs` — MCP types, `validate_mcp_instance_name()`
- `toolsets/toolset_objs.rs` — toolset types
- `app_access_requests/access_request_objs.rs` — access request types
- `shared_objs/` — `error_wrappers.rs`, `utils.rs`, `log.rs`, `token.rs` (JWT parsing)
- `tenants/` — tenant management module
- `inference/` — `InferenceService` trait, `LlmEndpoint`, `InferenceError`

## Error Handling

### IoError Pattern
Single `Io(#[from] IoError)` variant per service error enum. Convenience constructors: `IoError::file_read()`, `IoError::file_write()`, `IoError::dir_create()`, `IoError::file_delete()`. Bridge external errors via `impl_error_from!` macro.

### impl_error_from! Macro
Bridges orphan rule: `std::io::Error → IoError → DataServiceError::Io`. Signature: `impl_error_from!(source_type, target_enum::variant, intermediate_type)`.

### Auth-Scoped Service Return Types
- `AuthScopedTokenService` → `TokenServiceError`
- `AuthScopedMcpService` → `McpError`
- `AuthScopedToolService` → `ToolsetError`
- `AuthScopedUserService` → `AuthScopedUserError`
- `AuthScopedDataService` → `DataServiceError`

Blanket `From<T: AppError> for ApiError` auto-converts in route handlers.

## Cross-Crate Coordination

### Alias Resolution Priority
1. User aliases (YAML files)
2. Model aliases (auto-discovered GGUF)
3. API aliases (database, prefix-aware routing via `supports_model()`)

### MCP Service
- CRUD for MCP server instances with slug-based identification
- Auth header preservation: switching auth type away from `Header` does NOT delete auth headers (admin-managed). OAuth tokens ARE cleaned up.
- OAuth token refresh has per-key concurrency guard (keyed by `oauth_refresh:{config_id}`)
- CASCADE FK constraints on MCP tables

### Access Request Workflow
All requests start as drafts. Status: `draft` → `approved` | `denied` | `failed`. `requested_role` vs `approved_role` fields.

### Database
- SeaORM with dual SQLite/PostgreSQL support
- ID generation: ULID (`ulid::Ulid::new()`)
- Entity fields use typed enums via `DeriveValueType`
- Migrations in `src/db/sea_migrations/`

## Testing

### Canonical Pattern
- `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]` for async tests
- `#[awt]` only when `#[future]` fixture params are used
- `assert_eq!(expected, actual)` with `use pretty_assertions::assert_eq;`
- Error code assertions via `.code()`, never message text
- Return `-> anyhow::Result<()>`

### Key Infrastructure
- **TestDbService**: wraps `DefaultDbService` with event broadcasting + `FrozenTimeService`. See `src/test_utils/db.rs`
- **AppServiceStub**: builder-based full service composition. See `src/test_utils/app.rs`
- **SeaTestContext**: dual SQLite/PG fixture. See `src/test_utils/sea.rs`
- **OfflineHubService**: local-only hub ops. See `src/test_utils/hf.rs`
- **AuthContext test factories**: `src/test_utils/auth_context.rs` — `test_anonymous()`, `test_session()`, `test_api_token()`, `test_external_app()`
- **TEST_TENANT_ID / TEST_USER_ID**: constants in `src/test_utils/db.rs`

### Test File Organization
Sibling `test_*.rs` pattern for files over ~500 lines. Inline `mod tests` for smaller files. Always `mod tests` (not `mod test`).

### Skill Reference
`.claude/skills/test-services/SKILL.md` — quick reference and migration checklist.

## Service Initialization Order
1. TimeService → 2. DbService → 3. SettingService → 4. AuthService → 5. SessionService → 6. TenantService → 7. HubService, DataService, CacheService → 8. ConcurrencyService, NetworkService → 9. AiApiService, ToolService, ExaService → 10. McpService → 11. TokenService → 12. InferenceService → 13. AccessRequestService → 14. QueueProducer
