# PACKAGE.md - bodhi/src-tauri

This document provides detailed technical information for the bodhi/src-tauri crate, focusing on BodhiApp-specific dual-mode application architecture patterns and deployment orchestration.

## Architecture Position

The bodhi/src-tauri crate serves as BodhiApp's application entry point orchestration layer, providing sophisticated dual-mode deployment capabilities through feature-based conditional compilation. It coordinates complete application embedding with lib_bodhiserver while supporting both native desktop and container deployment scenarios.

## Implementation Architecture

### Dual-Mode Application System

The crate implements sophisticated conditional compilation architecture enabling multiple deployment modes through feature flags:

**Feature-Based Compilation Pattern** (see src/lib.rs:1-8):

```rust
#[cfg(feature = "native")]
mod native_init;
#[cfg(not(feature = "native"))]
mod server_init;
```

The `native` feature flag controls compilation of different initialization modules, enabling the same codebase to produce either a Tauri desktop application or a headless server executable.

**Unified CLI Interface** (see src/app.rs:15-45):
The application provides a unified command-line interface through clap integration with feature-conditional subcommand availability. The CLI adapts behavior based on compilation features while maintaining consistent user experience.

**AppCommand Enum Pattern** (see src/app.rs:8-12):

```rust
#[derive(Debug, Clone)]
pub enum AppCommand {
  Server(Option<String>, Option<u16>), // host, port
  Default,
}
```

The unified command representation supports both server deployment and native desktop modes with parameter extraction and validation.

### Native Desktop Implementation

**Tauri Framework Integration** (see src/native_init.rs:45-120):
Native mode leverages Tauri framework for cross-platform desktop application functionality with system tray integration, menu management, and embedded web UI serving. The implementation coordinates embedded server lifecycle with automatic startup and graceful shutdown.

**NativeCommand Structure** (see src/native_init.rs:25-35):

```rust
pub struct NativeCommand {
  service: Arc<dyn AppService>,
  ui: bool,
}

impl NativeCommand {
  pub fn new(service: Arc<dyn AppService>, ui: bool) -> Self {
    Self { service, ui }
  }
}
```

**System Tray Integration Pattern** (see src/native_init.rs:95-110):

```rust
let homepage = MenuItem::with_id(app, "homepage", "Open Homepage", true, None::<&str>)?;
let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
let menu = Menu::with_items(app, &[&homepage, &quit])?;
TrayIconBuilder::new()
  .menu(&menu)
  .show_menu_on_left_click(true)
  .icon(app.default_window_icon().unwrap().clone())
  .on_menu_event(move |app, event| {
    on_menu_event(app, event, &addr);
  })
  .build(app)?;
```

**System Integration Patterns**:

- System tray with menu-driven controls for homepage access and application shutdown
- Embedded server orchestration through lib_bodhiserver with ServeCommand coordination
- Web browser integration with automatic launching and fallback error handling
- Platform-specific activation policies and window management

### Container Deployment Implementation

**Headless Server Architecture** (see src/server_init.rs:85-130):
Container mode provides headless server deployment optimized for containerized environments with comprehensive file-based logging and configuration management. The implementation leverages lib_bodhiserver's ServeCommand for HTTP API serving.

**Configuration Override Pattern** (see src/server_init.rs:45-65):

```rust
if let AppCommand::Server(host, port) = command {
  if let Some(host) = host {
    SettingService::set_setting_with_source(
      &setting_service,
      BODHI_HOST,
      &Value::String(host),
      SettingSource::CommandLine,
    );
  }
  if let Some(port) = port {
    SettingService::set_setting_with_source(
      &setting_service,
      BODHI_PORT,
      &Value::Number(Number::from(port)),
      SettingSource::CommandLine,
    );
  }
}
```

**Logging Infrastructure Patterns** (see src/server_init.rs:95-140):

```rust
fn setup_logs(setting_service: &lib_bodhiserver::DefaultSettingService) -> WorkerGuard {
  let logs_dir = setting_service.logs_dir();
  let file_appender = tracing_appender::rolling::daily(logs_dir, "bodhi.log");
  let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
  let log_level: LevelFilter = setting_service.log_level().into();
  // Environment-specific logging configuration with filtering
}
```

**Logging Features**:

- File-based logging with daily rotation and configurable output targets
- Structured logging with environment-specific configuration and filtering
- Comprehensive error logging with context preservation and recovery guidance

## Cross-Crate Integration Implementation

### lib_bodhiserver Integration

**Service Composition Coordination** (see src/native_init.rs:130-150 and src/server_init.rs:95-115):
Both deployment modes coordinate with lib_bodhiserver for complete application service composition through AppServiceBuilder pattern with dependency injection and error handling.

