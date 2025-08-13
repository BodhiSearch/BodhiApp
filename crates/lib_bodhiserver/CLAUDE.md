# CLAUDE.md - lib_bodhiserver

This file provides guidance to Claude Code when working with the `lib_bodhiserver` crate, which provides an embeddable server library for BodhiApp.

## Purpose

The `lib_bodhiserver` crate provides a library interface for embedding BodhiApp server functionality:

- **Embeddable Server**: Library API for integrating BodhiApp into other applications
- **Programmatic Control**: Start, stop, and manage server instances programmatically
- **Custom Configuration**: Flexible configuration options for embedded scenarios
- **Resource Management**: Proper lifecycle management for embedded deployments
- **API Access**: Direct access to server functionality without HTTP overhead

## Key Components

### Server Library Interface
- `BodhiServer` - Main server instance with lifecycle management
- Configuration builders for customizing server behavior
- Service access methods for direct API interaction
- Resource management and cleanup handling

### Service Exposure
- Direct access to application services without HTTP layer
- Programmatic model management and configuration
- Authentication service integration
- Database and storage management

### Lifecycle Management
- Server initialization with custom configuration
- Graceful startup and shutdown procedures
- Resource cleanup and memory management
- Error handling and recovery

## Dependencies

### Core Server Components
- `server_app` - Main server implementation
- `services` - Business logic services
- `routes_all` - HTTP routing (optional for HTTP mode)
- `auth_middleware` - Authentication services

### Infrastructure
- `tokio` - Async runtime management
- `axum` - HTTP server (when HTTP interface is enabled)
- Database and storage backends

## Architecture Position

The `lib_bodhiserver` crate sits at the library interface layer:
- **Abstracts**: Complex server initialization and management
- **Provides**: Clean programmatic API for embedding
- **Manages**: Server lifecycle and resource allocation
- **Exposes**: Core functionality without HTTP overhead

## Usage Patterns

### Basic Server Embedding
```rust
use lib_bodhiserver::{BodhiServer, BodhiServerBuilder};

let server = BodhiServerBuilder::new()
    .database_url("sqlite:///path/to/db.sqlite")
    .data_dir("/path/to/data")
    .build()
    .await?;

// Start the server
server.start().await?;

// Use server functionality
let models = server.list_models().await?;
let response = server.chat_completion(request).await?;

// Shutdown when done
server.shutdown().await?;
```

### Custom Configuration
```rust
let config = BodhiServerConfig {
    database_url: "sqlite:memory:".to_string(),
    enable_http: false,  // Disable HTTP interface
    log_level: "debug".to_string(),
    cache_size: 1000,
    // ... other configuration
};

let server = BodhiServer::with_config(config).await?;
```

### Service Access
```rust
// Direct service access without HTTP overhead
let data_service = server.data_service();
let auth_service = server.auth_service();
let hub_service = server.hub_service();

// Use services directly
let alias = data_service.find_alias("my-model")?;
let models = hub_service.list_models().await?;
```

### HTTP Mode (Optional)
```rust
let server = BodhiServerBuilder::new()
    .enable_http(true)
    .bind_address("127.0.0.1:8080")
    .build()
    .await?;

server.start().await?;
// Server now accepts HTTP requests on specified address
```

## Configuration Options

### Database Configuration
- SQLite database path or in-memory database
- Connection pool settings
- Migration management options

### Storage Configuration  
- Data directory for models and files
- Cache directory and size limits
- Temporary file handling

### Authentication Configuration
- OAuth provider settings (optional)
- JWT signing configuration
- Session management options

### Performance Settings
- Cache size and eviction policies
- Connection pool limits
- Async runtime configuration

## Service Interface

### Model Management
```rust
// List available models
let models = server.list_models().await?;

// Create new model alias
let create_request = CreateModelRequest { /* ... */ };
server.create_model(create_request).await?;

// Pull model from repository
let pull_request = PullModelRequest { /* ... */ };
server.pull_model(pull_request).await?;
```

