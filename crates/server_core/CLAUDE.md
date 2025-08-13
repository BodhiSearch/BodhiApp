# CLAUDE.md - server_core

This file provides guidance to Claude Code when working with the `server_core` crate, which provides HTTP server infrastructure and streaming capabilities for BodhiApp.

## Purpose

The `server_core` crate provides foundational HTTP server infrastructure:

- **Router State Management**: Centralized state container for HTTP route handlers
- **Server-Sent Events (SSE)**: Real-time streaming for chat completions and responses
- **Context Management**: Shared context for managing LLM server instances and connections  
- **Request Proxying**: Efficient forwarding of requests to underlying LLM services
- **Error Handling**: Comprehensive error management for server operations
- **Async Operations**: High-performance asynchronous HTTP and streaming operations

## Key Components

### Router State (`src/router_state.rs`)
- `RouterState` trait - Interface for accessing application services from HTTP handlers
- `DefaultRouterState` - Concrete implementation managing shared context and services
- Service access methods for dependency injection in route handlers
- Chat completion orchestration with model alias resolution

### Server-Sent Events Implementation

#### Direct SSE (`src/direct_sse.rs`)
- `DirectSse` - Custom SSE implementation for direct event streaming
- `DirectEvent` - Event formatting with data payload support
- Keep-alive mechanisms for connection stability
- Axum integration for HTTP response streaming

#### Forwarded SSE (`src/fwd_sse.rs`)
- `ForwardedSse` - Proxy SSE streams from external services
- Efficient request forwarding with streaming response handling
- Connection management and error propagation

### Shared Context (`src/shared_rw.rs`)
- `SharedContext` trait - Interface for managing LLM server instances
- Context lifecycle management (start/stop operations)
- Request routing to appropriate server instances
- Resource management and cleanup

### Error Handling (`src/error.rs`)
- `ContextError` - Context management and server operation errors
- `RouterStateError` - Route handler and state management errors
- Integration with application-wide error handling system

## Dependencies

### Core Infrastructure
- `objs` - Domain objects, validation, and error handling
- `services` - Business logic services and application state
- `llama_server_proc` - LLM server process management

### HTTP and Networking
- `axum` - Modern async web framework with extractors and handlers
- `reqwest` - HTTP client for proxying requests to LLM servers
- `tokio` - Async runtime for concurrent operations
- `futures` - Stream processing and async utilities

### Streaming and Data Handling
- `bytes` - Efficient byte buffer management
- `tokio-stream` - Async stream utilities
- `http-body` - HTTP body trait implementations
- `pin-project-lite` - Safe pin projection for async types

### Development and Testing
- `mockall` - Mock object generation for testing
- `rstest` - Parameterized testing framework

## Architecture Position

The `server_core` crate sits at the HTTP infrastructure layer:
- **Above**: Services and business logic layer
- **Below**: Route implementations and HTTP handlers
- **Coordinates**: Request routing, streaming, and context management
- **Provides**: Foundation for building HTTP APIs with real-time streaming

## Usage Patterns

### Router State Setup and Dependency Injection
```rust
use server_core::{DefaultRouterState, RouterState, SharedContext};
use services::AppService;

let router_state = DefaultRouterState::new(
    shared_context,
    app_service,
);

// Access services from route handlers
let app_service = router_state.app_service();
let data_service = app_service.data_service();
```

### Chat Completion Processing
```rust
use server_core::RouterState;
use async_openai::types::CreateChatCompletionRequest;

async fn chat_handler(
    state: Arc<dyn RouterState>,
    request: CreateChatCompletionRequest,
) -> Result<reqwest::Response, RouterStateError> {
    // Router state handles alias resolution and request forwarding
    let response = state.chat_completions(request).await?;
    Ok(response)
}
```

### Server-Sent Events Streaming
```rust
use server_core::{DirectSse, DirectEvent};
use tokio_stream::wrappers::UnboundedReceiverStream;

async fn sse_handler() -> DirectSse<impl Stream<Item = Result<DirectEvent>>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    
    // Send events asynchronously
    tokio::spawn(async move {
        for i in 0..10 {
            let event = DirectEvent::new()
                .data(format!("Event {}", i));
            tx.send(Ok(event)).unwrap();
        }
    });

    DirectSse::new(UnboundedReceiverStream::new(rx))
        .keep_alive(KeepAlive::default())
}
```

