# FFI UI Testing Research: Exposing lib_bodhiserver for TypeScript/JavaScript Integration

## Executive Summary

This document analyzes technical approaches for exposing the `lib_bodhiserver` Rust crate through FFI interfaces to enable UI testing from TypeScript/JavaScript. After comprehensive research, **NAPI-RS emerges as the optimal solution** for BodhiApp's requirements, offering the best balance of simplicity, reliability, and ROI.

## Current Architecture Analysis

### lib_bodhiserver Structure

The `lib_bodhiserver` crate provides a clean, platform-agnostic initialization API:

**Core Components:**
- `AppOptions` - Configuration struct using `derive_builder` pattern
- `setup_app_dirs()` - Directory and settings initialization  
- `build_app_service()` - Complete service dependency resolution
- `AppServiceBuilder` - Flexible service construction with dependency injection

**Current Usage Pattern:**
```rust
// From crates/bodhi/src-tauri/src/server_init.rs:46-61
let app_options = build_app_options(env_wrapper, APP_TYPE)?;
let setting_service = setup_app_dirs(app_options)?;
let app_service = build_app_service(Arc::new(setting_service)).await?;
```

**FFI Compatibility Challenges:**
1. **Complex Types**: `Arc<dyn EnvWrapper>`, `Arc<dyn SettingService>` 
2. **Async Functions**: `build_app_service()` returns `Result<DefaultAppService, ErrorMessage>`
3. **Builder Pattern**: `AppOptionsBuilder` with method chaining
4. **Error Handling**: Custom error types with `thiserror` integration

## FFI Technology Analysis

### 1. NAPI-RS (Recommended)

**Strengths:**
- **Mature Ecosystem**: Used by major projects (SWC, Prisma, Next.js, Parcel)
- **Async Support**: Native async/await integration with JavaScript Promises
- **Complex Type Handling**: Automatic serialization/deserialization
- **Error Propagation**: Rust errors become JavaScript exceptions
- **TypeScript Generation**: Automatic `.d.ts` file generation
- **Zero-Copy Operations**: Efficient data transfer via Buffer/TypedArray

**Architecture Fit:**
```rust
#[napi]
pub struct AppInitializer {
    inner: Option<Arc<DefaultAppService>>,
}

#[napi]
impl AppInitializer {
    #[napi(constructor)]
    pub fn new() -> Self { /* ... */ }
    
    #[napi]
    pub async fn initialize(&mut self, config: AppConfig) -> napi::Result<()> {
        // Wraps lib_bodhiserver initialization
    }
    
    #[napi]
    pub async fn shutdown(&mut self) -> napi::Result<()> { /* ... */ }
}
```

**Limitations:**
- Node.js specific (not browser-compatible)
- Requires native compilation for each platform
- Learning curve for NAPI-specific patterns

### 2. UniFFI

**Strengths:**
- **Multi-language**: Supports Kotlin, Swift, Python, Ruby
- **Interface Definition**: Clean UDL-based API definition
- **Mozilla Backing**: Well-maintained by Mozilla
- **Growing Ecosystem**: Third-party JavaScript bindings available

**Limitations:**
- **JavaScript Support**: Third-party, less mature than NAPI-RS
- **Async Complexity**: Limited async function support
- **Complex Types**: Requires significant type mapping for Arc/async patterns
- **Additional Toolchain**: UDL compilation step adds complexity

### 3. WebAssembly + wasm-bindgen

**Strengths:**
- **Universal Compatibility**: Works in browsers and Node.js
- **Type Safety**: Strong TypeScript integration
- **Performance**: Near-native execution speed

**Limitations:**
- **Async Restrictions**: Limited async/await support
- **System Integration**: Cannot access file system, environment variables
- **Architecture Mismatch**: BodhiApp requires system-level operations
- **Complex Setup**: Requires significant refactoring for WASM compatibility

### 4. Traditional C FFI + cbindgen

**Strengths:**
- **Universal**: Works with any language supporting C FFI
- **Performance**: Direct function calls, minimal overhead

**Limitations:**
- **Async Incompatibility**: No support for async functions
- **Complex Type Mapping**: Manual serialization for complex types
- **Memory Management**: Manual lifetime management across FFI boundary
- **Error Handling**: Limited error propagation mechanisms

## Technical Feasibility Assessment

### NAPI-RS Implementation Strategy

