# PACKAGE.md - services Crate Implementation Index

*For architectural documentation and design rationale, see [crates/services/CLAUDE.md](crates/services/CLAUDE.md)*

## Module Structure

### Core Service Registry
- `src/lib.rs` - Crate root with module exports and feature flags
- `src/app_service.rs` - Central `AppService` trait and `DefaultAppService` implementation (16 service accessors)
- `src/service_ext.rs` - Service extension utilities
- `src/macros.rs` - `asref_impl!` macro for service trait AsRef implementations

### Authentication and Security Services
- `src/auth_service.rs` - OAuth2 PKCE flows with Keycloak integration, token exchange, dynamic client registration
- `src/secret_service.rs` - AES-GCM encryption with PBKDF2 key derivation for secret storage
- `src/keyring_service.rs` - Platform-specific credential storage (Keychain, Secret Service, Windows Credential Manager)
- `src/session_service.rs` - SQLite-backed HTTP session management with `AppSessionStore` wrapper
- `src/token.rs` - JWT token parsing, validation, and claims extraction
- `src/concurrency_service.rs` - Distributed lock abstraction with `LocalConcurrencyService` for auth token refresh

### Model Management Services
- `src/hub_service.rs` - HuggingFace Hub API integration, local model discovery, GGUF file resolution
- `src/data_service.rs` - Local model storage, alias management (User/Model/Api), remote model listing
- `src/cache_service.rs` - Mini-moka based caching layer

### AI and Tool Services
- `src/ai_api_service.rs` - External AI API integration with test prompt, model listing, and request forwarding
- `src/tool_service/mod.rs` - Module root for toolset management
- `src/tool_service/service.rs` - `ToolService` trait and `DefaultToolService` implementation for LLM function calling
- `src/tool_service/error.rs` - `ToolsetError` domain error enum
- `src/tool_service/tests.rs` - Toolset service unit tests
- `src/exa_service.rs` - Exa AI semantic search API integration (search, find similar, contents, answer)

### MCP Services
- `src/mcp_service/mod.rs` - Module root for MCP server management
- `src/mcp_service/service.rs` - `McpService` trait and `DefaultMcpService` implementation for CRUD, tool discovery, execution
- `src/mcp_service/error.rs` - `McpError` domain error enum

### Access Control Services
- `src/access_request_service/mod.rs` - Module root for access request management
- `src/access_request_service/service.rs` - `AccessRequestService` trait and implementation
- `src/access_request_service/error.rs` - Access request error types

### Infrastructure Services
- `src/db/mod.rs` - Database module exports
- `src/db/service.rs` - SQLite operations with migration support, `TimeService` trait, `DbService` trait
- `src/db/sqlite_pool.rs` - Connection pool management
- `src/db/encryption.rs` - Database-level API key encryption utilities
- `src/db/error.rs` - Database error types (`SqlxError`, `SqlxMigrateError`, `ItemNotFound`)
- `src/db/objs.rs` - Database domain objects (`DownloadRequest`, `ApiToken`, `UserAccessRequest`, `ModelMetadataRow`, `McpRow`, `McpServerRow`, `ToolsetRow`, `AppToolsetConfigRow`, etc.)
- `src/db/mcp_repository.rs` - MCP server and instance persistence
- `src/db/toolset_repository.rs` - Toolset and app toolset config persistence
- `src/db/access_repository.rs` - Access control persistence
- `src/db/access_request_repository.rs` - User access request persistence
- `src/db/user_alias_repository.rs` - User alias persistence

### Configuration and Environment
- `src/setting_service.rs` - Application configuration management with `SettingsChangeListener` notification
- `src/env_wrapper.rs` - Environment variable abstraction
- `src/progress_tracking.rs` - Download progress monitoring
- `src/objs.rs` - Service-specific domain objects (`AppRegInfo`, `AppStatus`)
- `src/queue_service.rs` - Background metadata extraction queue with async processing

