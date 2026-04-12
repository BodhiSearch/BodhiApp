# PACKAGE.md - services Crate Implementation Index

*For architecture and critical rules, see [CLAUDE.md](CLAUDE.md)*

## Module Structure

### Crate Root and Re-exports
- `src/lib.rs` — module declarations, re-exports all public types, re-exports `errmeta` types for downstream convenience, `pub use db::*`
- `src/app_service/` — `AppService` trait (18 accessors), `DefaultAppService`, `AuthScopedAppService`, auth-scoped sub-services
- `src/macros.rs` — `asref_impl!` macro for service trait AsRef implementations
- `src/env_wrapper.rs` — `EnvWrapper` trait for environment variable abstraction

### Cross-Cutting Types (`shared_objs/`)
- `src/shared_objs/error_wrappers.rs` — `SerdeJsonError`, `SerdeYamlError`, `ReqwestError`, `ObjValidationError`
- `src/shared_objs/utils.rs` — `is_default()`, `ILLEGAL_CHARS` regex, `to_safe_filename()`
- `src/shared_objs/log.rs` — `mask_sensitive_value()`, `mask_form_params()`, HTTP request/response logging
- `src/shared_objs/token.rs` — JWT token parsing, validation, claims extraction

### Authentication and Security (`auth/`, `tokens/`)
- `src/auth/auth_objs.rs` — `ResourceRole`, `TokenScope`, `UserScope`, `AppRole`, `UserInfo`
- `src/auth/auth_context.rs` — `AuthContext` enum (Anonymous, Session, ApiToken, ExternalApp), `AuthContextError`
- `src/auth/auth_service.rs` — `AuthService` trait: OAuth2 PKCE, token exchange, dynamic client registration
- `src/auth/session_service.rs` — `SessionService` trait, `AppSessionStoreExt`, `DefaultSessionService`
- `src/auth/session_store.rs` — `SessionStoreBackend`, `InnerStoreShared`, `is_postgres_url()`
- `src/auth/session_error.rs` — `SessionServiceError` enum
- `src/auth/postgres.rs`, `src/auth/sqlite.rs` — backend-specific session store creation
- `src/tokens/token_objs.rs` — `TokenStatus`, `TokenDetail`
- `src/tokens/token_service.rs` — `TokenService` trait
- `src/tokens/token_repository.rs` — `TokenRepository` trait
- `src/tokens/error.rs` — `TokenServiceError`

### Tenants (`tenants/`)
- `src/tenants/tenant_objs.rs` — `AppStatus` enum, tenant types
- `src/tenants/tenant_service.rs` — `TenantService` trait, `DefaultTenantService`
- `src/tenants/tenant_repository.rs` — `TenantRepository` trait
- `src/tenants/error.rs` — tenant error types

### Inference (`inference/`)
- `src/inference/inference_service.rs` — `InferenceService` trait, `LlmEndpoint` enum (ChatCompletions, Embeddings)
- `src/inference/error.rs` — `InferenceError`
- `src/inference/noop.rs` — `NoopInferenceService`

### Model and Data Management (`models/`)
- `src/models/model_objs.rs` — `Repo`, `HubFile`, `Alias` (User/Model/Api variants), `UserAlias`, `ModelAlias`, `ApiAlias`, `OAIRequestParams`, `ModelMetadata`, `JsonVec`, `DownloadStatus`, `BuilderError`, GGUF constants
- `src/models/data_service.rs` — `DataService` trait: local model storage, alias management, three-tier resolution
- `src/models/hub_service.rs` — `HubService` trait: HuggingFace Hub API, local model discovery
- `src/models/gguf/` — GGUF format parsing (header, metadata extraction)
- `src/models/progress_tracking.rs` — Download progress monitoring

### User Management (`users/`)
- `src/users/user_objs.rs` — `UserAccessRequestStatus`
- `src/users/access_repository.rs` — User access request repository trait

### AI Services (`ai_apis/`)
- `src/ai_apis/ai_api_service.rs` — `AiApiService` trait: external AI API integration
- `src/ai_apis/ai_provider_client.rs` — `AIProviderClient` strategy trait; `OpenAIProviderClient`, `OpenAIResponsesProviderClient`, `AnthropicProviderClient`, `AnthropicOAuthProviderClient`. `merge_extra_body()` merges config `extra_body` into incoming request body (prepends `"system"` arrays; other keys fall back when incoming lacks them).
- `src/ai_apis/error.rs` — `AiApiServiceError`

