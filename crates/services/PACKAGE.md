# PACKAGE.md - services Crate Implementation Index

*For architectural documentation and design rationale, see [crates/services/CLAUDE.md](crates/services/CLAUDE.md)*

## Module Structure

### Crate Root and Re-exports
- `src/lib.rs` - Crate root: module declarations organized by domain, re-exports all public types, re-exports `errmeta` types (`AppError`, `ErrorType`, `IoError`, `EntityError`, `RwLockReadError`, `impl_error_from!`) for downstream convenience
- `src/app_service.rs` - Central `AppService` trait and `DefaultAppService` implementation (16 service accessors)
- `src/macros.rs` - `asref_impl!` macro for service trait AsRef implementations
- `src/env_wrapper.rs` - Environment variable abstraction (`EnvWrapper` trait)

### Cross-Cutting Types (`shared_objs/`)
- `src/shared_objs/mod.rs` - Module declarations and re-exports
- `src/shared_objs/error_api.rs` - `ApiError` struct: captures `AppError` metadata, `From<T: AppError>`, converts to `OpenAIApiError`, implements `axum::IntoResponse`
- `src/shared_objs/error_oai.rs` - `OpenAIApiError`, `ErrorBody`: OpenAI-compatible error envelope with utoipa `ToSchema`
- `src/shared_objs/error_wrappers.rs` - `SerdeJsonError`, `SerdeYamlError`, `ReqwestError`, `JsonRejectionError`, `ObjValidationError`: framework-dependent error wrappers with `AppError` implementations
- `src/shared_objs/utils.rs` - `is_default()` helper, `ILLEGAL_CHARS` regex, `to_safe_filename()`
- `src/shared_objs/log.rs` - `mask_sensitive_value()`, `mask_form_params()`, `log_http_request()`, `log_http_response()`, `log_http_error()`

### Authentication and Security (`auth/`, `apps/`, `tokens/`)
- `src/auth/mod.rs` - Module declarations for auth domain
- `src/auth/auth_objs.rs` - **Domain types**: `ResourceRole` (User/PowerUser/Manager/Admin hierarchy), `TokenScope` (User/PowerUser), `UserScope` (User/PowerUser), `AppRole` (union type), `UserInfo`, `RoleError`, `TokenScopeError`, `UserScopeError`
- `src/auth/auth_service.rs` - `AuthService` trait and `DefaultAuthService`: OAuth2 PKCE flows, token exchange, dynamic client registration, access request consent
- `src/auth/session_service.rs` - `SessionService` trait and `DefaultSessionService` with `SessionStoreBackend`
- `src/auth/session_store.rs` - `SessionStoreBackend`, `InnerStoreShared`, `is_postgres_url()`
- `src/auth/session_error.rs` - `SessionServiceError` enum
- `src/auth/postgres.rs`, `src/auth/sqlite.rs` - Backend-specific session store creation
- `src/apps/mod.rs` - Module declarations for app instance domain
- `src/apps/app_objs.rs` - **Domain types**: `AppStatus` enum (`Setup`, `Ready`, `PreRegistered`, `ResourceAdmin`)
- `src/apps/app_instance_service.rs` - `AppInstanceService` trait and `DefaultAppInstanceService`
- `src/apps/error.rs` - `AppInstanceError` domain error enum
- `src/shared_objs/token.rs` - JWT token parsing, validation, and claims extraction (moved from `src/token.rs`)
- `src/tokens/mod.rs` - Module declarations for token domain
- `src/tokens/token_objs.rs` - **Domain types**: `TokenStatus` (Active/Inactive), `ApiTokenRow` (moved from `db/` shims)
- `src/tokens/token_service.rs` - Token management service
- `src/tokens/token_repository.rs` - Token repository trait

### Model and Data Management (`models/`)
- `src/models/mod.rs` - Module declarations for model domain
- `src/models/model_objs.rs` - **Domain types**: `Repo`, `HubFile`, `Alias` (User/Model/Api variants), `UserAlias`, `ModelAlias`, `ApiAlias`, `OAIRequestParams`, `ModelMetadata`, `JsonVec`, `DownloadStatus`, `BuilderError`, `ModelValidationError`, GGUF constants (`TOKENIZER_CONFIG_JSON`, `GGUF`, `GGUF_EXTENSION`)
- `src/models/gguf/` - GGUF format parsing module (header parsing, metadata extraction)
- `src/models/data_service.rs` - `DataService` trait: local model storage, alias management (User/Model/Api), three-tier resolution
- `src/models/hub_service.rs` - `HubService` trait: HuggingFace Hub API integration, local model discovery, GGUF file resolution
- `src/models/progress_tracking.rs` - Download progress monitoring with event broadcasting

