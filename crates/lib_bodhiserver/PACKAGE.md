# PACKAGE.md - lib_bodhiserver

This document provides detailed technical information for the `lib_bodhiserver` crate, focusing on BodhiApp's embeddable server library architecture, sophisticated service composition patterns, and comprehensive application bootstrap coordination.

## Embeddable Server Library Architecture

The `lib_bodhiserver` crate serves as BodhiApp's **embeddable server library orchestration layer**, implementing advanced service composition, application directory management, and configuration systems for embedding BodhiApp functionality into external applications.

### AppServiceBuilder Dependency Injection Architecture
Sophisticated service composition with automatic dependency resolution and comprehensive error handling:

```rust
// Core service builder pattern - see crates/lib_bodhiserver/src/app_service_builder.rs
#[derive(Debug)]
pub struct AppServiceBuilder {
  setting_service: Arc<dyn SettingService>,
  hub_service: Option<Arc<dyn HubService>>,
  data_service: Option<Arc<dyn DataService>>,
  time_service: Option<Arc<dyn TimeService>>,
  db_service: Option<Arc<dyn DbService>>,
  session_service: Option<Arc<dyn SessionService>>,
  secret_service: Option<Arc<dyn SecretService>>,
  cache_service: Option<Arc<dyn CacheService>>,
  auth_service: Option<Arc<dyn AuthService>>,
  localization_service: Option<Arc<dyn LocalizationService>>,
}

impl AppServiceBuilder {
  pub async fn build(mut self) -> Result<DefaultAppService, ErrorMessage> {
    // Build services in dependency order with automatic resolution
    let localization_service = self.get_or_build_localization_service()?;
    let hub_service = self.get_or_build_hub_service();
    let data_service = self.get_or_build_data_service(hub_service.clone());
    let time_service = self.get_or_build_time_service();
    let db_service = self.get_or_build_db_service(time_service.clone()).await?;
    let session_service = self.get_or_build_session_service().await?;
    let secret_service = self.get_or_build_secret_service()?;
    let cache_service = self.get_or_build_cache_service();
    let auth_service = self.get_or_build_auth_service();

    // Complete service composition with dependency injection
    let app_service = DefaultAppService::new(
      self.setting_service, hub_service, data_service, auth_service,
      db_service, session_service, secret_service, cache_service,
      localization_service, time_service,
    );
    Ok(app_service)
  }
}
```

### Service Composition Implementation
Complete service registry initialization with sophisticated dependency management:

```rust
// Automatic service resolution with dependency coordination - see crates/lib_bodhiserver/src/app_service_builder.rs
fn get_or_build_hub_service(&mut self) -> Arc<dyn HubService> {
  if let Some(service) = self.hub_service.take() {
    return service;
  }
  
  let hf_cache = self.setting_service.hf_cache();
  let hf_token = self.setting_service.get_env(HF_TOKEN);
  Arc::new(HfHubService::new_from_hf_cache(hf_cache, hf_token, true))
}

fn get_or_build_data_service(&mut self, hub_service: Arc<dyn HubService>) -> Arc<dyn DataService> {
  if let Some(service) = self.data_service.take() {
    return service;
  }
  
  let bodhi_home = self.setting_service.bodhi_home();
  Arc::new(LocalDataService::new(bodhi_home, hub_service))
}

async fn get_or_build_db_service(&mut self, time_service: Arc<dyn TimeService>, encryption_key: Vec<u8>) -> Result<Arc<dyn DbService>, ApiError> {
  if let Some(service) = self.db_service.take() {
    return Ok(service);
  }
  
  let app_db_pool = DbPool::connect(&format!("sqlite:{}", self.setting_service.app_db_path().display())).await?;
  let db_service = SqliteDbService::new(app_db_pool, time_service, encryption_key);
  db_service.migrate().await?;
  Ok(Arc::new(db_service))
}
```

**Key Service Composition Features**:
- Automatic dependency resolution with proper service ordering and initialization
- Comprehensive error handling with service-specific error types and recovery strategies
- Database migration management with SQLite connection pooling and transaction support
- Platform-specific credential storage integration with keyring services and encryption
- Localization resource loading from all workspace crates with fallback support and error handling

## Application Directory Management Architecture

