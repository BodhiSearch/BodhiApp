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
            server_core
                    ↓
                  routes_app (includes middleware)
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
- **Services layer**: Domain errors (`TokenServiceError`, `McpError`, etc.) — all implement `AppError` via `errmeta_derive`
- **Auth context errors**: `AuthContextError` in `src/auth/auth_context.rs`
- **HTTP layer**: `ApiError` / `OpenAIApiError` / `ErrorBody` live in `routes_app::shared` (NOT here)

### ApiError Is NOT in Services
`ApiError`, `OpenAIApiError`, `ErrorBody` moved to `routes_app::shared`. Do not add them back here.

## AppService Trait

Central service registry with 20 service accessors. Defined in `src/app_service/app_service.rs`. Includes `api_model_service()`, `download_service()`, `inference_service()`, `token_service()`, `tenant_service()`. All services are `Arc<dyn Trait>` with `#[mockall::automock]`.

## AuthScopedAppService

Wraps `Arc<dyn AppService>` + `AuthContext`. Defined in `src/app_service/auth_scoped.rs`.

**Auth-aware sub-services** (co-located with their domains):
- `tokens()` → `AuthScopedTokenService` (in `tokens/auth_scoped.rs`)
- `mcps()` → `AuthScopedMcpService` (in `mcps/auth_scoped.rs`)
- `users()` → `AuthScopedUserService` (in `users/auth_scoped.rs`)
- `user_access_requests()` → `AuthScopedUserAccessRequestService` (in `users/auth_scoped_access_requests.rs`)
- `data()` → `AuthScopedDataService` (in `models/auth_scoped_data.rs`)
- `api_models()` → `AuthScopedApiModelService` (in `models/auth_scoped_api_models.rs`)
- `downloads()` → `AuthScopedDownloadService` (in `models/auth_scoped_downloads.rs`)
- `tenants()` → `AuthScopedTenantService` (in `tenants/auth_scoped.rs`)

**Short-name passthrough accessors** (D1-D9, excluding D2 which is auth-scoped above): `settings()`, `auth_flow()`, `network()`, `sessions()`, `db()`, `hub()`, `ai_api()`, `time()`, `inference()`.

**Non-auth-scoped passthrough**: `access_request_service()` — intentionally not auth-scoped (see `AccessRequestService` doc comment).

**Removed passthroughs**: `token_service()`, `mcp_service()`, `data_service()` — use auth-scoped sub-services instead.

**Architecture rule**: Route handlers use `AuthScopedAppService`. Infrastructure (bootstrap, middleware) uses `AppService` directly.

## Domain Module Layout

Each domain module follows `*_objs.rs` pattern for types and `error.rs` for errors:
- `auth/auth_objs.rs` — `ResourceRole` (Anonymous/Guest/User/PowerUser/Manager/Admin), `TokenScope`, `UserScope`, `AppRole`, `UserInfo`
- `auth/auth_context.rs` — `AuthContext` enum (Anonymous{deployment}/Session/MultiTenantSession/ApiToken/ExternalApp), `AuthContextError`. `Session.role` and `MultiTenantSession.role` are `ResourceRole` (not Option).
- `tokens/token_objs.rs` — `TokenStatus`, `TokenDetail`, `CreateTokenRequest`, `UpdateTokenRequest`
- `models/model_objs.rs` — `Repo`, `HubFile`, `Alias` (User/Model/Api), `OAIRequestParams`, `JsonVec`, `DownloadStatus`, `ApiModelRequest`, `ApiAliasResponse` (has `has_api_key: bool`), `UserAliasRequest`
- `settings/setting_objs.rs` — `Setting`, `EnvType`, `AppType`, `LogLevel`
- `tenants/tenant_objs.rs` — `DeploymentMode` (Standalone/MultiTenant), `AppStatus` (Setup/Ready/ResourceAdmin), `Tenant` (includes `created_by: Option<String>`)
- `tenants/spi_types.rs` — `SpiTenant`, `SpiTenantListResponse`, `SpiCreateTenantRequest`, `SpiCreateTenantResponse`
- `mcps/mcp_objs.rs` — MCP types, `McpRequest`, `McpServerRequest` (both derive `Validate`)
- `app_access_requests/access_request_objs.rs` — `AppAccessRequest` (renamed from `AppAccessRequestRow`)
- `shared_objs/` — `error_wrappers.rs`, `utils.rs`, `log.rs`, `token.rs` (JWT parsing)
- `tenants/` — tenant management module
- `inference/` — `InferenceService` trait, `LlmEndpoint`, `InferenceError`

