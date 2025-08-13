# CLAUDE.md - integration-tests

This file provides guidance to Claude Code when working with the `integration-tests` crate, which provides end-to-end testing for BodhiApp.

## Purpose

The `integration-tests` crate provides comprehensive end-to-end testing:

- **Live Server Testing**: Real server startup and shutdown testing with actual models
- **API Integration Testing**: Complete HTTP API testing with authentication flows
- **Chat Completion Testing**: Streaming and non-streaming chat completion validation
- **Model Management Testing**: End-to-end model loading and management workflows
- **Cross-Component Testing**: Integration testing across all application components

## Key Components

### Live Server Tests
- Complete server lifecycle testing with real llama.cpp integration
- Model loading and server startup validation
- Shared context and server factory testing
- Resource cleanup and proper shutdown testing

### API Integration Tests
- HTTP endpoint testing with real authentication
- Chat completion API validation (streaming and non-streaming)
- API ping and health check validation
- Error handling and edge case testing

### Test Utilities
- Live server setup and teardown utilities
- Test data management and fixture setup
- Authentication token generation for testing
- Model file management for test scenarios

### Test Data
- Sample model files for testing (Llama-68M variants)
- HuggingFace hub cache simulation
- Configuration files and test aliases
- Mock data for offline testing scenarios

## Dependencies

### Core Components (dev-dependencies)
- `server_app` - Complete server application for integration testing
- `lib_bodhiserver` - Embeddable server library testing
- `services` - Business logic service testing
- `routes_app` - Application API endpoint testing
- `server_core` - HTTP server infrastructure testing
- `auth_middleware` - Authentication middleware testing
- `llama_server_proc` - LLM process management testing
- `objs` - Domain objects validation

### Testing Framework
- `rstest` - Parameterized testing framework
- `tokio` - Async runtime for async test execution
- `anyhow` - Error handling in tests
- `reqwest` - HTTP client for API testing
- `serial_test` - Sequential test execution for resource conflicts

### Test Support
- `tempfile` - Temporary file management for tests
- `pretty_assertions` - Enhanced assertion output
- `maplit` - HashMap literal macros for test data
- `fs_extra` - Extended file system operations

## Architecture Position

The `integration-tests` crate sits at the testing validation layer:
- **Validates**: Complete application functionality end-to-end
- **Tests**: Cross-component integration and workflows
- **Ensures**: Real-world usage scenarios work correctly
- **Verifies**: Production-like deployment scenarios

## Usage Patterns

### Live Server Integration Testing
```rust
use llama_server_proc::{LlamaServer, LlamaServerArgsBuilder};
use rstest::{fixture, rstest};

#[fixture]
fn test_model_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/live/huggingface/hub")
        .join("models--afrideva--Llama-68M-Chat-v1-GGUF")
        .join("snapshots/4bcbc666d2f0d2b04d06f046d6baccdab79eac61")
        .join("llama-68m-chat-v1.q8_0.gguf")
}

#[rstest]
#[tokio::test]
async fn test_server_startup_with_model(
    test_model_path: PathBuf,
    lookup_path: PathBuf,
) -> anyhow::Result<()> {
    let exec_path = exec_path_from(&lookup_path, DEFAULT_VARIANT);
    let server = LlamaServer::new(
        &exec_path,
        LlamaServerArgsBuilder::default()
            .alias("test-model")
            .model(test_model_path)
            .build()?,
    )?;

    let result = server.start().await;
    server.stop_unboxed().await?;
    
    assert!(result.is_ok(), "Server startup failed: {:?}", result);
    Ok(())
}
```

### HTTP API Integration Testing
```rust
use reqwest::Client;
use serde_json::json;

#[rstest]
#[tokio::test]
async fn test_chat_completion_non_streamed(
    live_server: LiveServer,
) -> anyhow::Result<()> {
    let client = Client::new();
    
    let request = json!({
        "model": "test-model",
        "messages": [
            {"role": "user", "content": "Hello, how are you?"}
        ],
        "stream": false,
        "max_tokens": 50
    });

    let response = client
        .post(&format!("{}/v1/chat/completions", live_server.base_url()))
        .header("Authorization", format!("Bearer {}", live_server.token()))
        .json(&request)
        .send()
        .await?;

    assert_eq!(response.status(), 200);
    
    let chat_response: serde_json::Value = response.json().await?;
    assert!(chat_response["choices"].is_array());
    assert!(!chat_response["choices"].as_array().unwrap().is_empty());
    
    Ok(())
}
```

