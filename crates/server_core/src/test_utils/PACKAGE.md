# PACKAGE.md - server_core/test_utils

This document provides detailed technical information for the `server_core/test_utils` module, focusing on BodhiApp's HTTP infrastructure testing architecture, sophisticated SSE streaming validation, and LLM server context testing coordination.

## HTTP Infrastructure Testing Architecture

### HTTP Response Testing Extensions
Comprehensive HTTP response testing utilities for server infrastructure validation:

```rust
// Pattern structure (see src/test_utils/http.rs:10-25 for complete trait definition)
#[async_trait::async_trait]
pub trait ResponseTestExt {
  async fn json<T>(self) -> anyhow::Result<T> where T: DeserializeOwned;
  async fn json_obj<T>(self) -> anyhow::Result<T> where T: for<'a> Deserialize<'a>;
  async fn text(self) -> anyhow::Result<String>;
  async fn sse<T>(self) -> anyhow::Result<Vec<T>> where T: DeserializeOwned;
  async fn direct_sse(self) -> anyhow::Result<Vec<String>>;
}

pub trait RequestTestExt {
  fn json<T: serde::Serialize>(self, value: T) -> Result<Request<Body>, anyhow::Error>;
  fn json_str(self, value: &str) -> Result<Request<Body>, anyhow::Error>;
}
```

### RouterState Testing Foundation
Comprehensive HTTP infrastructure testing with service mock coordination:

```rust
// Pattern structure (see src/test_utils/state.rs:8-15 for complete fixture)
#[fixture]
#[awt]
pub async fn router_state_stub(#[future] app_service_stub: AppServiceStub) -> DefaultRouterState {
  DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service_stub),
  )
}
```

### HTTP Infrastructure Mock Composition Implementation
Sophisticated testing infrastructure for HTTP server coordination:

**RouterState Testing Integration**:
- `router_state_stub()`: Pre-configured RouterState with MockSharedContext and services integration
- Service registry access testing through AppServiceStub coordination
- HTTP context management testing with realistic LLM server lifecycle simulation
- Error propagation testing from services through HTTP infrastructure to route responses

**Cross-Service HTTP Testing Features**:
- Integration with services `AppServiceStub` for comprehensive business logic coordination
- MockSharedContext provides LLM server context testing without external dependencies
- HTTP error translation testing with service error scenario simulation
- Dependency injection validation for HTTP route handler service access

## SharedContext Mock Architecture

### MockSharedContext Implementation for HTTP Testing
Advanced LLM server context mocking for HTTP infrastructure validation:

```rust
#[cfg(test)]
mod http_context_tests {
    use server_core::{MockSharedContext, DefaultRouterState, RouterStateError};
    use services::test_utils::AppServiceStub;
    use async_openai::types::CreateChatCompletionRequest;
    use objs::{Alias, test_utils::setup_l10n};
    use mockall::predicate::*;
    
    #[rstest]
    #[awt]
    async fn test_router_state_chat_completions_success(
        setup_l10n: (),
        #[future] app_service_stub: AppServiceStub
    ) {
        let mut mock_context = MockSharedContext::new();
        
        // Configure SharedContext mock for HTTP request processing
        mock_context
            .expect_chat_completions()
            .with(
                function(|req: &CreateChatCompletionRequest| req.model == "testalias:instruct"),
                function(|alias: &Alias| alias.alias == "testalias:instruct")
            )
            .returning(|_req, _alias| {
                // Simulate successful LLM server response
                let response = reqwest::Response::from(
                    http::Response::builder()
                        .status(200)
                        .header("Content-Type", "text/event-stream")
                        .body("data: {\"id\":\"test\",\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}")
                        .unwrap()
                );
                Ok(response)
            })
            .times(1);
        
        let router_state = DefaultRouterState::new(
            Arc::new(mock_context),
            Arc::new(app_service_stub.await),
        );
        
        // Test HTTP request processing through RouterState
        let request = CreateChatCompletionRequest {
            model: "testalias:instruct".to_string(),
            messages: vec![],
            ..Default::default()
        };
        
        let result = router_state.chat_completions(request).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "text/event-stream"
        );
    }
}
```

