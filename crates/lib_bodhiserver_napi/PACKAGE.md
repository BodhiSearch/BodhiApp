# PACKAGE.md - lib_bodhiserver_napi Crate

*For architectural insights and design decisions, see [crates/lib_bodhiserver_napi/CLAUDE.md](crates/lib_bodhiserver_napi/CLAUDE.md)*

## Implementation Index

The `lib_bodhiserver_napi` crate provides sophisticated Node.js bindings for BodhiApp server functionality with comprehensive configuration management and cross-platform support.

### Core NAPI Files

**src/lib.rs**
- Module structure and feature-gated test utilities
- Public API exports for BodhiServer and configuration functions

**src/config.rs**
- NapiAppOptions configuration structure for Node.js integration
- Multi-layer configuration with environment variables, app settings, system settings
- OAuth2 client credentials management and validation
- Constants export for safe JavaScript usage

**src/server.rs**
- BodhiServer class implementation with complete lifecycle management
- Async server operations with Promise-based API
- Health monitoring and state management
- Logging integration with proper cleanup

**build.rs**
- NAPI build configuration for cross-platform compilation
- Native module build coordination

### NAPI Interface Examples

#### Server Lifecycle Management

```rust
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
      // Configuration translation and service composition
      let builder = try_build_app_options_internal(self.config.clone())?;
      let app_options = builder.build()?;

      // Complete application bootstrap
      let setting_service = Arc::new(setup_app_dirs(&app_options)?);
      let app_service = Arc::new(build_app_service(setting_service.clone()).await?);

      // HTTP server coordination
      let serve_command = ServeCommand::ByParams {
        host: self.host(),
        port: self.port(),
      };
      let handle = serve_command
        .get_server_handle(app_service, Some(&EMBEDDED_UI_ASSETS))
        .await?;

      // Thread-safe state management
      let mut handle_guard = self.shutdown_handle.lock().await;
      *handle_guard = Some(handle);
      Ok(())
    }
  }
```

#### Configuration Management

```rust
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
```

## TypeScript Integration

### Auto-Generated Type Definitions

```typescript
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
    host(): string;
    port(): number;
    start(): Promise<void>;
    stop(): Promise<void>;
    isRunning(): Promise<boolean>;
    ping(): Promise<boolean>;
  }

  export const BODHI_HOME: string;
  export const BODHI_HOST: string;
  export const BODHI_PORT: string;
  export const DEFAULT_HOST: string;
  export const DEFAULT_PORT: number;
```

### JavaScript Usage Examples

```javascript
  import {
    BodhiServer,
    createNapiAppOptions,
    setEnvVar,
    setSystemSetting,
    BODHI_HOME,
    BODHI_HOST,
    BODHI_PORT
  } from '@bodhiapp/app-bindings';

  // Configuration setup
  let config = createNapiAppOptions();
  config = setEnvVar(config, BODHI_HOME, '/tmp/bodhi');
  config = setEnvVar(config, BODHI_HOST, 'localhost');
  config = setEnvVar(config, BODHI_PORT, '3000');
  config = setSystemSetting(config, 'BODHI_ENV_TYPE', 'development');

  // Server lifecycle
  const server = new BodhiServer(config);
  await server.start();

  console.log(`Server running: ${await server.isRunning()}`);
  console.log(`Server URL: ${server.serverUrl()}`);
  console.log(`Health check: ${await server.ping()}`);

  await server.stop();
```

## Cross-Platform Build System

### Package Configuration

```json
  {
    "name": "@bodhiapp/app-bindings",
    "main": "index.js",
    "types": "index.d.ts",
    "napi": {
      "name": "app-bindings",
      "triples": {
        "defaults": false,
        "additional": [
          "aarch64-apple-darwin",
          "x86_64-apple-darwin",
          "x86_64-pc-windows-msvc",
          "x86_64-unknown-linux-gnu"
        ]
      }
    },
    "scripts": {
      "build": "napi build --platform",
      "build:release": "napi build --platform --release",
      "test": "npm run test:run && npm run test:playwright",
      "test:run": "vitest run",
      "test:playwright": "playwright test"
    }
  }
```

### Build Commands

```bash
# Development build
npm run build

# Release build for all platforms
npm run build:release

# Run JavaScript tests
npm run test:run

# Run Playwright integration tests
npm run test:playwright

# Test Rust NAPI bindings
cargo test -p lib_bodhiserver_napi

# Test with utilities
cargo test -p lib_bodhiserver_napi --features test-utils
```

