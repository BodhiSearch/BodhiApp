# PACKAGE.md - lib_bodhiserver_napi

This document provides detailed technical information for the `lib_bodhiserver_napi` crate, focusing on BodhiApp's Node.js binding architecture, sophisticated NAPI integration patterns, and comprehensive cross-platform deployment coordination.

## Node.js Binding Architecture

The `lib_bodhiserver_napi` crate serves as BodhiApp's **Node.js binding orchestration layer**, implementing advanced NAPI-based bindings for embedding complete BodhiApp server functionality into Node.js applications with sophisticated configuration management and cross-platform support.

### NAPI Binding Implementation Architecture

Advanced Node.js integration with comprehensive server lifecycle management:

```rust
// Core NAPI server wrapper (see src/server.rs:15-35 for complete implementation)
#[napi]
pub struct BodhiServer {
  config: NapiAppOptions,
  shutdown_handle: Arc<Mutex<Option<ServerShutdownHandle>>>,
  temp_dir: Option<TempDir>,
  log_guard: Option<WorkerGuard>,
}

#[napi]
impl BodhiServer {
  #[napi(constructor)]
  pub fn new(config: NapiAppOptions) -> Result<Self> {
    Ok(Self {
      config,
      shutdown_handle: Arc::new(Mutex::new(None)),
      temp_dir: None,
      log_guard: None,
    })
  }

  #[napi]
  pub async unsafe fn start(&mut self) -> Result<()> {
    // Configuration translation and validation
    let builder = try_build_app_options_internal(self.config.clone())?;
    let app_options = builder.build()?;

    // Service composition coordination
    let setting_service = Arc::new(setup_app_dirs(&app_options)?);
    let app_service: Arc<dyn AppService> = Arc::new(build_app_service(setting_service.clone()).await?);

    // Server lifecycle management
    let serve_command = ServeCommand::ByParams { host: self.host(), port: self.port() };
    let handle = serve_command.get_server_handle(app_service, Some(&EMBEDDED_UI_ASSETS)).await?;

    // State synchronization
    let mut handle_guard = self.shutdown_handle.lock().await;
    *handle_guard = Some(handle);
    Ok(())
  }
}
```

### Configuration Management Implementation

Sophisticated multi-layer configuration system bridging Node.js and Rust environments:

```rust
// Flexible NAPI configuration structure (see src/config.rs:8-25 for complete implementation)
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NapiAppOptions {
  pub env_vars: HashMap<String, String>,
  pub app_settings: HashMap<String, String>,
  pub system_settings: HashMap<String, String>,
  pub client_id: Option<String>,
  pub client_secret: Option<String>,
  pub app_status: Option<String>,
}

// Configuration builder functions for JavaScript integration
#[napi]
pub fn create_napi_app_options() -> NapiAppOptions {
  NapiAppOptions {
    env_vars: HashMap::new(),
    app_settings: HashMap::new(),
    system_settings: HashMap::new(),
    client_id: None,
    client_secret: None,
    app_status: None,
  }
}

#[napi]
pub fn set_env_var(mut config: NapiAppOptions, key: String, value: String) -> NapiAppOptions {
  config.env_vars.insert(key, value);
  config
}

// Internal configuration translation (see src/config.rs:85-125 for complete implementation)
pub fn try_build_app_options_internal(config: NapiAppOptions) -> Result<AppOptionsBuilder, AppOptionsError> {
  let mut builder = AppOptionsBuilder::default();

  // Environment variable coordination
  for (key, value) in config.env_vars {
    builder = builder.set_env(&key, &value);
  }

  // OAuth2 credentials management
  if let (Some(client_id), Some(client_secret)) = (config.client_id, config.client_secret) {
    builder = builder.set_app_reg_info(&client_id, &client_secret);
  }

  Ok(builder)
}
```

**Key Configuration Features**:

- Multi-layer configuration with environment variables, app settings, and system settings coordination
- OAuth2 client credentials management with secure storage and authentication flow support
- Application status management with validation and state synchronization across language boundaries
- Configuration validation with comprehensive error handling and recovery guidance for Node.js applications
- Environment-specific configuration with development/production mode coordination and resource management

## Cross-Platform Integration Architecture

### Native Module Build System Implementation

Advanced cross-platform compilation with automated build pipeline:

