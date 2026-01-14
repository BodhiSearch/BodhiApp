# PACKAGE.md - integration-tests

This document provides detailed technical information for the `integration-tests` crate, focusing on BodhiApp-specific end-to-end testing patterns, live server testing infrastructure, and comprehensive authentication integration testing.

## Architecture Position in BodhiApp

The `integration-tests` crate operates as BodhiApp's **highest-level testing validation layer**, providing comprehensive end-to-end testing that validates complete application workflows in production-like environments. It coordinates testing across all application crates through sophisticated service composition and live server management.

### Key Architectural Decisions
- **Real Authentication Integration**: Uses actual OAuth2 flows rather than mocked authentication for realistic testing
- **Live Server Testing**: Starts real server instances with actual llama.cpp integration for comprehensive validation
- **Serial Test Execution**: All live server tests run serially to prevent resource conflicts and ensure proper isolation
- **Production Environment Simulation**: Uses real model files, OAuth2 servers, and database-backed sessions

### Cross-Crate Dependencies
- **Complete Application Stack**: Integrates with all BodhiApp crates through dev-dependencies with test-utils features
- **Service Coordination**: Validates service dependencies and integration patterns across application layers
- **Authentication Middleware**: Tests complete OAuth2 flows with session management and cookie handling
- **Model Management**: Validates llama.cpp integration with real model files and HuggingFace cache simulation

## Live Server Testing Infrastructure Implementation

### TestServerHandle Pattern
Core infrastructure for managing complete server lifecycle with authentication and resource management:

```rust
// TestServerHandle structure (see tests/utils/live_server_utils.rs)
pub struct TestServerHandle {
  pub temp_cache_dir: TempDir,
  pub host: String,
  pub port: u16,
  pub handle: ServerShutdownHandle,
  pub app_service: Arc<dyn AppService>,
}

// Live server fixture with complete service setup (see tests/utils/live_server_utils.rs)
#[fixture]
#[awt]
pub async fn live_server(
  #[future] llama2_7b_setup: anyhow::Result<(TempDir, Arc<dyn AppService>)>,
) -> anyhow::Result<TestServerHandle> {
  let host = String::from("127.0.0.1");
  let port = rand::rng().random_range(2000..60000);
  let (temp_cache_dir, app_service) = llama2_7b_setup?;
  let serve_command = ServeCommand::ByParams { host: host.clone(), port };
  let handle = serve_command.get_server_handle(app_service.clone(), None).await?;
  // Returns complete server handle with cleanup coordination
}
```

**Key Implementation Features**:
- **Random Port Allocation**: Uses random ports (2000-60000 range) to prevent conflicts during test execution
- **Temporary Directory Management**: Each test gets isolated temporary directories with proper cleanup
- **Server Lifecycle Management**: Uses `ServerShutdownHandle` from server_app for proper shutdown coordination
- **Service Access**: Provides access to complete `AppService` for authentication and testing utilities

### **OAuth2 Authentication Testing Infrastructure**
Complete authentication flow testing with real OAuth2 tokens and session management:

```rust
// OAuth2 token acquisition (see tests/utils/live_server_utils.rs)
pub async fn get_oauth_tokens(app_service: &dyn AppService) -> anyhow::Result<(String, String)> {
  let setting_service = app_service.setting_service();
  let auth_url = setting_service.auth_url();
  let realm = setting_service.auth_realm();
  let app_reg_info = app_service.secret_service().app_reg_info()?.expect("AppRegInfo is not set");
  
  let token_url = format!("{}/realms/{}/protocol/openid-connect/token", 
    auth_url.trim_end_matches('/'), realm);
  
  let params = [
    ("grant_type", "password"),
    ("client_id", &app_reg_info.client_id),
    ("client_secret", &app_reg_info.client_secret),
    ("username", &username), ("password", &password),
    ("scope", &["openid", "email", "profile", "roles"].join(" ")),
  ];
  // Returns (access_token, refresh_token) for session creation
}

// Session creation with database-backed storage (see tests/utils/live_server_utils.rs)
pub async fn create_authenticated_session(
  app_service: &Arc<dyn AppService>,
  access_token: &str,
  refresh_token: &str,
) -> anyhow::Result<String> {
  let session_service = app_service.session_service();
  let session_id = Id::default();
  let session_data = maplit::hashmap! {
    SESSION_KEY_ACCESS_TOKEN.to_string() => Value::String(access_token.to_string()),
    SESSION_KEY_REFRESH_TOKEN.to_string() => Value::String(refresh_token.to_string()),
  };
  
  let mut record = Record {
    id: session_id, data: session_data,
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  session_service.get_session_store().create(&mut record).await?;
  Ok(session_id.to_string())
}
```

