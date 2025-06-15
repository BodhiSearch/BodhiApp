# NAPI-RS FFI UI Testing Implementation

## 1. Overview

This specification defines the implementation of a NAPI-RS based FFI layer to expose `lib_bodhiserver` functionality for TypeScript/JavaScript UI testing. The solution leverages the **completed dependency isolation refactoring** that transformed `lib_bodhiserver` into a clean, FFI-ready interface with centralized re-exports and simplified server management.

**Objective**: Enable UI testing with fine-grained programmatic control over BodhiApp server lifecycle, superior to CLI execution with environment variables.

**Technical Approach**: NAPI-RS wrapper crate providing TypeScript bindings for the isolated `lib_bodhiserver` interface, with a utility function to handle the `Dir<'static>` asset dependency.

**Key Advantage**: The dependency isolation refactoring eliminated complex external dependencies, making FFI implementation significantly simpler and more reliable.

## 2. Architecture

### 2.1. Component Structure

```
crates/
├── lib_bodhiserver_napi/          # New NAPI-RS wrapper crate
│   ├── src/
│   │   ├── lib.rs                 # NAPI exports and module definition
│   │   ├── app_initializer.rs     # Core app initialization wrapper
│   │   ├── config.rs              # FFI-compatible configuration types
│   │   ├── server.rs              # Server lifecycle management
│   │   └── error.rs               # Error type conversions
│   ├── Cargo.toml                 # NAPI-RS dependencies
│   ├── package.json               # npm package configuration
│   └── build.rs                   # Build script for bindings
├── lib_bodhiserver/               # Existing library (unchanged)
└── bodhi/                         # Existing Tauri app (unchanged)
```

### 2.2. API Design

**Core TypeScript Interface:**
```typescript
export interface AppConfig {
  envType: 'Development' | 'Production';
  appType: 'Container' | 'Native';
  appVersion: string;
  authUrl: string;
  authRealm: string;
  bodhiHome?: string;
}

export class BodhiApp {
  constructor();
  initialize(config: AppConfig): Promise<void>;
  startServer(host: string, port: number, assetsPath?: string): Promise<string>;
  shutdown(): Promise<void>;
  getStatus(): Promise<'Uninitialized' | 'Ready' | 'Running' | 'Shutdown'>;
}
```

### 2.3. Integration Points

**Leveraging Dependency Isolation:**
- ✅ **Single Dependency**: Only `lib_bodhiserver` import needed
- ✅ **Clean Re-exports**: All types accessible through `lib_bodhiserver::`
- ✅ **Simplified Server Interface**: `ServeCommand::aexecute()` accepts `Option<&'static Dir<'static>>`
- ✅ **No Complex Dependencies**: Eliminated `axum`, `server_app`, `services` direct usage

**Dir<'static> Asset Handling:**
- **Challenge**: `Dir<'static>` requires compile-time embedding via `include_dir!` macro
- **Solution**: New utility function `lib_bodhiserver::create_static_dir_from_path(path: &str)`
- **FFI Benefit**: Enables dynamic asset path specification from TypeScript

**New Capabilities:**
- TypeScript/JavaScript programmatic server control
- UI test automation with server lifecycle management
- Dynamic asset path configuration for testing scenarios
- Clean error propagation from Rust to JavaScript

## 3. Implementation Plan

### Phase 1: Core NAPI-RS Infrastructure

**Scope**: Basic NAPI-RS crate setup and core initialization wrapper leveraging isolated `lib_bodhiserver`

**Deliverables:**
1. **New Crate**: `crates/lib_bodhiserver_napi/`
2. **Dir<'static> Utility**: `lib_bodhiserver::create_static_dir_from_path()` function
3. **Core Types**: FFI-compatible configuration and error types
4. **Basic Initialization**: Wrapper using `lib_bodhiserver` re-exports
5. **Build Configuration**: NAPI-RS build setup and npm package

**Key Files:**
- `crates/lib_bodhiserver/src/static_dir_utils.rs` (new utility)
- `crates/lib_bodhiserver_napi/Cargo.toml`
- `crates/lib_bodhiserver_napi/src/lib.rs`
- `crates/lib_bodhiserver_napi/src/config.rs`
- `crates/lib_bodhiserver_napi/src/app_initializer.rs`

**Success Criteria:**
- `lib_bodhiserver::create_static_dir_from_path()` function implemented
- `npm install` successfully builds native bindings
- Basic TypeScript types generated automatically
- Can initialize app service from JavaScript using re-exported types
- `cargo test -p lib_bodhiserver_napi` passes

**Testing Strategy:**
- Unit tests for `create_static_dir_from_path()` utility
- Unit tests for type conversions using re-exported types
- Integration test for basic initialization flow
- Verify no regressions in existing `lib_bodhiserver` tests

### Phase 2: Server Lifecycle Management

**Scope**: HTTP server startup, shutdown, and status management using isolated interface