```json
// Cross-platform build configuration (see package.json:15-25 for complete configuration)
{
  "name": "@bodhiapp/app-bindings",
  "napi": {
    "name": "app-bindings",
    "triples": {
      "defaults": false,
      "additional": ["aarch64-apple-darwin", "x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"]
    }
  },
  "scripts": {
    "build": "napi build --platform",
    "build:release": "napi build --platform --release",
    "test": "npm run test:run && npm run test:playwright"
  }
}
```

### TypeScript Integration Implementation

Auto-generated type definitions with comprehensive interface coverage:

```typescript
// Auto-generated TypeScript definitions (see index.d.ts:5-25 for complete interfaces)
export interface NapiAppOptions {
  envVars: Record<string, string>;
  appSettings: Record<string, string>;
  systemSettings: Record<string, string>;
  clientId?: string;
  clientSecret?: string;
  appStatus?: string;
}

export declare class BodhiServer {
  constructor(config: NapiAppOptions);
  get config(): NapiAppOptions;
  serverUrl(): string;
  start(): Promise<void>;
  stop(): Promise<void>;
  isRunning(): Promise<boolean>;
  ping(): Promise<boolean>;
}

// Configuration constants for safe JavaScript usage
export const BODHI_HOME: string;
export const BODHI_HOST: string;
export const BODHI_PORT: string;
export const DEFAULT_HOST: string;
export const DEFAULT_PORT: number;
```

**Cross-Platform Integration Features**:

- Native module compilation for Windows, macOS (Intel/ARM), and Linux with automated build pipeline
- Auto-generated TypeScript definitions with comprehensive interface coverage and type validation
- Cross-platform resource management with proper permissions and cleanup coordination
- Platform-specific optimization with efficient native binary size and minimal JavaScript wrapper code

## Server Lifecycle Management Architecture

### Async Server Operations Implementation

Advanced server lifecycle coordination with comprehensive state management:

```rust
// Server lifecycle management with async coordination (see src/server.rs:145-185 for complete implementation)
impl BodhiServer {
  pub async unsafe fn start(&mut self) -> Result<()> {
    // State validation
    let handle_guard = self.shutdown_handle.lock().await;
    if handle_guard.is_some() {
      return Err(Error::new(Status::InvalidArg, "Server is already running".to_string()));
    }
    drop(handle_guard);

    // Configuration and service setup
    let builder = try_build_app_options_internal(self.config.clone())?;
    let app_options = builder.build()?;
    let setting_service = Arc::new(setup_app_dirs(&app_options)?);

    // Logging setup with proper cleanup
    let log_guard = setup_logs(&setting_service);
    self.log_guard = Some(log_guard);

    // Complete service composition
    let app_service: Arc<dyn AppService> = Arc::new(build_app_service(setting_service.clone()).await?);
    update_with_option(&app_service, (&app_options).into())?;

    // HTTP server coordination
    let serve_command = ServeCommand::ByParams { host: self.host(), port: self.port() };
    let handle = serve_command.get_server_handle(app_service, Some(&EMBEDDED_UI_ASSETS)).await?;

    // Thread-safe state management
    let mut handle_guard = self.shutdown_handle.lock().await;
    *handle_guard = Some(handle);
    Ok(())
  }

  pub async unsafe fn stop(&mut self) -> Result<()> {
    // Graceful shutdown coordination
    let handle = {
      let mut handle_guard = self.shutdown_handle.lock().await;
      handle_guard.take()
    };

    if let Some(handle) = handle {
      handle.shutdown().await?;
    }

    // Resource cleanup
    if let Some(guard) = self.log_guard.take() {
      drop(guard);
    }
    Ok(())
  }
}
```

### Health Monitoring Implementation

Advanced server health checking with HTTP validation:

```rust
// Server health monitoring (see src/server.rs:225-245 for complete implementation)
pub async unsafe fn ping(&self) -> Result<bool> {
  let is_running = {
    let handle_guard = self.shutdown_handle.lock().await;
    handle_guard.is_some()
  };

  if !is_running {
    return Ok(false);
  }

  // HTTP health check coordination
  let url = format!("{}/ping", self.server_url());
  match reqwest::get(&url).await {
    Ok(response) => Ok(response.status().is_success()),
    Err(_) => Ok(false),
  }
}
```

**Server Lifecycle Features**:

- Promise-based async operations with proper error handling and state management
- Thread-safe server state management with Mutex coordination and concurrent access support
- Comprehensive resource cleanup with Drop trait implementation and automatic garbage collection coordination
- HTTP health monitoring with connection validation and server status checking
- Graceful shutdown coordination with proper resource cleanup and error handling

## Testing Infrastructure Architecture

### Rust Test Utils Implementation

Comprehensive testing infrastructure for NAPI binding validation:

```rust
// Test configuration fixtures (see src/test_utils/config.rs:15-45 for complete implementation)
#[fixture]
pub fn test_config(temp_dir: TempDir) -> (NapiAppOptions, TempDir) {
  let bodhi_home = temp_dir.path().to_string_lossy().to_string();
  let port = rand::rng().random_range(20000..30000);

  let mut config = create_napi_app_options();

  // Basic configuration setup
  config = set_env_var(config, BODHI_HOME.to_string(), bodhi_home);
  config = set_env_var(config, BODHI_HOST.to_string(), "127.0.0.1".to_string());
  config = set_env_var(config, BODHI_PORT.to_string(), port.to_string());

  // System settings for complete app setup
  config = set_system_setting(config, BODHI_ENV_TYPE.to_string(), "development".to_string());
  config = set_system_setting(config, BODHI_APP_TYPE.to_string(), "container".to_string());
  config = set_system_setting(config, BODHI_VERSION.to_string(), "1.0.0-test".to_string());

  (config, temp_dir)
}

// Custom configuration testing
pub fn test_config_with_settings(mut config: NapiAppOptions, settings: HashMap<String, String>) -> NapiAppOptions {
  for (key, value) in settings {
    match key.as_str() {
      "host" => config = set_env_var(config, BODHI_HOST.to_string(), value),
      "port" => config = set_env_var(config, BODHI_PORT.to_string(), value),
      _ => config = set_env_var(config, key, value),
    }
  }
  config
}
```

### JavaScript Test Integration Implementation

Advanced JavaScript testing with comprehensive NAPI validation:

```javascript
// Test helper functions (see tests-js/test-helpers.mjs:15-45 for complete implementation)
async function loadBindings() {
  const appBindingsModule = await import('../index.js');
  return appBindingsModule.default;
}

function createTestServer(bindings, options = {}) {
  const config = createFullTestConfig(bindings, options);
  const server = new bindings.BodhiServer(config);
  return server;
}

function createFullTestConfig(bindings, options = {}) {
  const appHome = createTempDir();
  const { host = 'localhost', port = randomPort() } = options;

  let config = bindings.createNapiAppOptions();

  // Environment variable setup
  config = bindings.setEnvVar(config, 'HOME', appHome);
  config = bindings.setEnvVar(config, bindings.BODHI_HOST, host);
  config = bindings.setEnvVar(config, bindings.BODHI_PORT, port.toString());

  // System settings coordination
  config = bindings.setSystemSetting(config, bindings.BODHI_ENV_TYPE, 'development');
  config = bindings.setSystemSetting(config, bindings.BODHI_VERSION, '1.0.0-test');

  return config;
}
```

### Integration Test Implementation

Comprehensive integration testing with server lifecycle validation:

```javascript
// Integration test patterns (see tests-js/integration.test.js:25-65 for complete tests)
describe('Server Lifecycle and State Management', () => {
  test('should initially report not running and maintain proper server state', async () => {
    const server = createTestServer(bindings, { host: '127.0.0.1', port: 20001 });
    runningServers.push(server);

    // State validation
    expect(await server.isRunning()).toBe(false);
    expect(await server.isRunning()).toBe(false);
  });

  test('should contain all required configuration variables for complete setup', () => {
    const server = createTestServer(bindings, { host: 'test-host', port: 12345 });
    const serverConfig = server.config;

    // Configuration validation
    expect(serverConfig.envVars['HOME']).toBeDefined();
    expect(serverConfig.envVars[bindings.BODHI_HOST]).toBeDefined();
    expect(serverConfig.systemSettings[bindings.BODHI_ENV_TYPE]).toBeDefined();
  });
});
```

**Testing Infrastructure Features**:

- Comprehensive Rust test fixtures with temporary directory management and random port generation
- JavaScript test helpers with dynamic binding loading and configuration management
- Integration testing with server lifecycle validation and state management verification
- Cross-platform testing support with automated build pipeline and platform-specific validation
- Memory leak detection and performance benchmarking for NAPI binding optimization

