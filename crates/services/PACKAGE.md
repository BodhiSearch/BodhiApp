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
