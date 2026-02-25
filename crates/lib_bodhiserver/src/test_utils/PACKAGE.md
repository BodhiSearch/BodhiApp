# PACKAGE.md - lib_bodhiserver test_utils

This document provides detailed technical information for the `lib_bodhiserver` test_utils module, focusing on testing infrastructure for embeddable server library scenarios and comprehensive service composition testing.

## Test Utils Architecture

The `lib_bodhiserver` test_utils module provides sophisticated testing infrastructure for embeddable library scenarios, service composition testing, and configuration management validation.

### AppOptionsBuilder Test Extensions
Comprehensive testing utilities for configuration and embedded library scenarios:

```rust
// Development configuration builder for testing (see src/test_utils/app_options_builder.rs:5-15 for complete implementation)
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

**Key Testing Features**:
- Development configuration presets with realistic authentication and environment settings
- BODHI_HOME configuration for isolated testing with temporary directories
- Container application type setup for embedded library testing scenarios
- Test authentication provider configuration with development-specific endpoints

## Testing Patterns for Embeddable Library

### Service Composition Testing
Comprehensive testing infrastructure for AppServiceBuilder and service dependency injection:

```rust
// Example service composition testing pattern
#[rstest]
#[tokio::test]
async fn test_app_service_builder_with_custom_services(
  empty_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
  let setting_service = Arc::new(setup_app_dirs(&options)?);
  
  // Create mock services for testing
  let mut mock_secret_service = MockSecretService::new();
  mock_secret_service.expect_set_secret_string().returning(|_, _| Ok(()));
  mock_secret_service.expect_get_secret_string().returning(|_| Ok(Some("test_value".to_string())));
  
  let time_service = Arc::new(FrozenTimeService::default());
  
  let app_service = AppServiceBuilder::new(setting_service.clone())
    .secret_service(Arc::new(mock_secret_service))?
    .time_service(time_service.clone())?
    .build()
    .await?;
  
  // Verify all services are properly initialized
  assert_eq!(app_service.setting_service().bodhi_home(), bodhi_home);
  assert!(setting_service.app_db_path().exists());
  assert!(setting_service.session_db_url().exists());
  
  Ok(())
}
```

### Configuration Testing Patterns
Comprehensive configuration validation and error handling testing:

```rust
// Configuration validation testing with error scenarios
#[rstest]
fn test_service_already_set_errors() -> anyhow::Result<()> {
  let setting_service = Arc::new(services::test_utils::SettingServiceStub::default());
  let mock_secret_service = Arc::new(MockSecretService::new());
  
  let builder = AppServiceBuilder::new(setting_service).secret_service(mock_secret_service.clone())?;
  
  // Test duplicate service setting error
  let result = builder.secret_service(mock_secret_service);
  
  assert!(result.is_err());
  assert!(matches!(
    result.unwrap_err(),
    AppServiceBuilderError::ServiceAlreadySet(service) if service == *"secret_service"
  ));
  
  Ok(())
}
```

### Directory Setup Testing
Comprehensive testing for application directory management and filesystem operations:

```rust
// Directory setup testing with realistic scenarios
#[rstest]
fn test_setup_app_dirs_integration(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let options = AppOptionsBuilder::development()
    .set_env(BODHI_HOME, &bodhi_home.display().to_string())
    .build()?;
    
  let _settings_service = setup_app_dirs(&options)?;
  
  // Verify all required directories and files are created
  assert!(bodhi_home.join(TEST_ALIASES_DIR).exists());
  assert!(bodhi_home.join(TEST_PROD_DB).exists());
  assert!(bodhi_home.join(TEST_SESSION_DB).exists());
  
  Ok(())
}
```

## Test Fixture Patterns

### Temporary Directory Management
Sophisticated temporary directory management for isolated testing:

```rust
// Temporary directory fixtures for isolated testing
use objs::test_utils::{empty_bodhi_home, temp_dir};
use tempfile::TempDir;