### User Management (`users/`)
- `src/users/mod.rs` - Module declarations for user domain
- `src/users/user_objs.rs` - **Domain types**: `UserAccessRequestStatus` (Pending/Approved/Rejected) (moved from `db/` shims)
- `src/users/access_repository.rs` - User access request repository trait
- `src/users/access_request_entity.rs` - SeaORM entity for user access requests

### AI and Tool Services (`ai_apis/`, `toolsets/`)
- `src/ai_apis/ai_api_service.rs` - `AiApiService` trait: external AI API integration with test prompt, model listing, and request forwarding
- `src/ai_apis/error.rs` - `AiApiServiceError` domain error enum
- `src/toolsets/mod.rs` - Module declarations for toolset domain
- `src/toolsets/toolset_objs.rs` - **Domain types**: `Toolset`, `ToolsetScope`, `ToolsetType`, validation constants (`MAX_TOOLSET_SLUG_LEN`, etc.)
- `src/toolsets/tool_service.rs` - `ToolService` trait and `DefaultToolService` for LLM function calling
- `src/toolsets/exa_service.rs` - `ExaService`: Exa AI semantic search API integration
- `src/toolsets/error.rs` - `ToolsetError` and `ExaError` domain error enums
- `src/toolsets/execution.rs` - Toolset execution logic

### MCP Services (`mcps/`)
- `src/mcps/mod.rs` - Module declarations, re-exports `mcp_client::McpTool`
- `src/mcps/mcp_objs.rs` - **Domain types**: `McpServer`, `Mcp`, MCP auth config types, validation constants (`MAX_MCP_SLUG_LEN`, etc.)
- `src/mcps/mcp_service.rs` - `McpService` trait and `DefaultMcpService`: CRUD, tool discovery, execution
- `src/mcps/error.rs` - `McpError` domain error enum

### Access Control (`app_access_requests/`)
- `src/app_access_requests/mod.rs` - Module declarations for access request domain
- `src/app_access_requests/access_request_objs.rs` - **Domain types**: `AppAccessRequest`, `AppAccessResponse`, `AppAccessRequestDetail`, status enums
- `src/app_access_requests/access_request_service.rs` - `AccessRequestService` trait and `DefaultAccessRequestService` (role-based: `requested_role`/`approved_role`)
- `src/app_access_requests/error.rs` - `AccessRequestError` domain error enum

### Configuration (`settings/`)
- `src/settings/mod.rs` - Module declarations for settings domain
- `src/settings/setting_objs.rs` - **Domain types**: `Setting`, `EnvType`, `AppType`, `LogLevel`, `AppCommand`
- `src/settings/setting_service.rs` - `SettingService` trait
- `src/settings/default_service.rs` - `DefaultSettingService` implementation
- `src/settings/bootstrap_parts.rs` - `BootstrapParts` data carrier
- `src/settings/constants.rs` - Setting key constants
- `src/settings/error.rs` - `SettingsMetadataError` and `SettingServiceError` enums

### Utility Services (`utils/`)
- `src/utils/cache_service.rs` - Mini-moka based caching layer
- `src/utils/concurrency_service.rs` - Distributed lock abstraction with `LocalConcurrencyService`
- `src/utils/keyring_service.rs` - Platform-specific credential storage
- `src/utils/network_service.rs` - Network connectivity checks
- `src/utils/queue_service.rs` - Background metadata extraction queue with async processing