### Context Management
```rust
use server_core::SharedContext;

// Start context and associated resources
context.start().await?;

// Process requests through context
let response = context.chat_completions(request, alias).await?;

// Clean shutdown
context.stop().await?;
```

### Error Handling in Routes
```rust
use server_core::{RouterStateError, ContextError};

async fn route_handler() -> Result<impl IntoResponse, RouterStateError> {
    match operation().await {
        Err(RouterStateError::AliasNotFound(alias)) => {
            // Handle specific error types with appropriate HTTP responses
            Err(RouterStateError::AliasNotFound(alias))
        }
        Err(RouterStateError::ContextError(ctx_err)) => {
            // Context errors map to internal server errors
            Err(RouterStateError::ContextError(ctx_err))
        }
        Ok(result) => Ok(result),
    }
}
```

## Integration Points

### With HTTP Routes Layer
- Router state injected into route handlers via Axum's state system
- Error types automatically convert to appropriate HTTP status codes
- Streaming responses integrate with Axum's response system

### With Services Layer
- App service provides access to all business logic services
- Data service used for model alias resolution
- Hub service integration for model management

### With LLM Processing Layer
- Shared context manages LLM server instances
- Request forwarding to llama.cpp servers
- Response streaming from LLM inference

## Streaming Architecture

### Direct Streaming
- Custom SSE implementation for application-generated events
- Efficient memory usage with BytesMut buffers
- Keep-alive support for long-lived connections

### Forwarded Streaming  
- Proxy streaming from external services
- Maintains request/response streaming semantics
- Error handling during stream processing

### Connection Management
- Automatic cleanup of resources on client disconnect
- Proper error propagation through stream chains
- Keep-alive mechanisms for connection stability

## Performance Considerations

### Async Operations
- All operations are fully asynchronous for high concurrency
- Non-blocking I/O for HTTP requests and responses
- Efficient stream processing with minimal buffering

### Memory Management
- Zero-copy operations where possible
- Efficient byte buffer reuse with BytesMut
- Streaming responses to minimize memory usage

### Connection Pooling
- HTTP client connection reuse via reqwest
- Shared context for resource pooling
- Proper resource cleanup on shutdown

## Error Handling Strategy

### Context Errors
- Server startup and shutdown failures
- LLM server communication errors
- Resource management failures

### Router State Errors
- Model alias resolution failures
- Service unavailability errors
- Request validation failures

### Streaming Errors
- Connection interruption handling
- Partial response recovery
- Client disconnect cleanup

## Development Guidelines

### Adding New Streaming Endpoints
1. Implement stream source (async iterator or channel)
2. Use appropriate SSE type (Direct vs Forwarded)
3. Handle connection lifecycle and cleanup
4. Add proper error handling and recovery

### Context Management
- Implement SharedContext trait for new resource types
- Ensure proper async lifecycle management
- Add comprehensive error handling
- Test startup/shutdown sequences

### Testing Strategy
- Mock SharedContext for isolated testing
- Test streaming with simulated network conditions
- Validate error propagation through all layers
- Test concurrent request handling

### Performance Optimization
- Profile streaming performance under load
- Optimize buffer sizes for typical payloads
- Monitor connection pool utilization
- Test memory usage patterns

## Security Considerations

### Request Validation
- Validate all incoming requests before processing
- Sanitize data before forwarding to LLM servers
- Implement rate limiting and abuse prevention

### Connection Security
- Secure handling of streaming connections
- Proper cleanup to prevent resource leaks
- Validate client disconnect scenarios

### Error Information
- Avoid exposing internal system details in error messages
- Log sensitive information securely
- Implement proper error boundaries

## Future Extensions

The server_core crate is designed for extensibility:
- Additional streaming protocols (WebSockets, gRPC streaming)
- Enhanced connection management and load balancing
- Metrics collection and monitoring integration
- Advanced caching and request optimization