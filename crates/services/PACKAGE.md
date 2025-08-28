# PACKAGE.md - services

This document provides detailed technical information for the `services` crate, focusing on BodhiApp-specific service implementation patterns and coordination mechanisms.

## Service Registry Implementation

### AppService Pattern
BodhiApp's comprehensive service registry provides centralized dependency injection for 10 services:

```rust
// Pattern structure (see src/app_service.rs:8-25 for complete trait definition)
#[cfg_attr(test, mockall::automock)]
pub trait AppService: std::fmt::Debug + Send + Sync {
  fn setting_service(&self) -> Arc<dyn SettingService>;
  fn hub_service(&self) -> Arc<dyn HubService>;
  fn auth_service(&self) -> Arc<dyn AuthService>;
  fn db_service(&self) -> Arc<dyn DbService>;
  fn session_service(&self) -> Arc<dyn SessionService>;
  fn secret_service(&self) -> Arc<dyn SecretService>;
  fn cache_service(&self) -> Arc<dyn CacheService>;
  fn localization_service(&self) -> Arc<dyn LocalizationService>;
  fn time_service(&self) -> Arc<dyn TimeService>;
  // Complete service registry with all dependencies
}

// Implementation with derive_new pattern (see src/app_service.rs:27-40)
#[derive(Clone, Debug, derive_new::new)]
pub struct DefaultAppService {
  env_service: Arc<dyn SettingService>,
  hub_service: Arc<dyn HubService>,
  data_service: Arc<dyn DataService>,
  auth_service: Arc<dyn AuthService>,
  db_service: Arc<dyn DbService>,
  session_service: Arc<dyn SessionService>,
  secret_service: Arc<dyn SecretService>,
  cache_service: Arc<dyn CacheService>,
  localization_service: Arc<dyn LocalizationService>,
  time_service: Arc<dyn TimeService>,
}
```

**Key Implementation Details**:
- All services wrapped in `Arc<dyn Trait>` for thread-safe sharing across async contexts
- Service registration happens at application startup with derive_new constructor pattern
- Dependency injection enables comprehensive testing with mock services via mockall
- Service interdependencies managed through registry pattern with proper lifecycle management

## Authentication Flow Implementation

### OAuth2 PKCE Flow
BodhiApp implements complete OAuth2 authorization with PKCE security and Keycloak integration:

```rust
// Core authentication patterns (see src/auth_service.rs:45-89 for complete trait)
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait AuthService: Send + Sync + std::fmt::Debug {
  async fn register_client(&self, name: String, description: String, redirect_uris: Vec<String>) -> Result<AppRegInfo>;
  async fn exchange_auth_code(&self, code: AuthorizationCode, client_id: ClientId, client_secret: ClientSecret, redirect_uri: RedirectUrl, code_verifier: PkceCodeVerifier) -> Result<(AccessToken, RefreshToken)>;
  async fn refresh_token(&self, client_id: &str, client_secret: &str, refresh_token: &str) -> Result<(String, Option<String>)>;
  async fn exchange_app_token(&self, client_id: &str, client_secret: &str, subject_token: &str, scopes: Vec<String>) -> Result<(String, Option<String>)>;
  async fn make_resource_admin(&self, client_id: &str, client_secret: &str, email: &str) -> Result<()>;
  async fn request_access(&self, client_id: &str, client_secret: &str, app_client_id: &str) -> Result<String>;
}

// Keycloak implementation with custom Bodhi endpoints (see src/auth_service.rs:91-156)
#[derive(Debug)]
pub struct KeycloakAuthService {
  app_version: String,
  auth_url: String,
  realm: String,
  client: reqwest::Client,
}
```
```

### Token Exchange Protocol
Service-to-service authentication through RFC 8693 token exchange:

```rust
// Token exchange implementation (see src/auth_service.rs:289-325 for complete implementation)
async fn exchange_app_token(&self, client_id: &str, client_secret: &str, subject_token: &str, scopes: Vec<String>) -> Result<(String, Option<String>)> {
  let params = [
    ("grant_type", "urn:ietf:params:oauth:grant-type:token-exchange"),
    ("subject_token", subject_token),
    ("client_id", client_id),
    ("client_secret", client_secret),
    ("audience", client_id),
    ("scope", &scopes.join(" ")),
  ];
  // HTTP request with proper error handling and logging
}

