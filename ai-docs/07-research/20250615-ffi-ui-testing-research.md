# FFI UI Testing Research: Exposing lib_bodhiserver for TypeScript/JavaScript Integration

## Executive Summary

This document analyzes technical approaches for exposing the `lib_bodhiserver` Rust crate through FFI interfaces to enable UI testing from TypeScript/JavaScript. After comprehensive research and analysis of the **dependency isolation refactoring**, **NAPI-RS emerges as the optimal solution** for BodhiApp's requirements, offering the best balance of simplicity, reliability, and ROI.

## Current Architecture Analysis (Post-Dependency Isolation)

### lib_bodhiserver Structure

The `lib_bodhiserver` crate has been significantly refactored to provide a clean, FFI-ready interface:

**Core Components:**
- `AppOptions` - Configuration struct using `derive_builder` pattern
- `setup_app_dirs()` - Directory and settings initialization  
- `build_app_service()` - Complete service dependency resolution
- `AppServiceBuilder` - Flexible service construction with dependency injection
- **Re-exports** - All domain objects and service interfaces centralized

**Dependency Isolation Achievements:**
- ✅ **Single Dependency Gateway**: All functionality accessible through `lib_bodhiserver`
- ✅ **Clean Re-exports**: Domain objects (`AppType`, `ErrorMessage`) and service interfaces re-exported
- ✅ **Simplified Server Interface**: `ServeCommand::aexecute()` accepts `Option<&'static Dir<'static>>`
- ✅ **Eliminated Complex Dependencies**: No direct `axum`, `server_app`, or `services` dependencies in client code

**Current Usage Pattern:**
```rust
// From crates/bodhi/src-tauri/src/server_init.rs:46-61
let app_options = build_app_options(env_wrapper, APP_TYPE)?;
let setting_service = setup_app_dirs(app_options)?;
let app_service = build_app_service(Arc::new(setting_service)).await?;

// Server startup with static assets
let command = ServeCommand::ByParams { host, port };
command.aexecute(app_service, Some(&crate::ui::ASSETS)).await?;
```

**Remaining FFI Compatibility Challenges:**
1. **Complex Types**: `Arc<dyn AppService>`, `Arc<dyn SettingService>` 
2. **Async Functions**: `build_app_service()` returns `Result<DefaultAppService, ErrorMessage>`
3. **Builder Pattern**: `AppOptionsBuilder` with method chaining
4. **Dir<'static> Dependency**: Static asset handling requires compile-time embedding

### Dir<'static> Challenge and Solution

**Current Implementation:**
```rust
// crates/bodhi/src-tauri/src/ui.rs
pub static ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../out");

// Usage in server_init.rs
command.aexecute(app_service, Some(&crate::ui::ASSETS)).await?;
```

**FFI Challenge**: `Dir<'static>` requires compile-time embedding via `include_dir!` macro, which cannot be used dynamically from FFI.

**Proposed Solution**: Create utility function in `lib_bodhiserver` that accepts a path and returns `Dir<'static>`:

```rust
// New utility function in lib_bodhiserver
pub fn create_static_dir_from_path(path: &str) -> Result<&'static Dir<'static>, ErrorMessage> {
    // Implementation that analyzes path and creates Dir<'static>
    // This enables FFI clients to pass asset paths dynamically
}

// FFI usage pattern
let assets_dir = lib_bodhiserver::create_static_dir_from_path("/path/to/assets")?;
command.aexecute(app_service, Some(assets_dir)).await?;
```

## FFI Technology Analysis

### 1. NAPI-RS (Recommended)

**Strengths:**
- **Mature Ecosystem**: Used by major projects (SWC, Prisma, Next.js, Parcel)
- **Async Support**: Native async/await integration with JavaScript Promises
- **Complex Type Handling**: Automatic serialization/deserialization
- **Error Propagation**: Rust errors become JavaScript exceptions
- **TypeScript Generation**: Automatic `.d.ts` file generation
- **Zero-Copy Operations**: Efficient data transfer via Buffer/TypedArray

