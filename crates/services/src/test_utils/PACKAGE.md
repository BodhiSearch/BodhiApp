# PACKAGE.md - services/test_utils

This document provides detailed technical information for the `services/test_utils` module, focusing on BodhiApp-specific service testing patterns and mock coordination implementation.

## Service Composition Testing Implementation

### AppServiceStub Pattern
BodhiApp's comprehensive service testing infrastructure with builder pattern and dependency management:

```rust
// Core testing infrastructure (see app.rs:47-67 for complete struct)
#[derive(Debug, Default, Builder)]
#[builder(default, setter(strip_option))]
pub struct AppServiceStub {
  pub temp_home: Option<Arc<TempDir>>,
  #[builder(default = "self.default_setting_service()")]
  pub setting_service: Option<Arc<dyn SettingService>>,
  #[builder(default = "self.default_hub_service()")]
  pub hub_service: Option<Arc<dyn HubService>>,
  pub data_service: Option<Arc<dyn DataService>>,
  #[builder(default = "self.default_auth_service()")]
  pub auth_service: Option<Arc<dyn AuthService>>,
  pub db_service: Option<Arc<dyn DbService>>,
  pub session_service: Option<Arc<dyn SessionService>>,
  #[builder(default = "self.default_secret_service()")]
  pub secret_service: Option<Arc<dyn SecretService>>,
  #[builder(default = "self.default_cache_service()")]
  pub cache_service: Option<Arc<dyn CacheService>>,
  pub localization_service: Option<Arc<dyn LocalizationService>>,
  #[builder(default = "self.default_time_service()")]
  pub time_service: Option<Arc<dyn TimeService>>,
}

// Implements AppService trait for seamless testing (see app.rs:156-189)
impl AppService for AppServiceStub {
  fn setting_service(&self) -> Arc<dyn SettingService> { self.setting_service.clone().unwrap() }
  fn data_service(&self) -> Arc<dyn DataService> { self.data_service.clone().unwrap() }
  // All service accessors with proper Arc cloning
}
```

### Service Builder Pattern Implementation
Flexible service composition with sensible defaults and dependency management:

```rust
// Builder implementation with dependency resolution (see app.rs:69-145)
impl AppServiceStubBuilder {
  fn default_setting_service(&self) -> Option<Arc<dyn SettingService>> {
    Some(Arc::new(SettingServiceStub::default()))
  }

  fn default_auth_service(&self) -> Option<Arc<dyn AuthService>> {
    Some(Arc::new(MockAuthService::default()))
  }

  pub fn with_hub_service(&mut self) -> &mut Self {
    let temp_home = self.setup_temp_home();
    let hf_home = temp_home.path().join("huggingface");
    copy_test_dir("tests/data/huggingface", &hf_home);
    let hf_cache = hf_home.join("hub");
    let hub_service = OfflineHubService::new(HfHubService::new(hf_cache, false, None));
    self.hub_service = Some(Some(Arc::new(hub_service)));
    self
  }

  pub fn with_data_service(&mut self) -> &mut Self {
    let temp_home = self.setup_temp_home();
    let hub_service = self.with_hub_service().hub_service.clone().unwrap().unwrap().clone();
    let bodhi_home = temp_home.path().join("bodhi");
    copy_test_dir("tests/data/bodhi", &bodhi_home);
    let data_service = LocalDataService::new(bodhi_home, hub_service);
    self.data_service = Some(Some(Arc::new(data_service)));
    self
  }

  pub async fn with_session_service(&mut self) -> &mut Self {
    let temp_home = self.setup_temp_home();
    let dbfile = temp_home.path().join("test.db");
    self.build_session_service(dbfile).await;
    self
  }

  pub fn with_secret_service(&mut self) -> &mut Self {
    let secret_service = SecretServiceStub::default()
      .with_app_reg_info(&AppRegInfoBuilder::test_default().build().unwrap());
    self.secret_service = Some(Some(Arc::new(secret_service)));
    self
  }
}
```

**Key Implementation Details**:
- Builder pattern with configurable service implementations and automatic dependency resolution
- Automatic temp directory setup with realistic test data copying from tests/data/
- Sensible defaults for testing scenarios with mock services where appropriate
- Support for mixed real and mock service composition for targeted testing scenarios
- Dependency ordering ensures services are initialized in correct sequence