### LLM Server Context Testing Patterns
HTTP context testing with comprehensive lifecycle management:

```rust
#[rstest]
#[awt]
async fn test_shared_context_lifecycle_management(
    setup_l10n: (),
    #[future] app_service_stub: AppServiceStub
) {
    let mut mock_context = MockSharedContext::new();
    
    // Test server lifecycle coordination
    mock_context
        .expect_is_loaded()
        .returning(|| false)
        .times(1);
    
    mock_context
        .expect_reload()
        .with(any())
        .returning(|_args| Ok(()))
        .times(1);
    
    mock_context
        .expect_chat_completions()
        .returning(|_req, _alias| {
            Ok(create_mock_sse_response())
        })
        .times(1);
    
    mock_context
        .expect_stop()
        .returning(|| Ok(()))
        .times(1);
    
    let router_state = DefaultRouterState::new(
        Arc::new(mock_context),
        Arc::new(app_service_stub.await),
    );
    
    // Test complete HTTP context lifecycle
    let request = CreateChatCompletionRequest::default();
    let _response = router_state.chat_completions(request).await.unwrap();
    
    // Test context cleanup
    let stop_result = router_state.stop().await;
    assert!(stop_result.is_ok());
}
```

## ServerFactory Testing Infrastructure

### ServerFactoryStub Implementation for HTTP Testing
Sophisticated LLM server factory testing coordinated with HTTP infrastructure:

```rust
#[derive(Debug)]
pub struct ServerFactoryStub {
    pub servers: Mutex<Vec<Box<dyn Server>>>,
}

impl ServerFactoryStub {
    pub fn new(instance: Box<dyn Server>) -> Self {
        Self {
            servers: Mutex::new(vec![instance]),
        }
    }
    
    pub fn new_with_instances(instances: Vec<Box<dyn Server>>) -> Self {
        Self {
            servers: Mutex::new(instances),
        }
    }
}

impl ServerFactory for ServerFactoryStub {
    fn create_server(
        &self,
        _executable_path: &Path,
        _server_args: &LlamaServerArgs,
    ) -> Result<Box<dyn Server>, ContextError> {
        Ok(self.servers.lock().unwrap().pop().unwrap())
    }
}
```

### HTTP Context Integration Testing Implementation
Comprehensive HTTP infrastructure testing with LLM server coordination:

```rust
#[cfg(test)]
mod server_factory_integration_tests {
    use super::*;
    use llama_server_proc::{MockServer, LlamaServerArgsBuilder};
    use std::path::PathBuf;
    
    #[rstest]
    #[awt]
    async fn test_server_factory_http_context_integration(
        setup_l10n: (),
        #[future] app_service_stub: AppServiceStub
    ) {
        // Create mock LLM server for HTTP context testing
        let mut mock_server = MockServer::new();
        mock_server
            .expect_start()
            .returning(|| Ok(()))
            .times(1);
        
        mock_server
            .expect_chat_completions()
            .returning(|_req| {
                Ok(create_streaming_response())
            })
            .times(1);
        
        mock_server
            .expect_stop()
            .returning(|| Ok(()))
            .times(1);
        
        // Create ServerFactory with mock server
        let factory = ServerFactoryStub::new(Box::new(mock_server));
        let shared_context = DefaultSharedContext::new(
            Arc::new(factory),
            app_service_stub.await.setting_service(),
            app_service_stub.await.hub_service(),
        );
        
        // Test HTTP context with real ServerFactory coordination
        let router_state = DefaultRouterState::new(
            Arc::new(shared_context),
            Arc::new(app_service_stub.await),
        );
        
        let request = CreateChatCompletionRequest {
            model: "testalias:instruct".to_string(),
            messages: vec![],
            ..Default::default()
        };
        
        let result = router_state.chat_completions(request).await;
        assert!(result.is_ok());
    }
}
```

## Server-Sent Events Testing Architecture

### DirectSSE Testing Implementation
Comprehensive application event streaming testing:

```rust
#[cfg(test)]
mod sse_testing {
    use server_core::{DirectSse, DirectEvent};
    use tokio_stream::wrappers::UnboundedReceiverStream;
    use axum::response::IntoResponse;
    use futures_util::StreamExt;
    
    #[tokio::test]
    async fn test_direct_sse_event_formatting() {
        let event = DirectEvent::new()
            .data("Hello, World!")
            .data("Second line");
        
        let bytes = event.finalize();
        let event_str = std::str::from_utf8(&bytes).unwrap();
        
        assert!(event_str.contains("Hello, World!"));
        assert!(event_str.contains("Second line"));
        assert!(event_str.ends_with('\n'));
    }
    
    #[tokio::test]
    async fn test_direct_sse_streaming_response() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Generate test events
        tokio::spawn(async move {
            for i in 0..3 {
                let event = DirectEvent::new()
                    .data(format!("Event {}", i));
                tx.send(Ok(event)).unwrap();
            }
        });
        
        let sse = DirectSse::new(UnboundedReceiverStream::new(rx));
        let response = sse.into_response();
        
        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "text/event-stream"
        );
        assert_eq!(
            response.headers().get("Cache-Control").unwrap(),
            "no-cache"
        );
    }
    
    #[tokio::test]
    async fn test_direct_sse_connection_lifecycle() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Test connection interruption scenarios
        tokio::spawn(async move {
            tx.send(Ok(DirectEvent::new().data("Event 1"))).unwrap();
            // Simulate connection error
            tx.send(Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                "Connection lost"
            ))).unwrap();
        });
        
        let sse = DirectSse::new(UnboundedReceiverStream::new(rx));
        let mut stream = sse.into_stream();
        
        // Validate event delivery before error
        let first_event = stream.next().await.unwrap();
        assert!(first_event.is_ok());
        
        // Validate error propagation
        let error_event = stream.next().await.unwrap();
        assert!(error_event.is_err());
    }
}
```

### RawSSE Testing Implementation
Comprehensive LLM server proxy streaming testing:

```rust
#[cfg(test)]
mod raw_sse_testing {
    use server_core::{fwd_sse, test_utils::ResponseTestExt};
    use axum::{body::Body, http::{Request, StatusCode}, routing::get, Router};
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_proxy_sse_handler() -> anyhow::Result<()> {
        let app = Router::new().route(
            "/sse",
            get(|| async {
                let (tx, rx) = mpsc::channel::<String>(100);
                tokio::spawn(async move {
                    for i in 1..=3 {
                        tx.send(format!("data: message {}\n\n", i)).await.unwrap();
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                });
                fwd_sse(rx)
            }),
        );

        let request = Request::builder().uri("/sse").body(Body::empty())?;
        let response = app.oneshot(request).await?;

        assert_eq!(StatusCode::OK, response.status());
        assert_eq!("text/event-stream", response.headers()["content-type"]);
        let response = response.direct_sse().await?;
        assert_eq!(6, response.len());
        assert_eq!(
            vec![
                "data: message 1".to_string(),
                "".to_string(),
                "data: message 2".to_string(),
                "".to_string(),
                "data: message 3".to_string(),
                "".to_string(),
            ],
            response
        );
        Ok(())
    }
}
```

## Cross-Service HTTP Testing Integration

### Services Integration HTTP Testing
HTTP infrastructure testing with comprehensive service coordination:

```rust
#[rstest]
#[awt]
async fn test_http_infrastructure_service_integration(
    setup_l10n: (),
    temp_bodhi_home: TempDir,
    #[future] app_service_stub: AppServiceStub
) {
    // Configure service integration for HTTP testing
    let app_service = app_service_stub.await;
    let data_service = app_service.data_service();
    
    // Create test alias for HTTP request processing
    let alias = AliasBuilder::default()
        .alias("testalias:instruct".to_string())
        .repo(Repo::testalias())
        .filename("model.gguf".to_string())
        .snapshot("main".to_string())
        .source(AliasSource::Model)
        .build()
        .unwrap();
    
    data_service.save_alias(&alias).unwrap();
    
    // Configure SharedContext with service integration
    let mut mock_context = MockSharedContext::new();
    mock_context
        .expect_chat_completions()
        .returning(|_req, _alias| {
            Ok(create_mock_streaming_response())
        })
        .times(1);
    
    let router_state = DefaultRouterState::new(
        Arc::new(mock_context),
        Arc::new(app_service),
    );
    
    // Test complete HTTP request processing with service integration
    let request = CreateChatCompletionRequest {
        model: "testalias:instruct".to_string(),
        messages: vec![],
        ..Default::default()
    };
    
    let result = router_state.chat_completions(request).await;
    assert!(result.is_ok());
    
    // Validate service integration in HTTP response
    let response = result.unwrap();
    assert_eq!(response.status(), 200);
}
```