// Resource administration (see src/auth_service.rs:401-445)
async fn make_resource_admin(&self, client_id: &str, client_secret: &str, email: &str) -> Result<()> {
  let access_token = self.get_client_access_token(client_id, client_secret).await?;
  // Custom Bodhi API endpoint for resource admin assignment
}
```

**Implementation Notes**:
- PKCE code verifier required for all authorization flows with proper validation
- JWT tokens with automatic refresh mechanism and expiration handling
- Token exchange enables service-to-service authentication following RFC 8693 standards
- Integration with Keycloak OAuth2 provider using custom Bodhi API endpoints
- Comprehensive error handling with localized messages and HTTP request logging

## Multi-Layer Security Implementation

### Secret Service Encryption
AES-GCM encryption with secure key derivation and comprehensive error handling:

```rust
// Core encryption pattern (see src/secret_service.rs:45-89 for complete implementation)
impl SecretService for DefaultSecretService {
  async fn store_secret(&self, key: &str, value: &str) -> Result<()> {
    let mut salt = vec![0u8; SALT_SIZE];
    let mut nonce = vec![0u8; NONCE_SIZE];
    rng().fill_bytes(&mut salt);
    rng().fill_bytes(&mut nonce);
    
    let mut derived_key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(&master_key, &salt, PBKDF2_ITERATIONS, &mut derived_key);
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&derived_key));
    let encrypted = cipher.encrypt(Nonce::from_slice(&nonce), value.as_bytes())?;
    
    let encrypted_data = EncryptedData { salt, nonce, data: encrypted };
    // Store with Base64 encoding and proper error handling
  }

  async fn retrieve_secret(&self, key: &str) -> Result<Option<String>> {
    // Decrypt with same key derivation and comprehensive error handling
  }
}
```

**Security Implementation**:
- PBKDF2 key derivation with 1000 iterations and cryptographically secure random salts
- Unique nonce per encryption operation using secure random number generation
- Base64 encoding for storage and transport with proper error handling
- Never store plaintext secrets in memory or logs with comprehensive parameter masking
- Integration with platform keyring services for persistent credential storage

### Platform Keyring Integration
Secure credential storage using platform-specific keyrings:

```rust
impl KeyringService for DefaultKeyringService {
  async fn store_credential(&self, service: &str, account: &str, password: &str) -> Result<()> {
    let entry = Entry::new(service, account)?;
    entry.set_password(password)?;
    Ok(())
  }
}
```

## Model Management Implementation

### Hub Service Integration
Sophisticated Hugging Face Hub integration with comprehensive error recovery and offline testing:

```rust
// Core hub service pattern (see src/hub_service.rs:45-89 for complete trait)
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait HubService: Send + Sync + std::fmt::Debug {
  async fn download_model(&self, repo: &Repo, snapshot: &str) -> Result<Vec<HubFile>, HubServiceError>;
  async fn list_models(&self, query: &str) -> Result<Vec<String>, HubServiceError>;
  // Additional methods for model discovery and validation
}

// Production implementation with comprehensive error handling
impl HubService for HfHubService {
  async fn download_model(&self, repo: &Repo, snapshot: &str) -> Result<Vec<HubFile>> {
    match self.cache.repo(repo.to_string()).revision(snapshot).get(&filename) {
      Ok(cache_file) => Ok(vec![HubFile::try_from(cache_file.path())?]),
      Err(hf_hub::Error::Http { code: 401, .. }) => {
        Err(HubApiError::new("Gated repository requires authentication".to_string(), 401, repo.to_string(), HubApiErrorKind::GatedAccess))
      }
      Err(hf_hub::Error::Http { code: 404, .. }) => {
        Err(HubApiError::new("Repository or file not found".to_string(), 404, repo.to_string(), HubApiErrorKind::NotExists))
      }
      Err(hf_hub::Error::Http { code, .. }) => {
        Err(HubApiError::new(format!("HTTP error: {}", code), code, repo.to_string(), HubApiErrorKind::Transport))
      }
    }
  }
}

// Offline testing implementation (see src/test_utils/hf.rs:45-89)
#[derive(Debug, Default)]
pub struct OfflineHubService {
  inner: HfHubService,
  test_data: HashMap<String, Vec<HubFile>>,
}
```

**Implementation Notes**:
- Gated repository handling with authentication token management and comprehensive error categorization
- Network error recovery with retry logic and exponential backoff for transient failures
- Local cache validation and reuse with Hugging Face cache structure validation
- Progress tracking for long downloads with status reporting and cancellation support
- OfflineHubService for testing without external dependencies using local test data

### Data Service Coordination
File system operations coordinated with metadata management:

```rust
impl DataService for LocalDataService {
  fn save_alias(&self, alias: &Alias) -> Result<PathBuf> {
    let config_path = self.aliases_dir.join(alias.config_filename());
    
    // Atomic write operation
    let temp_path = config_path.with_extension("tmp");
    let yaml_content = serde_yaml::to_string(alias)?;
    fs::write(&temp_path, yaml_content)?;
    fs::rename(&temp_path, &config_path)?;
    
    Ok(config_path)
  }
}
```

## Database Transaction System

### Migration Management
Versioned database schema evolution with comprehensive migration support:

```rust
// Migration pattern (see src/db/service.rs:45-67 for complete implementation)
impl DbService for SqliteDbService {
  async fn migrate(&self) -> Result<(), DbError> {
    sqlx::migrate!("./migrations").run(&self.pool).await?;
    Ok(())
  }

