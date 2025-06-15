# NAPI-RS FFI UI Testing Implementation

## 1. Overview

This specification defines the implementation of a NAPI-RS based FFI layer to expose `lib_bodhiserver` functionality for TypeScript/JavaScript UI testing. The solution enables programmatic control of BodhiApp server initialization, configuration, and lifecycle management from UI test suites.

**Objective**: Enable UI testing with fine-grained programmatic control over BodhiApp server lifecycle, superior to CLI execution with environment variables.

**Technical Approach**: NAPI-RS wrapper crate providing TypeScript bindings for `lib_bodhiserver` initialization and server management.

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
  startServer(host: string, port: number): Promise<string>;
  shutdown(): Promise<void>;
  getStatus(): Promise<'Uninitialized' | 'Ready' | 'Running' | 'Shutdown'>;
}
```

### 2.3. Integration Points

**Existing Architecture Preservation:**
- `lib_bodhiserver` API remains unchanged
- Current production usage (`native_init.rs`, `server_init.rs`) unaffected
- Integration tests continue using existing patterns

**New Capabilities:**
- TypeScript/JavaScript programmatic server control
- UI test automation with server lifecycle management
- Configuration override capabilities for testing scenarios

## 3. Implementation Plan

### Phase 1: Core NAPI-RS Infrastructure

**Scope**: Basic NAPI-RS crate setup and core initialization wrapper

**Deliverables:**
1. **New Crate**: `crates/lib_bodhiserver_napi/`
2. **Core Types**: FFI-compatible configuration and error types
3. **Basic Initialization**: Wrapper for `setup_app_dirs()` and `build_app_service()`
4. **Build Configuration**: NAPI-RS build setup and npm package

**Key Files:**
- `crates/lib_bodhiserver_napi/Cargo.toml`
- `crates/lib_bodhiserver_napi/src/lib.rs`
- `crates/lib_bodhiserver_napi/src/config.rs`
- `crates/lib_bodhiserver_napi/src/app_initializer.rs`

**Success Criteria:**
- `npm install` successfully builds native bindings
- Basic TypeScript types generated automatically
- Can initialize app service from JavaScript
- `cargo test -p lib_bodhiserver_napi` passes

**Testing Strategy:**
- Unit tests for type conversions
- Integration test for basic initialization flow
- Verify no regressions in existing `lib_bodhiserver` tests

### Phase 2: Server Lifecycle Management

**Scope**: HTTP server startup, shutdown, and status management

**Deliverables:**
1. **Server Management**: Wrapper for `ServeCommand` and `ServerShutdownHandle`
2. **Lifecycle Control**: Start, stop, status monitoring
3. **Error Handling**: Comprehensive error propagation to JavaScript
4. **Resource Cleanup**: Proper cleanup on shutdown/errors

**Key Files:**
- `crates/lib_bodhiserver_napi/src/server.rs`
- `crates/lib_bodhiserver_napi/src/error.rs`

**Success Criteria:**
- Can start HTTP server programmatically from TypeScript
- Server shutdown properly releases all resources
- Error conditions properly propagated to JavaScript
- Memory leaks verified absent through testing

**Testing Strategy:**
- Server startup/shutdown cycle tests
- Error condition testing (port conflicts, etc.)
- Resource cleanup verification
- Integration with existing server test patterns

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

**Core Wrapper Structure:**
```rust
#[napi]
pub struct BodhiApp {
    state: AppState,
    app_service: Option<Arc<DefaultAppService>>,
    server_handle: Option<ServerShutdownHandle>,
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

**Async Function Patterns:**
```rust
#[napi]
impl BodhiApp {
    #[napi]
    pub async fn initialize(&mut self, config: AppConfig) -> napi::Result<()> {
        // Convert AppConfig to lib_bodhiserver::AppOptions
        // Call setup_app_dirs() and build_app_service()
        // Update internal state
    }
    
    #[napi]
    pub async fn start_server(&mut self, host: String, port: u16) -> napi::Result<String> {
        // Start HTTP server using existing ServeCommand
        // Return actual server URL
    }
}
```

### 4.2. Error Handling Strategy

**Rust Error Conversion:**
```rust
impl From<lib_bodhiserver::AppDirsBuilderError> for napi::Error {
    fn from(err: lib_bodhiserver::AppDirsBuilderError) -> Self {
        napi::Error::new(napi::Status::GenericFailure, err.to_string())
    }
}
```

**TypeScript Error Handling:**
```typescript
try {
    await app.initialize(config);
} catch (error) {
    // Rust errors become JavaScript exceptions
    console.error('Initialization failed:', error.message);
}
```

### 4.3. Build Configuration

**Cargo.toml Dependencies:**
```toml
[dependencies]
lib_bodhiserver = { path = "../lib_bodhiserver" }
napi = { version = "2.0", features = ["async", "serde-json"] }
napi-derive = "2.0"
tokio = { version = "1.0", features = ["rt-multi-thread"] }
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
    test('initialization and server startup', async () => {
        const app = new BodhiApp();
        await app.initialize({
            envType: 'Development',
            appType: 'Container',
            appVersion: '1.0.0-test',
            authUrl: 'https://dev-id.getbodhi.app',
            authRealm: 'bodhi',
        });
        
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
- **Integration Test Patterns**: `crates/integration-tests/tests/utils/live_server_utils.rs`
- **FFI Research Document**: `ai-docs/06-knowledge-transfer/20250615-ffi-ui-testing-research.md`