### Streaming API Testing
```rust
use futures::StreamExt;

#[rstest]
#[tokio::test]
async fn test_chat_completion_streamed(
    live_server: LiveServer,
) -> anyhow::Result<()> {
    let client = Client::new();
    
    let request = json!({
        "model": "test-model",
        "messages": [
            {"role": "user", "content": "Count to 5"}
        ],
        "stream": true,
        "max_tokens": 30
    });

    let mut stream = client
        .post(&format!("{}/v1/chat/completions", live_server.base_url()))
        .header("Authorization", format!("Bearer {}", live_server.token()))
        .json(&request)
        .send()
        .await?
        .bytes_stream();

    let mut received_chunks = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        
        if text.starts_with("data: ") && !text.contains("[DONE]") {
            received_chunks += 1;
        }
    }

    assert!(received_chunks > 0, "No streaming chunks received");
    Ok(())
}
```

### Shared Context Testing
```rust
use server_core::{DefaultSharedContext, SharedContext};
use services::test_utils::OfflineHubService;

#[rstest]
#[tokio::test]
async fn test_shared_context_reload(
    lookup_path: PathBuf,
    tests_data: PathBuf,
) -> anyhow::Result<()> {
    let hub_service = OfflineHubService::new(HfHubService::new(
        tests_data.join("live/huggingface/hub"),
        false,
        None,
    ));
    
    let shared_context = DefaultSharedContext::with_args(
        Arc::new(hub_service),
        Box::new(DefaultServerFactory),
        &lookup_path,
        DEFAULT_VARIANT,
    );

    // Test initial load
    let result = shared_context.reload(None).await;
    assert!(result.is_ok(), "Initial reload failed: {:?}", result);

    // Test reload with specific model
    let server_args = LlamaServerArgsBuilder::default()
        .alias("test-alias")
        .model(test_model_path)
        .build()?;
    
    let result = shared_context.reload(Some(server_args)).await;
    assert!(result.is_ok(), "Model reload failed: {:?}", result);

    shared_context.stop().await?;
    Ok(())
}
```

## Test Infrastructure

### LiveServer Test Utility
```rust
pub struct LiveServer {
    server_handle: JoinHandle<()>,
    base_url: String,
    auth_token: String,
    shutdown_sender: tokio::sync::oneshot::Sender<()>,
}

impl LiveServer {
    pub async fn start() -> anyhow::Result<Self> {
        // Initialize test database and services
        let app_service = setup_test_app_service().await?;
        
        // Create router with test configuration
        let router = create_test_router(app_service).await?;
        
        // Start server on random port
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        
        // Generate test authentication token
        let auth_token = generate_test_token().await?;
        
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        
        let server_handle = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    shutdown_rx.await.ok();
                })
                .await
                .unwrap();
        });

        Ok(Self {
            server_handle,
            base_url: format!("http://{}", addr),
            auth_token,
            shutdown_sender: shutdown_tx,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn token(&self) -> &str {
        &self.auth_token
    }

    pub async fn shutdown(self) -> anyhow::Result<()> {
        self.shutdown_sender.send(()).ok();
        self.server_handle.await?;
        Ok(())
    }
}
```