**Configuration Management Integration** (see src/common.rs:5-15):

```rust
pub fn build_app_options(app_type: AppType) -> Result<AppOptions, ErrorMessage> {
  Ok(
    AppOptionsBuilder::default()
      .env_type(ENV_TYPE.clone())
      .app_type(app_type)
      .app_version(env!("CARGO_PKG_VERSION"))
      .app_commit_sha("not-set")
      .auth_url(AUTH_URL)
      .auth_realm(AUTH_REALM)
      .build()?,
  )
}
```

**Environment-Specific Configuration** (see src/env.rs:5-25):

```rust
#[cfg(feature = "production")]
mod env_config {
  pub static ENV_TYPE: EnvType = EnvType::Production;
  pub static AUTH_URL: &str = "https://id.getbodhi.app";
  pub static AUTH_REALM: &str = "bodhi";
}

#[cfg(not(feature = "production"))]
mod env_config {
  pub static ENV_TYPE: EnvType = EnvType::Development;
  pub static AUTH_URL: &str = "https://main-id.getbodhi.app";
  pub static AUTH_REALM: &str = "bodhi";
}
```

**UI Asset Integration** (see src/ui.rs:1-5):

```rust
// Re-export embedded UI assets from lib_bodhiserver
pub use lib_bodhiserver::EMBEDDED_UI_ASSETS as ASSETS;
```

**Configuration Features**:

- AppOptions construction with environment detection and OAuth endpoint configuration
- Settings service coordination for configuration management with file-based and environment variable overrides
- Resource path management for BODHI_EXEC_LOOKUP_PATH and LLM server binary discovery
- Embedded UI asset serving through lib_bodhiserver integration

### Error Handling Integration