### Test Utilities (`test-utils` feature)
- `src/test_utils/mod.rs` - Test fixture exports
- `src/test_utils/app.rs` - `AppServiceStub` builder for full service composition testing
- `src/test_utils/auth.rs` - Authentication service mocks with embedded RSA keys
- `src/test_utils/data.rs` - Data service test helpers with temp directory fixtures
- `src/test_utils/db.rs` - Database test fixtures with event broadcasting
- `src/test_utils/envs.rs` - Environment test utilities
- `src/test_utils/hf.rs` - HuggingFace service mocks and `OfflineHubService`
- `src/test_utils/objs.rs` - Domain object test builders
- `src/test_utils/secret.rs` - Secret service stubs with in-memory storage
- `src/test_utils/session.rs` - Session service mocks
- `src/test_utils/settings.rs` - Settings test configuration

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
// src/data_service.rs - Single IoError variant per service error enum
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum DataServiceError {
  #[error(transparent)]
  Io(#[from] IoError),
  // ... other domain-specific variants
}

// Bridge std::io::Error -> IoError -> DataServiceError::Io
impl_error_from!(::std::io::Error, DataServiceError::Io, ::objs::IoError);

// Usage with convenience constructors
fs::write(filename.clone(), contents)
  .map_err(|err| IoError::file_write(err, alias.config_filename().clone()))?;
fs::read(&path)
  .map_err(|err| IoError::file_read(err, path.display().to_string()))?;
fs::remove_file(&filename)
  .map_err(|err| IoError::file_delete(err, filename))?;
fs::create_dir_all(parent)
  .map_err(|err| IoError::dir_create(err, parent.display().to_string()))?;
```

### Alias Resolution Priority
```rust
// src/data_service.rs - Three-tier alias resolution
async fn find_alias(&self, alias: &str) -> Option<Alias> {
  // Priority 1: User aliases (YAML files)
  if let Some(user_alias) = self.find_user_alias(alias) {
    return Some(Alias::User(user_alias));
  }
  // Priority 2: Model aliases (auto-discovered GGUF files)
  // Priority 3: API aliases (database, prefix-aware routing)
}
```

### Concurrency Control Pattern
```rust
// src/concurrency_service.rs
pub trait ConcurrencyService: Send + Sync + std::fmt::Debug {
  async fn with_lock_auth(
    &self,
    key: &str,
    f: Box<dyn FnOnce() -> BoxFuture<'static, AuthTokenResult> + Send + 'static>,
  ) -> AuthTokenResult;
}
```

### Error Enum Pattern with impl_error_from!
```rust
// Common pattern across services - bridge external errors via intermediate types
impl_error_from!(reqwest::Error, AuthServiceError::Reqwest, ::objs::ReqwestError);
impl_error_from!(serde_yaml::Error, DataServiceError::SerdeYamlError, ::objs::SerdeYamlError);
impl_error_from!(std::io::Error, HubServiceError::IoError, ::objs::IoError);
impl_error_from!(sqlx::Error, DbError::SqlxError, crate::db::SqlxError);
```

## Error Types by Service

| Service | Error Enum | IoError Variant | Other Key Variants |
|---------|-----------|-----------------|-------------------|
| DataService | `DataServiceError` | `Io(IoError)` | `DirMissing`, `DataFileNotFound`, `AliasNotExists`, `AliasExists`, `SerdeYaml`, `HubService`, `Db` |
| HubService | `HubServiceError` | `IoError(IoError)` | `HubApiError`, `HubFileNotFound`, `ObjValidationError` |
| SecretService | `SecretServiceError` | `IoError(IoError)` | `KeyMismatch`, `KeyNotFound`, `EncryptionError`, `DecryptionError` |
| SettingService | `SettingServiceError` | `Io(IoError)` | `SerdeYaml`, `LockError`, `InvalidSource` |
| AuthService | `AuthServiceError` | (none) | `Reqwest`, `AuthServiceApiError`, `TokenExchangeError` |
| DbService | `DbError` | (none) | `SqlxError`, `SqlxMigrateError`, `StrumParse`, `TokenValidation`, `PrefixExists` |
| SessionService | `SessionServiceError` | (none) | `SqlxError`, `SessionStoreError` |
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

let time_service = Arc::new(DefaultTimeService::new());
let db_service = Arc::new(SqliteDbService::new(pool, time_service.clone()));
let secret_service = Arc::new(DefaultSecretService::new(/* ... */));

let app_service = DefaultAppService::new(
  env_service, hub_service, data_service,
  auth_service, db_service, session_service,
  secret_service, cache_service, time_service,
  ai_api_service, concurrency_service,
  queue_producer, tool_service, network_service,
  access_request_service, mcp_service,
);
```

### IO Error Handling in Services
```rust
use objs::IoError;

// File read with path context
let content = fs::read_to_string(&path)
  .map_err(|err| IoError::file_read(err, path.display().to_string()))?;

// Directory creation with path context
fs::create_dir_all(parent)
  .map_err(|err| IoError::dir_create(err, parent.display().to_string()))?;

// Bare std::io::Error auto-converts via impl_error_from! macro
let entries = fs::read_dir(&aliases_dir)?;
```

### Model Management
```rust
use services::{HubService, DataService};

// Download model
let hub_file = hub_service
  .download(&repo, "model.gguf", None, Some(progress))
  .await?;

// Save user alias
let alias = UserAlias::new("my-model", repo, filename);
data_service.save_alias(&alias)?;

// Find alias with priority resolution
let alias = data_service.find_alias("gpt-4").await;
// Returns User > Model > Api alias priority
```

## Test Infrastructure

### Test Organization

All service modules contain inline `#[cfg(test)] mod tests` blocks. Three modules use separate test files:

| Module | Test Location |
|--------|--------------|
| `db` | `src/db/tests.rs` (separate file) |
| `setting_service` | `src/setting_service/tests.rs` (separate file) |
| `tool_service` | `src/tool_service/tests.rs` (separate file) |
| `mcp_service` | Inline in `src/mcp_service/service.rs` |
| `access_request_service` | Inline in `src/access_request_service/service.rs` |
| All other services | Inline `mod tests` at bottom of source file |

Additionally, `src/db/encryption.rs`, `src/db/error.rs`, and `src/db/sqlite_pool.rs` have inline test modules for their focused concerns.

### Modules with Inline Tests

- `src/auth_service.rs` -- OAuth2 flow tests with mockito
- `src/ai_api_service.rs` -- AI API tests with mockito
- `src/exa_service.rs` -- Exa search API tests with mockito
- `src/secret_service.rs` -- Encryption/decryption tests
- `src/hub_service.rs` -- HuggingFace integration tests
- `src/data_service.rs` -- Local model/alias management tests
- `src/session_service.rs` -- Session management tests
- `src/concurrency_service.rs` -- Per-key locking tests
- `src/progress_tracking.rs` -- Download progress tests with event broadcasting
- `src/queue_service.rs` -- Background queue tests
- `src/cache_service.rs` -- Cache layer tests
- `src/keyring_service.rs` -- Credential storage tests
- `src/token.rs` -- JWT parsing/validation tests
- `src/env_wrapper.rs` -- Environment variable tests
- `src/service_ext.rs` -- Service extension tests

### Test Utilities (`test-utils` feature)

The `src/test_utils/` module provides reusable test infrastructure:

| File | Key Exports | Purpose |
|------|-------------|---------|
| `db.rs` | `TestDbService`, `FrozenTimeService`, `MockDbService`, `test_db_service` | Real SQLite fixture with event broadcasting, frozen timestamps, composite mock |
| `app.rs` | `AppServiceStub`, `AppServiceStubBuilder` | Full service composition for integration-style tests |
| `auth.rs` | `test_auth_service`, embedded RSA keys | AuthService with configurable base URL for mockito |
| `data.rs` | Data service helpers | Temp directory fixtures for alias/model tests |
| `hf.rs` | `TestHfService`, `OfflineHubService` | HuggingFace mock with configurable real/mock modes |
| `secret.rs` | `SecretServiceStub`, `KeyringStoreStub` | In-memory secret/keyring storage |
| `session.rs` | Session mocks | Session service test helpers |
| `envs.rs` | `EnvWrapperStub` | In-memory environment variable stub |
| `objs.rs` | Domain object builders | Test data construction helpers |
| `settings.rs` | `bodhi_home_setting` | Setting service test configuration |

### Canonical Test Pattern

All tests follow the standardized pattern established in the services test revamp:

- **Annotations**: `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]` (async) or `#[rstest]` only (sync)
- **Return type**: `-> anyhow::Result<()>` with `Ok(())` at end
- **Assertions**: `assert_eq!(expected, actual)` with `pretty_assertions`
- **Error handling**: `?` operator instead of `.unwrap()`, error code assertions via `.code()`
- **Fixtures**: `#[awt]` + `#[future]` only for async fixture params like `test_db_service`

For detailed patterns and migration checklists, see `.claude/skills/test-services/SKILL.md`.

## Feature Flags

- `test-utils`: Enables comprehensive test utilities, mock services, and rstest fixtures
- `default = ["tokio"]`: Tokio runtime enabled by default

## Dependencies

### Core Dependencies
- `objs` - Domain objects, error types, `IoError` enum, `impl_error_from!` macro
- `llama_server_proc` - LLM process management
- `mcp_client` - MCP protocol client for tool discovery and execution
- `async-trait` - Async trait support
- `axum` - HTTP framework integration
- `sqlx` - Database operations (SQLite)
- `oauth2` - OAuth2 client
- `jsonwebtoken` - JWT handling
- `aes-gcm` / `pbkdf2` - Encryption and key derivation
- `keyring` - Platform credential storage (platform-specific features)
- `mini-moka` - In-memory caching
- `hf-hub` - HuggingFace API integration
- `reqwest` - HTTP client
- `walkdir` - Directory traversal for model discovery

### Optional Dependencies (test-utils)
- `mockall` - Mock generation for service traits
- `rstest` - Fixture-based testing
- `tempfile` - Temporary directories for test isolation
- `rsa` - RSA key pair generation for JWT testing
- `once_cell` - One-time initialization in test fixtures

## File References

See individual module files for complete implementation details:
- Service registry: `src/app_service.rs`
- Authentication: `src/auth_service.rs`, `src/session_service.rs`, `src/concurrency_service.rs`
- Security: `src/secret_service.rs`, `src/keyring_service.rs`
- Model management: `src/hub_service.rs`, `src/data_service.rs`
- AI integration: `src/ai_api_service.rs`, `src/tool_service/*.rs`, `src/exa_service.rs`
- MCP: `src/mcp_service/*.rs`
- Access control: `src/access_request_service/*.rs`
- Database: `src/db/*.rs` (includes `mcp_repository.rs`, `toolset_repository.rs`, `access_repository.rs`, `access_request_repository.rs`)
- Configuration: `src/setting_service.rs`, `src/env_wrapper.rs`
- Background processing: `src/queue_service.rs`, `src/progress_tracking.rs`
- Domain objects: `src/objs.rs`
- Token handling: `src/token.rs`
- Test utilities: `src/test_utils/*.rs`
