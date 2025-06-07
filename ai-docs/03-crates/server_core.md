# server_core - HTTP Server Foundation

## Overview

The `server_core` crate provides the foundational HTTP server infrastructure for BodhiApp. It implements core server functionality including state management, Server-Sent Events (SSE) handling, and shared utilities that are used by all route modules.

## Purpose

- **Server Foundation**: Core HTTP server infrastructure and utilities
- **State Management**: Shared application state and dependency injection
- **SSE Support**: Server-Sent Events for real-time communication
- **Middleware Foundation**: Base functionality for HTTP middleware
- **Testing Infrastructure**: Server testing utilities and helpers

## Key Components

### State Management

#### Router State (`router_state.rs`)
- Application state container for dependency injection
- Service registration and retrieval
- Request-scoped state management
- Thread-safe state sharing across handlers

The router state provides a centralized way to manage application dependencies and share them across HTTP handlers through Axum's extension system.

#### Shared Read-Write (`shared_rw.rs`)
- Thread-safe shared data structures
- Read-write lock management
- Concurrent access patterns
- State synchronization utilities

### Server-Sent Events (SSE)

#### Direct SSE (`direct_sse.rs`)
- Direct Server-Sent Events implementation
- Real-time data streaming to clients
- Event formatting and serialization
- Connection management and cleanup

#### Forward SSE (`fwd_sse.rs`)
- SSE forwarding and proxying
- Event stream transformation
- Multi-client broadcasting
- Stream aggregation and filtering

SSE support is crucial for real-time features like:
- Live chat message streaming
- Model download progress updates
- System status notifications
- Real-time log streaming

### Error Handling

#### Server Errors (`error.rs`)
- HTTP-specific error types
- Error response formatting
- Status code mapping
- Error middleware integration

## Directory Structure

```
src/
├── lib.rs                    # Main module exports
├── router_state.rs           # Application state management
├── shared_rw.rs              # Thread-safe shared data
├── direct_sse.rs             # Direct SSE implementation
├── fwd_sse.rs                # SSE forwarding utilities
├── error.rs                  # HTTP error handling
├── resources/                # Localization resources
│   └── en-US/
└── test_utils/               # Testing utilities
    ├── mod.rs
    ├── http.rs               # HTTP testing helpers
    ├── server.rs             # Server testing utilities
    └── state.rs              # State testing helpers
```

## Key Features

### Dependency Injection
The router state system enables clean dependency injection:

```rust
// Service registration
let state = RouterState::new()
    .with_service(app_service)
    .with_service(auth_service);

// Service retrieval in handlers
async fn handler(
    Extension(app_service): Extension<Arc<dyn AppService>>,
) -> Result<Response, ApiError> {
    // Use injected service
}
```

### Real-Time Communication
SSE support enables real-time features:

```rust
// Direct SSE streaming
async fn stream_handler() -> Sse<impl Stream<Item = Event>> {
    let stream = create_event_stream();
    Sse::new(stream)
}

// SSE forwarding
async fn forward_handler() -> Sse<impl Stream<Item = Event>> {
    let forwarded_stream = forward_sse_stream(source_stream);
    Sse::new(forwarded_stream)
}
```

### Thread-Safe State
Shared state management for concurrent access:

```rust
// Shared read-write data
let shared_data = SharedRw::new(initial_data);

// Concurrent read access
let read_guard = shared_data.read().await;

// Exclusive write access
let mut write_guard = shared_data.write().await;
```

## Dependencies

### Core Dependencies
- **objs**: Domain objects and error types
- **axum**: HTTP server framework
- **tokio**: Async runtime
- **tower**: Middleware and service abstractions

### SSE and Streaming
- **futures**: Stream utilities
- **tokio-stream**: Async stream support
- **serde**: Event serialization

### Concurrency
- **tokio**: Async synchronization primitives
- **parking_lot**: High-performance locks (if used)

## Usage Patterns

### State Injection
Services are injected into route handlers through the router state:

```rust
let app = Router::new()
    .route("/api/endpoint", get(handler))
    .with_state(router_state);

async fn handler(
    Extension(service): Extension<Arc<dyn SomeService>>,
) -> Result<Json<Response>, ApiError> {
    // Use service
}
```

### SSE Streaming
Real-time data streaming using SSE:

```rust
async fn events_handler() -> Sse<impl Stream<Item = Event>> {
    let (tx, rx) = mpsc::channel(100);
    
    // Spawn background task to send events
    tokio::spawn(async move {
        // Send events to stream
    });
    
    let stream = ReceiverStream::new(rx)
        .map(|data| Event::default().data(data));
    
    Sse::new(stream)
}
```

### Error Handling
Consistent error handling across the server:

```rust
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(ErrorResponse::from(self));
        (status, body).into_response()
    }
}
```

## Testing Support

The server_core crate provides comprehensive testing utilities:

### HTTP Testing (`test_utils/http.rs`)
- HTTP client helpers
- Request/response testing utilities
- Mock server setup

### Server Testing (`test_utils/server.rs`)
- Test server creation
- Integration test helpers
- Server lifecycle management

### State Testing (`test_utils/state.rs`)
- Mock state creation
- Service injection testing
- State validation utilities

## Integration Points

- **Route Modules**: All route crates use server_core for state management
- **Middleware**: Authentication and other middleware build on server_core
- **Services**: Services are injected through the router state system
- **Frontend**: SSE endpoints provide real-time updates to the frontend

## Performance Considerations

### Async/Await
- Full async/await support for non-blocking operations
- Efficient handling of concurrent requests
- Proper resource cleanup and cancellation

### Memory Management
- Efficient state sharing without unnecessary cloning
- Proper cleanup of SSE connections
- Memory-efficient event streaming

### Scalability
- Thread-safe state management for multi-threaded servers
- Efficient SSE connection management
- Proper resource pooling and reuse

## Future Extensions

The server_core foundation is designed to support:
- WebSocket integration
- Additional middleware types
- Enhanced monitoring and metrics
- Custom authentication schemes
- Advanced routing patterns
