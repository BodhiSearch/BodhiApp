# App Service Builder Migration Documentation

## Overview

The App Service Builder is a centralized service initialization system that orchestrates the creation of all required services and their dependencies for the Bodhi application. This module was created to:

- **Consolidate service initialization logic** from multiple locations into a single, reusable library
- **Enable dependency injection** for better testability with mocked services
- **Provide clean separation** between platform-specific code and service initialization
- **Support multiple deployment modes** (native, CLI, Docker) with consistent service setup

The builder follows a modular design pattern, breaking down complex service initialization into smaller, focused functions while maintaining proper error handling and dependency management.

## Architecture

### Main Functions

The app service builder provides two primary entry points:

1. **`build_app_service(setting_service)`** - Standard production use
   - Takes a `SettingService` and creates all required services with default implementations
   - Handles all service dependencies and initialization order
   - Returns a fully configured `DefaultAppService`

2. **`build_app_service_with_config(setting_service, config)`** - Configurable for testing
   - Allows dependency injection through `AppServiceConfig`
   - Enables mocking of specific services (secret service, etc.)
   - Maintains the same initialization flow while allowing customization

### Helper Functions

The builder is decomposed into focused helper functions:

- `setup_data_services()` - Initializes hub and data services
- `setup_secret_service()` - Handles encryption key management and secret service setup
- `setup_app_database()` - Connects to and migrates the application database
- `setup_session_database()` - Connects to and migrates the session database
- `setup_auth_service()` - Configures Keycloak authentication service

## Key Components

### AppServiceBuilder

```rust
pub struct AppServiceBuilder {
  setting_service: Arc<dyn SettingService>,
  hub_service: Option<Arc<dyn HubService>>,
  data_service: Option<Arc<dyn DataService>>,
  // ... other services
}
```

**Purpose**: Provides a fluent builder pattern for constructing app services with dependency injection support.

**Usage**:
- Production: Use `build_app_service()` for simple cases or `AppServiceBuilder::new().build()` for default services
- Testing: Use `AppServiceBuilder::new().service_name(mock_service).build()` to inject specific mock services