### Entity Aliases
All entities follow `pub type <Domain>Entity = Model;` pattern. Standard fields: `id` (ULID), `tenant_id`, `user_id` (for user-scoped), `created_at`/`updated_at`. Full entity index in `PACKAGE.md`.

### CRUD Conventions

**Request types** (in `*_objs.rs`): Named `*Request`, derive `Serialize, Deserialize, Validate, ToSchema`. Exclude `id`, `tenant_id`, `user_id`, timestamps.

**Service layer**: Services do NOT call `form.validate()` — input assumed validated by routes. Services return Entity types; route handlers convert Entity→Response via `.into()`. Business invariants requiring service deps stay in services. DB constraints handle uniqueness/FK violations → mapped to domain errors. Exceptions: Token returns `TokenDetail` directly, ApiModelService returns `ApiAliasResponse` directly.

**AuthScoped services**: Inject `tenant_id`/`user_id` from `AuthContext` only — no validation, no authorization. Return Entity types (pass through). Routes MUST use auth_scoped services exclusively.

**Route handlers**: `ValidatedJson<DomainRequest>` for body extraction. Handlers own field validation and operation-specific authorization. No `require_tenant_id()`/`require_user_id()` — AuthScoped handles.

**Response types** (in `*_objs.rs`): Separate struct from entity. Exclude `tenant_id`, `user_id`. Secret fields → `has_<secret>: bool`. `impl From<Entity> for ResponseType` defined in services.

## Error Handling

### IoError Pattern
Single `Io(#[from] IoError)` variant per service error enum. Convenience constructors: `IoError::file_read()`, `IoError::file_write()`, `IoError::dir_create()`, `IoError::file_delete()`. Bridge external errors via `impl_error_from!` macro.

### impl_error_from! Macro
Bridges orphan rule: `std::io::Error → IoError → DataServiceError::Io`. Signature: `impl_error_from!(source_type, target_enum::variant, intermediate_type)`.

### Auth-Scoped Service Return Types
Each auth-scoped service returns its own domain error type (e.g., `AuthScopedTokenService` → `TokenServiceError`). Blanket `From<T: AppError> for ApiError` auto-converts in route handlers. Full mapping in `PACKAGE.md`.

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

### AuthService SPI Proxy
- `create_tenant(bearer_token, name, description, redirect_uris)` — proxy to SPI `POST /realms/{realm}/bodhi/tenants`
- `TenantService.create_tenant()` takes `created_by: Option<String>` parameter
- `TenantService.set_client_ready(client_id, user_id)` — sets status to Ready AND created_by in one call (replaces separate `update_status` + `update_created_by`)
- `SettingService.multitenant_client_secret()` — reads `BODHI_MULTITENANT_CLIENT_SECRET` env var

### Session Keys
Namespaced session keys in `src/session_keys.rs`. Client-scoped tokens use `{client_id}:access_token` / `{client_id}:refresh_token` format. Dashboard tokens use separate `DASHBOARD_*` keys.

### Access Request Workflow
All requests start as drafts. Status: `draft` → `approved` | `denied` | `failed`. `requested_role` vs `approved_role` fields.

### Database
- SeaORM with dual SQLite/PostgreSQL support
- ID generation: ULID (`ulid::Ulid::new()`)
- Entity fields use typed enums via `DeriveValueType`
- Migrations in `src/db/sea_migrations/`

## Testing

Uses shared conventions from `crates/CLAUDE.md` "Shared Testing Conventions". Key infrastructure:
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
1. TimeService → 2. DbService → 3. SettingService → 4. AuthService → 5. SessionService → 6. TenantService → 7. HubService, DataService, CacheService → 8. ConcurrencyService, NetworkService → 9. AiApiService → 10. McpService → 11. TokenService → 12. InferenceService → 13. AccessRequestService → 14. QueueProducer