## Database Testing Infrastructure

### TestDbService Implementation
Sophisticated database testing with isolation, deterministic time, and event broadcasting:

```rust
// Test database service with event broadcasting (see db.rs:23-45 for complete implementation)
#[derive(Debug)]
pub struct TestDbService {
  _temp_dir: TempDir,
  inner: SqliteDbService,
  event_sender: Sender<String>,
  now: DateTime<Utc>,
}

impl TestDbService {
  pub fn new(_temp_dir: TempDir, inner: SqliteDbService, now: DateTime<Utc>) -> Self {
    let (event_sender, _) = channel(100);
    TestDbService { _temp_dir, inner, event_sender, now }
  }
  
  pub fn event_receiver(&self) -> Receiver<String> {
    self.event_sender.subscribe() // For reactive testing scenarios
  }

  pub fn pool(&self) -> &SqlitePool {
    self.inner.pool() // Direct pool access for service coordination
  }
}

// Fixture for easy test setup (see db.rs:67-89)
#[fixture]
#[awt]
pub async fn test_db_service(#[future] temp_dir: TempDir) -> TestDbService {
  let db_path = temp_dir.path().join("test.db");
  let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.display())).await.unwrap();
  sqlx::migrate!("./migrations").run(&pool).await.unwrap();
  
  let inner = SqliteDbService::new(pool);
  let now = chrono::Utc::now().with_nanosecond(0).unwrap();
  TestDbService::new(temp_dir, inner, now)
}
```

### FrozenTimeService Pattern
Deterministic time operations for reproducible testing:

```rust
#[derive(Debug)]
pub struct FrozenTimeService(DateTime<Utc>);

impl Default for FrozenTimeService {
  fn default() -> Self {
    FrozenTimeService(chrono::Utc::now().with_nanosecond(0).unwrap())
  }
}

impl TimeService for FrozenTimeService {
  fn utc_now(&self) -> DateTime<Utc> {
    self.0 // Always returns the same time
  }

  fn created_at(&self, _path: &Path) -> u32 {
    0 // Deterministic file creation time
  }
}
```

**Database Testing Features**:
- Isolated temporary SQLite databases for each test
- Frozen time service for deterministic testing
- Event broadcasting for reactive testing scenarios
- Migration testing with realistic data preservation

## Authentication Testing Implementation

### OAuth2 Flow Simulation
Comprehensive authentication flow testing:

```rust
impl AppRegInfoBuilder {
  pub fn mock_registration() -> Self {
    Self::default()
      .client_id("test_client_id".to_string())
      .client_secret("test_client_secret".to_string())
      .authorization_endpoint("http://localhost:8080/auth".to_string())
      .token_endpoint("http://localhost:8080/token".to_string())
      .redirect_uris(vec!["http://localhost:8080/callback".to_string()])
  }
}

// Mock auth service with configurable responses
let mut mock_auth_service = MockAuthService::new();
mock_auth_service
  .expect_register_client()
  .returning(|_name, _desc, _uris| {
    Ok(AppRegInfoBuilder::mock_registration().build().unwrap())
  });

mock_auth_service
  .expect_exchange_auth_code()
  .returning(|_code, _client_id, _client_secret, _redirect_uri, _code_verifier| {
    Ok((AccessToken::new("test_access_token".to_string()), RefreshToken::new("test_refresh_token".to_string())))
  });
```

### JWT Token Testing
Token validation and lifecycle testing:

```rust
#[rstest]
async fn test_jwt_token_lifecycle(
  #[future] app_service_stub: AppServiceStub
) -> Result<(), Box<dyn std::error::Error>> {
  let auth_service = app_service_stub.auth_service();
  let db_service = app_service_stub.db_service();
  
  // Test token creation
  let (access_token, refresh_token) = auth_service.exchange_auth_code(/* ... */).await?;
  
  // Validate token storage
  let stored_token = db_service.get_api_token(&token_id).await?;
  assert!(stored_token.is_some());
  
  // Test token refresh
  let (new_access, new_refresh) = auth_service.refresh_token(
    &client_id, &client_secret, refresh_token.secret()
  ).await?;
  
  Ok(())
}
```

## Hub Service Testing Implementation

### OfflineHubService Pattern
Local-only model management for testing without external dependencies using realistic test data:

```rust
// Offline hub service for testing (see hf.rs:23-45 for complete implementation)
#[derive(Debug)]
pub struct OfflineHubService {
  inner: HfHubService,
  test_data: HashMap<String, Vec<HubFile>>,
}

impl OfflineHubService {
  pub fn new(inner: HfHubService) -> Self {
    Self { inner, test_data: HashMap::new() }
  }

  pub fn with_test_model(mut self, repo: &str, files: Vec<HubFile>) -> Self {
    self.test_data.insert(repo.to_string(), files);
    self
  }
}

impl HubService for OfflineHubService {
  async fn download_model(&self, repo: &Repo, snapshot: &str) -> Result<Vec<HubFile>, HubServiceError> {
    // First try local test data, then fall back to real HF cache for realistic testing
    if let Some(files) = self.test_data.get(&repo.to_string()) {
      Ok(files.clone())
    } else {
      self.inner.download_model(repo, snapshot).await
    }
  }

  async fn list_models(&self, query: &str) -> Result<Vec<String>, HubServiceError> {
    let mut models = self.test_data.keys().cloned().collect::<Vec<_>>();
    if let Ok(real_models) = self.inner.list_models(query).await {
      models.extend(real_models);
    }
    Ok(models)
  }
}

// Test data builders for realistic scenarios (see hf.rs:67-89)
impl HubFileBuilder {
  pub fn testalias() -> Self {
    Self::default()
      .repo(Repo::llama3())
      .filename("model.gguf".to_string())
      .snapshot("main".to_string())
  }
}
```

### Hub Service Mock Coordination
Complex Hub API simulation with various scenarios:

```rust
let mut mock_hub_service = MockHubService::new();

// Successful download scenario
mock_hub_service
  .expect_download_model()
  .with(eq(Repo::llama3()), eq("main"))
  .returning(|_repo, _snapshot| {
    Ok(vec![HubFileBuilder::testalias().build().unwrap()])
  });

// Gated repository scenario
mock_hub_service
  .expect_download_model()
  .with(eq(Repo::from_str("meta-llama/Llama-2-70b-chat-hf").unwrap()), any())
  .returning(|repo, _snapshot| {
    Err(HubApiError::new(
      "Repository requires authentication".to_string(),
      401,
      repo.to_string(),
      HubApiErrorKind::GatedAccess
    ).into())
  });

// Network error scenario
mock_hub_service
  .expect_list_models()
  .returning(|_query| {
    Err(HubApiError::new(
      "Network timeout".to_string(),
      503,
      "test-repo".to_string(),
      HubApiErrorKind::Transport
    ).into())
  });
```

## Security Testing Infrastructure

### SecretServiceStub Implementation
Deterministic encryption testing without real cryptographic operations:

```rust
#[derive(Debug, Default)]
pub struct SecretServiceStub {
  secrets: std::sync::Mutex<HashMap<String, String>>,
}

impl SecretService for SecretServiceStub {
  async fn store_secret(&self, key: &str, value: &str) -> Result<(), SecretServiceError> {
    let mut secrets = self.secrets.lock().unwrap();
    secrets.insert(key.to_string(), format!("encrypted_{}", value));
    Ok(())
  }

  async fn retrieve_secret(&self, key: &str) -> Result<Option<String>, SecretServiceError> {
    let secrets = self.secrets.lock().unwrap();
    Ok(secrets.get(key).map(|v| v.strip_prefix("encrypted_").unwrap().to_string()))
  }
}
```

### Session Security Testing
HTTP session validation with secure cookie configuration:

```rust
#[rstest]
async fn test_session_security_configuration(
  #[future] app_service_stub: AppServiceStub
) -> Result<(), Box<dyn std::error::Error>> {
  let session_service = app_service_stub.session_service();
  let session_layer = session_service.session_layer();
  
  // Validate session configuration
  assert_eq!(session_layer.cookie_name(), "bodhiapp_session_id");
  assert_eq!(session_layer.cookie_same_site(), Some(SameSite::Strict));
  assert!(!session_layer.cookie_secure()); // TODO: Enable for HTTPS
  
  Ok(())
}
```

## Cross-Service Integration Testing

### Multi-Service Flow Testing
Complex scenarios involving multiple service coordination:

```rust
#[rstest]
#[awt]
async fn test_complete_model_download_flow(
  #[future] app_service_stub: AppServiceStub
) -> Result<(), Box<dyn std::error::Error>> {
  let hub_service = app_service_stub.hub_service();
  let data_service = app_service_stub.data_service();
  let auth_service = app_service_stub.auth_service();
  let cache_service = app_service_stub.cache_service();
  
  // Step 1: Authenticate for gated repository
  let (access_token, _) = auth_service.exchange_auth_code(/* ... */).await?;
  
  // Step 2: Download model with authentication
  let repo = Repo::from_str("meta-llama/Llama-2-7b-chat-hf")?;
  let model_files = hub_service.download_model(&repo, "main").await?;
  
  // Step 3: Create local alias
  let alias = AliasBuilder::default()
    .alias("llama2-chat".to_string())
    .repo(repo.clone())
    .filename(model_files[0].filename.clone())
    .snapshot("main".to_string())
    .build()?;
  
  let alias_path = data_service.save_alias(&alias)?;
  
  // Step 4: Cache model metadata
  cache_service.set(&format!("model:{}", repo), &serde_json::to_string(&model_files)?);
  
  // Step 5: Validate end-to-end flow
  let cached_models = cache_service.get(&format!("model:{}", repo));
  assert!(cached_models.is_some());
  assert!(alias_path.exists());
  
  Ok(())
}
```

### Error Propagation Testing
Cross-service error handling and recovery:

```rust
#[rstest]
#[awt]
async fn test_error_recovery_across_services(
  #[future] app_service_stub: AppServiceStub
) -> Result<(), Box<dyn std::error::Error>> {
  let hub_service = app_service_stub.hub_service();
  let db_service = app_service_stub.db_service();
  
  // Create download request
  let download_request = DownloadRequest::new(
    "test-id".to_string(),
    Repo::llama3(),
    "model.gguf".to_string(),
    "main".to_string(),
    DownloadStatus::Pending,
  );
  
  db_service.create_download_request(&download_request).await?;
  
  // Simulate network failure during download
  let result = hub_service.download_model(&download_request.repo, &download_request.snapshot).await;
  assert!(result.is_err());
  
  // Update request status to reflect failure
  let mut updated_request = download_request;
  updated_request.status = DownloadStatus::Failed;
  db_service.update_download_request(&updated_request).await?;
  
  // Verify error state is properly recorded
  let stored_request = db_service.get_download_request(&updated_request.id).await?;
  assert_eq!(stored_request.unwrap().status, DownloadStatus::Failed);
  
  Ok(())
}
```

## Extension Guidelines

### Adding New Service Tests
When creating tests for new services:

1. **Add service to AppServiceStub**: Include in builder pattern with sensible defaults
2. **Create mock service**: Use `#[mockall::automock]` with comprehensive expectation coverage
3. **Implement stub version**: Create simplified implementation for offline testing
4. **Add integration tests**: Test service interactions with other services
5. **Validate error scenarios**: Test error propagation and recovery across service boundaries

### Database Testing Patterns
For services requiring database integration:

1. **Use TestDbService**: Provides isolated database with event broadcasting
2. **Use FrozenTimeService**: Ensures deterministic timestamps for testing
3. **Test migrations**: Validate schema changes with realistic data
4. **Test transactions**: Verify rollback scenarios and consistency
5. **Test concurrent access**: Validate connection pooling and locking

### Authentication Flow Testing
For authentication-related services:

1. **Mock OAuth providers**: Simulate various provider responses and errors
2. **Test token lifecycle**: Creation, validation, refresh, and expiration
3. **Test session coordination**: HTTP sessions synchronized with authentication state
4. **Test security configuration**: Validate cookie settings and session isolation
5. **Test cross-service auth**: Verify token exchange and service-to-service authentication

### Performance Testing Utilities
For performance and caching validation:

1. **Use cache service stubs**: Test cache invalidation and consistency
2. **Simulate load scenarios**: Test connection pooling and resource management
3. **Test timeout handling**: Validate network timeout and retry logic
4. **Test resource cleanup**: Verify proper resource lifecycle management

## Commands

**Testing**: `cargo test -p services --features test-utils` (includes all service integration tests)  
**Database Tests**: `cargo test -p services test_db` (database-specific tests)  
**Auth Tests**: `cargo test -p services test_auth` (authentication flow tests)