### MCP Services (`mcps/`)
- `src/mcps/mcp_objs.rs` — `McpServer`, `Mcp`, MCP auth config types, validation constants
- `src/mcps/mcp_service.rs` — `McpService` trait (includes `resolve_auth_params`), `DefaultMcpService`
- `src/mcps/auth_scoped.rs` — `AuthScopedMcpService` (includes `resolve_auth_params` passthrough)
- `src/mcps/error.rs` — `McpError`
- `src/mcps/test_mcp_proxy_service.rs` — Tests for `resolve_auth_params` (MCP not found, public auth, disabled instance, header auth, OAuth token)

### Access Control (`app_access_requests/`)
- `src/app_access_requests/access_request_objs.rs` — `AppAccessRequest` (renamed from `AppAccessRequestRow`), `AppAccessResponse`, status enums
- `src/app_access_requests/access_request_service.rs` — `AccessRequestService` trait
- `src/app_access_requests/error.rs` — `AccessRequestError`

### Configuration (`settings/`)
- `src/settings/setting_objs.rs` — `Setting`, `EnvType`, `AppType`, `LogLevel`, `AppCommand`
- `src/settings/setting_service.rs` — `SettingService` trait
- `src/settings/default_service.rs` — `DefaultSettingService`
- `src/settings/bootstrap_parts.rs` — `BootstrapParts` data carrier
- `src/settings/constants.rs` — Setting key constants
- `src/settings/error.rs` — `SettingsMetadataError`, `SettingServiceError`
- `src/settings/settings_repository.rs` — `SettingsRepository` trait

### Utility Services (`utils/`)
- `src/utils/cache_service.rs` — Mini-moka based caching
- `src/utils/concurrency_service.rs` — Distributed lock abstraction, `LocalConcurrencyService`
- `src/utils/keyring_service.rs` — Platform-specific credential storage
- `src/utils/network_service.rs` — Network connectivity checks
- `src/utils/queue_service.rs` — Background metadata extraction queue

### Persistence (`db/`)
- `src/db/service.rs` — `DbService` trait, repository trait definitions
- `src/db/time_service.rs` — `TimeService` trait, `DefaultTimeService`
- `src/db/default_service.rs` — `DefaultDbService` (SeaORM, sole implementation)
- `src/db/db_core.rs` — `DbCore` trait with `begin_tenant_txn()`
- `src/db/encryption.rs` — API key encryption utilities
- `src/db/error.rs` — `DbError`
- `src/db/objs.rs` — `ApiKeyUpdate`
- `src/db/entities/` — SeaORM entity definitions with `Relation` enums
- `src/db/sea_migrations/` — migrations (SQLite + PostgreSQL)
- `src/db/service_*.rs` — repository implementations
- `src/db/*_repository.rs` — repository traits

## Error Types by Service

| Service | Error Enum | IoError Variant | Other Key Variants |
|---------|-----------|-----------------|-------------------|
| DataService | `DataServiceError` | `Io(IoError)` | `DirMissing`, `DataFileNotFound`, `AliasNotExists`, `AliasExists`, `SerdeYaml`, `HubService`, `Db` |
| HubService | `HubServiceError` | `IoError(IoError)` | `HubApiError`, `HubFileNotFound`, `ObjValidationError` |
| SettingService | `SettingServiceError` | `Io(IoError)` | `SerdeYaml`, `LockError`, `InvalidSource` |
| AuthService | `AuthServiceError` | (none) | `Reqwest`, `AuthServiceApiError`, `TokenExchangeError` |
| DbService | `DbError` | (none) | `SeaOrmError`, `StrumParse`, `TokenValidation`, `EncryptionError`, `PrefixExists`, `ItemNotFound`, `MultipleAppInstance`, `Conversion` |
| SessionService | `SessionServiceError` | (none) | `SqlxError`, `SessionStoreError`, `DbSetup` |
| AiApiService | `AiApiServiceError` | (none) | `Reqwest`, `ApiError`, `Unauthorized`, `NotFound`, `RateLimit`, `PromptTooLong` |
| McpService | `McpError` | (none) | `McpNotFound`, `McpUrlNotAllowed`, `McpDisabled`, `ToolNotFound`, `SlugExists`, `DbError` |
| TokenService | `TokenServiceError` | (none) | service-specific variants |
| InferenceService | `InferenceError` | (none) | `Unsupported` and other variants |

## Feature Flags

- `test-utils` — test utilities, mock services, rstest fixtures, `mcp_client/test-utils`
- `default = ["tokio"]` — Tokio runtime

## Entity Alias Index