## Error Handling Architecture

### Rust to JavaScript Error Translation Implementation

Sophisticated error handling with proper context preservation:

```rust
// Error translation patterns (see src/server.rs:95-115 for complete error handling)
impl BodhiServer {
  pub async unsafe fn start(&mut self) -> Result<()> {
    let builder = try_build_app_options_internal(self.config.clone()).map_err(|e| {
      Error::new(Status::GenericFailure, format!("Failed to build app options: {}", e))
    })?;

    let setting_service = Arc::new(setup_app_dirs(&app_options).map_err(|e| {
      Error::new(Status::GenericFailure, format!("Failed to setup app dirs: {}", e))
    })?);

    let handle = serve_command.get_server_handle(app_service, Some(&EMBEDDED_UI_ASSETS)).await.map_err(|e| {
      let err: ApiError = e.into();
      let err: OpenAIApiError = err.into();
      Error::new(Status::GenericFailure, format!("Failed to start server: {}", err))
    })?;

    Ok(())
  }
}
```

### JavaScript Error Handling Patterns

Comprehensive error handling with proper stack traces and context:

```javascript
// JavaScript error handling patterns (see tests-js/integration.test.js:85-105 for complete patterns)
describe('Error Handling and Validation', () => {
  test('should reject invalid app status', () => {
    const config = bindings.createNapiAppOptions();

    expect(() => {
      bindings.setAppStatus(config, 'invalid-status');
    }).toThrow();
  });

  test('should handle server startup errors gracefully', async () => {
    const server = createTestServer(bindings, { host: 'invalid-host' });

    try {
      await server.start();
      expect(false).toBe(true); // Should not reach here
    } catch (error) {
      expect(error.message).toContain('Failed to start server');
    }
  });
});
```

**Error Handling Features**:

- Comprehensive error translation from Rust error types to JavaScript Error objects with proper stack traces
- Context preservation with detailed error messages and recovery guidance for Node.js applications
- Configuration validation errors with specific guidance for invalid settings and missing requirements
- Server lifecycle error handling with graceful degradation and proper resource cleanup coordination

## Extension Guidelines for NAPI Bindings

### Adding New NAPI Bindings

When creating new Node.js bindings for BodhiApp functionality:

1. **NAPI Function Design**: Use proper NAPI annotations with async support and comprehensive error handling
2. **Configuration Integration**: Extend NapiAppOptions with new configuration options and validation rules
3. **Type Safety**: Update TypeScript definitions with new interfaces and comprehensive type coverage
4. **Error Handling**: Create NAPI-specific errors that translate to JavaScript Error objects with context
5. **Testing Infrastructure**: Use comprehensive JavaScript and Rust testing for isolated binding validation

### Extending Configuration Management

For new configuration capabilities and Node.js integration patterns:

1. **Multi-Layer Configuration**: Extend configuration system with new environment variables and settings
2. **Validation Integration**: Add comprehensive validation with error translation and recovery guidance
3. **Cross-Platform Support**: Ensure new configuration works across all supported platforms
4. **TypeScript Integration**: Update type definitions with new configuration options and interfaces
5. **Configuration Testing**: Test configuration management with different Node.js scenarios and validation

### Cross-Application Integration Patterns

For new Node.js embedding scenarios and application integration:

1. **Runtime Coordination**: Design async runtime coordination that integrates efficiently with Node.js event loop
2. **Memory Management**: Implement proper resource lifecycle management between Rust and JavaScript
3. **Error Boundaries**: Provide comprehensive error handling with proper isolation and recovery
4. **Performance Optimization**: Optimize NAPI bindings and resource management for minimal overhead
5. **Integration Testing**: Support comprehensive Node.js integration testing with realistic scenarios

## Commands for NAPI Testing

**NAPI Integration Tests**: `cargo test -p lib_bodhiserver_napi` (includes Rust NAPI binding tests)  
**JavaScript Tests**: `npm run test:run` (includes Vitest-based JavaScript integration tests)  
**Playwright Tests**: `npm run test:playwright` (includes browser-based UI integration tests)  
**Cross-Platform Build**: `npm run build:release` (includes native module compilation for all platforms)  
**Test Utils Tests**: `cargo test -p lib_bodhiserver_napi --features test-utils` (includes comprehensive testing infrastructure)
