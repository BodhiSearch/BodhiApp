# Enhanced Configuration Management for lib_bodhiserver

## 1. Overview

**Status**: 📋 **PLANNED** - Next phase implementation for flexible configuration management

This specification defines the enhancement of `lib_bodhiserver`'s configuration management to eliminate hardcoded values and provide maximum flexibility for FFI clients. The implementation builds upon the completed NAPI FFI layer to create a comprehensive configuration builder pattern that supports all aspects of BodhiApp initialization.

**Objective**: Remove all hardcoded authentication credentials and configuration values from `lib_bodhiserver`, replacing them with a flexible builder pattern that allows clients to specify all configuration parameters.

**Technical Approach**: Extend the existing `AppOptions` builder with comprehensive configuration methods, implement proper validation, and enhance the NAPI bridge to expose all configuration capabilities.

## 2. Architecture Enhancement

### 2.1. Enhanced AppOptions Builder Pattern

**Current State**: Basic `AppOptions` with limited configuration options
**Target State**: Comprehensive configuration builder supporting all settings categories

```rust
// Enhanced AppOptions builder interface
impl AppOptionsBuilder {
    // Environment configuration
    pub fn set_env(mut self, key: &str, value: &str) -> Self
    
    // App settings (configurable via settings.yaml)
    pub fn set_app_setting(mut self, key: &str, value: &str) -> Self
    
    // System settings (immutable)
    pub fn set_system_setting(mut self, key: &str, value: &str) -> Self
    
    // OAuth client credentials (optional)
    pub fn set_app_reg_info(mut self, client_id: &str, client_secret: &str) -> Self
    
    // App initialization status (optional)
    pub fn set_app_status(mut self, status: AppStatus) -> Self
    
    // Secret encryption key
    pub fn set_secret_key(mut self, key: &str) -> Self
    
    // Validation and build
    pub fn build(self) -> Result<AppOptions, ConfigurationError>
}
```

### 2.2. Configuration Categories Integration

**System Settings** (Set via `set_system_setting`):
- `BODHI_ENV_TYPE`, `BODHI_APP_TYPE`, `BODHI_VERSION`
- `BODHI_AUTH_URL`, `BODHI_AUTH_REALM`, `BODHI_HOME`

**App Settings** (Set via `set_app_setting`):
- `BODHI_LOGS`, `BODHI_LOG_LEVEL`, `BODHI_LOG_STDOUT`
- `BODHI_SCHEME`, `BODHI_HOST`, `BODHI_PORT`
- `BODHI_EXEC_VARIANT`, `BODHI_EXEC_LOOKUP_PATH`, `BODHI_KEEP_ALIVE_SECS`
- `HF_HOME`

**Environment Variables** (Set via `set_env`):
- Any environment variable needed by the application
- Replaces direct `env_wrapper` dependency in client code

**Sensitive Settings** (Set via `set_secret_key` and `set_app_reg_info`):
- `BODHI_ENCRYPTION_KEY` - Secret encryption key
- OAuth client credentials (`client_id`, `client_secret`)

## 3. Implementation Phases

### 3.1. Phase 1: Enhanced AppOptions Builder

**Scope**: Extend `AppOptions` and `AppOptionsBuilder` with comprehensive configuration methods

**Implementation Tasks**:
1. **Add Configuration Storage**:
   ```rust
   #[derive(Debug, Clone, Builder)]
   pub struct AppOptions {
       // Existing fields
       pub env_wrapper: Arc<dyn EnvWrapper>,
       pub env_type: EnvType,
       // ... existing fields
       
       // New configuration storage
       pub environment_vars: HashMap<String, String>,
       pub app_settings: HashMap<String, String>,
       pub system_settings: HashMap<String, String>,
       pub app_reg_info: Option<AppRegInfo>,
       pub app_status: Option<AppStatus>,
       pub secret_key: Option<String>,
   }
   ```

2. **Implement Builder Methods**:
   - `set_env()` - Store environment variables
   - `set_app_setting()` - Store app configuration
   - `set_system_setting()` - Store system configuration
   - `set_app_reg_info()` - Store OAuth credentials
   - `set_app_status()` - Store app initialization status
   - `set_secret_key()` - Store encryption key

3. **Configuration Validation**:
   ```rust
   pub fn build(self) -> Result<AppOptions, ConfigurationError> {
       // Validate required settings
       self.validate_required_settings()?;
       // Build enhanced env_wrapper from collected environment variables
       let env_wrapper = self.build_env_wrapper()?;
       // Return validated AppOptions
       Ok(AppOptions { /* ... */ })
   }
   ```

**Acceptance Criteria**:
- ✅ All configuration methods implemented and tested
- ✅ Proper validation with descriptive error messages
- ✅ Backward compatibility with existing `AppOptions` usage
- ✅ Comprehensive unit tests for all builder methods

### 3.2. Phase 2: Service Initialization Enhancement