### HTTP Error Testing with Service Coordination
Comprehensive HTTP error handling testing with service error scenarios:

```rust
#[rstest]
#[awt]
async fn test_http_error_service_coordination(
    setup_l10n: (),
    #[future] app_service_stub: AppServiceStub
) {
    let app_service = app_service_stub.await;
    let data_service = app_service.data_service();
    
    // Configure service error scenario
    let mut mock_data_service = MockDataService::new();
    mock_data_service
        .expect_load_alias()
        .with(eq("nonexistent:alias"))
        .returning(|alias| {
            Err(AliasNotFoundError::new(alias.to_string()))
        })
        .times(1);
    
    let mut mock_context = MockSharedContext::new();
    let router_state = DefaultRouterState::new(
        Arc::new(mock_context),
        Arc::new(app_service),
    );
    
    // Test HTTP error handling with service error propagation
    let request = CreateChatCompletionRequest {
        model: "nonexistent:alias".to_string(),
        messages: vec![],
        ..Default::default()
    };
    
    let result = router_state.chat_completions(request).await;
    assert!(result.is_err());
    
    // Validate HTTP error translation
    match result.unwrap_err() {
        RouterStateError::AliasNotFound(alias_err) => {
            assert_eq!(alias_err.alias, "nonexistent:alias");
            
            // Validate error converts to proper HTTP response
            let api_error = objs::ApiError::from(alias_err);
            assert_eq!(api_error.error_type, objs::ErrorType::NotFound);
        }
        _ => panic!("Expected AliasNotFound error"),
    }
}
```

## Extension Guidelines for HTTP Infrastructure Testing

### Adding New HTTP Infrastructure Tests
When creating tests for new HTTP infrastructure functionality:

1. **RouterState Integration**: Design HTTP tests with proper dependency injection and service coordination
2. **SSE Testing Patterns**: Create comprehensive streaming tests for both DirectSSE and ForwardedSSE scenarios
3. **Context Management Testing**: Implement SharedContext tests with lifecycle management and state synchronization
4. **Service Mock Coordination**: Coordinate HTTP tests with services test utilities for realistic scenarios
5. **Error Scenario Testing**: Test comprehensive HTTP error handling with service error translation

### HTTP Testing Infrastructure Extensions
When extending HTTP testing capabilities:

1. **Mock Coordination**: Extend MockSharedContext and ServerFactoryStub for new HTTP testing scenarios
2. **Streaming Test Patterns**: Create reusable SSE testing patterns for different streaming scenarios
3. **Service Integration**: Coordinate HTTP testing with services and LLM server process testing infrastructure
4. **Performance Testing**: Add HTTP infrastructure performance testing under realistic load conditions
5. **Integration Testing**: Create end-to-end HTTP testing patterns across infrastructure and route layers

### Cross-Infrastructure HTTP Testing Coordination
HTTP infrastructure testing must coordinate across multiple systems:

1. **Service Testing Integration**: Coordinate HTTP tests with services test utilities for realistic business logic scenarios
2. **Process Testing Integration**: Coordinate HTTP context tests with LLM server process testing infrastructure
3. **Route Testing Support**: Provide HTTP infrastructure testing foundation for route testing scenarios
4. **Error Testing Coordination**: Coordinate HTTP error testing across service boundaries and infrastructure layers
5. **Performance Testing Integration**: Coordinate HTTP infrastructure performance testing with service and process testing

## Commands for HTTP Infrastructure Testing

**HTTP Infrastructure Tests**: `cargo test -p server_core --features test-utils` (includes RouterState and SharedContext testing)  
**SSE Streaming Tests**: `cargo test -p server_core sse` (includes DirectSSE and ForwardedSSE validation)  
**Context Integration Tests**: `cargo test -p server_core context` (includes LLM server context coordination testing)