**Deliverables:**
1. **Server Management**: Wrapper for re-exported `ServeCommand` and `ServerShutdownHandle`
2. **Asset Handling**: Integration with `create_static_dir_from_path()` utility
3. **Lifecycle Control**: Start, stop, status monitoring
4. **Error Handling**: Comprehensive error propagation to JavaScript
5. **Resource Cleanup**: Proper cleanup on shutdown/errors

**Key Files:**
- `crates/lib_bodhiserver_napi/src/server.rs`
- `crates/lib_bodhiserver_napi/src/error.rs`

**Success Criteria:**
- Can start HTTP server programmatically from TypeScript with dynamic asset paths
- Server shutdown properly releases all resources
- Error conditions properly propagated to JavaScript
- Memory leaks verified absent through testing
- Asset serving works with dynamically specified paths

**Testing Strategy:**
- Server startup/shutdown cycle tests with various asset configurations
- Error condition testing (port conflicts, invalid asset paths, etc.)
- Resource cleanup verification
- Integration with existing server test patterns
- Asset serving functionality verification

### Phase 3: UI Testing Integration

**Scope**: Complete UI testing workflow integration and documentation

**Deliverables:**
1. **Test Utilities**: TypeScript helper classes for UI testing
2. **Example Tests**: Reference implementation for UI test patterns
3. **Documentation**: API documentation and usage examples
4. **CI Integration**: Build and test automation

**Key Files:**
- `crates/lib_bodhiserver_napi/examples/ui-testing.ts`
- `crates/lib_bodhiserver_napi/README.md`
- Updated CI configuration for NAPI-RS builds

**Success Criteria:**
- Complete UI test example demonstrating server control
- TypeScript API documentation generated
- CI builds and tests NAPI-RS bindings
- Performance comparable to existing integration tests

**Testing Strategy:**
- End-to-end UI testing workflow
- Performance benchmarking vs existing approaches
- Cross-platform build verification
- Documentation accuracy verification

## 4. Technical Specifications

### 4.1. NAPI-RS Implementation Details

**Core Wrapper Structure (Using Isolated lib_bodhiserver):**
```rust
#[napi]
pub struct BodhiApp {
    state: AppState,
    app_service: Option<Arc<dyn lib_bodhiserver::AppService>>,
    server_handle: Option<lib_bodhiserver::ServerShutdownHandle>,
}

#[napi]
pub enum AppState {
    Uninitialized,
    Ready,
    Running,
    Shutdown,
}

#[napi(object)]
pub struct AppConfig {
    pub env_type: String,
    pub app_type: String,
    pub app_version: String,
    pub auth_url: String,
    pub auth_realm: String,
    pub bodhi_home: Option<String>,
}
```

**Async Function Patterns with Re-exported Types:**
```rust
#[napi]
impl BodhiApp {
    #[napi]
    pub async fn initialize(&mut self, config: AppConfig) -> napi::Result<()> {
        // Use re-exported types from lib_bodhiserver
        let env_wrapper = Arc::new(lib_bodhiserver::DefaultEnvWrapper::default());
        let app_options = lib_bodhiserver::AppOptionsBuilder::default()
            .env_wrapper(env_wrapper)
            .env_type(config.env_type.parse()?)
            .app_type(config.app_type.parse()?)
            .app_version(config.app_version)
            .auth_url(config.auth_url)
            .auth_realm(config.auth_realm)
            .build()?;

        let setting_service = lib_bodhiserver::setup_app_dirs(app_options)?;
        let app_service = lib_bodhiserver::build_app_service(Arc::new(setting_service)).await?;

        self.app_service = Some(Arc::new(app_service));
        self.state = AppState::Ready;
        Ok(())
    }

    #[napi]
    pub async fn start_server(&mut self, host: String, port: u16, assets_path: Option<String>) -> napi::Result<String> {
        let app_service = self.app_service.as_ref()
            .ok_or_else(|| napi::Error::from_reason("App not initialized"))?;

        // Handle assets using utility function
        let assets_dir = if let Some(path) = assets_path {
            Some(lib_bodhiserver::create_static_dir_from_path(&path)?)
        } else {
            None
        };

        let command = lib_bodhiserver::ServeCommand::ByParams { host: host.clone(), port };
        let handle = command.get_server_handle(app_service.clone(), assets_dir).await?;

        self.server_handle = Some(handle);
        self.state = AppState::Running;
        Ok(format!("http://{}:{}", host, port))
    }
}
```

### 4.2. Error Handling Strategy

**Rust Error Conversion (Using Re-exported Error Types):**
```rust
impl From<lib_bodhiserver::AppDirsBuilderError> for napi::Error {
    fn from(err: lib_bodhiserver::AppDirsBuilderError) -> Self {
        napi::Error::new(napi::Status::GenericFailure, err.to_string())
    }
}

impl From<lib_bodhiserver::ErrorMessage> for napi::Error {
    fn from(err: lib_bodhiserver::ErrorMessage) -> Self {
        napi::Error::new(napi::Status::GenericFailure, err.to_string())
    }
}

impl From<lib_bodhiserver::ServeError> for napi::Error {
    fn from(err: lib_bodhiserver::ServeError) -> Self {
        napi::Error::new(napi::Status::GenericFailure, err.to_string())
    }
}
```