**Error Translation Patterns** (see src/native_init.rs:15-25):
The crate implements comprehensive error handling with errmeta_derive integration for consistent error metadata and localization support:

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum NativeError {
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, code = "tauri", args_delegate = false)]
  Tauri(#[from] tauri::Error),
  #[error(transparent)]
  Serve(#[from] ServeError),
}
```

**Application Setup Error Handling** (see src/error.rs:5-15):

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppSetupError {
  #[error("io_error: error spawning async runtime: {0}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  AsyncRuntime(#[from] io::Error),
}
```

**Error Message Conversion Pattern** (see src/error.rs:15-20):

```rust
impl From<AppSetupError> for ErrorMessage {
  fn from(value: AppSetupError) -> Self {
    ErrorMessage::new(value.code(), value.error_type(), value.to_string())
  }
}
```

## Testing Architecture

### CLI Testing Patterns

The crate implements comprehensive CLI testing with feature-conditional test scenarios:

**Feature-Conditional Testing** (see src/app.rs:85-150):

```rust
#[cfg(not(feature = "native"))]
#[cfg(test)]
mod server_test {
  // Server-specific CLI tests
}

#[cfg(feature = "native")]
#[cfg(test)]
mod native_test {
  // Native desktop-specific CLI tests
}
```

**CLI Validation Testing** (see src/app.rs:95-120):

```rust
#[rstest]
#[case(vec!["bodhi", "serve", "-H", "0.0.0.0"], Some("0.0.0.0"), None)]
#[case(vec!["bodhi", "serve", "-p", "8080"], None, Some(8080))]
#[case(vec!["bodhi", "serve", "-H", "0.0.0.0", "-p", "8080"], Some("0.0.0.0"), Some(8080))]
fn test_cli_serve_valid(
  #[case] args: Vec<&str>,
  #[case] expected_host: Option<&str>,
  #[case] expected_port: Option<u16>,
) -> anyhow::Result<()> {
  // CLI parameter validation testing
}
```

**Error Scenario Testing** (see src/error.rs:25-40):

```rust
#[test]
fn test_app_setup_error_async_runtime_to_error_message() {
  let io_err = io::Error::other("simulated async runtime failure");
  let app_setup_err = AppSetupError::AsyncRuntime(io_err);
  let err_msg: ErrorMessage = app_setup_err.into();
  // Error message validation with expected format
}
```

### Test Utils Architecture

**Current Test Utils Structure** (see src/test_utils/mod.rs):
The test_utils module is currently minimal, providing a foundation for future desktop application testing utilities. The module supports both feature-conditional compilation and test-only compilation patterns.

**Testing Infrastructure Features**:

- Feature-conditional test compilation with `#[cfg(feature = "test-utils")]`
- Test-only compilation with `#[cfg(all(not(feature = "test-utils"), test))]`
- CLI parameter validation with rstest fixtures
- Error handling testing with comprehensive error message validation
- Cross-platform CLI testing with feature-specific test scenarios

## Extension Guidelines

### Adding New Deployment Modes

When implementing additional deployment scenarios:

1. **Feature Flag Design**: Create new feature flags with appropriate conditional compilation patterns following the existing `native`/`not(native)` model
2. **Initialization Module**: Implement new initialization modules following the pattern established by `native_init.rs` and `server_init.rs`
3. **CLI Integration**: Extend the clap command structure in `src/app.rs` with new subcommands and parameter validation
4. **Configuration Coordination**: Coordinate with lib_bodhiserver's AppOptions pattern for new deployment-specific configuration

### Extending Native Desktop Features

For new native desktop functionality:

1. **Tauri Integration**: Leverage Tauri framework capabilities through proper plugin integration and system API access
2. **System Integration**: Design cross-platform system features with proper conditional compilation for platform-specific functionality
3. **Resource Management**: Implement proper resource lifecycle management with cleanup coordination through the existing server handle patterns
4. **Error Handling**: Follow the established error handling patterns with errmeta_derive integration for consistent error reporting

### Container Deployment Extensions

For new container and server deployment capabilities:

1. **Configuration Management**: Extend command-line parameter processing with settings service integration following the existing host/port override patterns
2. **Logging Infrastructure**: Design comprehensive logging strategies following the established file-based logging patterns with rotation and filtering
3. **Service Orchestration**: Coordinate with lib_bodhiserver following the established ServeCommand patterns for HTTP server functionality
4. **Resource Optimization**: Optimize resource usage for containerized environments with proper cleanup and error handling

### Desktop Application Testing Extensions

For comprehensive desktop application testing:

1. **Tauri Testing Integration**: Leverage Tauri's testing capabilities for desktop application lifecycle testing
2. **System Tray Testing**: Design test patterns for system tray functionality and menu interaction validation
3. **Server Lifecycle Testing**: Test embedded server coordination with proper startup/shutdown validation
4. **Cross-Platform Testing**: Validate desktop features across different operating systems with platform-specific test scenarios
5. **UI Integration Testing**: Test web browser integration and embedded UI serving with realistic user interaction patterns

### Test Utils Enhancement Patterns

For expanding the test_utils module:

1. **Desktop Application Fixtures**: Create fixtures for Tauri application testing with proper lifecycle management
2. **CLI Testing Utilities**: Extend CLI testing utilities with comprehensive parameter validation and error scenario testing
3. **Configuration Testing**: Design configuration testing patterns for both native and container deployment modes
4. **Error Testing Infrastructure**: Expand error testing with comprehensive error message validation and recovery testing
5. **Integration Testing Support**: Support integration testing with realistic service coordination and resource management

## Commands

### Development Commands

```bash
# Build native desktop application
cargo build -p bodhi --features native

# Build container/server executable
cargo build -p bodhi

# Run tests with native features
cargo test -p bodhi --features native

# Run tests without native features
cargo test -p bodhi

# Run with test-utils feature for enhanced testing
cargo test -p bodhi --features test-utils
```

### Testing Commands

```bash
# Test CLI parsing with native features
cargo test -p bodhi --features native -- test_cli_native

# Test CLI parsing without native features
cargo test -p bodhi -- test_cli_non_native

# Test application initialization and CLI validation
cargo test -p bodhi -- test_cli_debug_assert

# Test error handling patterns
cargo test -p bodhi -- test_app_setup_error

# Test CLI parameter validation with rstest
cargo test -p bodhi -- test_cli_serve_valid

# Test CLI error scenarios
cargo test -p bodhi -- test_cli_serve_invalid_port
```

### Build Configuration

The crate uses sophisticated build configuration through Cargo.toml features:

- `native`: Enables Tauri desktop application functionality with system integration (includes tauri, tauri-plugin-log, webbrowser)
- `test-utils`: Provides testing utilities and fixtures (currently minimal foundation)
- `production`: Enables production-specific configuration and optimizations

**Key Dependencies**:

- **Core**: errmeta_derive, objs, lib_bodhiserver for application foundation
- **Native Desktop**: tauri with tray-icon, tauri-plugin-log, webbrowser for desktop functionality
- **CLI**: clap with derive features for command-line interface
- **Logging**: tracing, tracing-appender, tracing-subscriber for comprehensive logging
- **Async Runtime**: tokio with full features for async coordination

Build dependencies include tauri-build for native desktop builds and comprehensive file system utilities (fs2, fs_extra) for build-time asset management.