### Chat Interface
```rust
use async_openai::types::CreateChatCompletionRequest;

let request = CreateChatCompletionRequest {
    model: "my-model".to_string(),
    messages: vec![/* ... */],
    // ... other parameters
};

let response = server.chat_completion(request).await?;
```

### Authentication (when enabled)
```rust
// User authentication
let login_url = server.initiate_login().await?;
let tokens = server.complete_login(callback_params).await?;

// Token management
let api_token = server.create_api_token(token_request).await?;
```

## Integration Scenarios

### Desktop Application Integration
```rust
// Embedded in Tauri or similar desktop framework
let server = BodhiServer::new()
    .database_url(&app_data_dir.join("bodhi.db").to_string_lossy())
    .data_dir(&app_data_dir)
    .enable_http(false)  // Use direct API
    .build()
    .await?;
```

### Web Application Backend
```rust
// Embedded as backend service
let server = BodhiServer::new()
    .enable_http(true)
    .bind_address("0.0.0.0:8080")
    .build()
    .await?;

// Expose HTTP API to frontend
server.start().await?;
```

### Plugin or Extension
```rust
// Embedded in larger application as plugin
pub struct BodhiPlugin {
    server: BodhiServer,
}

impl Plugin for BodhiPlugin {
    async fn initialize(&mut self) -> Result<()> {
        self.server.start().await
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        self.server.shutdown().await
    }
}
```

## Error Handling

### Initialization Errors
- Configuration validation failures
- Database connection errors
- File system permission issues
- Port binding conflicts (HTTP mode)

### Runtime Errors
- Service operation failures
- Resource exhaustion
- Network connectivity issues
- Authentication failures

### Graceful Error Recovery
- Automatic retry for transient failures
- Graceful degradation for non-critical errors
- Proper error propagation to embedding application

## Resource Management

### Memory Management
- Efficient resource allocation and cleanup
- Cache size management with bounds
- Connection pool limits and cleanup

### File System Management
- Temporary file cleanup
- Database file management
- Model file organization

### Network Resources (HTTP mode)
- Port binding and release
- Connection management
- Request/response resource cleanup

## Performance Considerations

### Async Runtime
- Efficient task scheduling
- Non-blocking I/O operations
- Proper resource sharing

### Caching Strategy
- In-memory caching with LRU eviction
- Persistent caching for model metadata
- Query result caching

### Database Optimization
- Connection pooling
- Prepared statement caching
- Index optimization

## Development Guidelines

### Adding New Library Features
1. Define public API with appropriate error handling
2. Implement feature with proper resource management
3. Add configuration options as needed
4. Include comprehensive documentation and examples
5. Add unit and integration tests

### Configuration Management
- Use builder pattern for complex configuration
- Provide sensible defaults for all options
- Validate configuration during construction
- Support both programmatic and file-based configuration

### Error Handling Best Practices
- Use typed errors with clear descriptions
- Provide context for debugging
- Handle resource cleanup in error paths
- Document error conditions and recovery strategies

## Testing Strategy

### Unit Testing
- Individual service functionality
- Configuration validation
- Error handling paths
- Resource management

### Integration Testing
- Complete server lifecycle
- Service interaction testing
- Database integration
- Error recovery scenarios

### Embedding Testing
- Test integration in various scenarios
- Memory leak detection
- Performance benchmarking
- Concurrent usage testing

## Security Considerations

### Embedded Security
- Secure default configuration
- Input validation for all APIs
- Resource access controls
- Audit logging capabilities

### Data Protection
- Secure database connections
- Encrypted storage options
- Memory-safe operations
- Secret management

## Future Extensions

The lib_bodhiserver crate can be extended with:
- Plugin system for extensibility
- Event system for notifications
- Streaming API for real-time updates
- Multi-tenancy support
- Advanced monitoring and metrics
- Configuration hot-reloading