**TypeScript Error Handling:**
```typescript
try {
    await app.initialize(config);
    const serverUrl = await app.startServer('127.0.0.1', 3000, '/path/to/assets');
} catch (error) {
    // Rust errors become JavaScript exceptions
    console.error('Operation failed:', error.message);
}
```

### 4.3. Build Configuration

**Cargo.toml Dependencies (Leveraging Isolated Interface):**
```toml
[dependencies]
# Single dependency on isolated lib_bodhiserver
lib_bodhiserver = { path = "../lib_bodhiserver" }

# NAPI-RS dependencies
napi = { version = "2.0", features = ["async", "serde-json"] }
napi-derive = "2.0"

# Async runtime (already included in lib_bodhiserver)
tokio = { version = "1.0", features = ["rt-multi-thread"] }

# Error handling
thiserror = "1.0"
```

**package.json Configuration:**
```json
{
  "name": "@bodhiapp/server-bindings",
  "version": "0.1.0",
  "main": "index.js",
  "types": "index.d.ts",
  "napi": {
    "name": "bodhiapp-server-bindings",
    "triples": {
      "defaults": true
    }
  },
  "scripts": {
    "build": "napi build --platform --release",
    "build:debug": "napi build --platform",
    "prepublishOnly": "napi prepublish -t npm",
    "test": "vitest",
    "artifacts": "napi artifacts"
  }
}
```

## 5. Testing Strategy

### 5.1. Unit Testing

**Rust Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_app_initialization() {
        let mut app = BodhiApp::new();
        let config = AppConfig::development();
        assert!(app.initialize(config).await.is_ok());
    }
}
```

**TypeScript Integration Tests:**
```typescript
describe('BodhiApp NAPI Bindings', () => {
    test('initialization and server startup with assets', async () => {
        const app = new BodhiApp();
        await app.initialize({
            envType: 'Development',
            appType: 'Container',
            appVersion: '1.0.0-test',
            authUrl: 'https://dev-id.getbodhi.app',
            authRealm: 'bodhi',
        });

        // Test with dynamic asset path
        const serverUrl = await app.startServer('127.0.0.1', 0, '/path/to/test/assets');
        expect(serverUrl).toMatch(/^http:\/\/127\.0\.0\.1:\d+$/);

        await app.shutdown();
    });

    test('server startup without assets', async () => {
        const app = new BodhiApp();
        await app.initialize({
            envType: 'Development',
            appType: 'Container',
            appVersion: '1.0.0-test',
            authUrl: 'https://dev-id.getbodhi.app',
            authRealm: 'bodhi',
        });

        // Test without assets (API-only mode)
        const serverUrl = await app.startServer('127.0.0.1', 0);
        expect(serverUrl).toMatch(/^http:\/\/127\.0\.0\.1:\d+$/);

        await app.shutdown();
    });
});
```

### 5.2. Integration Testing

**Compatibility Verification:**
- Verify existing `lib_bodhiserver` tests continue passing
- Ensure no performance regressions in initialization
- Validate memory usage patterns

**Cross-Platform Testing:**
- macOS, Linux, Windows build verification
- Node.js version compatibility testing
- TypeScript version compatibility

## 6. Success Criteria

### 6.1. Functional Requirements

- ✅ Initialize BodhiApp server from TypeScript/JavaScript
- ✅ Start HTTP server with configurable host/port
- ✅ Clean shutdown with proper resource cleanup
- ✅ Error propagation from Rust to JavaScript
- ✅ TypeScript type safety with generated definitions

### 6.2. Performance Requirements

- Initialization time comparable to existing integration tests
- Memory usage within 10% of direct Rust usage
- Server startup time equivalent to CLI execution

### 6.3. Quality Requirements

- Zero memory leaks in server lifecycle
- Comprehensive error handling for all failure modes
- Complete TypeScript API documentation
- CI/CD integration with automated testing

## 7. Future Extensions

### 7.1. Enhanced Configuration

- Runtime configuration updates
- Service-level dependency injection
- Advanced logging configuration

### 7.2. Multi-Language Support

- Consider UniFFI for Python/Ruby bindings
- Evaluate WebAssembly for browser compatibility
- Explore additional language bindings as needed

### 7.3. Performance Optimization

- Connection pooling for repeated test runs
- Shared server instances across test suites
- Optimized build configurations for testing

## 8. References

- **NAPI-RS Documentation**: https://napi.rs/docs/introduction/getting-started
- **lib_bodhiserver Implementation**: `crates/lib_bodhiserver/src/`
- **Dependency Isolation Analysis**: `ai-docs/02-features/completed-stories/20250615-bodhi-dependency-isolation-analysis.md`
- **Integration Test Patterns**: `crates/integration-tests/tests/utils/live_server_utils.rs`
- **FFI Research Document**: `ai-docs/06-knowledge-transfer/20250615-ffi-ui-testing-research.md`