### Test Data Management
```rust
// Test model files and HuggingFace cache structure
tests/
├── data/
│   └── live/
│       ├── bodhi/
│       │   ├── aliases/
│       │   │   └── phi4--mini-instruct.yaml
│       │   ├── logs/
│       │   └── models.yaml
│       └── huggingface/
│           └── hub/
│               ├── models--afrideva--Llama-68M-Chat-v1-GGUF/
│               │   ├── blobs/
│               │   │   └── cdd6bad08258f53c637c233309c3b41ccd91907359364aaa02e18df54c34b836
│               │   ├── refs/
│               │   │   └── main
│               │   └── snapshots/
│               │       └── 4bcbc666d2f0d2b04d06f046d6baccdab79eac61/
│               │           └── llama-68m-chat-v1.q8_0.gguf
│               └── models--TheBloke--TinyLlama-1.1B-Chat-v1.0-GGUF/
│                   └── blobs/
│                       └── da3087fb14aede55fde6eb81a0e55e886810e43509ec82ecdc7aa5d62a03b556
```

### Test Configuration
```rust
// Test fixtures for consistent setup
#[fixture]
fn lookup_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../llama_server_proc/bin")
}

#[fixture]
fn tests_data() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
}

#[fixture]
fn test_model_68m(tests_data: PathBuf) -> PathBuf {
    tests_data
        .join("live/huggingface/hub")
        .join("models--afrideva--Llama-68M-Chat-v1-GGUF")
        .join("snapshots/4bcbc666d2f0d2b04d06f046d6baccdab79eac61")
        .join("llama-68m-chat-v1.q8_0.gguf")
}
```

## Testing Categories

### Component Integration Tests
- Server startup and shutdown lifecycle
- Database integration and migration testing
- Authentication flow validation
- Service dependency injection testing

### API Endpoint Tests
- HTTP request/response validation
- Authentication and authorization testing
- Error handling and status code validation
- Content type and header validation

### Model Management Tests
- Model loading and unloading
- Alias creation and management
- HuggingFace integration testing
- File system operations testing

### Chat Completion Tests
- OpenAI API compatibility validation
- Streaming response testing
- Token counting and usage reporting
- Error handling for invalid requests

### Performance Tests
- Concurrent request handling
- Memory usage validation
- Response time benchmarking
- Resource cleanup verification

## Development Guidelines

### Adding New Integration Tests
1. Identify the integration scenario to test
2. Create appropriate test fixtures and data
3. Use `LiveServer` utility for full server testing
4. Include proper cleanup in test teardown
5. Add both positive and negative test cases

### Test Data Management
- Use realistic but small model files for testing
- Maintain test data in version control
- Document test data requirements and setup
- Clean up temporary files and resources

### Error Handling in Tests
- Test both success and failure scenarios
- Validate error messages and status codes
- Ensure proper resource cleanup on failures
- Use descriptive assertion messages

### Test Performance
- Keep integration tests focused and fast
- Use `serial_test` for tests that conflict
- Mock external dependencies when possible
- Profile test execution times

## Testing Strategy

### Test Execution
```bash
# Run all integration tests
cargo test --package integration-tests

# Run specific test category
cargo test --package integration-tests test_live_api

# Run tests with output
cargo test --package integration-tests -- --nocapture

# Run tests serially (for resource conflicts)
cargo test --package integration-tests -- --test-threads=1
```

### Continuous Integration
- Integration tests run in CI pipeline
- Use cached test data for faster execution
- Parallel execution where safe
- Proper resource isolation between tests

### Test Environment
- Isolated test database for each test
- Temporary directories for file operations
- Clean state before each test execution
- Proper cleanup after test completion

## Error Scenarios Tested

### Server Startup Failures
- Invalid model file paths
- Missing executable dependencies
- Port binding conflicts
- Database connection failures

### API Request Failures
- Invalid authentication tokens
- Malformed request payloads
- Unsupported model requests
- Network connectivity issues

### Model Management Failures
- Corrupted model files
- Insufficient disk space
- Permission denied errors
- Invalid model formats

## Security Testing

### Authentication Testing
- Valid and invalid token validation
- Session management testing
- Authorization level verification
- Token expiration handling

### Input Validation Testing
- SQL injection prevention
- Path traversal protection
- Request size limits
- Content type validation

## Future Extensions

The integration-tests crate can be extended with:
- Load testing with realistic traffic patterns
- Cross-platform testing automation
- Performance regression testing
- Security vulnerability scanning
- End-to-end UI testing with browser automation
- Multi-tenant testing scenarios