# PACKAGE.md - Services Crate

*For architectural insights and design decisions, see [crates/services/CLAUDE.md](crates/services/CLAUDE.md)*

## Implementation Index

The `services` crate implements BodhiApp's comprehensive business logic layer through sophisticated service orchestration patterns, multi-layer security, and cross-service coordination.

### Core Service Files

**src/lib.rs**
- Module exports and feature-gated test utilities
- Comprehensive service re-exports for unified API access

**src/app_service.rs**
- AppService trait defining the central service registry
- DefaultAppService implementation with dependency injection
- Comprehensive service composition for all 11 business services

**src/auth_service.rs**
- OAuth2 PKCE authentication with Keycloak integration
- JWT token management and refresh mechanisms
- User access request and resource administration workflows

**src/hub_service.rs**
- HuggingFace Hub API integration with error categorization
- Model download coordination and local cache management
- Gated repository handling with authentication token support

**src/data_service.rs**
- File system operations for model alias and metadata management
- YAML-based configuration with atomic write operations
- Local model discovery and validation with GGUF support

**src/db/service.rs**
- SQLite database operations with migration management
- Download request tracking and user access control
- Transaction coordination and connection pooling

### Service Architecture Examples

#### Service Registry Pattern

```rust
  #[cfg_attr(test, mockall::automock)]
  pub trait AppService: std::fmt::Debug + Send + Sync {
    fn setting_service(&self) -> Arc<dyn SettingService>;
    fn data_service(&self) -> Arc<dyn DataService>;
    fn hub_service(&self) -> Arc<dyn HubService>;
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
    data_service: Arc<dyn DataService>,
    auth_service: Arc<dyn AuthService>,
    db_service: Arc<dyn DbService>,
    session_service: Arc<dyn SessionService>,
    secret_service: Arc<dyn SecretService>,
    cache_service: Arc<dyn CacheService>,
    localization_service: Arc<dyn LocalizationService>,
    time_service: Arc<dyn TimeService>,
    ai_api_service: Arc<dyn AiApiService>,
  }
```

#### OAuth2 Authentication Flow

```rust
  #[async_trait]
  pub trait AuthService: Send + Sync + std::fmt::Debug {
    async fn register_client(
      &self,
      name: String,
      description: String,
      redirect_uris: Vec<String>
    ) -> Result<AppRegInfo>;

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

    async fn exchange_app_token(
      &self,
      client_id: &str,
      client_secret: &str,
      subject_token: &str,
      scopes: Vec<String>,
    ) -> Result<(String, Option<String>)>;
  }
```

#### AI API Service Integration

```rust
  #[async_trait]
  pub trait AiApiService: Send + Sync + std::fmt::Debug {
    async fn test_api_key(&self, alias: &ApiAlias) -> Result<()>;

    async fn test_model(
      &self,
      alias: &ApiAlias,
      model_name: &str
    ) -> Result<()>;

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

## Multi-Layer Security Implementation

### Secret Service Encryption

```rust
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
      // Store with Base64 encoding
    }

    async fn retrieve_secret(&self, key: &str) -> Result<Option<String>> {
      // Decrypt with proper key derivation and error handling
    }
  }
```

### Platform Keyring Integration

```rust
  impl KeyringService for DefaultKeyringService {
    async fn store_credential(
      &self,
      service: &str,
      account: &str,
      password: &str,
    ) -> Result<()> {
      let entry = Entry::new(service, account)?;
      entry.set_password(password)?;
      Ok(())
    }

    async fn retrieve_credential(
      &self,
      service: &str,
      account: &str,
    ) -> Result<Option<String>> {
      let entry = Entry::new(service, account)?;
      match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(KeyringServiceError::KeyringError(e.to_string())),
      }
    }
  }
```

## Database Transaction System

### Migration Management

```rust
  impl DbService for SqliteDbService {
    async fn migrate(&self) -> Result<(), DbError> {
      sqlx::migrate!("./migrations").run(&self.pool).await?;
      Ok(())
    }

    async fn pool(&self) -> &SqlitePool {
      &self.pool
    }
  }