**Phase 1: Core FFI Layer**
```rust
// New crate: crates/lib_bodhiserver_napi/
#[napi(object)]
pub struct AppConfig {
    pub env_type: String,
    pub app_type: String, 
    pub app_version: String,
    pub auth_url: String,
    pub auth_realm: String,
    pub bodhi_home: Option<String>,
}

#[napi]
pub struct BodhiApp {
    app_service: Option<Arc<DefaultAppService>>,
    server_handle: Option<ServerShutdownHandle>,
}

#[napi]
impl BodhiApp {
    #[napi(constructor)]
    pub fn new() -> Self { /* ... */ }
    
    #[napi]
    pub async fn initialize(&mut self, config: AppConfig) -> napi::Result<()> {
        // Convert AppConfig to AppOptions
        // Call lib_bodhiserver::setup_app_dirs()
        // Call lib_bodhiserver::build_app_service()
    }
    
    #[napi]
    pub async fn start_server(&mut self, host: String, port: u16) -> napi::Result<String> {
        // Start HTTP server, return server URL
    }
    
    #[napi]
    pub async fn shutdown(&mut self) -> napi::Result<()> {
        // Clean shutdown of all services
    }
}
```

**Phase 2: TypeScript Integration**
```typescript
// Generated types from NAPI-RS
import { BodhiApp, AppConfig } from '@bodhiapp/server-bindings';

export class BodhiTestServer {
    private app: BodhiApp;
    
    constructor() {
        this.app = new BodhiApp();
    }
    
    async initialize(config: Partial<AppConfig> = {}): Promise<void> {
        const defaultConfig: AppConfig = {
            envType: 'Development',
            appType: 'Container',
            appVersion: '1.0.0-test',
            authUrl: 'https://dev-id.getbodhi.app',
            authRealm: 'bodhi',
        };
        
        await this.app.initialize({ ...defaultConfig, ...config });
    }
    
    async startServer(port = 0): Promise<string> {
        return await this.app.startServer('127.0.0.1', port);
    }
    
    async shutdown(): Promise<void> {
        await this.app.shutdown();
    }
}
```

### Integration with Existing Test Infrastructure

**Current Test Pattern (Integration Tests):**
```rust
// From crates/integration-tests/tests/utils/live_server_utils.rs
let app_options = AppOptionsBuilder::development().build()?;
let setting_service = setup_app_dirs(app_options)?;
let app_service = build_app_service(Arc::new(setting_service)).await?;
```

**New TypeScript Test Pattern:**
```typescript
// UI tests using NAPI-RS bindings
import { BodhiTestServer } from '@bodhiapp/server-bindings';

describe('UI Integration Tests', () => {
    let server: BodhiTestServer;
    
    beforeEach(async () => {
        server = new BodhiTestServer();
        await server.initialize({
            bodhiHome: await createTempDir(),
        });
        const serverUrl = await server.startServer();
        // Configure UI test environment with serverUrl
    });
    
    afterEach(async () => {
        await server.shutdown();
    });
    
    test('chat completion flow', async () => {
        // UI automation tests with programmatic server control
    });
});
```

## Implementation Complexity Analysis

### NAPI-RS Complexity: **Low-Medium**
- **Type Mapping**: Straightforward for most types
- **Async Handling**: Native support with JavaScript Promises  
- **Error Handling**: Automatic conversion to JavaScript exceptions
- **Build Integration**: Standard Rust + npm toolchain

### Required Refactoring: **Minimal**
- Create new `lib_bodhiserver_napi` crate
- Implement wrapper types for FFI compatibility
- Add NAPI-RS build configuration
- No changes to existing `lib_bodhiserver` API

### Maintenance Overhead: **Low**
- NAPI-RS handles most FFI complexity automatically
- TypeScript types generated automatically
- Standard Rust testing patterns apply
- Well-documented ecosystem with active community

## Recommendation: NAPI-RS

**Rationale:**
1. **Best ROI**: Minimal implementation effort for maximum functionality
2. **Proven Reliability**: Used by major production systems
3. **Excellent Async Support**: Native Promise integration
4. **Strong TypeScript Integration**: Automatic type generation
5. **Minimal Refactoring**: Preserves existing architecture
6. **Active Ecosystem**: Strong community and documentation

**Alternative Consideration:**
UniFFI could be viable for future multi-language support, but NAPI-RS provides the most direct path to TypeScript/JavaScript integration for UI testing requirements.

## Next Steps

1. **Create Implementation Specification**: Detailed phase-wise implementation plan
2. **Prototype Development**: Basic NAPI-RS wrapper for core functionality
3. **Integration Testing**: Verify compatibility with existing test infrastructure
4. **Documentation**: TypeScript API documentation and usage examples

## References

- **NAPI-RS Documentation**: https://napi.rs/
- **UniFFI Documentation**: https://mozilla.github.io/uniffi-rs/
- **Current lib_bodhiserver Implementation**: `crates/lib_bodhiserver/src/`
- **Integration Test Patterns**: `crates/integration-tests/tests/utils/live_server_utils.rs`