**Scope**: Update service initialization to use enhanced configuration

**Implementation Tasks**:
1. **Enhanced `setup_app_dirs`**:
   ```rust
   pub fn setup_app_dirs(options: AppOptions) -> Result<DefaultSettingService, AppDirsBuilderError> {
       // Initialize env_wrapper from collected environment variables
       let env_wrapper = build_enhanced_env_wrapper(&options)?;
       
       // Apply system settings during initialization
       let system_settings = build_system_settings_from_options(&options);
       
       // Create settings service with enhanced configuration
       let setting_service = DefaultSettingService::new_with_enhanced_config(
           env_wrapper,
           system_settings,
           options.app_settings,
           settings_file,
       );
       
       Ok(setting_service)
   }
   ```

2. **Enhanced `build_app_service`**:
   ```rust
   pub async fn build_app_service_with_config(
       setting_service: Arc<dyn SettingService>,
       options: &AppOptions,
   ) -> Result<DefaultAppService, ErrorMessage> {
       let mut builder = AppServiceBuilder::new(setting_service);
       
       // Configure secret service with provided key
       if let Some(secret_key) = &options.secret_key {
           let secret_service = build_secret_service_with_key(secret_key)?;
           builder = builder.secret_service(secret_service)?;
       }
       
       let app_service = builder.build().await?;
       
       // Set app registration info if provided
       if let Some(app_reg_info) = &options.app_reg_info {
           app_service.secret_service().set_app_reg_info(app_reg_info)?;
       }
       
       // Set app status if provided
       if let Some(app_status) = &options.app_status {
           app_service.secret_service().set_app_status(*app_status)?;
       }
       
       Ok(app_service)
   }
   ```

**Acceptance Criteria**:
- ✅ Service initialization uses enhanced configuration
- ✅ No hardcoded values in service initialization
- ✅ Proper error handling for missing required configuration
- ✅ Integration tests verify complete configuration flow

### 3.3. Phase 3: NAPI Bridge Enhancement

**Scope**: Update NAPI bridge to expose enhanced configuration capabilities

**Implementation Tasks**:
1. **Enhanced AppConfig Interface**:
   ```typescript
   export interface AppConfig {
       // Existing fields
       envType: string;
       appType: string;
       // ... existing fields
       
       // New configuration options
       environmentVars?: Record<string, string>;
       appSettings?: Record<string, string>;
       systemSettings?: Record<string, string>;
       clientId?: string;
       clientSecret?: string;
       appStatus?: string;
       secretKey?: string;
   }
   ```

2. **Enhanced Configuration Conversion**:
   ```rust
   impl TryFrom<AppConfig> for lib_bodhiserver::AppOptions {
       fn try_from(config: AppConfig) -> Result<Self, String> {
           let mut builder = AppOptionsBuilder::default()
               .env_type(config.env_type.parse()?)
               .app_type(config.app_type.parse()?)
               .app_version(config.app_version)
               .auth_url(config.auth_url)
               .auth_realm(config.auth_realm);
           
           // Apply environment variables
           if let Some(env_vars) = config.environment_vars {
               for (key, value) in env_vars {
                   builder = builder.set_env(&key, &value);
               }
           }
           
           // Apply app settings
           if let Some(app_settings) = config.app_settings {
               for (key, value) in app_settings {
                   builder = builder.set_app_setting(&key, &value);
               }
           }
           
           // Apply OAuth credentials
           if let (Some(client_id), Some(client_secret)) = (config.client_id, config.client_secret) {
               builder = builder.set_app_reg_info(&client_id, &client_secret);
           }
           
           // Apply app status
           if let Some(app_status) = config.app_status {
               let status = AppStatus::from_str(&app_status)?;
               builder = builder.set_app_status(status);
           }
           
           // Apply secret key
           if let Some(secret_key) = config.secret_key {
               builder = builder.set_secret_key(&secret_key);
           }
           
           builder.build().map_err(|e| e.to_string())
       }
   }
   ```

3. **Remove Hardcoded Values from app_initializer.rs**:
   - Remove hardcoded OAuth credentials
   - Remove hardcoded app status setting
   - All configuration comes from client via enhanced `AppConfig`

**Acceptance Criteria**:
- ✅ NAPI bridge exposes all configuration options
- ✅ No hardcoded values in `app_initializer.rs`
- ✅ TypeScript client can specify all configuration parameters
- ✅ Error forwarding from `lib_bodhiserver` to JavaScript client
- ✅ Comprehensive integration tests with various configuration scenarios

## 4. Testing Strategy

### 4.1. Unit Testing