```

### Download Request Management

```rust
  impl DbService for SqliteDbService {
    async fn create_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
      let mut tx = self.pool.begin().await?;

      query!(
        r#"INSERT INTO download_requests
           (id, repo, filename, status, created_at, updated_at, total_bytes, downloaded_bytes)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        request.id,
        request.repo,
        request.filename,
        request.status.to_string(),
        request.created_at,
        request.updated_at,
        request.total_bytes,
        request.downloaded_bytes
      ).execute(&mut *tx).await?;

      tx.commit().await?;
      Ok(())
    }

    async fn update_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
      query!(
        r#"UPDATE download_requests
           SET status = ?, updated_at = ?, total_bytes = ?, downloaded_bytes = ?
           WHERE id = ?"#,
        request.status.to_string(),
        request.updated_at,
        request.total_bytes,
        request.downloaded_bytes,
        request.id
      ).execute(&self.pool).await?;
      Ok(())
    }
  }
```

### User Access Request Management

```rust
  impl DbService for SqliteDbService {
    async fn create_user_access_request(
      &self,
      request: &UserAccessRequest,
    ) -> Result<(), DbError> {
      query!(
        r#"INSERT INTO user_access_requests
           (username, user_id, reason, status, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?)"#,
        request.username,
        request.user_id,
        request.reason,
        request.status.to_string(),
        request.created_at,
        request.updated_at
      ).execute(&self.pool).await?;
      Ok(())
    }

    async fn update_user_access_request_status(
      &self,
      id: i64,
      status: UserAccessRequestStatus,
      updated_at: DateTime<Utc>,
    ) -> Result<(), DbError> {
      query!(
        "UPDATE user_access_requests SET status = ?, updated_at = ? WHERE id = ?",
        status.to_string(),
        updated_at,
        id
      ).execute(&self.pool).await?;
      Ok(())
    }
  }
```

## Model Management Coordination

### Hub Service Implementation

```rust
  #[async_trait]
  pub trait HubService: Send + Sync + std::fmt::Debug {
    async fn download(
      &self,
      repo: &Repo,
      filename: &str,
      snapshot: Option<String>,
      progress: Option<Progress>,
    ) -> Result<HubFile, HubServiceError>;

    fn local_file_exists(
      &self,
      repo: &Repo,
      filename: &str,
      snapshot: Option<String>,
    ) -> Result<bool, HubServiceError>;

    fn find_local_file(
      &self,
      repo: &Repo,
      filename: &str,
      snapshot: Option<String>,
    ) -> Result<HubFile, HubServiceError>;
  }