| Domain | Alias | Entity File |
|--------|-------|-------------|
| MCP Instance | `McpEntity` | `src/mcps/mcp_entity.rs` |
| MCP Server | `McpServerEntity` | `src/mcps/mcp_server_entity.rs` |
| MCP Auth Param | `McpAuthParamEntity` | `src/mcps/mcp_auth_param_entity.rs` |
| MCP Auth Config | `McpAuthConfigEntity` | `src/mcps/mcp_auth_config_entity.rs` |
| MCP Auth Config Param | `McpAuthConfigParamEntity` | `src/mcps/mcp_auth_config_param_entity.rs` |
| MCP OAuth Config Detail | `McpOAuthConfigDetailEntity` | `src/mcps/mcp_oauth_config_detail_entity.rs` |
| MCP OAuth Token | `McpOAuthTokenEntity` | `src/mcps/mcp_oauth_token_entity.rs` |
| MCP + Server join | `McpWithServerEntity` | `src/mcps/mcp_entity.rs` |
| API Model | `ApiModelEntity` | `src/models/api_model_alias_entity.rs` |
| User Alias | `UserAliasEntity` | `src/models/user_alias_entity.rs` |
| Token | `TokenEntity` | `src/tokens/api_token_entity.rs` |
| Download | `DownloadEntity` | `src/models/download_request_entity.rs` |
| Model Metadata | `ModelMetadataEntity` | `src/models/model_metadata_entity.rs` |
| User Access Req | `UserAccessRequestEntity` | `src/users/access_request_entity.rs` |
| App Access Req | `AppAccessRequestEntity` | `src/app_access_requests/app_access_request_entity.rs` |

## Auth-Scoped Service Return Types

| Auth-Scoped Service | Error Type |
|---|---|
| `AuthScopedTokenService` | `TokenServiceError` |
| `AuthScopedMcpService` | `McpError` |
| `AuthScopedUserService` | `AuthScopedUserError` |
| `AuthScopedDataService` | `DataServiceError` |
| `AuthScopedApiModelService` | `ApiModelServiceError` |
| `AuthScopedDownloadService` | `DownloadServiceError` |
| `AuthScopedTenantService` | `TenantError` |

## Database Upsert Pattern
`SettingsRepository::upsert_setting` returns a struct with `created_at` set to `now` even on update path (placeholder). Callers needing actual `created_at` should query. See `src/db/service_settings.rs`.

## AppService and AuthScopedAppService

**AppService**: Central service registry with 20 service accessors. Defined in `src/app_service/app_service.rs`. Includes `api_model_service()`, `download_service()`, `inference_service()`, `token_service()`, `tenant_service()`. All services are `Arc<dyn Trait>` with `#[mockall::automock]`.

**AuthScopedAppService**: Wraps `Arc<dyn AppService>` + `AuthContext`. Defined in `src/app_service/auth_scoped.rs`.

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

**AiApiService**: `forward_request_with_method(method, url, body, api_key)` uses `http::Method` for type-safe HTTP method dispatch. `SafeReqwest` provides `request(method, url)` as a generic method alongside `get`/`post`/`delete`. `fetch_models` returns `Vec<ApiModel>` and delegates to `AIProviderClient` strategy pattern.

**AIProviderClient** (`ai_apis/ai_provider_client.rs`): Strategy trait for multi-provider model fetching. Concrete impls: `OpenAIProviderClient`, `OpenAIResponsesProviderClient`, `AnthropicProviderClient`, `AnthropicOAuthProviderClient`. Instantiated directly in `DefaultAiApiService` methods based on `ApiFormat`.

**ApiModelService**: `create()`/`update()` validate model IDs against the remote provider (fetches model list). `ModelNotFoundAtProvider` error if a requested model doesn't exist at the provider.

**Non-auth-scoped passthrough**: `access_request_service()` — intentionally not auth-scoped (see `AccessRequestService` doc comment).

**Removed passthroughs**: `token_service()`, `mcp_service()`, `data_service()` — use auth-scoped sub-services instead.

## IoError Pattern

Single `Io(#[from] IoError)` variant per service error enum. Convenience constructors: `IoError::file_read()`, `IoError::file_write()`, `IoError::dir_create()`, `IoError::file_delete()`. Bridge external errors via `impl_error_from!` macro.

## impl_error_from! Macro

Bridges orphan rule: `std::io::Error → IoError → DataServiceError::Io`. Signature: `impl_error_from!(source_type, target_enum::variant, intermediate_type)`.

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
- **Auth params resolution**: `resolve_auth_params(tenant_id, user_id, id)` returns `Option<McpAuthParams>` (headers + query params for upstream requests). Used by `mcp_proxy_handler` in `routes_app`. `AuthScopedMcpService.resolve_auth_params(id)` is the auth-scoped passthrough.

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

## Service Initialization Order

1. TimeService → 2. DbService → 3. SettingService → 4. AuthService → 5. SessionService → 6. TenantService → 7. HubService, DataService, CacheService → 8. ConcurrencyService, NetworkService → 9. AiApiService → 10. McpService → 11. TokenService → 12. InferenceService → 13. AccessRequestService → 14. QueueProducer
