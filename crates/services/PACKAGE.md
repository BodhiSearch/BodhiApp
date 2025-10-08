# PACKAGE.md - services Crate Implementation Index

*For architectural documentation and design rationale, see [crates/services/CLAUDE.md](crates/services/CLAUDE.md)*

## Module Structure

### Core Service Registry
- `src/lib.rs` - Crate root with module exports and feature flags
- `src/app_service.rs` - Central AppService trait and DefaultAppService implementation
- `src/service_ext.rs` - Service extension utilities
- `src/macros.rs` - Service macro definitions

### Authentication & Security Services
- `src/auth_service.rs` - OAuth2 PKCE flows with Keycloak integration
- `src/secret_service.rs` - AES-GCM encryption with PBKDF2 key derivation
- `src/keyring_service.rs` - Platform-specific credential storage
- `src/session_service.rs` - SQLite-backed HTTP session management
- `src/token.rs` - JWT token utilities and validation

### Model Management Services
- `src/hub_service.rs` - HuggingFace Hub API integration
- `src/data_service.rs` - Local model storage and alias management
- `src/ai_api_service.rs` - External AI API integration with OpenAI compatibility
- `src/cache_service.rs` - Mini-moka based caching layer

### Infrastructure Services
- `src/db/mod.rs` - Database module exports
- `src/db/service.rs` - SQLite operations with migration support
- `src/db/sqlite_pool.rs` - Connection pool management
- `src/db/encryption.rs` - Database-level encryption utilities
- `src/db/error.rs` - Database error types
- `src/db/objs.rs` - Database domain objects

### Configuration & Environment
- `src/setting_service.rs` - Application configuration management
- `src/env_wrapper.rs` - Environment variable abstraction
- `src/progress_tracking.rs` - Download progress monitoring
- `src/objs.rs` - Service-specific domain objects

### Test Utilities (`test-utils` feature)
- `src/test_utils/mod.rs` - Test fixture exports
- `src/test_utils/app.rs` - AppService test builders
- `src/test_utils/auth.rs` - Authentication service mocks
- `src/test_utils/data.rs` - Data service test helpers
- `src/test_utils/db.rs` - Database test fixtures
- `src/test_utils/envs.rs` - Environment test utilities
- `src/test_utils/hf.rs` - HuggingFace service mocks
- `src/test_utils/objs.rs` - Domain object test builders
- `src/test_utils/secret.rs` - Secret service test helpers
- `src/test_utils/session.rs` - Session service mocks
- `src/test_utils/settings.rs` - Settings test configuration

## Key Implementation Examples

### Service Registry Pattern
```rust
// src/app_service.rs
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
  fn localization_service(&self) -> Arc<dyn LocalizationService>;
  fn time_service(&self) -> Arc<dyn TimeService>;
  fn ai_api_service(&self) -> Arc<dyn AiApiService>;
}

#[derive(Clone, Debug, derive_new::new)]
pub struct DefaultAppService {
  env_service: Arc<dyn SettingService>,
  hub_service: Arc<dyn HubService>,
  // ... all 11 services
}
```

### OAuth2 Authentication Flow
```rust
// src/auth_service.rs
#[async_trait]
pub trait AuthService: Send + Sync + std::fmt::Debug {
  async fn exchange_auth_code(
    &self,
    code: AuthorizationCode,
    client_id: ClientId,
    client_secret: ClientSecret,
    redirect_uri: RedirectUrl,
    code_verifier: PkceCodeVerifier,
  ) -> Result<(AccessToken, RefreshToken)>;

  async fn refresh_token(
    &self,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
  ) -> Result<(String, Option<String>)>;
}
```

### Multi-Layer Encryption
```rust
// src/secret_service.rs
pub async fn store_secret(&self, key: &str, value: &str) -> Result<()> {
  let mut salt = vec![0u8; SALT_SIZE];
  let mut nonce = vec![0u8; NONCE_SIZE];
  rng().fill_bytes(&mut salt);
  rng().fill_bytes(&mut nonce);

  let mut derived_key = [0u8; 32];
  pbkdf2_hmac::<Sha256>(&master_key, &salt, 1000, &mut derived_key);

  let cipher = Aes256Gcm::new(&derived_key);
  let encrypted = cipher.encrypt(&nonce, value.as_bytes())?;
  // Store with Base64 encoding
}
```

### Model Download Coordination
```rust
// src/hub_service.rs
#[async_trait]
pub trait HubService: Send + Sync + std::fmt::Debug {
  async fn download(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: Option<String>,
    progress: Option<Progress>,
  ) -> Result<HubFile, HubServiceError>;

  fn find_local_file(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: Option<String>,
  ) -> Result<HubFile, HubServiceError>;
}
```

### AI API Integration
```rust
// src/ai_api_service.rs
#[async_trait]
pub trait AiApiService: Send + Sync + std::fmt::Debug {
  async fn test_api_key(&self, alias: &ApiAlias) -> Result<()>;

  async fn chat_completion(
    &self,
    alias: &ApiAlias,
    request: CreateChatCompletionRequest,
  ) -> Result<Response>;

  async fn forward_chat_completion(
    &self,
    alias_id: &str,
    request: CreateChatCompletionRequest,
  ) -> Result<axum::Response>;
}
```

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
use services::{DefaultAppService, TimeService, DbService, SecretService};

// Initialize services in dependency order
let time_service = Arc::new(DefaultTimeService::new());
let db_service = Arc::new(SqliteDbService::new(pool, time_service.clone()));
let secret_service = Arc::new(DefaultSecretService::new(db_service.clone()));

let app_service = DefaultAppService::new(
  env_service,
  hub_service,
  data_service,
  auth_service,
  db_service,
  session_service,
  secret_service,
  cache_service,
  localization_service,
  time_service,
  ai_api_service,
);
```

### Authentication Workflow
```rust
use services::{AuthService, SessionService};

// OAuth2 code exchange
let (access_token, refresh_token) = auth_service
  .exchange_auth_code(code, client_id, client_secret, redirect_uri, verifier)
  .await?;

// Create session
let session_id = session_service
  .create_session(user_id, &access_token)
  .await?;
```

### Model Management
```rust
use services::{HubService, DataService};

// Download model
let hub_file = hub_service
  .download(&repo, "model.gguf", None, Some(progress))
  .await?;

// Save alias
let alias = UserAlias::new("my-model", repo, filename);
data_service.save_alias(&alias)?;
```

## Feature Flags

- `test-utils`: Enables comprehensive test utilities and mock services

## Dependencies

### Core Dependencies
- `async-trait`: Async trait support
- `axum`: HTTP framework integration
- `sqlx`: Database operations
- `oauth2`: OAuth2 client
- `jsonwebtoken`: JWT handling
- `aes-gcm`: Encryption
- `pbkdf2`: Key derivation
- `keyring`: Platform credential storage
- `mini-moka`: Caching
- `hf-hub`: HuggingFace API

### Optional Dependencies (test-utils)
- `mockall`: Mock generation
- `rstest`: Fixture-based testing
- `tempfile`: Temporary directories

## File References

See individual module files for complete implementation details:
- Service registry: `src/app_service.rs`
- Authentication: `src/auth_service.rs`, `src/session_service.rs`
- Security: `src/secret_service.rs`, `src/keyring_service.rs`
- Model management: `src/hub_service.rs`, `src/data_service.rs`
- Database: `src/db/*.rs`
- AI APIs: `src/ai_api_service.rs`
- Configuration: `src/setting_service.rs`
- Test utilities: `src/test_utils/*.rs`