**Architecture Fit with Isolated lib_bodhiserver:**
```rust
#[napi]
pub struct BodhiApp {
    app_service: Option<Arc<dyn lib_bodhiserver::AppService>>,
    server_handle: Option<lib_bodhiserver::ServerShutdownHandle>,
}

#[napi]
impl BodhiApp {
    #[napi(constructor)]
    pub fn new() -> Self { /* ... */ }
    
    #[napi]
    pub async fn initialize(&mut self, config: AppConfig) -> napi::Result<()> {
        // Convert AppConfig to lib_bodhiserver::AppOptions
        // Call lib_bodhiserver::setup_app_dirs()
        // Call lib_bodhiserver::build_app_service()
    }
    
    #[napi]
    pub async fn start_server(&mut self, host: String, port: u16, assets_path: Option<String>) -> napi::Result<String> {
        let assets_dir = if let Some(path) = assets_path {
            Some(lib_bodhiserver::create_static_dir_from_path(&path)?)
        } else {
            None
        };
        
        let command = lib_bodhiserver::ServeCommand::ByParams { host, port };
        command.aexecute(self.app_service.clone().unwrap(), assets_dir).await?;
        Ok(format!("http://{}:{}", host, port))
    }
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

### NAPI-RS Implementation Strategy (Updated)

**Phase 1: Core FFI Layer with Isolated Dependencies**
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
    app_service: Option<Arc<dyn lib_bodhiserver::AppService>>,
    server_handle: Option<lib_bodhiserver::ServerShutdownHandle>,
}

#[napi]
impl BodhiApp {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            app_service: None,
            server_handle: None,
        }
    }
    
    #[napi]
    pub async fn initialize(&mut self, config: AppConfig) -> napi::Result<()> {
        // Convert AppConfig to lib_bodhiserver::AppOptions using re-exported types
        let env_wrapper = Arc::new(lib_bodhiserver::DefaultEnvWrapper::default());
        let app_options = lib_bodhiserver::AppOptionsBuilder::default()
            .env_wrapper(env_wrapper)
            .env_type(config.env_type.parse().map_err(|e| napi::Error::from_reason(format!("Invalid env_type: {}", e)))?)
            .app_type(config.app_type.parse().map_err(|e| napi::Error::from_reason(format!("Invalid app_type: {}", e)))?)
            .app_version(config.app_version)
            .auth_url(config.auth_url)
            .auth_realm(config.auth_realm)
            .build()
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
            
        let setting_service = lib_bodhiserver::setup_app_dirs(app_options)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
            
        let app_service = lib_bodhiserver::build_app_service(Arc::new(setting_service)).await
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
            
        self.app_service = Some(Arc::new(app_service));
        Ok(())
    }
    
    #[napi]
    pub async fn start_server(&mut self, host: String, port: u16, assets_path: Option<String>) -> napi::Result<String> {
        let app_service = self.app_service.as_ref()
            .ok_or_else(|| napi::Error::from_reason("App not initialized"))?;
            
        let assets_dir = if let Some(path) = assets_path {
            Some(lib_bodhiserver::create_static_dir_from_path(&path)
                .map_err(|e| napi::Error::from_reason(e.to_string()))?)
        } else {
            None
        };
        
        let command = lib_bodhiserver::ServeCommand::ByParams { host: host.clone(), port };
        let handle = command.get_server_handle(app_service.clone(), assets_dir).await
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
            
        self.server_handle = Some(handle);
        Ok(format!("http://{}:{}", host, port))
    }
    
    #[napi]
    pub async fn shutdown(&mut self) -> napi::Result<()> {
        if let Some(handle) = self.server_handle.take() {
            handle.shutdown().await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        }
        self.app_service = None;
        Ok(())
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
    
    async startServer(port = 0, assetsPath?: string): Promise<string> {
        return await this.app.startServer('127.0.0.1', port, assetsPath);
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
        const serverUrl = await server.startServer(0, '/path/to/test/assets');
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
- **Type Mapping**: Straightforward with isolated dependencies
- **Async Handling**: Native support with JavaScript Promises  
- **Error Handling**: Automatic conversion to JavaScript exceptions
- **Build Integration**: Standard Rust + npm toolchain

### Required Refactoring: **Minimal**
- Create new `lib_bodhiserver_napi` crate
- Implement `create_static_dir_from_path()` utility function
- Add NAPI-RS build configuration
- **No changes to existing `lib_bodhiserver` API** (already isolated)

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
5. **Minimal Refactoring**: Leverages existing dependency isolation
6. **Active Ecosystem**: Strong community and documentation
7. **Clean Architecture**: Works perfectly with isolated `lib_bodhiserver`

**Alternative Consideration:**
UniFFI could be viable for future multi-language support, but NAPI-RS provides the most direct path to TypeScript/JavaScript integration for UI testing requirements.

## Next Steps

1. **Implement Dir<'static> Utility**: Create `create_static_dir_from_path()` function
2. **Create Implementation Specification**: Detailed phase-wise implementation plan
3. **Prototype Development**: Basic NAPI-RS wrapper for core functionality
4. **Integration Testing**: Verify compatibility with existing test infrastructure
5. **Documentation**: TypeScript API documentation and usage examples

## References

- **NAPI-RS Documentation**: https://napi.rs/
- **UniFFI Documentation**: https://mozilla.github.io/uniffi-rs/
- **Current lib_bodhiserver Implementation**: `crates/lib_bodhiserver/src/`
- **Dependency Isolation Analysis**: `ai-docs/02-features/completed-stories/20250615-bodhi-dependency-isolation-analysis.md`
- **Integration Test Patterns**: `crates/integration-tests/tests/utils/live_server_utils.rs`