### Persistence (`db/`)
- `src/db/mod.rs` - Database module exports
- `src/db/service.rs` - `DbService` trait, repository trait definitions
- `src/db/time_service.rs` - `TimeService` trait and `DefaultTimeService` (all timestamp operations must use this, never `Utc::now()` directly)
- `src/db/default_service.rs` - `DefaultDbService` (SeaORM-based, the sole DbService implementation)
- `src/db/db_core.rs` - Database core utilities
- `src/db/encryption.rs` - Database-level API key encryption utilities
- `src/db/error.rs` - `DbError` with `ItemNotFound`, `StrumParse`, `TokenValidation`, `EncryptionError`, `PrefixExists`, `MultipleAppInstance`, `Conversion`
- `src/db/objs.rs` - Database row objects: `ApiKeyUpdate` (remaining after shim elimination; `TokenStatus`, `AppAccessRequestRow`, `UserAccessRequestStatus`, `ApiTokenRow` moved to domain modules)
- `src/db/entities/` - SeaORM entity definitions with populated `Relation` enums for FK relationships
- `src/db/sea_migrations/` - SeaORM migrations (14+ migration files, supporting both SQLite and PostgreSQL)
- `src/db/service_*.rs` - Repository implementations using SeaORM (model, token, access, mcp, toolset, user_alias, settings, app_instance, access_request)
- `src/db/model_repository.rs` - Model metadata repository trait
- `src/db/mcp_repository.rs` - MCP server and instance repository trait
- `src/db/toolset_repository.rs` - Toolset and app toolset config repository trait
- `src/db/access_repository.rs` - Access control repository trait
- `src/db/access_request_repository.rs` - App access request repository trait
- `src/db/user_alias_repository.rs` - User alias repository trait

### Test Utilities (`test-utils` feature)
- `src/test_utils/mod.rs` - Test fixture exports (17 sub-modules)
- `src/test_utils/app.rs` - `AppServiceStub` builder for full service composition testing
- `src/test_utils/auth.rs` - Authentication service mocks with embedded RSA keys
- `src/test_utils/bodhi.rs` - `temp_bodhi_home` fixture for isolated home directory testing
- `src/test_utils/data.rs` - Data service test helpers with temp directory fixtures
- `src/test_utils/db.rs` - `TestDbService`, `FrozenTimeService`, `MockDbService`, `test_db_service` fixture
- `src/test_utils/envs.rs` - `EnvWrapperStub` in-memory environment variable stub
- `src/test_utils/hf.rs` - `TestHfService`, `OfflineHubService` for HuggingFace mock
- `src/test_utils/http.rs` - HTTP test utilities
- `src/test_utils/io.rs` - IO test helpers
- `src/test_utils/logs.rs` - Log capture for test assertions
- `src/test_utils/fixtures.rs` - Domain object test builders (renamed from `objs.rs`)
- `src/test_utils/model_fixtures.rs` - Model-specific test fixture builders
- `src/test_utils/network.rs` - Network service test utilities
- `src/test_utils/queue.rs` - Queue service test helpers
- `src/test_utils/sea.rs` - `SeaTestContext`, `sea_context()` dual-database fixture
- `src/test_utils/session.rs` - Session service mocks
- `src/test_utils/settings.rs` - `bodhi_home_setting` and settings test configuration
- `src/test_utils/test_data.rs` - Static test data constants

## Key Implementation Patterns

### Service Registry Pattern
```rust
// src/app_service.rs - 16 services in the registry
#[cfg_attr(test, mockall::automock)]
pub trait AppService: std::fmt::Debug + Send + Sync {
  fn setting_service(&self) -> Arc<dyn SettingService>;
  fn hub_service(&self) -> Arc<dyn HubService>;
  fn data_service(&self) -> Arc<dyn DataService>;
  fn auth_service(&self) -> Arc<dyn AuthService>;
  fn db_service(&self) -> Arc<dyn DbService>;
  fn session_service(&self) -> Arc<dyn SessionService>;
  fn secret_service(&self) -> Arc<dyn SecretService>;
  fn cache_service(&self) -> Arc<dyn CacheService>;
  fn time_service(&self) -> Arc<dyn TimeService>;
  fn ai_api_service(&self) -> Arc<dyn AiApiService>;
  fn concurrency_service(&self) -> Arc<dyn ConcurrencyService>;
  fn queue_producer(&self) -> Arc<dyn QueueProducer>;
  fn tool_service(&self) -> Arc<dyn ToolService>;
  fn network_service(&self) -> Arc<dyn NetworkService>;
  fn access_request_service(&self) -> Arc<dyn AccessRequestService>;
  fn mcp_service(&self) -> Arc<dyn McpService>;
}
```