**Rust Tests**:
```rust
#[rstest]
fn test_enhanced_app_options_builder() {
    let options = AppOptionsBuilder::default()
        .set_env("TEST_VAR", "test_value")
        .set_app_setting("BODHI_PORT", "8080")
        .set_system_setting("BODHI_ENV_TYPE", "development")
        .set_app_reg_info("test_client_id", "test_client_secret")
        .set_app_status(AppStatus::Ready)
        .set_secret_key("test_secret_key")
        .build()
        .expect("Should build successfully");
    
    // Verify all configuration is properly stored
    assert_eq!(options.environment_vars.get("TEST_VAR"), Some(&"test_value".to_string()));
    // ... additional assertions
}
```

**TypeScript Tests**:
```typescript
describe('Enhanced Configuration', () => {
    test('should accept comprehensive configuration', async () => {
        const config: AppConfig = {
            envType: 'development',
            appType: 'container',
            // ... other required fields
            environmentVars: { 'TEST_VAR': 'test_value' },
            appSettings: { 'BODHI_PORT': '8080' },
            clientId: 'test_client_id',
            clientSecret: 'test_client_secret',
            appStatus: 'Ready',
            secretKey: 'test_secret_key'
        };
        
        const app = new BodhiApp();
        await expect(app.initialize(config)).resolves.not.toThrow();
    });
});
```

### 4.2. Integration Testing

**Complete Authentication Flow**:
```typescript
test('should complete login scenario with provided credentials', async () => {
    const config: AppConfig = {
        // ... basic configuration
        clientId: process.env.TEST_CLIENT_ID!,
        clientSecret: process.env.TEST_CLIENT_SECRET!,
        appStatus: 'Ready',
        secretKey: 'test-encryption-key'
    };
    
    const app = new BodhiApp();
    await app.initialize(config);
    const serverUrl = await app.start('localhost', 0);
    
    // Launch Playwright and complete login flow
    const browser = await playwright.chromium.launch({ headless: false });
    const page = await browser.newPage();
    
    await page.goto(serverUrl);
    // Verify redirect to login page
    await expect(page).toHaveURL(/\/ui\/login/);
    
    // Complete login with test credentials
    await page.fill('[data-testid="username"]', process.env.TEST_USERNAME!);
    await page.fill('[data-testid="password"]', process.env.TEST_PASSWORD!);
    await page.click('[data-testid="login-button"]');
    
    // Verify successful login
    await expect(page).toHaveURL(/\/ui\/chat/);
    
    await browser.close();
    await app.shutdown();
});
```

### 4.3. Error Handling Tests

**Configuration Validation**:
```rust
#[rstest]
fn test_configuration_validation_errors() {
    // Test missing required settings
    let result = AppOptionsBuilder::default().build();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing required setting"));
    
    // Test invalid setting values
    let result = AppOptionsBuilder::default()
        .set_system_setting("BODHI_ENV_TYPE", "invalid_env_type")
        .build();
    assert!(result.is_err());
}
```

## 5. Migration Strategy

### 5.1. Backward Compatibility

**Preserve Existing Interfaces**:
- Keep existing `AppOptionsBuilder::development()` method
- Maintain compatibility with current `setup_app_dirs()` usage
- Provide migration path for existing code

### 5.2. Gradual Migration

**Phase-wise Implementation**:
1. **Phase 1**: Implement enhanced builder without breaking existing code
2. **Phase 2**: Update service initialization with backward compatibility
3. **Phase 3**: Update NAPI bridge and remove hardcoded values
4. **Phase 4**: Update documentation and examples

## 6. Verification Commands

**Rust Testing**:
```bash
# Test enhanced configuration builder
cargo test -p lib_bodhiserver test_enhanced_app_options

# Test service initialization
cargo test -p lib_bodhiserver test_build_app_service_with_config

# Test NAPI bridge
cargo test -p lib_bodhiserver_napi

# Full workspace test
cargo test

# Format code
cargo fmt
```

**TypeScript Testing**:
```bash
# From crates/lib_bodhiserver_napi
npm test

# Integration tests with Playwright
npm run test:integration
```

## 7. Success Criteria

### 7.1. Functional Requirements

- ✅ No hardcoded authentication credentials in any Rust code
- ✅ All configuration parameters can be specified by client
- ✅ Proper validation with descriptive error messages
- ✅ Complete authentication flow works with provided credentials
- ✅ Settings.yaml created automatically from app settings
- ✅ Localization service initialized automatically in production

### 7.2. Technical Requirements

- ✅ Backward compatibility maintained
- ✅ Comprehensive test coverage (>90%)
- ✅ Clean error handling and propagation
- ✅ Performance equivalent to current implementation
- ✅ Memory usage within acceptable bounds

### 7.3. Integration Requirements

- ✅ NAPI bridge exposes all configuration capabilities
- ✅ TypeScript client can control all aspects of initialization
- ✅ Playwright tests demonstrate complete functionality
- ✅ Error scenarios properly handled and tested

---

*Specification created on 2025-06-16*  
*Builds upon: `ai-docs/02-features/completed-stories/20250616-napi-ffi-implementation-completion.md`*  
*Reference: `ai-docs/01-architecture/backend-settings-service.md`*