### Comprehensive Directory Setup Implementation
Advanced filesystem orchestration with environment-specific configuration and error handling:

```rust
// Primary directory setup orchestration - see crates/lib_bodhiserver/src/app_dirs_builder.rs
pub fn setup_app_dirs(options: &AppOptions) -> Result<DefaultSettingService, AppDirsBuilderError> {
  let file_defaults = load_defaults_yaml();
  let (bodhi_home, source) = create_bodhi_home(options.env_wrapper.clone(), &options.env_type, &file_defaults)?;
  let setting_service = setup_settings(options, bodhi_home, source, file_defaults)?;
  setup_bodhi_subdirs(&setting_service)?;
  setup_hf_home(&setting_service)?;
  setup_logs_dir(&setting_service)?;
  Ok(setting_service)
}

// BODHI_HOME creation with environment-specific paths - see crates/lib_bodhiserver/src/app_dirs_builder.rs
fn find_bodhi_home(
  env_wrapper: Arc<dyn EnvWrapper>,
  env_type: &EnvType,
  file_defaults: &HashMap<String, Value>,
) -> Result<(PathBuf, SettingSource), AppDirsBuilderError> {
  let value = env_wrapper.var(BODHI_HOME);
  let bodhi_home = match value {
    Ok(value) => (PathBuf::from(value), SettingSource::Environment),
    Err(_) => {
      if let Some(file_value) = file_defaults.get(BODHI_HOME) {
        if let Some(path_str) = file_value.as_str() {
          return Ok((PathBuf::from(path_str), SettingSource::Default));
        }
      }
      
      // Environment-specific default paths
      let home_dir = env_wrapper.home_dir();
      match home_dir {
        Some(home_dir) => {
          let path = if env_type.is_production() { "bodhi" } else { "bodhi-dev" };
          (home_dir.join(".cache").join(path), SettingSource::Default)
        }
        None => return Err(AppDirsBuilderError::BodhiHomeNotFound),
      }
    }
  };
  Ok(bodhi_home)
}
```

### Database and Resource Directory Setup
Sophisticated resource management with proper permissions and error handling:

```rust
// Subdirectory creation with comprehensive error handling - see crates/lib_bodhiserver/src/app_dirs_builder.rs
fn setup_bodhi_subdirs(setting_service: &dyn SettingService) -> Result<(), AppDirsBuilderError> {
  let alias_home = setting_service.aliases_dir();
  if !alias_home.exists() {
    fs::create_dir_all(&alias_home).map_err(|err| AppDirsBuilderError::DirCreate {
      source: err, path: alias_home.display().to_string(),
    })?;
  }
  
  let db_path = setting_service.app_db_path();
  if !db_path.exists() {
    File::create_new(&db_path).map_err(|err| AppDirsBuilderError::IoFileWrite {
      source: err, path: db_path.display().to_string(),
    })?;
  }
  
  let session_db_path = setting_service.session_db_path();
  if !session_db_path.exists() {
    File::create_new(&session_db_path).map_err(|err| AppDirsBuilderError::IoFileWrite {
      source: err, path: session_db_path.display().to_string(),
    })?;
  }
  Ok(())
}

// HuggingFace cache setup with automatic configuration - see crates/lib_bodhiserver/src/app_dirs_builder.rs
fn setup_hf_home(setting_service: &dyn SettingService) -> Result<PathBuf, AppDirsBuilderError> {
  let hf_home = match setting_service.get_setting(HF_HOME) {
    Some(hf_home) => PathBuf::from(hf_home),
    None => match setting_service.home_dir() {
      Some(home_dir) => {
        let hf_home = home_dir.join(".cache").join("huggingface");
        setting_service.set_setting(HF_HOME, &hf_home.display().to_string());
        hf_home
      }
      None => return Err(AppDirsBuilderError::HfHomeNotFound),
    },
  };
  
  let hf_hub = hf_home.join("hub");
  if !hf_hub.exists() {
    fs::create_dir_all(&hf_hub).map_err(|err| AppDirsBuilderError::DirCreate {
      source: err, path: "$HF_HOME/hub".to_string(),
    })?;
  }
  Ok(hf_home)
}
```