  async fn pool(&self) -> &SqlitePool {
    &self.pool // Connection pool access for service coordination
  }
}
```

**Migration Files Structure** (see migrations/ directory):
```text
migrations/
├── 0001_download-requests.up.sql      # Download request tracking
├── 0001_download-requests.down.sql    # Rollback support
├── 0002_pending-access-requests.up.sql # Access request management
├── 0002_pending-access-requests.down.sql
└── 0003_create_api_tokens.up.sql      # API token storage
```

### Transaction Coordination
Cross-service transaction support:

```rust
impl DbService for SqliteDbService {
  async fn create_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    let mut tx = self.pool.begin().await?;
    
    query_as!(
      DownloadRequest,
      "INSERT INTO download_requests (id, repo, filename, snapshot, status, created_at) VALUES (?, ?, ?, ?, ?, ?)",
      request.id, request.repo.to_string(), request.filename, request.snapshot, 
      request.status.to_string(), request.created_at
    ).execute(&mut *tx).await?;
    
    tx.commit().await?;
    Ok(())
  }
}
```

### Time Service Abstraction
Testable time operations with deterministic testing and comprehensive time management:

```rust
// Production time service (see src/db/service.rs:234-267 for complete implementation)
pub struct DefaultTimeService;

impl TimeService for DefaultTimeService {
  fn utc_now(&self) -> DateTime<Utc> {
    let now = chrono::Utc::now();
    now.with_nanosecond(0).unwrap_or(now) // Remove nanoseconds for database consistency
  }

  fn created_at(&self, path: &Path) -> u32 {
    // File creation time for model metadata
  }
}

// Test implementation with frozen time (see src/test_utils/db.rs:45-67)
#[derive(Debug)]
pub struct FrozenTimeService(DateTime<Utc>);

impl Default for FrozenTimeService {
  fn default() -> Self {
    FrozenTimeService(chrono::Utc::now().with_nanosecond(0).unwrap())
  }
}

impl TimeService for FrozenTimeService {
  fn utc_now(&self) -> DateTime<Utc> {
    self.0 // Always returns the same time for deterministic testing
  }

  fn created_at(&self, _path: &Path) -> u32 {
    0 // Deterministic file creation time
  }
}
```

## Session Management Implementation

### SQLite Session Store
HTTP session management with database backend:

```rust
impl SessionService for SqliteSessionService {
  fn session_layer(&self) -> SessionManagerLayer<SqliteStore> {
    SessionManagerLayer::new(self.session_store.clone())
      .with_secure(false) // TODO: Enable when HTTPS supported
      .with_same_site(SameSite::Strict) // Prevents CSRF attacks
      .with_name("bodhiapp_session_id")
  }
}
```

**Session Security Configuration**:
- SameSite::Strict prevents cross-origin requests
- Session data encrypted and stored in SQLite
- Automatic session cleanup and expiration
- Integration with authentication state

## Cache Service Implementation

### Mini-Moka Integration
High-performance caching with TTL support:

```rust
impl CacheService for MokaCacheService {
  fn get(&self, key: &str) -> Option<String> {
    self.cache.get(&key.to_string())
  }

  fn set(&self, key: &str, value: &str) {
    self.cache.insert(key.to_string(), value.to_string());
  }
}
```

**Caching Strategy**:
- In-memory cache for frequently accessed data
- Automatic eviction based on size and TTL
- Thread-safe concurrent access
- Cache invalidation coordinated with data changes

## Error Handling Patterns

### Service-Specific Error Types
Each service defines domain-specific errors:

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum HubServiceError {
  #[error(transparent)]
  HubApiError(#[from] HubApiError),
  
  #[error("hub_file_missing")]
  #[error_meta(error_type = ErrorType::NotFound)]
  HubFileNotFound(#[from] HubFileNotFoundError),
}
```

### Error Propagation Macros
Automatic error conversion between service boundaries:

```rust
impl_error_from!(
  reqwest::Error,
  AuthServiceError::Reqwest,
  ::objs::ReqwestError
);
```