### Consolidated IO Error Usage
```rust
// Single IoError variant per service error enum
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum DataServiceError {
  #[error(transparent)]
  Io(#[from] IoError),
  // ... other domain-specific variants
}

// Bridge std::io::Error -> IoError -> DataServiceError::Io
impl_error_from!(::std::io::Error, DataServiceError::Io, ::errmeta::IoError);

// Usage with convenience constructors
fs::write(filename.clone(), contents)
  .map_err(|err| IoError::file_write(err, alias.config_filename().clone()))?;
fs::read(&path)
  .map_err(|err| IoError::file_read(err, path.display().to_string()))?;
```

### Database Upsert Return Value Pattern
`SettingsRepository::upsert_setting` constructs a `DbSetting` return value from the input and the computed `now` timestamp. On update (ON CONFLICT path), the returned struct's `created_at` field is set to `now` even though the database preserves the original `created_at` (it is excluded from the UPDATE SET clause). This is analogous to setting `id: 0` on a struct before insert -- the database assigns the real value but the returned struct carries a placeholder. Callers needing the actual `created_at` should query the database. See `src/db/service_settings.rs`.

### Alias Resolution Priority
```rust
// src/models/data_service.rs - Three-tier alias resolution
async fn find_alias(&self, alias: &str) -> Option<Alias> {
  // Priority 1: User aliases (YAML files)
  // Priority 2: Model aliases (auto-discovered GGUF files)
  // Priority 3: API aliases (database, prefix-aware routing)
}
```

### Error Enum Pattern with impl_error_from!
```rust
// Common pattern: bridge external errors via intermediate wrapper types
impl_error_from!(reqwest::Error, AuthServiceError::Reqwest, ::services::ReqwestError);
impl_error_from!(serde_yaml::Error, DataServiceError::SerdeYamlError, ::services::SerdeYamlError);
impl_error_from!(std::io::Error, HubServiceError::IoError, ::errmeta::IoError);
```

## Error Types by Service

| Service | Error Enum | IoError Variant | Other Key Variants |
|---------|-----------|-----------------|-------------------|
| DataService | `DataServiceError` | `Io(IoError)` | `DirMissing`, `DataFileNotFound`, `AliasNotExists`, `AliasExists`, `SerdeYaml`, `HubService`, `Db` |
| HubService | `HubServiceError` | `IoError(IoError)` | `HubApiError`, `HubFileNotFound`, `ObjValidationError` |
| SecretService | `SecretServiceError` | `IoError(IoError)` | `KeyMismatch`, `KeyNotFound`, `EncryptionError`, `DecryptionError` |
| SettingService | `SettingServiceError` | `Io(IoError)` | `SerdeYaml`, `LockError`, `InvalidSource` |
| AuthService | `AuthServiceError` | (none) | `Reqwest`, `AuthServiceApiError`, `TokenExchangeError` |
| DbService | `DbError` | (none) | `SeaOrmError`, `StrumParse`, `TokenValidation`, `EncryptionError`, `PrefixExists`, `ItemNotFound`, `MultipleAppInstance`, `Conversion` |
| SessionService | `SessionServiceError` | (none) | `SqlxError`, `SessionStoreError`, `DbSetup` |
| AiApiService | `AiApiServiceError` | (none) | `Reqwest`, `ApiError`, `Unauthorized`, `NotFound`, `RateLimit`, `PromptTooLong` |
| ToolService | `ToolsetError` | (none) | `ToolsetNotFound`, `MethodNotFound`, `ToolsetNotConfigured`, `ToolsetDisabled`, `ToolsetAppDisabled`, `SlugExists`, `InvalidSlug`, `InvalidDescription`, `InvalidToolsetType`, `DbError`, `ExaError` |
| McpService | `McpError` | (none) | `McpNotFound`, `McpUrlNotAllowed`, `McpDisabled`, `ToolNotAllowed`, `ToolNotFound`, `SlugExists`, `InvalidSlug`, `InvalidDescription`, `NameRequired`, `ConnectionFailed`, `ExecutionFailed`, `DbError` |
| ExaService | `ExaError` | (none) | `RequestFailed`, `RateLimited`, `InvalidApiKey`, `Timeout` |
| KeyringService | `KeyringError` | (none) | `KeyringError`, `DecodeError` |
| Token | `TokenError` | (none) | `InvalidToken`, `SerdeJson`, `InvalidIssuer`, `ScopeEmpty`, `Expired` |