**Authentication Testing Features**:
- **Real OAuth2 Flows**: Uses actual OAuth2 server with client credentials and password flows
- **Database-Backed Sessions**: Creates real sessions with tower-sessions and SQLite storage
- **Cookie Management**: Secure session cookie creation for HTTP-based authentication testing
- **Multi-Client Support**: OAuth2 client creation and resource management for comprehensive testing

## Test Data Management Implementation

### **HuggingFace Cache Simulation**
Complete hub cache structure simulation for realistic model management testing:

```rust
// Test data copying with proper structure (see tests/utils/live_server_utils.rs)
pub fn copy_test_dir(src: &str, dst_path: &Path) {
  let src_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(src);
  copy(src_path, dst_path, &COPY_OPTIONS).unwrap();
}

// HuggingFace cache structure (see tests/data/live/huggingface/hub/)
tests/data/live/huggingface/hub/
├── models--afrideva--Llama-68M-Chat-v1-GGUF/
│   ├── blobs/cdd6bad08258f53c637c233309c3b41ccd91907359364aaa02e18df54c34b836
│   ├── refs/main
│   └── snapshots/4bcbc666d2f0d2b04d06f046d6baccdab79eac61/
│       └── llama-68m-chat-v1.q8_0.gguf
└── models--Felladrin--Llama-68M-Chat-v1/
    ├── blobs/afeed3e0dce0ce1ebd2be65c3b9bdabdabddf2ed
    └── snapshots/180d584580aa5cf33558d2bce51f1d125e20c7c7/
```

### **Bodhi Configuration Management**
Test-specific configuration with aliases and model definitions:

```rust
// Bodhi configuration structure (see tests/data/live/bodhi/)
tests/data/live/bodhi/
├── aliases/qwen3--1.7b-instruct.yaml
├── logs/
└── models.yaml

// Environment configuration loading (see tests/utils/live_server_utils.rs)
let env_test_path = Path::new(env!("CARGO_MANIFEST_DIR"))
  .join("tests").join("resources").join(".env.test");
if env_test_path.exists() {
  dotenv::from_filename(&env_test_path).ok();
}
```

**Test Data Features**:
- **Small Model Files**: Uses Llama-68M variants for fast test execution while maintaining realism
- **Proper Hub Structure**: Maintains complete HuggingFace hub cache structure with blobs and snapshots
- **Version Control**: All test data maintained in version control for consistency across environments
- **Environment Isolation**: Each test gets isolated temporary directories with proper cleanup

## API Integration Testing Patterns

### **Non-Streaming Chat Completion Testing**
Complete API testing with authentication and content validation:

```rust
// Non-streaming API test pattern (see tests/test_live_chat_completions_non_streamed.rs)
#[rstest::rstest]
#[awt]
#[tokio::test]
#[timeout(Duration::from_secs(5 * 60))]
#[serial_test::serial(live)]
async fn test_live_chat_completions_non_streamed(
  #[future] live_server: anyhow::Result<TestServerHandle>,
) -> anyhow::Result<()> {
  let TestServerHandle { host, port, handle, app_service, .. } = live_server?;
  
  // OAuth2 token acquisition and session creation
  let (access_token, refresh_token) = get_oauth_tokens(app_service.as_ref()).await?;
  let session_id = create_authenticated_session(&app_service, &access_token, &refresh_token).await?;
  let session_cookie = create_session_cookie(&session_id);
  
  // API request with session-based authentication
  let response = reqwest::Client::new()
    .post(&format!("http://{host}:{port}/v1/chat/completions"))
    .header("Content-Type", "application/json")
    .header("Cookie", session_cookie.to_string())
    .json(&serde_json::json!({
      "model": "qwen3:1.7b-instruct",
      "seed": 42,
      "messages": [{"role": "user", "content": "Answer in one word. What day comes after Monday?"}]
    }))
    .send().await?;
  
  // Content validation and cleanup
  let response = response.json::<Value>().await?;
  handle.shutdown().await?;
  assert!(response["choices"][0]["message"]["content"].as_str().unwrap().contains("Tuesday"));
}
```

### **Streaming API Testing Implementation**
Server-sent events testing with streaming response validation:

```rust
// Streaming API test pattern (see tests/test_live_chat_completions_streamed.rs)
async fn test_live_chat_completions_stream(
  #[future] live_server: anyhow::Result<TestServerHandle>,
) -> anyhow::Result<()> {
  // Authentication setup identical to non-streaming test
  
  let response = client.post(&chat_endpoint)
    .json(&serde_json::json!({
      "model": "qwen3:1.7b-instruct", "seed": 42, "stream": true,
      "messages": [{"role": "user", "content": "Answer in one word. What day comes after Monday?"}]
    }))
    .send().await?;
  
  // Streaming response parsing
  let response = response.text().await?;
  let streams = response.lines()
    .filter_map(|line| {
      if line.is_empty() || line == "data: [DONE]" { None }
      else if line.starts_with("data: ") {
        let value: Value = serde_json::from_str(line.strip_prefix("data: ").unwrap()).unwrap();
        Some(value)
      } else { None }
    })
    .collect::<Vec<_>>();
  
  // Content validation across streaming chunks
  let full_content = streams[0..max(streams.len() - 1, 0)]
    .iter()
    .map(|stream| stream["choices"][0]["delta"]["content"].as_str().unwrap_or_default())
    .collect::<Vec<_>>().join("");
  assert!(full_content.contains("Tuesday") || full_content.contains("Tues"));
}
```

**API Testing Features**:
- **Session-Based Authentication**: Uses secure HTTP cookies rather than Bearer tokens for realistic testing
- **Content Validation**: Validates actual model responses for correctness and completeness
- **Streaming Protocol Testing**: Parses server-sent events and validates streaming response format
- **Error Handling**: Comprehensive error handling with proper resource cleanup on failures

## Library Integration Testing Implementation

### **LLM Server Process Testing**
Direct llama.cpp server testing with various model file formats:

```rust
// Direct server testing pattern (see tests/test_live_lib.rs)
#[rstest]
#[tokio::test]
async fn test_live_llama_server_load_exec_with_server(
  tests_data: PathBuf, lookup_path: PathBuf,
) -> anyhow::Result<()> {
  let llama_68m = tests_data.join("live/huggingface/hub/models--afrideva--Llama-68M-Chat-v1-GGUF/snapshots/4bcbc666d2f0d2b04d06f046d6baccdab79eac61/llama-68m-chat-v1.q8_0.gguf");
  let exec_path = &lookup_path.join(BUILD_TARGET).join(DEFAULT_VARIANT).join(EXEC_NAME);
  
  let server = LlamaServer::new(&exec_path, 
    LlamaServerArgsBuilder::default().alias("testalias").model(llama_68m).build().unwrap())?;
  let result = server.start().await;
  server.stop_unboxed().await?;
  assert!(result.is_ok(), "server start failed with error: {:?}", result);
}

// Shared context testing with service coordination (see tests/test_live_lib.rs)
async fn test_live_shared_rw_reload(lookup_path: PathBuf, tests_data: PathBuf) -> anyhow::Result<()> {
  let hub_service = OfflineHubService::new(HfHubService::new(
    tests_data.join("live/huggingface/hub"), false, None));
  
  let mut mock_setting_service = MockSettingService::new();
  mock_setting_service.expect_exec_path_from().returning(move || {
    lookup_path.clone().join(BUILD_TARGET).join(DEFAULT_VARIANT).join(EXEC_NAME)
  });
  
  let shared_rw = DefaultSharedContext::with_args(
    Arc::new(hub_service), Arc::new(mock_setting_service), Box::new(DefaultServerFactory));
  let result = shared_rw.reload(None).await;
  shared_rw.stop().await?;
  assert!(result.is_ok(), "shared rw reload failed with error: {:?}", result);
}
```

**Library Testing Features**:
- **Direct Server Testing**: Tests llama.cpp server startup and shutdown without HTTP layer
- **Model File Format Testing**: Validates various model file formats (symlinks, direct files, blobs)
- **Shared Context Testing**: Tests server factory and shared context lifecycle management
- **Service Coordination**: Validates cross-service integration with mock and real implementations

## Test Execution and Resource Management

### **Serial Test Execution Pattern**
Resource conflict prevention with proper isolation:

```rust
// Serial test execution attributes (see all test files)
#[rstest::rstest]
#[awt]
#[tokio::test]
#[timeout(Duration::from_secs(5 * 60))]
#[serial_test::serial(live)]
async fn test_function_name() {
  // Test implementation with guaranteed serial execution
}
```

### **Environment Configuration Management**
Test environment setup with OAuth2 server configuration:

```rust
// Environment configuration (see tests/resources/.env.test)
INTEG_TEST_AUTH_URL=https://test-id.getbodhi.app
INTEG_TEST_AUTH_REALM=bodhi
INTEG_TEST_DEV_CONSOLE_CLIENT_ID=client-bodhi-dev-console
INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET=change-me
INTEG_TEST_USERNAME=user@email.com
INTEG_TEST_PASSWORD=pass

// Environment loading in tests (see tests/utils/live_server_utils.rs)
let env_test_path = Path::new(env!("CARGO_MANIFEST_DIR"))
  .join("tests").join("resources").join(".env.test");
if env_test_path.exists() {
  dotenv::from_filename(&env_test_path).ok();
}
```

**Execution Management Features**:
- **Serial Execution**: All live server tests run serially using `#[serial_test::serial(live)]` to prevent conflicts
- **Timeout Management**: 5-minute timeouts prevent hanging tests in CI environments
- **Resource Cleanup**: Proper cleanup of temporary directories, server processes, and authentication sessions
- **Environment Isolation**: Each test gets isolated environments with proper configuration management

## Cross-Crate Integration Validation

### **Service Composition Testing**
Complete application service setup with dependency injection:

```rust
// Complete service setup fixture (see tests/utils/live_server_utils.rs)
#[fixture]
pub async fn llama2_7b_setup(
  #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
) -> anyhow::Result<(TempDir, Arc<dyn AppService>)> {
  // Environment setup with temporary directories
  let temp_dir = tempfile::tempdir()?;
  let cache_dir = temp_dir.path().join(".cache");
  let bodhi_home = cache_dir.join("bodhi");
  copy_test_dir("tests/data/live/bodhi", &bodhi_home);
  
  // OAuth2 client creation and configuration
  let config = AuthServerConfigBuilder::default()
    .auth_server_url(&auth_server_url).realm(&realm)
    .dev_console_client_id(client_id).dev_console_client_secret(client_secret).build()?;
  let auth_client = AuthServerTestClient::new(config);
  let resource_client = auth_client.create_resource_client("integration_test").await?;
  
  // Service composition with dependency injection
  let service = AppServiceBuilder::new(setting_service)
    .hub_service(hub_service)?.data_service(data_service)?
    .auth_service(Arc::new(auth_service))?.secret_service(Arc::new(secret_service))?
    .localization_service(localization_service.clone())?.build().await?;
  Ok((temp_dir, Arc::new(service)))
}
```

**Integration Validation Features**:
- **Complete Service Setup**: Tests full service dependency injection and coordination
- **OAuth2 Integration**: Creates real OAuth2 clients and validates authentication flows
- **Cross-Service Communication**: Validates data flow between services, middleware, and route handlers
- **Resource Management**: Tests proper resource lifecycle management across service boundaries

## Extension Guidelines for Integration Testing

### Adding New API Integration Tests
When creating new API endpoint integration tests:

1. **Use Live Server Fixture**: Leverage `live_server` fixture for complete server setup with authentication
2. **Follow Authentication Pattern**: Use `get_oauth_tokens` and `create_authenticated_session` for authenticated requests
3. **Implement Serial Execution**: Add `#[serial_test::serial(live)]` for tests using shared resources
4. **Add Proper Timeouts**: Use `#[timeout(Duration::from_secs(5 * 60))]` for tests that may hang
5. **Validate Content**: Test actual response content, not just status codes, for comprehensive validation

### Extending Test Data Management
For new test scenarios and model management:

1. **Maintain Hub Structure**: Follow proper HuggingFace hub cache structure with blobs and snapshots
2. **Use Small Models**: Add small but realistic model files for fast test execution
3. **Version Control Data**: Maintain all test data in version control for consistency
4. **Environment Isolation**: Ensure each test gets isolated temporary directories with proper cleanup
5. **Configuration Management**: Add test-specific configuration files with proper structure

### Cross-Service Integration Testing
For new service coordination and integration scenarios:

1. **Service Composition**: Test complete service dependency injection and coordination patterns
2. **Mock Integration**: Use appropriate service mocks for controlled testing environments
3. **Error Propagation**: Test error handling and propagation across service boundaries
4. **Resource Lifecycle**: Validate proper resource management and cleanup across services
5. **Authentication Integration**: Test complete authentication flows with session management

## Commands for Integration Testing

**All Integration Tests**: `cargo test --package integration-tests` (requires OAuth2 server access and environment configuration)  
**Specific Test Files**: `cargo test --package integration-tests test_live_api_ping`  
**Debug Output**: `cargo test --package integration-tests -- --nocapture` (useful for debugging authentication issues)  
**Environment Setup**: Configure `.env.test` file with OAuth2 server details and test credentials before running tests