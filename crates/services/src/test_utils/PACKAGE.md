# PACKAGE.md - services/test_utils

This document provides detailed implementation information for the `services/test_utils` module, focusing on BodhiApp's service testing infrastructure with accurate file references and navigation aids.

## Module Organization

### Core Module Structure
Main entry point and module organization:

```rust
// Module registration (see crates/services/src/test_utils/mod.rs:1-22)
mod app;
mod auth;
mod data;
mod db;
mod envs;
mod hf;
mod objs;
mod secret;
mod session;
mod settings;

pub use app::*;
pub use auth::*;
pub use data::*;
pub use db::*;
pub use envs::*;
pub use hf::*;
pub use objs::*;
pub use secret::*;
pub use session::*;
pub use settings::*;
```

**Key Features**:
- Centralized re-exports for all test utility components
- Modular organization by service domain (auth, db, hf, etc.)
- Feature-gated availability via `test-utils` feature in Cargo.toml

## Service Composition Testing

### AppServiceStub Architecture
Complete service registry implementation for complex integration testing:

```rust
// Main service stub (see crates/services/src/test_utils/app.rs:45-65)
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
```

### Builder Pattern Implementation
Service composition with dependency management:

```rust
// Service defaults and composition (see crates/services/src/test_utils/app.rs:67-91)
impl AppServiceStubBuilder {
  fn default_setting_service(&self) -> Option<Arc<dyn SettingService>> {
    Some(Arc::new(SettingServiceStub::default()))
  }

  fn default_auth_service(&self) -> Option<Arc<dyn AuthService>> {
    Some(Arc::new(MockAuthService::default()))
  }

  fn default_time_service(&self) -> Option<Arc<dyn TimeService>> {
    Some(Arc::new(FrozenTimeService::default()))
  }
}

// Service builder methods with test data setup
impl AppServiceStubBuilder {
  pub fn with_hub_service(&mut self) -> &mut Self {
    // Implementation at crates/services/src/test_utils/app.rs:135-155
    let temp_home = self.setup_temp_home();
    let hf_home = temp_home.path().join("huggingface");
    copy_test_dir("tests/data/huggingface", &hf_home);
    // Creates OfflineHubService with realistic test data
  }
}
```

**Implementation Details**:
- Builder pattern with configurable service implementations 
- Automatic test data copying from `crates/services/tests/data/`
- Dependency injection with proper Arc wrapping for shared ownership
- Fixture integration with rstest for seamless test setup

## Database Testing Infrastructure

### TestDbService Implementation
Advanced database testing with event broadcasting and temporal control:

```rust
// Test database service (see crates/services/src/test_utils/db.rs:55-85)
#[derive(Debug)]
pub struct TestDbService {
  _temp_dir: Arc<TempDir>,
  inner: SqliteDbService,
  event_sender: Sender<String>,
  now: DateTime<Utc>,
}

impl TestDbService {
  pub fn new(_temp_dir: Arc<TempDir>, inner: SqliteDbService, now: DateTime<Utc>) -> Self {
    let (event_sender, _) = channel(100);
    TestDbService { _temp_dir, inner, event_sender, now }
  }
  
  pub fn subscribe(&self) -> Receiver<String> {
    self.event_sender.subscribe()
  }
}

// Fixture for easy test setup (see crates/services/src/test_utils/db.rs:15-34)
#[fixture]
#[awt]
pub async fn test_db_service(temp_dir: TempDir) -> TestDbService {
  test_db_service_with_temp_dir(Arc::new(temp_dir)).await
}
```

### FrozenTimeService Pattern
Deterministic time operations for reproducible testing:

```rust
// Time service implementation (see crates/services/src/test_utils/db.rs:36-53)
#[derive(Debug)]
pub struct FrozenTimeService(DateTime<Utc>);

impl Default for FrozenTimeService {
  fn default() -> Self {
    FrozenTimeService(chrono::Utc::now().with_nanosecond(0).unwrap())
  }
}

impl TimeService for FrozenTimeService {
  fn utc_now(&self) -> DateTime<Utc> {
    self.0 // Always returns the same frozen time
  }

  fn created_at(&self, _path: &Path) -> u32 {
    0 // Deterministic file creation time
  }
}
```