## Crate Commands

### Building
```bash
cargo build -p services
cargo build -p services --features test-utils
```

### Testing
```bash
cargo test -p services
cargo test -p services --features test-utils
cargo test -p services -- --nocapture  # Show test output
```

### Documentation
```bash
cargo doc -p services --open
cargo doc -p services --features test-utils --open
```

## Usage Examples

### Service Initialization
```rust
use services::{DefaultAppService, DefaultTimeService};
use services::db::DefaultDbService;

let time_service = Arc::new(DefaultTimeService);
let db = Database::connect(&db_url).await?;
Migrator::fresh(&db).await?;
let db_service = Arc::new(DefaultDbService::new(
  db, time_service.clone(), encryption_key
));
```

### IO Error Handling in Services
```rust
use services::IoError;  // re-exported from errmeta

// File read with path context
let content = fs::read_to_string(&path)
  .map_err(|err| IoError::file_read(err, path.display().to_string()))?;

// Directory creation with path context
fs::create_dir_all(parent)
  .map_err(|err| IoError::dir_create(err, parent.display().to_string()))?;

// Bare std::io::Error auto-converts via impl_error_from! macro
let entries = fs::read_dir(&aliases_dir)?;
```

### Domain Type Usage
```rust
use services::{Repo, HubFile, Alias, ResourceRole, UserScope};

// Repo parsing with validation
let repo: Repo = "username/model-name".parse()?;

// Role hierarchy checks
assert!(ResourceRole::Admin.has_access_to(&ResourceRole::User));
assert!(UserScope::PowerUser.has_access_to(&UserScope::User));
```

## Test Infrastructure

### Test Organization

Domain modules use the sibling test file pattern (`test_*.rs`). Service modules vary between separate test files and inline tests:

| Module | Test Location |
|--------|--------------|
| `auth/auth_objs.rs` | `test_auth_objs_role.rs`, `test_auth_objs_token_scope.rs`, `test_auth_objs_user_scope.rs` |
| `models/model_objs.rs` | Inline and sibling test files |
| `shared_objs/` | Inline `mod tests` in `error_api.rs`, `error_wrappers.rs`, `utils.rs`, `log.rs` |
| `db` | `src/db/tests.rs` (separate file) |
| `settings` | Sibling test files |
| `toolsets` | Sibling test files |
| `auth` | Sibling test files |
| All other services | Inline `mod tests` at bottom of source file |

### Test Utilities (`test-utils` feature)

The `src/test_utils/` module provides reusable test infrastructure:

| File | Key Exports | Purpose |
|------|-------------|---------|
| `db.rs` | `TestDbService`, `FrozenTimeService`, `MockDbService`, `test_db_service` | TestDbService wraps `DefaultDbService` (SeaORM) with event broadcasting, frozen timestamps, composite mock |
| `sea.rs` | `SeaTestContext`, `sea_context()` | Dual-database test fixture (SQLite or PostgreSQL) with `DefaultDbService` and fresh migrations |
| `app.rs` | `AppServiceStub`, `AppServiceStubBuilder` | Full service composition for integration-style tests |
| `auth.rs` | `test_auth_service`, embedded RSA keys | AuthService with configurable base URL for mockito |
| `bodhi.rs` | `temp_bodhi_home` | Isolated bodhi home directory fixture |
| `data.rs` | Data service helpers | Temp directory fixtures for alias/model tests |
| `hf.rs` | `TestHfService`, `OfflineHubService` | HuggingFace mock with configurable real/mock modes |
| `fixtures.rs` | Domain object builders | Test data construction helpers (renamed from `objs.rs`) |
| `model_fixtures.rs` | Model fixture builders | Model-specific test fixture construction |
| `network.rs` | Network service test utils | Network service stubs and test helpers |
| `envs.rs` | `EnvWrapperStub` | In-memory environment variable stub |
| `settings.rs` | `bodhi_home_setting` | Setting service test configuration |
| `session.rs` | Session mocks | Session service test helpers |
| `http.rs` | HTTP test utilities | Request/response test helpers |
| `io.rs` | IO helpers | File system test utilities |
| `logs.rs` | Log capture | Tracing subscriber for test log assertions |
| `queue.rs` | Queue helpers | Queue service test infrastructure |
| `test_data.rs` | Static constants | SNAPSHOT hash and other test data |

