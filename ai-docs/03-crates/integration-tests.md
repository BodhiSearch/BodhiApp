# integration-tests - End-to-End Testing

## Overview

The `integration-tests` crate provides comprehensive end-to-end testing for BodhiApp, validating the complete system functionality including API endpoints, LLM integration, and real-world usage scenarios. It ensures that all components work together correctly in production-like environments.

## Purpose

- **End-to-End Testing**: Complete system integration testing
- **API Validation**: Comprehensive API endpoint testing
- **LLM Integration Testing**: Real LLM inference testing with live models
- **Performance Testing**: System performance and load testing
- **Regression Testing**: Prevent regressions in core functionality

## Key Test Categories

### Live API Testing

#### API Ping Tests (`test_live_api_ping.rs`)
- **Health Check Validation**: Test system health endpoints
- **Service Availability**: Verify all services are running correctly
- **Response Time Testing**: Validate response time requirements
- **Error Handling**: Test error responses and recovery

#### Chat Completions - Non-Streamed (`test_live_chat_completions_non_streamed.rs`)
- **OpenAI Compatibility**: Test OpenAI API compatibility
- **Model Integration**: Test with real LLM models
- **Request/Response Validation**: Validate complete request/response cycle
- **Parameter Testing**: Test various inference parameters

#### Chat Completions - Streamed (`test_live_chat_completions_streamed.rs`)
- **Streaming API**: Test Server-Sent Events streaming
- **Real-time Response**: Validate real-time response streaming
- **Connection Management**: Test connection lifecycle
- **Stream Error Handling**: Test stream interruption and recovery

### Test Infrastructure

#### Live Server Utilities (`utils/live_server_utils.rs`)
- **Server Setup**: Automated test server setup and teardown
- **Model Management**: Test model loading and configuration
- **Environment Setup**: Test environment configuration
- **Resource Cleanup**: Proper test resource cleanup

#### Test Library (`test_live_lib.rs`)
- **Common Test Functions**: Shared test utilities and helpers
- **Test Data Management**: Test data generation and management
- **Assertion Helpers**: Custom assertion functions for BodhiApp
- **Mock Data**: Mock data generation for testing

## Directory Structure

```
src/
└── lib.rs                    # Test library exports

tests/
├── test_live_api_ping.rs                           # Health check tests
├── test_live_chat_completions_non_streamed.rs      # Non-streamed chat tests
├── test_live_chat_completions_streamed.rs          # Streamed chat tests
├── test_live_lib.rs                                # Test library
├── data/                                           # Test data
│   └── live/                                       # Live test data
└── utils/                                          # Test utilities
    ├── mod.rs
    └── live_server_utils.rs                        # Server test utilities
```

## Test Scenarios

### API Endpoint Testing

#### Health Check Tests
```rust
#[tokio::test]
async fn test_health_endpoint() {
    let server = setup_test_server().await;
    
    let response = server
        .get("/health")
        .await
        .expect("Health endpoint should respond");
    
    assert_eq!(response.status(), 200);
    assert!(response.json::<HealthResponse>().is_ok());
}
```

#### Authentication Tests
```rust
#[tokio::test]
async fn test_authenticated_endpoint() {
    let server = setup_test_server().await;
    let token = create_test_token(&server).await;
    
    let response = server
        .get("/api/models")
        .header("Authorization", format!("Bearer {}", token))
        .await
        .expect("Authenticated request should succeed");
    
    assert_eq!(response.status(), 200);
}
```

### LLM Integration Testing

#### Model Loading Tests
```rust
#[tokio::test]
async fn test_model_loading() {
    let server = setup_test_server().await;
    
    // Test model loading
    let load_response = server
        .post("/api/models/load")
        .json(&LoadModelRequest {
            model_path: test_model_path(),
            config: default_model_config(),
        })
        .await
        .expect("Model loading should succeed");
    
    assert_eq!(load_response.status(), 200);
    
    // Verify model is available
    let models_response = server
        .get("/api/models")
        .await
        .expect("Models list should be available");
    
    let models: ModelsResponse = models_response.json().await.unwrap();
    assert!(!models.data.is_empty());
}
```

#### Chat Completion Tests
```rust
#[tokio::test]
async fn test_chat_completion() {
    let server = setup_test_server_with_model().await;
    
    let request = ChatCompletionRequest {
        model: "test-model".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Hello, how are you?".to_string(),
            }
        ],
        temperature: Some(0.7),
        max_tokens: Some(100),
        stream: Some(false),
    };
    
    let response = server
        .post("/v1/chat/completions")
        .json(&request)
        .await
        .expect("Chat completion should succeed");
    
    assert_eq!(response.status(), 200);
    
    let completion: ChatCompletionResponse = response.json().await.unwrap();
    assert!(!completion.choices.is_empty());
    assert!(!completion.choices[0].message.content.is_empty());
}
```

### Streaming Tests

#### SSE Streaming Tests
```rust
#[tokio::test]
async fn test_streaming_chat_completion() {
    let server = setup_test_server_with_model().await;
    
    let request = ChatCompletionRequest {
        model: "test-model".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Tell me a short story".to_string(),
            }
        ],
        stream: Some(true),
    };
    
    let mut stream = server
        .post("/v1/chat/completions")
        .json(&request)
        .stream()
        .await
        .expect("Streaming should start");
    
    let mut chunks = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.expect("Stream chunk should be valid");
        chunks.push(chunk);
    }
    
    assert!(!chunks.is_empty());
    assert!(chunks.iter().any(|chunk| chunk.choices[0].delta.content.is_some()));
}
```