**Database Testing Features**:
- Isolated SQLite databases in temporary directories
- Event broadcasting system for reactive testing patterns
- Automatic migration with rollback testing
- Deterministic timestamp generation via `FrozenTimeService`

## Authentication Testing Infrastructure

### OAuth2 Flow Simulation
Comprehensive authentication testing with embedded keys:

```rust
// Test constants and keys (see crates/services/src/test_utils/auth.rs:14-61)
pub const TEST_CLIENT_ID: &str = "test-client";
pub const TEST_CLIENT_SECRET: &str = "test-client-secret";
pub const ISSUER: &str = "https://id.mydomain.com/realms/myapp";
pub const TEST_KID: &str = "test-kid";

static PUBLIC_KEY: Lazy<RsaPublicKey> = Lazy::new(|| {
  RsaPublicKey::from_public_key_pem(include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/test_public_key.pem"
  )))
  .expect("Failed to parse public key")
});
```

### JWT Token Generation
Token lifecycle testing with proper claims structure:

```rust
// Token generation (see crates/services/src/test_utils/auth.rs:72-144)
pub fn access_token_claims() -> Value {
  access_token_with_exp(Utc::now().timestamp() + 3600)
}

pub fn build_token(claims: Value) -> anyhow::Result<(String, String)> {
  sign_token(&PRIVATE_KEY, &PUBLIC_KEY, &claims)
}

// Claims structure includes proper resource_access roles
pub fn access_token_with_exp(exp: i64) -> Value {
  json!({
    "exp": exp,
    "iss": "https://id.mydomain.com/realms/myapp",
    "resource_access": {
      TEST_CLIENT_ID: {
        "roles": [
          "resource_manager",
          "resource_power_user", 
          "resource_user",
          "resource_admin"
        ]
      }
    },
    "preferred_username": "testuser@email.com",
    "name": "Test User"
  })
}
```

**Authentication Testing Features**:
- Embedded RSA key pairs for deterministic signing
- Multi-token type support (Bearer, Offline, Refresh)
- Comprehensive claims structure with resource access roles
- Keycloak-specific OAuth2 flow simulation

## Hub Service Testing

### OfflineHubService Implementation
Local-only model management without external dependencies:

```rust
// Offline hub service (see crates/services/src/test_utils/hf.rs:112-169)
#[derive(Debug, new)]
pub struct OfflineHubService {
  inner: HfHubService,
}

#[async_trait::async_trait]
impl HubService for OfflineHubService {
  async fn download(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: Option<String>,
    progress: Option<Progress>,
  ) -> Result<HubFile> {
    if !self.inner.local_file_exists(repo, filename, snapshot.clone())? {
      panic!("tried to download file in test");
    }
    self.inner.download(repo, filename, snapshot, progress).await
  }
}
```

### TestHfService Pattern  
Hybrid service supporting both real and mock operations:

```rust
// Test HF service (see crates/services/src/test_utils/hf.rs:21-110)
#[derive(Debug)]
pub struct TestHfService {
  _temp_dir: TempDir,
  inner: HfHubService,
  pub inner_mock: MockHubService,
  allow_downloads: bool,
}

impl TestHfService {
  pub fn hf_cache(&self) -> PathBuf {
    self._temp_dir.path().join("huggingface").join("hub")
  }
}

// Configurable download behavior
#[async_trait::async_trait]
impl HubService for TestHfService {
  async fn download(&self, repo: &Repo, filename: &str, snapshot: Option<String>, progress: Option<Progress>) -> Result<HubFile> {
    if self.allow_downloads {
      self.inner.download(repo, filename, snapshot, progress).await
    } else {
      self.inner_mock.download(repo, filename, snapshot, progress).await
    }
  }
}
```

**Hub Service Testing Features**:
- Configurable real/mock operation modes
- Automatic test data setup from `tests/data/huggingface/`
- Network call prevention with realistic local file operations
- MockAll integration for comprehensive API simulation

## Secret Management Testing

### SecretServiceStub Implementation
In-memory secret storage with application status management:

```rust
// Secret service stub (see crates/services/src/test_utils/secret.rs:9-89)
#[derive(Debug)]
pub struct SecretServiceStub {
  store: Mutex<HashMap<String, String>>,
}

impl SecretServiceStub {
  pub fn with_app_status(self, status: &AppStatus) -> Self {
    self.set_app_status(status).unwrap();
    self
  }

  pub fn with_app_reg_info(self, app_reg_info: &AppRegInfo) -> Self {
    self.set_app_reg_info(app_reg_info).unwrap();
    self
  }
}

impl SecretService for SecretServiceStub {
  fn set_secret_string(&self, key: &str, value: &str) -> Result<()> {
    let mut store = self.store.lock().unwrap();
    store.insert(key.to_string(), value.to_string());
    Ok(())
  }
}
```

### KeyringStoreStub Pattern
Cross-platform credential storage simulation:

```rust
// Keyring store stub (see crates/services/src/test_utils/secret.rs:91-133)
#[derive(Debug)]
pub struct KeyringStoreStub {
  store: Mutex<HashMap<String, String>>,
}

impl KeyringStore for KeyringStoreStub {
  fn set_password(&self, key: &str, value: &str) -> std::result::Result<(), KeyringError> {
    let mut store = self.store.lock().unwrap();
    store.insert(key.to_string(), value.to_string());
    Ok(())
  }
}
```

**Secret Management Features**:
- In-memory storage for deterministic testing
- Application status and registration management
- Cross-platform keyring simulation without system dependencies
- Configurable secret storage for various test scenarios

## Session Management Testing

### Session Testing Extensions
HTTP session validation and lifecycle testing:

```rust
// Session test extensions (see crates/services/src/test_utils/session.rs:24-45)
#[async_trait::async_trait]
pub trait SessionTestExt {
  async fn get_session_value(&self, session_id: &str, key: &str) -> Option<Value>;
  async fn get_session_record(&self, session_id: &str) -> Option<Record>;
}

#[async_trait::async_trait]
impl SessionTestExt for SqliteSessionService {
  async fn get_session_value(&self, session_id: &str, key: &str) -> Option<Value> {
    let record = self.get_session_record(session_id).await.unwrap();
    record.data.get(key).cloned()
  }
}

// Session service builder (see crates/services/src/test_utils/session.rs:10-22)
impl SqliteSessionService {
  pub async fn build_session_service(dbfile: PathBuf) -> SqliteSessionService {
    if !dbfile.exists() {
      File::create(&dbfile).expect("Failed to create database file");
    }
    let pool = SqlitePool::connect(&format!("sqlite:{}", dbfile.display())).await.unwrap();
    let session_service = SqliteSessionService::new(pool);
    session_service.migrate().await.unwrap();
    session_service
  }
}
```

## Data Service Testing

### TestDataService Integration
Data service testing with hub and database coordination:

```rust
// Test data service (see crates/services/src/test_utils/data.rs:11-92)
#[derive(Debug)]
pub struct TestDataService {
  pub temp_bodhi_home: TempDir,
  pub inner: LocalDataService,
}

impl TestDataService {
  pub fn bodhi_home(&self) -> PathBuf {
    self.temp_bodhi_home.path().join("bodhi")
  }
}

// Fixture setup with service coordination (see crates/services/src/test_utils/data.rs:11-27)
#[fixture]
#[awt]
pub async fn test_data_service(
  temp_bodhi_home: TempDir,
  test_hf_service: TestHfService,
  #[future] test_db_service: TestDbService,
) -> TestDataService {
  let inner = LocalDataService::new(
    temp_bodhi_home.path().join("bodhi"),
    Arc::new(test_hf_service),
    Arc::new(test_db_service),
  );
  TestDataService { temp_bodhi_home, inner }
}
```

## Object Creation Utilities

### Test Object Builders
Domain object creation for testing scenarios:

```rust
// API alias creation utilities (see crates/services/src/test_utils/objs.rs:14-96)
pub fn create_test_api_model_alias(
  alias: &str,
  models: Vec<String>,
  created_at: DateTime<Utc>,
) -> ApiAlias {
  ApiAlias::new(
    alias,
    ApiFormat::OpenAI,
    "https://api.openai.com/v1",
    models,
    None,
    created_at,
  )
}

// Database seeding with test data
pub async fn seed_test_api_models(
  db_service: &dyn crate::db::DbService,
  base_time: DateTime<Utc>,
) -> anyhow::Result<Vec<ApiAlias>> {
  let aliases = vec![
    create_test_api_model_alias("openai-gpt4", vec!["gpt-4".to_string()], base_time),
    create_test_api_model_alias("azure-openai", vec!["gpt-4".to_string()], base_time - chrono::Duration::seconds(10)),
    // Additional test models with incremental timestamps
  ];

  for alias in &aliases {
    db_service.create_api_model_alias(alias, "sk-test-key-123456789").await?;
  }

  Ok(aliases)
}
```

## Environment and Settings Testing

### SettingServiceStub Implementation
Configuration management testing with environment variables:

```rust
// Settings service stub (see crates/services/src/test_utils/envs.rs:38-150)
#[derive(Debug, Clone)]
pub struct SettingServiceStub {
  settings: Arc<RwLock<HashMap<String, serde_yaml::Value>>>,
  envs: HashMap<String, String>,
  temp_dir: Arc<TempDir>,
}

// Test environment setup (see crates/services/src/test_utils/envs.rs:21-29)
pub fn hf_test_token_allowed() -> Option<String> {
  dotenv::from_filename(".env.test").ok();
  Some(std::env::var("HF_TEST_TOKEN_ALLOWED").unwrap())
}
```

### Test Constants
Standard test configuration values:

```rust
// Test settings constants (see crates/services/src/test_utils/settings.rs:1-6)
pub const TEST_LOGS_DIR: &str = "logs";
pub const TEST_ALIASES_DIR: &str = "aliases";
pub const TEST_PROD_DB: &str = "bodhi.sqlite";
pub const TEST_SESSION_DB: &str = "session.sqlite";
pub const TEST_SETTINGS_YAML: &str = "settings.yaml";
```

## Feature Configuration

### Cargo.toml Test Features
Test utilities are conditionally compiled via features:

```toml
# Feature configuration (see crates/services/Cargo.toml:94-106)
[features]
test-utils = [
  "rstest",
  "mockall", 
  "once_cell",
  "rsa",
  "tap",
  "tempfile",
  "anyhow",
  "objs/test-utils",
  "tokio",
]
```

### Library Integration  
Test utilities are conditionally exposed:

```rust
// Library exposure (see crates/services/src/lib.rs:1-4)
#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;
```

## Usage Examples

### Complete Integration Test
Multi-service coordination example:

```rust
#[rstest]
#[awt]
async fn test_complete_model_download_flow(
  #[future] app_service_stub: AppServiceStub
) -> Result<(), Box<dyn std::error::Error>> {
  let hub_service = app_service_stub.hub_service();
  let data_service = app_service_stub.data_service();
  let db_service = app_service_stub.db_service();
  
  // Test complete download lifecycle
  let repo = Repo::from_str("microsoft/DialoGPT-medium")?;
  let download_request = DownloadRequest::new(
    "test-request".to_string(),
    repo.clone(),
    "pytorch_model.bin".to_string(),
    "main".to_string(),
    DownloadStatus::Pending,
  );
  
  // Create download tracking
  db_service.create_download_request(&download_request).await?;
  
  // Attempt download (will use offline test data)
  let hub_files = hub_service.download(&repo, "pytorch_model.bin", Some("main".to_string()), None).await?;
  
  // Create alias for downloaded model
  let alias = UserAlias {
    alias: "dialogpt-test".to_string(),
    repo: repo.clone(),
    filename: hub_files.filename,
    snapshot: "main".to_string(),
  };
  
  let alias_path = data_service.save_alias(&alias)?;
  assert!(alias_path.exists());
  
  Ok(())
}
```

## Commands

**Run All Service Tests**: `cargo test -p services --features test-utils`  
**Database Tests Only**: `cargo test -p services test_db --features test-utils`  
**Authentication Tests**: `cargo test -p services test_auth --features test-utils`  
**Hub Service Tests**: `cargo test -p services test_hf --features test-utils`  
**Integration Tests**: `cargo test -p services integration --features test-utils`  

**Format Code**: `cargo fmt --manifest-path crates/services/Cargo.toml`  
**Check Features**: `cargo check -p services --features test-utils`