### AppServiceBuilderError

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppServiceBuilderError {
  EncryptionKeyError(#[from] services::KeyringError),
  SecretServiceError(#[from] services::SecretServiceError),
  AppDbConnectionError(#[from] services::db::DbError),
  AppDbMigrationError(#[source] services::db::DbError),
  SessionDbConnectionError(#[source] services::db::DbError),
  SessionDbMigrationError(#[source] services::SessionServiceError),
}
```

**Purpose**: Provides comprehensive error handling for all service initialization failures.

**Design**: Follows the same simple error pattern as `AppDirsBuilderError` - uses plain `thiserror::Error` with user-friendly messages directly in the error templates.

## Migration Summary

### Before Migration

**File: `crates/bodhi/src-tauri/src/app.rs`** (~60 lines of service initialization)
```rust
async fn aexecute(setting_service: Arc<dyn SettingService>) -> Result<()> {
  let bodhi_home = setting_service.bodhi_home();
  let hf_cache = setting_service.hf_cache();
  let hub_service = Arc::new(HfHubService::new_from_hf_cache(hf_cache, true));
  let data_service = LocalDataService::new(bodhi_home.clone(), hub_service.clone());
  // ... 50+ more lines of complex service setup
  let app_service = DefaultAppService::new(/* 10+ parameters */);
}
```

**File: `crates/integration-tests/tests/utils/live_server_utils.rs`** (~55 lines of duplicated logic)
```rust
// Similar complex service initialization with test-specific modifications
let hub_service = Arc::new(OfflineHubService::new(/* ... */));
let data_service = LocalDataService::new(/* ... */);
// ... extensive service setup duplication
```

### After Migration

**File: `crates/bodhi/src-tauri/src/app.rs`** (3 lines)
```rust
async fn aexecute(setting_service: Arc<dyn SettingService>) -> Result<()> {
  let app_service = build_app_service(setting_service.clone()).await?;
  let service = Arc::new(app_service);
  // ... rest of function unchanged
}
```

**File: `crates/integration-tests/tests/utils/live_server_utils.rs`** (simplified with builder pattern)
```rust
let service = AppServiceBuilder::new(setting_service)
  .hub_service(hub_service)?
  .data_service(data_service)?
  .auth_service(Arc::new(auth_service))?
  .secret_service(Arc::new(secret_service))?
  .build()
  .await?;
```

**Reduction**:
- **Main app**: 60+ lines → 3 lines (95% reduction)
- **Integration tests**: 55+ lines → 8 lines (85% reduction)
- **Total**: ~115 lines of duplicated logic consolidated into reusable library

## Usage Examples

### Standard Production Usage

```rust
use lib_bodhiserver::build_app_service;

async fn initialize_app(setting_service: Arc<dyn SettingService>) -> Result<DefaultAppService> {
    let app_service = build_app_service(setting_service).await?;
    Ok(app_service)
}
```

### Testing with Dependency Injection

```rust
use lib_bodhiserver::AppServiceBuilder;
use services::MockSecretService;

#[tokio::test]
async fn test_with_mocked_secret_service() -> anyhow::Result<()> {
    let setting_service = Arc::new(setup_test_settings()?);

    // Create mock secret service
    let mut mock_secret_service = MockSecretService::new();
    mock_secret_service
        .expect_get_secret_string()
        .returning(|_| Ok(Some("test_value".to_string())));

    let app_service = AppServiceBuilder::new(setting_service)
        .secret_service(Arc::new(mock_secret_service))?
        .build()
        .await?;

    // Test with mocked services
    assert!(app_service.setting_service().app_db_path().exists());
    Ok(())
}
```

### Integration Test Setup

```rust
// Create test-specific services
let secret_service = create_test_secret_service_with_app_reg_info()?;
let hub_service = Arc::new(OfflineHubService::new(HfHubService::new(hf_cache, false, None)));
let auth_service = test_auth_service(&auth_url);

let app_service = AppServiceBuilder::new(setting_service)
    .hub_service(hub_service)?
    .auth_service(Arc::new(auth_service))?
    .secret_service(Arc::new(secret_service))?
    .build()
    .await?;
```

## Testing Strategy

The dependency injection pattern enables comprehensive testing strategies:

### 1. **Unit Testing of Helper Functions**
Each helper function can be tested independently:
- `setup_data_services()` - Test service creation and configuration
- `setup_secret_service()` - Test encryption key handling
- `setup_app_database()` - Test database connection and migration
- `setup_session_database()` - Test session storage setup

### 2. **Integration Testing with Mocks**
The `AppServiceBuilder` allows selective mocking:
- Mock secret service for testing without keyring dependencies
- Mock time service for deterministic time-based testing
- Test error scenarios by injecting failing mock services

### 3. **End-to-End Testing**
Full service integration can be tested with real services in controlled environments using temporary directories and test databases.

## Error Handling

### Error Integration

The `AppServiceBuilderError` is integrated into the main application error system:

```rust
// In crates/bodhi/src-tauri/src/error.rs
pub enum BodhiError {
    // ... other variants
    #[error("app_service_builder_error")]
    #[error_meta(error_type = ErrorType::InternalServer)]
    AppServiceBuilder(#[source] AppServiceBuilderError),
}

impl From<AppServiceBuilderError> for BodhiError {
    fn from(err: AppServiceBuilderError) -> Self {
        BodhiError::AppServiceBuilder(err)
    }
}
```

### Error Messages

Error messages are defined directly in thiserror templates:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppServiceBuilderError {
  #[error("Failed to generate or retrieve encryption key.")]
  EncryptionKeyError(#[from] services::KeyringError),
  #[error("Failed to initialize secret service.")]
  SecretServiceError(#[from] services::SecretServiceError),
  #[error("Failed to connect to application database.")]
  AppDbConnectionError(#[from] services::db::DbError),
  // ... additional error variants
}
```

### Error Propagation

Errors are properly propagated through the call stack:
1. Helper functions return specific `AppServiceBuilderError` variants
2. Main builder functions propagate these errors
3. Application code converts to `BodhiError` via `From` trait
4. User-friendly error messages are displayed via thiserror templates

## Files Changed

### Created Files
- `crates/lib_bodhiserver/src/app_service_builder.rs` - Main implementation (175 lines)
- `crates/lib_bodhiserver/app_service_builder.md` - This documentation

### Modified Files

#### Core Implementation
- `crates/lib_bodhiserver/src/lib.rs` - Added module export
- `crates/lib_bodhiserver/Cargo.toml` - Added tokio test dependency

#### Application Integration
- `crates/bodhi/src-tauri/src/app.rs` - Replaced 60+ lines with 3 lines
- `crates/bodhi/src-tauri/src/error.rs` - Added error variant and conversion

#### Test Integration
- `crates/integration-tests/tests/utils/live_server_utils.rs` - Simplified service setup

### Impact Summary
- **Lines of code reduced**: ~115 lines of duplicated service initialization
- **Maintainability**: Centralized service initialization logic
- **Testability**: Enhanced through dependency injection pattern
- **Reusability**: Service builder can be used across deployment modes
- **Error handling**: Comprehensive error coverage with user-friendly thiserror messages

The migration successfully consolidates complex service initialization logic into a clean, testable, and reusable library component that serves as the foundation for all Bodhi application deployments.