## Test Configuration

### Test Environment Setup
```rust
pub struct TestConfig {
    pub server_host: String,
    pub server_port: u16,
    pub test_model_path: PathBuf,
    pub test_timeout: Duration,
    pub enable_gpu: bool,
}

impl TestConfig {
    pub fn from_env() -> Self {
        Self {
            server_host: env::var("TEST_SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("TEST_SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            test_model_path: PathBuf::from(
                env::var("TEST_MODEL_PATH")
                    .unwrap_or_else(|_| "./test_models/test-model.gguf".to_string())
            ),
            test_timeout: Duration::from_secs(
                env::var("TEST_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()
                    .unwrap_or(300)
            ),
            enable_gpu: env::var("TEST_ENABLE_GPU")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        }
    }
}
```

### Test Data Management
```rust
pub struct TestDataManager {
    pub test_models: Vec<TestModel>,
    pub test_conversations: Vec<TestConversation>,
    pub test_users: Vec<TestUser>,
}

impl TestDataManager {
    pub fn load_test_data() -> Result<Self, TestError> {
        // Load test data from files
    }
    
    pub fn create_test_model(&self) -> TestModel {
        // Create test model configuration
    }
    
    pub fn create_test_conversation(&self) -> TestConversation {
        // Create test conversation data
    }
}
```

## Test Utilities

### Server Setup Utilities
```rust
pub async fn setup_test_server() -> TestServer {
    let config = TestConfig::from_env();
    let server = TestServer::new(config).await;
    server.wait_for_ready().await;
    server
}

pub async fn setup_test_server_with_model() -> TestServer {
    let server = setup_test_server().await;
    server.load_test_model().await;
    server
}

pub async fn teardown_test_server(server: TestServer) {
    server.cleanup().await;
}
```

### Assertion Helpers
```rust
pub fn assert_valid_chat_response(response: &ChatCompletionResponse) {
    assert!(!response.choices.is_empty());
    assert!(!response.choices[0].message.content.is_empty());
    assert!(response.usage.is_some());
    assert!(response.usage.as_ref().unwrap().total_tokens > 0);
}

pub fn assert_valid_streaming_chunk(chunk: &ChatCompletionChunk) {
    assert!(!chunk.choices.is_empty());
    assert!(chunk.choices[0].delta.content.is_some() || 
            chunk.choices[0].finish_reason.is_some());
}
```

## Performance Testing

### Load Testing
```rust
#[tokio::test]
async fn test_concurrent_requests() {
    let server = setup_test_server_with_model().await;
    let num_concurrent = 10;
    
    let tasks: Vec<_> = (0..num_concurrent)
        .map(|_| {
            let server = server.clone();
            tokio::spawn(async move {
                let request = create_test_chat_request();
                server.post("/v1/chat/completions")
                    .json(&request)
                    .await
            })
        })
        .collect();
    
    let results = futures::future::join_all(tasks).await;
    
    // All requests should succeed
    for result in results {
        let response = result.unwrap().unwrap();
        assert_eq!(response.status(), 200);
    }
}
```

### Memory Usage Testing
```rust
#[tokio::test]
async fn test_memory_usage() {
    let server = setup_test_server_with_model().await;
    let initial_memory = server.get_memory_usage().await;
    
    // Perform many requests
    for _ in 0..100 {
        let request = create_test_chat_request();
        server.post("/v1/chat/completions")
            .json(&request)
            .await
            .unwrap();
    }
    
    let final_memory = server.get_memory_usage().await;
    
    // Memory usage should not grow excessively
    assert!(final_memory < initial_memory * 2);
}
```

## Dependencies

### Testing Framework
- **tokio-test**: Async testing utilities
- **rstest**: Parameterized testing
- **serial_test**: Sequential test execution
- **pretty_assertions**: Enhanced assertion output

### HTTP Testing
- **reqwest**: HTTP client for testing
- **axum-test**: Axum-specific testing utilities
- **tower-test**: Tower service testing

### Test Data
- **serde_json**: JSON test data handling
- **tempfile**: Temporary file management
- **uuid**: Test ID generation

## CI/CD Integration

### GitHub Actions
```yaml
name: Integration Tests
on: [push, pull_request]

jobs:
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
      - name: Download test models
        run: ./scripts/download-test-models.sh
      - name: Run integration tests
        run: cargo test --package integration-tests
```

### Test Reporting
- **Test Coverage**: Code coverage reporting
- **Performance Metrics**: Performance regression detection
- **Test Results**: Detailed test result reporting
- **Failure Analysis**: Automatic failure analysis and reporting

## Future Extensions

The integration-tests crate is designed to support:
- **Multi-Model Testing**: Testing with multiple LLM models
- **Cross-Platform Testing**: Testing on different operating systems
- **Stress Testing**: Extended stress and endurance testing
- **Security Testing**: Security vulnerability testing
- **Compatibility Testing**: Testing with different client libraries