## Testing Infrastructure

### Rust Test Utilities

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
    config = set_system_setting(config, BODHI_VERSION.to_string(), "1.0.0-test".to_string());

    (config, temp_dir)
  }
```

### JavaScript Test Helpers

```javascript
  function createTestServer(bindings, options = {}) {
    const config = createFullTestConfig(bindings, options);
    const server = new bindings.BodhiServer(config);
    return server;
  }

  function createFullTestConfig(bindings, options = {}) {
    const appHome = createTempDir();
    const { host = 'localhost', port = randomPort() } = options;

    let config = bindings.createNapiAppOptions();
    config = bindings.setEnvVar(config, 'HOME', appHome);
    config = bindings.setEnvVar(config, bindings.BODHI_HOST, host);
    config = bindings.setEnvVar(config, bindings.BODHI_PORT, port.toString());
    config = bindings.setSystemSetting(config, bindings.BODHI_ENV_TYPE, 'development');

    return config;
  }
```

### Integration Test Examples

```javascript
  describe('Server Lifecycle', () => {
    test('should manage server state correctly', async () => {
      const server = createTestServer(bindings, { host: '127.0.0.1', port: 20001 });

      // Initial state
      expect(await server.isRunning()).toBe(false);

      // Start server
      await server.start();
      expect(await server.isRunning()).toBe(true);

      // Health check
      expect(await server.ping()).toBe(true);

      // Stop server
      await server.stop();
      expect(await server.isRunning()).toBe(false);
    });
  });
```

## Configuration Management Patterns

### Multi-Layer Configuration

```rust
  pub fn try_build_app_options_internal(
    config: NapiAppOptions,
  ) -> Result<AppOptionsBuilder, AppOptionsError> {
    let mut builder = AppOptionsBuilder::default();

    // Environment variables
    for (key, value) in config.env_vars {
      builder = builder.set_env(&key, &value);
    }

    // App settings
    for (key, value) in config.app_settings {
      builder = builder.set_app_setting(&key, &value);
    }

    // System settings
    for (key, value) in config.system_settings {
      builder = builder.set_system_setting(&key, &value)?;
    }

    // OAuth2 credentials
    if let (Some(client_id), Some(client_secret)) = (config.client_id, config.client_secret) {
      builder = builder.set_app_reg_info(&client_id, &client_secret);
    }

    // App status validation
    if let Some(status_str) = config.app_status {
      let status = status_str.parse::<AppStatus>().map_err(|_| {
        AppOptionsError::ValidationError(format!("Invalid app status: {}", status_str))
      })?;
      builder = builder.set_app_status(status);
    }

    Ok(builder)
  }
```

## Error Handling Patterns

### Rust to JavaScript Error Translation

```rust
  impl BodhiServer {
    pub async unsafe fn start(&mut self) -> Result<()> {
      let builder = try_build_app_options_internal(self.config.clone()).map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to build app options: {}", e),
        )
      })?;

      let app_service = build_app_service(setting_service.clone())
        .await
        .map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to build app service: {}", e),
          )
        })?;

      Ok(())
    }
  }
```

### JavaScript Error Handling

```javascript
  try {
    await server.start();
  } catch (error) {
    console.error('Server startup failed:', error.message);
    // Error contains detailed context and stack trace
  }
```

## Memory Management

### Resource Cleanup Implementation

```rust
  impl Drop for BodhiServer {
    fn drop(&mut self) {
      // Cleanup temporary directories
      if let Some(_temp_dir) = self.temp_dir.take() {
        // temp_dir automatically cleaned up when dropped
      }

      // Cleanup logging resources
      if let Some(_log_guard) = self.log_guard.take() {
        // log_guard automatically cleaned up when dropped
      }
    }
  }
```

## Recent Architecture Changes

### Enhanced Configuration Management
- Multi-layer configuration system with environment, app, and system settings
- OAuth2 client credentials management with secure validation
- Application status management with comprehensive validation
- Constants export for safe JavaScript usage

### Improved Error Handling
- Comprehensive error translation from Rust to JavaScript
- Context preservation with detailed error messages
- Configuration validation with recovery guidance
- Graceful degradation for server lifecycle errors

### Cross-Platform Optimization
- Native module compilation for multiple platforms
- Auto-generated TypeScript definitions with type safety
- Memory management optimization between Rust and JavaScript
- Performance improvements for NAPI binding overhead