**Directory Management Features**:
- Environment-specific directory paths with development/production mode coordination
- Automatic directory creation with proper permissions and comprehensive error handling
- Database file initialization with SQLite setup and migration support
- HuggingFace cache integration with hub directory creation and model management
- Logs directory setup with proper permissions and cleanup coordination

## Configuration Management Architecture

### AppOptions Builder Pattern Implementation
Flexible configuration system with comprehensive validation and environment integration:

```rust
// Configuration builder with environment and settings management - see crates/lib_bodhiserver/src/app_options.rs
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AppOptionsBuilder {
  environment_vars: HashMap<String, String>,
  env_type: Option<EnvType>,
  app_type: Option<AppType>,
  app_version: Option<String>,
  app_commit_sha: Option<String>,
  auth_url: Option<String>,
  auth_realm: Option<String>,
  app_settings: HashMap<String, String>,
  app_reg_info: Option<AppRegInfo>,
  app_status: Option<AppStatus>,
}

impl AppOptionsBuilder {
  pub fn set_env(mut self, key: &str, value: &str) -> Self {
    self.environment_vars.insert(key.to_string(), value.to_string());
    self
  }
  
  pub fn set_app_setting(mut self, key: &str, value: &str) -> Self {
    self.app_settings.insert(key.to_string(), value.to_string());
    self
  }
  
  pub fn set_system_setting(self, key: &str, value: &str) -> Result<Self, AppOptionsError> {
    match key {
      BODHI_ENV_TYPE => {
        let env_type = value.parse::<EnvType>()?;
        Ok(self.env_type(env_type))
      }
      BODHI_APP_TYPE => {
        let app_type = value.parse::<AppType>()?;
        Ok(self.app_type(app_type))
      }
      BODHI_VERSION => Ok(self.app_version(value)),
      BODHI_COMMIT_SHA => Ok(self.app_commit_sha(value)),
      BODHI_AUTH_URL => Ok(self.auth_url(value)),
      BODHI_AUTH_REALM => Ok(self.auth_realm(value)),
      key => Err(AppOptionsError::UnknownSystemSetting(key.to_string())),
    }
  }
}
```

### Settings Service Integration Implementation
Advanced settings management with file-based configuration and environment overrides:

```rust
// Settings service setup with comprehensive configuration management - see crates/lib_bodhiserver/src/app_dirs_builder.rs
fn setup_settings(
  options: &AppOptions,
  bodhi_home: PathBuf,
  source: SettingSource,
  file_defaults: HashMap<String, Value>,
) -> Result<DefaultSettingService, AppDirsBuilderError> {
  let settings_file = bodhi_home.join(SETTINGS_YAML);
  let app_version = file_defaults.get(BODHI_VERSION).map(|v| v.as_str()).unwrap_or(None);
  let app_commit_sha = file_defaults.get(BODHI_COMMIT_SHA).map(|v| v.as_str()).unwrap_or(None);
  let app_settings = build_system_settings(options, app_version, app_commit_sha);
  
  let setting_service = DefaultSettingService::new_with_defaults(
    options.env_wrapper.clone(),
    Setting {
      key: BODHI_HOME.to_string(),
      value: Value::String(bodhi_home.display().to_string()),
      source, metadata: SettingMetadata::String,
    },
    app_settings, file_defaults, settings_file,
  );
  
  // Load environment and apply app settings
  setting_service.load_default_env();
  for (key, value) in &options.app_settings {
    let metadata = setting_service.get_setting_metadata(key);
    let parsed_value = metadata.parse(Value::String(value.clone()));
    setting_service.set_setting_with_source(key, &parsed_value, SettingSource::SettingsFile);
  }
  
  Ok(setting_service)
}
```

**Configuration Management Features**:
- Environment variable integration with override precedence and validation
- YAML configuration file support with parsing and error handling
- System settings injection with version, commit SHA, and authentication configuration
- OAuth2 application registration information with secure credential storage
- Development/production mode coordination with environment-specific defaults

## Localization Resource Management

### Multi-Crate Resource Loading Implementation
Comprehensive localization support with resource loading from all workspace crates:

```rust
// Localization resource loading from all crates - see crates/lib_bodhiserver/src/app_service_builder.rs
fn load_all_localization_resources(localization_service: &FluentLocalizationService) -> Result<(), ErrorMessage> {
  localization_service
    .load_resource(objs::l10n::L10N_RESOURCES)?
    .load_resource(objs::gguf::l10n::L10N_RESOURCES)?
    .load_resource(llama_server_proc::l10n::L10N_RESOURCES)?
    .load_resource(services::l10n::L10N_RESOURCES)?
    .load_resource(commands::l10n::L10N_RESOURCES)?
    .load_resource(server_core::l10n::L10N_RESOURCES)?
    .load_resource(auth_middleware::l10n::L10N_RESOURCES)?
    .load_resource(routes_oai::l10n::L10N_RESOURCES)?
    .load_resource(routes_app::l10n::L10N_RESOURCES)?
    .load_resource(routes_all::l10n::L10N_RESOURCES)?
    .load_resource(server_app::l10n::L10N_RESOURCES)?
    .load_resource(crate::l10n::L10N_RESOURCES)?;
  Ok(())
}

// Localization service initialization with comprehensive error handling
fn get_or_build_localization_service(&mut self) -> Result<Arc<dyn LocalizationService>, ErrorMessage> {
  if let Some(service) = self.localization_service.take() {
    return Ok(service);
  }
  
  let localization_service = FluentLocalizationService::get_instance();
  load_all_localization_resources(&localization_service)?;
  Ok(localization_service)
}
```

**Localization Features**:
- Multi-language support with resource loading from all workspace crates
- Fluent message template integration with comprehensive error handling
- Singleton localization service with thread-safe access and caching
- Fallback support for missing translations with graceful degradation

## UI Asset Management Architecture

### Embedded Static Asset Integration
Next.js frontend integration with embedded asset serving for complete application embedding:

```rust
// Embedded UI assets for complete application integration - see crates/lib_bodhiserver/src/ui_assets.rs
pub static EMBEDDED_UI_ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../bodhi/out");

// Localization resources embedded at compile time
pub mod l10n {
  use include_dir::Dir;
  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
```

**UI Asset Features**:
- Compile-time embedding of Next.js frontend assets with static file serving
- Complete application integration with embedded web interface for desktop and NAPI scenarios
- Resource optimization with compressed assets and efficient serving
- Development/production asset coordination with appropriate caching and optimization

## Error Handling Architecture

### Comprehensive Error Types Implementation
Sophisticated error handling with localized messages and recovery strategies:

```rust
// Configuration error types with comprehensive handling - see crates/lib_bodhiserver/src/error.rs
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppOptionsError {
  #[error("validation_error: required property '{0}' is not set")]
  #[error_meta(code = "app_options_error-validation_error", error_type = ErrorType::BadRequest)]
  ValidationError(String),
  
  #[error(transparent)]
  #[error_meta(code = "app_options_error-parse_error", error_type = ErrorType::BadRequest, args_delegate = false)]
  Parse(#[from] strum::ParseError),
  
  #[error("unknown_system_setting: {0}")]
  #[error_meta(code = "app_options_error-unknown_system_setting", error_type = ErrorType::BadRequest)]
  UnknownSystemSetting(String),
}

// Directory setup error types with detailed context
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppDirsBuilderError {
  #[error("failed to automatically set BODHI_HOME. Set it through environment variable $BODHI_HOME and try again.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  BodhiHomeNotFound,
  
  #[error("io_error: failed to create directory {path}, error: {source}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  DirCreate { #[source] source: io::Error, path: String },
  
  #[error("Service already set: {0}")]
  #[error_meta(error_type=ErrorType::InternalServer)]
  ServiceAlreadySet(String),
}
```

**Error Handling Features**:
- Comprehensive error types with localized messages and error codes
- Context preservation with source error chaining and detailed information
- Recovery guidance with actionable error messages and resolution steps
- Integration with objs error system for consistent error handling across crates

## Cross-Crate Integration Implementation

### Service Registry Re-exports
Complete service interface exposure for external application integration:

```rust
// Service re-exports for external application access - see crates/lib_bodhiserver/src/lib.rs
pub use services::{
  AppRegInfo, AppService, AppStatus, DefaultAppService, DefaultEnvWrapper,
  DefaultSecretService, DefaultSettingService, EnvWrapper, SecretServiceExt,
  SettingService, BODHI_APP_TYPE, BODHI_AUTH_REALM, BODHI_AUTH_URL,
  BODHI_COMMIT_SHA, BODHI_ENCRYPTION_KEY, BODHI_ENV_TYPE, BODHI_EXEC_LOOKUP_PATH,
  BODHI_EXEC_VARIANT, BODHI_HOME, BODHI_HOST, BODHI_KEEP_ALIVE_SECS, BODHI_LOGS,
  BODHI_LOG_LEVEL, BODHI_LOG_STDOUT, BODHI_PORT, BODHI_PUBLIC_HOST,
  BODHI_PUBLIC_PORT, BODHI_PUBLIC_SCHEME, BODHI_SCHEME, BODHI_VERSION,
  DEFAULT_HOST, DEFAULT_PORT, DEFAULT_SCHEME, HF_HOME,
};

// Domain object re-exports for consistent API
pub use objs::{
  ApiError, AppError, AppType, EnvType, ErrorMessage, ErrorType,
  FluentLocalizationService, LogLevel, OpenAIApiError,
};

// Server management re-exports
pub use server_app::{ServeCommand, ServeError, ServerShutdownHandle};
```

### Test Utils Integration
Comprehensive testing infrastructure for embedded library scenarios:

```rust
// Test utilities for embedded library testing (see crates/lib_bodhiserver/src/test_utils/app_options_builder.rs for complete implementation)
impl AppOptionsBuilder {
  pub fn development() -> Self {
    Self::default()
      .env_type(EnvType::Development)
      .app_type(AppType::Container)
      .app_version(env!("CARGO_PKG_VERSION"))
      .auth_url("https://test-id.getbodhi.app")
      .auth_realm("bodhi")
  }
  
  pub fn with_bodhi_home(bodhi_home: &str) -> Self {
    Self::development().set_env(BODHI_HOME, bodhi_home)
  }
}
```

**Integration Features**:
- Complete service interface exposure for external applications with consistent API
- Test utilities for embedded library testing with realistic configuration scenarios
- Domain object re-exports for consistent error handling and data types across boundaries
- Server management integration for complete HTTP server functionality when needed

## Extension Guidelines for Embeddable Library

### Adding New Service Integration
When creating new service integration for embeddable library functionality:

1. **AppServiceBuilder Extensions**: Add new service methods with dependency injection patterns and automatic resolution
2. **Configuration Integration**: Extend AppOptions with new configuration options and validation rules for service setup
3. **Error Handling**: Create service-specific errors that implement AppError trait for consistent error reporting
4. **Resource Management**: Implement proper resource lifecycle management with cleanup and error recovery
5. **Testing Infrastructure**: Use comprehensive service mocking for isolated library integration testing

### Extending Configuration Management
For new configuration capabilities and embedded deployment scenarios:

1. **AppOptions Extensions**: Add new configuration options with builder pattern and validation support
2. **Settings Integration**: Coordinate with SettingService for new configuration management and environment variables
3. **Directory Management**: Extend directory setup for new resource types with proper permissions and error handling
4. **Environment Support**: Support new deployment environments with appropriate configuration defaults
5. **Configuration Testing**: Test configuration management with different embedded scenarios and validation failures

### Cross-Application Integration Patterns
For new embedding scenarios and external application integration:

1. **Service Composition**: Design service composition patterns that support different embedding scenarios
2. **Resource Management**: Implement resource lifecycle management that coordinates with external applications
3. **Error Boundaries**: Provide comprehensive error handling with proper isolation and recovery
4. **Performance Optimization**: Optimize service composition and resource management for embedded scenarios
5. **Integration Testing**: Support comprehensive embedding testing with realistic external application integration

## Commands for Embeddable Library Testing

**Library Integration Tests**: `cargo test -p lib_bodhiserver` (includes service composition and configuration testing)  
**Service Builder Tests**: `cargo test -p lib_bodhiserver app_service_builder` (includes dependency injection testing)  
**Configuration Tests**: `cargo test -p lib_bodhiserver app_options` (includes AppOptions and directory setup testing)  
**Test Utils Tests**: `cargo test -p lib_bodhiserver --features test-utils` (includes comprehensive testing infrastructure)