### Canonical Test Pattern

All tests follow the standardized pattern:

- **Annotations**: `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]` (async) or `#[rstest]` only (sync)
- **Return type**: `-> anyhow::Result<()>` with `Ok(())` at end
- **Assertions**: `assert_eq!(expected, actual)` with `pretty_assertions`
- **Error handling**: `?` operator instead of `.unwrap()`, error code assertions via `.code()`
- **Fixtures**: `#[awt]` + `#[future]` only for async fixture params like `test_db_service`

For detailed patterns and migration checklists, see `.claude/skills/test-services/SKILL.md`.

## Feature Flags

- `test-utils`: Enables comprehensive test utilities, mock services, rstest fixtures, and `mcp_client/test-utils`
- `default = ["tokio"]`: Tokio runtime enabled by default

## Dependencies

### Core Dependencies
- `errmeta` - `AppError` trait, `ErrorType` enum, `IoError`, `EntityError`, `RwLockReadError`, `impl_error_from!` macro
- `errmeta_derive` - `#[derive(ErrorMeta)]` proc macro for error type generation
- `llama_server_proc` - LLM process management
- `mcp_client` - MCP protocol client for tool discovery and execution
- `sea-orm` - Database ORM (SQLite and PostgreSQL backends)
- `sea-orm-migration` - Schema migrations with dual-database support
- `axum` - HTTP framework integration (for `ApiError`, `JsonRejectionError`)
- `serde` / `serde_json` / `serde_yaml` - Serialization framework
- `utoipa` - OpenAPI schema generation (`ToSchema` derives on domain types)
- `validator` - Input validation (`Validate` derive on domain types)
- `strum` - Enum string conversion for domain enums
- `oauth2` - OAuth2 client
- `jsonwebtoken` - JWT handling
- `aes-gcm` / `pbkdf2` - Encryption and key derivation
- `keyring` - Platform credential storage (platform-specific features)
- `mini-moka` - In-memory caching
- `hf-hub` - HuggingFace API integration
- `reqwest` - HTTP client
- `walkdir` - Directory traversal for model discovery
- `ulid` - ULID-based ID generation (replaced UUID)
- `clap` - CLI argument parsing (for `AppCommand`)
- `derive_builder` - Builder pattern generation for domain types

### Optional Dependencies (test-utils)
- `mockall` - Mock generation for service traits
- `rstest` - Fixture-based testing
- `tempfile` - Temporary directories for test isolation
- `rsa` - RSA key pair generation for JWT testing
- `fs_extra` - File copy utilities for test data setup

## File References

See individual module files for complete implementation details:
- Service registry: `src/app_service.rs`
- Cross-cutting types: `src/shared_objs/*.rs` (ApiError, OpenAIApiError, error wrappers, utils, logging)
- Auth domain types + services: `src/auth/*.rs` (auth_objs, auth_service, session_*)
- Token domain types + services: `src/tokens/*.rs` (token_objs, token_service, token_repository)
- User domain types: `src/users/*.rs` (user_objs, access_repository, access_request_entity)
- App instance: `src/apps/*.rs` (app_objs, app_instance_service, error)
- Model domain types + services: `src/models/*.rs` (model_objs, data_service, hub_service, gguf/, progress_tracking)
- Settings domain types + services: `src/settings/*.rs` (setting_objs, setting_service, default_service, bootstrap_parts)
- Toolset domain types + services: `src/toolsets/*.rs` (toolset_objs, tool_service, exa_service, error, execution)
- MCP domain types + services: `src/mcps/*.rs` (mcp_objs, mcp_service, error)
- Access control: `src/app_access_requests/*.rs` (access_request_objs, access_request_service, error)
- Utility services: `src/utils/*.rs` (cache, concurrency, keyring, network, queue)
- Token handling: `src/shared_objs/token.rs` (JWT parsing), `src/tokens/*.rs` (token domain types + service)
- AI API: `src/ai_apis/*.rs` (ai_api_service, error)
- Database: `src/db/*.rs` (default_service, time_service, db_core, entities/, sea_migrations/, service_*.rs repository impls, *_repository.rs traits)
- Test utilities: `src/test_utils/*.rs`
