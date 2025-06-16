# Backend Development Conventions

## Overview

This document defines the development conventions and best practices for Rust backend development in the BodhiApp project, covering dependency management, crate organization, and architectural patterns.

## Dependency Management

### Workspace Dependencies

**Rule**: Always declare dependencies in the workspace `Cargo.toml` and import them using `workspace = true` for internal crates.

**Rationale**: Ensures consistent versioning across the workspace, simplifies dependency updates, and prevents version conflicts.

**Implementation:**

```toml
# Workspace Cargo.toml
[workspace.dependencies]
anyhow = "1.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }

# Individual crate Cargo.toml
[dependencies]
anyhow = { workspace = true }
tokio = { workspace = true, features = ["rt-multi-thread"] }
serde = { workspace = true }
```

**Exception**: Only specify versions directly in individual crates for:
- Features that differ from workspace defaults
- Development/test-only dependencies with specific requirements

### Internal Crate Dependencies

**Single Dependency Principle**: Library crates should minimize their dependency surface area and provide clean re-export interfaces.

**Example**: `lib_bodhiserver` serves as the single dependency gateway:
```toml
# Good: NAPI crate depends only on lib_bodhiserver
[dependencies]
lib_bodhiserver = { workspace = true }

# Bad: Multiple internal dependencies
[dependencies]
lib_bodhiserver = { workspace = true }
services = { workspace = true }  # VIOLATION
objs = { workspace = true }      # VIOLATION
```

### Test Dependencies

**Rule**: Use `test-utils` features for test-specific functionality, but avoid them in production code.

```toml
# For test utilities
[dev-dependencies]
services = { workspace = true, features = ["test-utils"] }

# For production code - use re-exports instead
[dependencies]
lib_bodhiserver = { workspace = true }  # Should re-export needed test types
```

## Crate Architecture

### Re-export Strategy

**Gateway Crates**: Library crates should re-export all types needed by their consumers.

```rust
// lib_bodhiserver/src/lib.rs
// Re-exports for dependency isolation
pub use objs::{AppError, AppType, EnvType, ErrorMessage, ErrorType, LogLevel};
pub use services::{AppService, DefaultAppService, SettingService, SecretServiceExt};
pub use server_app::{ServeCommand, ServeError, ServerShutdownHandle};

// Test utilities (behind feature flag)
#[cfg(feature = "test-utils")]
pub use services::test_utils::{EnvWrapperStub, SecretServiceStub};
#[cfg(feature = "test-utils")]
pub use objs::test_utils::{setup_l10n, set_mock_localization_service};
```

### Feature Flags

**Rule**: Use feature flags for optional functionality, especially test utilities and platform-specific code.

```toml
[features]
default = []
test-utils = ["services/test-utils", "objs/test-utils"]
native = ["tauri"]
```

```rust
#[cfg(feature = "test-utils")]
pub mod test_utils;

#[cfg(feature = "native")]
pub use tauri_specific_module::*;
```

## Configuration Management

### Settings Categories

**System Settings** (cannot be overridden):
- `BODHI_HOME`, `BODHI_ENV_TYPE`, `BODHI_APP_TYPE`
- `BODHI_VERSION`, `BODHI_AUTH_URL`, `BODHI_AUTH_REALM`

**App Settings** (configurable via settings.yaml):
- `HF_HOME`, `BODHI_LOGS`, `BODHI_LOG_LEVEL`, `BODHI_LOG_STDOUT`
- `BODHI_SCHEME`, `BODHI_HOST`, `BODHI_PORT`
- `BODHI_EXEC_VARIANT`, `BODHI_EXEC_LOOKUP_PATH`, `BODHI_KEEP_ALIVE_SECS`

**Environment Settings** (environment variables only):
- `BODHI_ENCRYPTION_KEY`

**Secret Settings** (encrypted storage):
- App registration info, client secrets, app status

### Unified Configuration Interface

**Preferred Pattern**: Use settings-based configuration instead of multiple individual parameters.

```rust
// Good: Unified settings approach
pub fn set_setting(&self, key: &str, value: &str) -> Result<()>;
pub fn set_system_setting(&self, key: &str, value: &str) -> Result<()>;
pub fn set_env_setting(&self, key: &str, value: &str) -> Result<()>;
pub fn set_secret(&self, key: &str, value: &str) -> Result<()>;

// Bad: Multiple individual parameters
pub struct AppConfig {
  pub env_type: String,
  pub app_type: String,
  pub app_version: String,
  pub auth_url: String,
  // ... many more fields
}
```

## Error Handling

### Error Types

**Rule**: Use `thiserror` for error enums with proper error conversion chains.

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ServiceError {
  #[error("Configuration error: {message}")]
  #[error_meta(error_type = ErrorType::InvalidRequest)]
  Config { message: String },
  
  #[error(transparent)]
  Io(#[from] std::io::Error),
}
```

### Error Conversion

**Pattern**: Implement proper error conversion chains for clean error propagation.

```rust
impl From<ServiceError> for ErrorMessage {
  fn from(value: ServiceError) -> Self {
    ErrorMessage::new(value.code(), value.error_type(), value.to_string())
  }
}
```

## Testing Conventions

### Test Organization

**Rule**: Co-locate tests with source code and use descriptive naming.

```rust
#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn test_service_initialization_success() {
    // Test implementation
  }
  
  #[test]
  fn test_service_initialization_failure_invalid_config() {
    // Test implementation
  }
}
```

### Test Utilities

**Pattern**: Provide test utilities through feature-flagged modules.

```rust
#[cfg(feature = "test-utils")]
pub mod test_utils {
  pub fn create_test_service() -> TestService {
    // Test utility implementation
  }
}
```

## Documentation

### Code Documentation

**Rule**: Document public APIs with examples and error conditions.

```rust
/// Creates a new service instance with the provided configuration.
///
/// # Arguments
/// * `config` - Service configuration parameters
///
/// # Returns
/// * `Ok(Service)` - Successfully created service
/// * `Err(ServiceError)` - Configuration validation failed
///
/// # Example
/// ```rust
/// let config = ServiceConfig::default();
/// let service = create_service(config)?;
/// ```
pub fn create_service(config: ServiceConfig) -> Result<Service, ServiceError> {
  // Implementation
}
```

### README Requirements

**Rule**: Each crate must have a README.md explaining:
- Purpose and scope
- Exported objects and functions
- Usage examples
- Dependencies and features

## Memory

- Always use workspace dependencies with `workspace = true` for internal crates
- Avoid direct dependencies on internal crates when a gateway crate provides re-exports
- Use feature flags for test utilities and optional functionality
- Implement unified configuration management instead of multiple parameters
- Follow proper error handling patterns with thiserror and error conversion chains