// Usage in tests with automatic cleanup
#[rstest]
fn test_with_isolated_environment(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
  
  // Test operations with isolated environment
  let setting_service = setup_app_dirs(&options)?;
  assert_eq!(setting_service.bodhi_home(), bodhi_home);
  
  Ok(())
  // Automatic cleanup when TempDir is dropped
}
```

### Service Mock Coordination
Comprehensive service mocking patterns for isolated testing:

```rust
// Service mock coordination for complex testing scenarios
use services::{MockSecretService, MockCacheService};
use services::test_utils::FrozenTimeService;

// Mock service setup with realistic expectations
let mut mock_secret_service = MockSecretService::new();
mock_secret_service
  .expect_set_secret_string()
  .returning(|_, _| Ok(()));
mock_secret_service
  .expect_get_secret_string()
  .returning(|_| Ok(Some("test_value".to_string())));

let app_service = AppServiceBuilder::new(setting_service)
  .secret_service(Arc::new(mock_secret_service))?
  .cache_service(Arc::new(MockCacheService::new()))?
  .build()
  .await?;
```

### Configuration Testing Fixtures
Comprehensive configuration testing with different scenarios:

```rust
// Configuration testing with app settings and environment variables
#[rstest]
fn test_setup_app_dirs_with_app_settings(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
  use services::BODHI_PORT;
  
  let bodhi_home = empty_bodhi_home.path().join("bodhi_enhanced");
  let bodhi_home_str = bodhi_home.display().to_string();
  
  // Create options with app settings
  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str)
    .set_env("TEST_VAR", "test_value")
    .set_app_setting(BODHI_PORT, "9090")
    .set_system_setting(services::BODHI_ENV_TYPE, "development")?
    .build()?;
  
  let setting_service = setup_app_dirs(&options)?;
  
  // Verify configuration was applied
  assert_eq!(setting_service.get_setting(BODHI_PORT).unwrap(), "9090");
  assert!(!setting_service.is_production());
  assert!(setting_service.bodhi_home().exists());
  
  Ok(())
}
```

## Error Testing Patterns

### Configuration Error Testing
Comprehensive error scenario testing for configuration validation:

```rust
// Configuration error testing with validation failures
#[rstest]
fn test_find_bodhi_home_fails_when_not_found() -> anyhow::Result<()> {
  let mut mock_env_wrapper = MockEnvWrapper::default();
  mock_env_wrapper
    .expect_var()
    .with(eq(BODHI_HOME))
    .times(1)
    .return_once(|_| Err(VarError::NotPresent));
  mock_env_wrapper
    .expect_home_dir()
    .times(1)
    .return_once(|| None);
    
  let env_wrapper: Arc<dyn EnvWrapper> = Arc::new(mock_env_wrapper);
  let options = AppOptions::new(/* ... */);
  
  let result = super::find_bodhi_home(options.env_wrapper.clone(), &options.env_type, &file_defaults);
  
  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AppDirsBuilderError::BodhiHomeNotFound));
  
  Ok(())
}
```

### Service Initialization Error Testing
Comprehensive error handling testing for service composition failures:

```rust
// Service initialization error testing with realistic failure scenarios
#[rstest]
fn test_setup_hf_home_fails_when_no_home_dir() -> anyhow::Result<()> {
  let mut mock = MockSettingService::default();
  mock.expect_get_setting().with(eq(HF_HOME)).times(1).return_const(None);
  mock.expect_home_dir().times(1).return_const(None);
  
  let setting_service: Arc<dyn SettingService> = Arc::new(mock);
  let result = super::setup_hf_home(setting_service.as_ref());
  
  assert!(matches!(result, Err(AppDirsBuilderError::HfHomeNotFound)));
  
  Ok(())
}
```

## Integration Testing Patterns

### Complete Application Bootstrap Testing
End-to-end testing for complete application initialization:

```rust
// Complete application bootstrap testing with realistic scenarios
#[rstest]
#[tokio::test]
async fn test_complete_app_service_composition(
  empty_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let bodhi_home_str = bodhi_home.display().to_string();
  
  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str).build()?;
  let setting_service = Arc::new(setup_app_dirs(&options)?);
  
  let app_service = AppServiceBuilder::new(setting_service.clone()).build().await?;
  
  // Verify complete service composition
  assert_eq!(app_service.setting_service().bodhi_home(), bodhi_home);
  assert!(setting_service.app_db_path().exists());
  assert!(setting_service.session_db_url().exists());
  
  // Test service interactions
  let data_service = app_service.data_service();
  let hub_service = app_service.hub_service();
  let auth_service = app_service.auth_service();
  
  // Verify services are properly initialized and accessible
  assert!(data_service.aliases_dir().exists());
  
  Ok(())
}
```

### Cross-Service Integration Testing
Comprehensive testing for service interactions and coordination:

```rust
// Cross-service integration testing with realistic workflows
#[rstest]
#[tokio::test]
async fn test_service_integration_workflows(
  empty_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
  let setting_service = Arc::new(setup_app_dirs(&options)?);
  
  let app_service = AppServiceBuilder::new(setting_service).build().await?;
  
  // Test cross-service workflows
  let data_service = app_service.data_service();
  let secret_service = app_service.secret_service();
  let db_service = app_service.db_service();
  
  // Test service coordination patterns
  // (Add specific service interaction tests based on actual workflows)
  
  Ok(())
}
```

## Performance Testing Patterns

### Service Composition Performance Testing
Performance testing for service initialization and composition:

```rust
// Service composition performance testing
#[rstest]
#[tokio::test]
async fn test_service_composition_performance(
  empty_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
  let setting_service = Arc::new(setup_app_dirs(&options)?);
  
  let start = std::time::Instant::now();
  let app_service = AppServiceBuilder::new(setting_service).build().await?;
  let duration = start.elapsed();
  
  // Verify reasonable initialization time
  assert!(duration.as_millis() < 1000, "Service composition took too long: {:?}", duration);
  
  // Verify all services are properly initialized
  assert!(app_service.setting_service().bodhi_home().exists());
  
  Ok(())
}
```

## Extension Guidelines for Test Utils

### Adding New Test Fixtures
When creating new test fixtures for embeddable library testing:

1. **Configuration Fixtures**: Create realistic configuration scenarios with proper environment and settings management
2. **Service Mock Fixtures**: Provide comprehensive service mocking patterns with realistic expectations and behaviors
3. **Directory Fixtures**: Support isolated testing with temporary directories and proper cleanup coordination
4. **Error Scenario Fixtures**: Create comprehensive error testing scenarios with realistic failure conditions
5. **Integration Fixtures**: Support end-to-end testing with complete service composition and realistic workflows

### Extending Testing Patterns
For new testing capabilities and embedded library scenarios:

1. **Service Composition Testing**: Extend testing patterns for new service types and dependency injection scenarios
2. **Configuration Testing**: Add new configuration validation and error handling testing patterns
3. **Performance Testing**: Create performance testing patterns for service initialization and resource management
4. **Integration Testing**: Support comprehensive integration testing with realistic external application scenarios
5. **Error Recovery Testing**: Test error recovery and graceful degradation patterns for embedded library failures

## Commands for Test Utils

**Test Utils Tests**: `cargo test -p lib_bodhiserver --features test-utils` (includes comprehensive testing infrastructure)  
**Configuration Tests**: `cargo test -p lib_bodhiserver test_utils::app_options_builder` (includes AppOptionsBuilder testing)  
**Service Composition Tests**: `cargo test -p lib_bodhiserver app_service_builder` (includes dependency injection testing)  
**Integration Tests**: `cargo test -p lib_bodhiserver --test integration` (includes end-to-end testing scenarios)