## CLI Command Service Orchestration

### Multi-Service CLI Coordination
Commands crate orchestrates multiple services for complex CLI workflows:

```rust
// CLI commands coordinate services through AppService registry (see crates/commands/src/cmd_create.rs:45-89)
impl CreateCommand {
    pub async fn execute(self, service: Arc<dyn AppService>) -> Result<()> {
        // DataService coordination for alias conflict detection
        if service.data_service().find_alias(&self.alias).is_some() && !self.update {
            return Err(AliasExistsError(self.alias.clone()).into());
        }
        
        // HubService coordination for model file management
        let file_exists = service.hub_service()
            .local_file_exists(&self.repo, &self.filename, self.snapshot.clone())?;
        
        // Multi-service workflow with auto-download coordination
        let local_model_file = match file_exists {
            true => service.hub_service().find_local_file(/* ... */)?,
            false if self.auto_download => service.hub_service().download(/* ... */).await?,
            false => return Err(/* HubFileNotFoundError */),
        };
        
        // Alias creation and persistence coordination
        service.data_service().save_alias(&alias)?;
        Ok(())
    }
}
```

**CLI Service Integration Features**:
- Multi-service workflow orchestration through AppService registry pattern
- CLI-specific error translation with transparent service error wrapping
- Progress feedback coordination for long-running download operations
- Service mock composition for comprehensive CLI command testing scenarios

## Extension Guidelines

### Adding New Services
When implementing new business services:

1. **Define service trait** with async methods and proper error handling
2. **Implement concrete service** with dependency injection support  
3. **Add mock generation** via `#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]`
4. **Register in AppService** trait and DefaultAppService implementation
5. **Add error types** with localization support via `errmeta_derive`

### Database Schema Changes
For database modifications:

1. **Create migration files** with both up and down SQL
2. **Update service methods** to handle new schema
3. **Test migrations** with existing data in development
4. **Coordinate with related services** that depend on the data

### External API Integration
When integrating with external services:

1. **Implement retry logic** with exponential backoff
2. **Handle API rate limiting** and quota restrictions
3. **Add proper error categorization** (network, auth, not found, etc.)
4. **Cache responses** when appropriate for performance
5. **Add comprehensive testing** with mock HTTP clients

## Testing Infrastructure

### Service Composition Testing
Complex service interactions require sophisticated test setup with comprehensive service coordination:

```rust
// Service composition testing pattern (see src/test_utils/app.rs:23-45 for complete fixture)
#[fixture]
#[awt]
pub async fn app_service_stub(#[future] app_service_stub_builder: AppServiceStubBuilder) -> AppServiceStub {
  app_service_stub_builder.build().unwrap()
}

#[fixture]
#[awt]
pub async fn app_service_stub_builder(#[future] test_db_service: TestDbService) -> AppServiceStubBuilder {
  AppServiceStubBuilder::default()
    .with_hub_service()
    .with_data_service()
    .db_service(Arc::new(test_db_service))
    .with_session_service().await
    .with_secret_service()
    .to_owned()
}

// Integration testing with service coordination
#[rstest]
#[awt]
async fn test_authentication_flow(#[future] app_service_stub: AppServiceStub) -> Result<(), Box<dyn std::error::Error>> {
  let auth_service = app_service_stub.auth_service();
  let db_service = app_service_stub.db_service();
  let secret_service = app_service_stub.secret_service();
  
  // Test complete authentication flow across services
  let app_reg = auth_service.register_client("Test App".to_string(), "Test Description".to_string(), vec!["http://localhost:8080/callback".to_string()]).await?;
  let (access_token, refresh_token) = auth_service.exchange_auth_code(/* OAuth2 parameters */).await?;
  
  // Verify cross-service coordination
  let stored_token = db_service.get_api_token(&token_id).await?;
  assert!(stored_token.is_some());
  
  let stored_credentials = secret_service.retrieve_secret("app_reg_info").await?;
  assert!(stored_credentials.is_some());
  
  Ok(())
}
```

### Mock Service Coordination
Testing service interdependencies with coordinated mocks:

```rust
let mut mock_hub_service = MockHubService::new();
mock_hub_service
  .expect_download_model()
  .returning(|repo, snapshot| Ok(vec![test_hub_file()]));

let mut mock_data_service = MockDataService::new();
mock_data_service
  .expect_save_alias()
  .returning(|alias| Ok(PathBuf::from("/tmp/alias.yaml")));
```

## Commands

**Testing**: `cargo test -p services` (includes database and mock service tests)  
**Building**: Standard `cargo build -p services`  
**Migration**: Via `DbService::migrate()` method in application startup