```

### Error Handling Implementation

```rust
  #[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
  #[error_meta(trait_to_impl = AppError)]
  pub enum HubServiceError {
    #[error(transparent)]
    HubApiError(#[from] HubApiError),

    #[error("hub_file_missing")]
    #[error_meta(error_type = ErrorType::NotFound)]
    HubFileNotFound(#[from] HubFileNotFoundError),

    #[error("io_error")]
    #[error_meta(error_type = ErrorType::InternalError)]
    IoError(#[from] std::io::Error),

    #[error("invalid_hub_file")]
    #[error_meta(error_type = ErrorType::ValidationError)]
    InvalidHubFile(String),
  }
```

## Session Management

### SQLite Session Store

```rust
  impl SessionService for SqliteSessionService {
    fn session_layer(&self) -> SessionManagerLayer<SqliteStore> {
      SessionManagerLayer::new(self.session_store.clone())
        .with_secure(false) // TODO: Enable when HTTPS supported
        .with_same_site(SameSite::Strict)
        .with_name("bodhiapp_session_id")
    }
  }
```

## Testing Infrastructure

### Service Composition Testing

```rust
  #[fixture]
  pub fn test_config(temp_dir: TempDir) -> (NapiAppOptions, TempDir) {
    let bodhi_home = temp_dir.path().to_string_lossy().to_string();
    let port = rand::rng().random_range(20000..30000);

    let mut config = create_napi_app_options();
    config = set_env_var(config, BODHI_HOME.to_string(), bodhi_home);
    config = set_env_var(config, BODHI_HOST.to_string(), "127.0.0.1".to_string());
    config = set_env_var(config, BODHI_PORT.to_string(), port.to_string());

    // System settings for complete setup
    config = set_system_setting(config, BODHI_ENV_TYPE.to_string(), "development".to_string());
    config = set_system_setting(config, BODHI_APP_TYPE.to_string(), "container".to_string());

    (config, temp_dir)
  }
```

### Mock Service Coordination

```rust
  #[rstest]
  #[tokio::test]
  async fn test_multi_service_workflow() -> anyhow::Result<()> {
    let mut mock_hub_service = MockHubService::new();
    mock_hub_service
      .expect_download()
      .with(eq(Repo::testalias()), eq("model.gguf"), eq(None), always())
      .return_once(|_, _, _, _| Ok(HubFile::testalias()));

    let mut mock_data_service = MockDataService::new();
    mock_data_service
      .expect_save_alias()
      .with(function(|alias: &UserAlias| alias.alias == "test"))
      .return_once(|_| Ok(PathBuf::from("/tmp/alias.yaml")));

    let service = AppServiceStub::new()
      .with_hub_service(Arc::new(mock_hub_service))
      .with_data_service(Arc::new(mock_data_service))
      .build()?;

    // Test coordinated service operations
    let hub_file = service.hub_service().download(&Repo::testalias(), "model.gguf", None, None).await?;
    let alias_path = service.data_service().save_alias(&alias)?;

    assert!(alias_path.exists());
    Ok(())
  }
```

## Build Commands

```bash
# Test services crate
cargo test -p services

# Test with all features
cargo test -p services --features test-utils

# Build services crate
cargo build -p services

# Run clippy
cargo clippy -p services

# Format code
cargo fmt -p services
```

## Extension Patterns

### Adding New Services

1. Define trait with async methods and proper error handling
2. Implement concrete service with dependency injection support
3. Add mock generation via mockall attributes
4. Register in AppService trait and DefaultAppService implementation
5. Add comprehensive error types with localization support

### Database Schema Changes

1. Create migration files with both up and down SQL
2. Update DbService methods to handle new schema
3. Test migrations with existing data
4. Coordinate with related services that depend on the data

### External API Integration

1. Implement retry logic with exponential backoff
2. Handle API rate limiting and quota restrictions
3. Add proper error categorization (network, auth, not found)
4. Cache responses when appropriate for performance
5. Add comprehensive testing with mock HTTP clients

## Service Dependencies

### Cross-Service Coordination

- **AuthService** ↔ **DbService**: Token storage and validation
- **AuthService** ↔ **SecretService**: Credential encryption and storage
- **HubService** ↔ **DataService**: Model download and alias creation
- **HubService** ↔ **CacheService**: Repository metadata caching
- **SessionService** ↔ **DbService**: HTTP session persistence
- **All services** ↔ **TimeService**: Consistent timestamp generation

### Service Mock Testing

```rust
  let service = AppServiceStubBuilder::default()
    .with_hub_service()
    .with_data_service()
    .await
    .with_session_service()
    .await
    .with_secret_service()
    .build()?;
```

## Recent Architecture Changes

### AI API Service Enhancement
- Comprehensive external AI provider integration with OpenAI compatibility
- Model testing and validation with configurable test prompts
- Streaming response support with proper Axum integration
- Enhanced error categorization for API failures

### User Access Management
- Advanced user access request workflow with status tracking
- Resource administration with dynamic admin assignment
- Comprehensive access control with approval workflows
- Database-backed user permission management

### Enhanced Security Architecture
- Multi-layer encryption with AES-GCM and PBKDF2 key derivation
- Platform-specific keyring integration for credential storage
- Session management with SQLite backend and security configuration
- Comprehensive audit trails and access logging