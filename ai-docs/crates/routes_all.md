# routes_all - Unified Route Composition

## Overview

The `routes_all` crate serves as the central composition layer that combines all HTTP routes from different route modules into a unified router. It provides the complete API surface for BodhiApp, including OpenAI-compatible endpoints, application-specific APIs, and additional utility routes.

## Purpose

- **Route Composition**: Combines all route modules into a single router
- **API Unification**: Provides a unified API surface for the entire application
- **Middleware Integration**: Applies cross-cutting concerns across all routes
- **Documentation Aggregation**: Combines OpenAPI documentation from all modules
- **Proxy Support**: Handles request proxying and forwarding

## Key Components

### Route Composition (`routes.rs`)
- **Router Assembly**: Combines routes from all route crates
- **Middleware Application**: Applies authentication, CORS, and other middleware
- **Path Organization**: Organizes routes under appropriate path prefixes
- **Service Integration**: Integrates with the service layer through dependency injection

### Proxy Routes (`routes_proxy.rs`)
- **Request Proxying**: Forwards requests to external services
- **Load Balancing**: Distributes requests across multiple backends
- **Fallback Handling**: Provides fallback mechanisms for service failures
- **Request Transformation**: Modifies requests/responses during proxying

## Directory Structure

```
src/
├── lib.rs                    # Main module exports
├── routes.rs                 # Main route composition
├── routes_proxy.rs           # Proxy and forwarding routes
├── resources/                # Localization resources
│   └── en-US/
└── test_utils/               # Testing utilities
    └── mod.rs
```

## Route Organization

### API Structure
The unified router organizes routes into logical groups:

```
/
├── /v1/                      # OpenAI-compatible API (routes_oai)
│   ├── /chat/completions
│   ├── /models
│   └── /embeddings
├── /api/                     # Application API (routes_app)
│   ├── /auth/
│   ├── /models/
│   ├── /settings/
│   ├── /tokens/
│   └── /pull/
├── /docs/                    # API documentation
│   ├── /swagger-ui/
│   └── /openapi.json
└── /health                   # Health check endpoints
```

### Route Composition Pattern
```rust
pub fn create_all_routes(state: RouterState) -> Router {
    Router::new()
        // OpenAI-compatible routes
        .nest("/v1", routes_oai::create_routes())
        // Application-specific routes
        .nest("/api", routes_app::create_routes())
        // Documentation routes
        .nest("/docs", create_docs_routes())
        // Health and utility routes
        .merge(create_utility_routes())
        // Apply global middleware
        .layer(cors_layer())
        .layer(tracing_layer())
        .with_state(state)
}
```

## Key Features

### Unified API Surface
- **Single Entry Point**: All APIs accessible through one router
- **Consistent Middleware**: Uniform middleware application
- **Centralized Configuration**: Single point for route configuration
- **Integrated Documentation**: Combined API documentation

### Cross-Cutting Concerns
- **CORS Handling**: Cross-origin request support
- **Request Tracing**: Distributed tracing across all routes
- **Error Handling**: Consistent error handling across APIs
- **Rate Limiting**: Unified rate limiting policies

### Documentation Integration
- **OpenAPI Aggregation**: Combines OpenAPI specs from all modules
- **Swagger UI**: Interactive API documentation
- **Type Generation**: TypeScript types for frontend
- **API Versioning**: Version management across all APIs

### Proxy Capabilities
- **Service Proxying**: Forward requests to external services
- **Request Transformation**: Modify requests/responses in transit
- **Load Balancing**: Distribute load across multiple backends
- **Circuit Breaking**: Handle service failures gracefully

## Middleware Stack

### Global Middleware
Applied to all routes in order:

1. **Request Tracing**: Distributed tracing and logging
2. **CORS**: Cross-origin resource sharing
3. **Compression**: Response compression (gzip, brotli)
4. **Rate Limiting**: Global rate limiting
5. **Security Headers**: Security-related HTTP headers

### Route-Specific Middleware
Applied to specific route groups:

1. **Authentication**: Applied to protected routes
2. **Authorization**: Role-based access control
3. **Validation**: Request/response validation
4. **Caching**: Response caching for appropriate endpoints

## Dependencies

### Core Dependencies
- **routes_oai**: OpenAI-compatible API routes
- **routes_app**: Application-specific API routes
- **server_core**: HTTP server infrastructure
- **auth_middleware**: Authentication and authorization

### HTTP and Middleware
- **axum**: HTTP framework and routing
- **tower**: Middleware abstractions
- **tower-http**: HTTP-specific middleware (CORS, compression, tracing)
- **hyper**: HTTP implementation

### Documentation
- **utoipa**: OpenAPI documentation generation
- **utoipa-swagger-ui**: Swagger UI integration

## Configuration

### Route Configuration
Routes can be configured through environment variables and settings:

```rust
pub struct RouteConfig {
    pub enable_swagger_ui: bool,
    pub cors_origins: Vec<String>,
    pub rate_limit_requests_per_minute: u32,
    pub enable_compression: bool,
}
```

### Middleware Configuration
Middleware behavior is configurable:

```rust
pub struct MiddlewareConfig {
    pub tracing_level: TracingLevel,
    pub cors_max_age: Duration,
    pub compression_level: CompressionLevel,
    pub rate_limit_burst: u32,
}
```

## Health Checks

### Health Endpoints
- **GET /health**: Basic health check
- **GET /health/ready**: Readiness probe
- **GET /health/live**: Liveness probe
- **GET /health/detailed**: Detailed health information

### Health Check Components
- **Database Connectivity**: Database connection health
- **Service Dependencies**: External service health
- **Resource Usage**: Memory and CPU usage
- **Model Availability**: LLM model status

## Error Handling

### Global Error Handler
Provides consistent error handling across all routes:

```rust
async fn global_error_handler(
    error: BoxError,
) -> Result<Response, Infallible> {
    let response = match error.downcast::<ApiError>() {
        Ok(api_error) => api_error.into_response(),
        Err(other_error) => {
            // Handle unexpected errors
            InternalServerError::new().into_response()
        }
    };
    Ok(response)
}
```

### Error Response Format
Consistent error format across all APIs:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {
      "field": "specific_field",
      "reason": "validation_failed"
    },
    "request_id": "req_123456789"
  }
}
```

## Testing Support

### Integration Testing
- **Full Router Testing**: Test complete router composition
- **Middleware Testing**: Test middleware integration
- **Cross-Route Testing**: Test interactions between different APIs
- **Documentation Testing**: Validate OpenAPI documentation

### Test Utilities
```rust
pub fn create_test_router() -> Router {
    create_all_routes(test_router_state())
}

pub async fn test_request(
    router: &Router,
    request: Request,
) -> Response {
    // Test helper implementation
}
```

## Performance Considerations

### Request Routing
- **Efficient Routing**: Optimized route matching
- **Middleware Ordering**: Optimal middleware execution order
- **Connection Pooling**: Efficient connection reuse
- **Resource Sharing**: Shared resources across routes

### Caching Strategy
- **Response Caching**: Cache appropriate responses
- **Route Compilation**: Pre-compiled route patterns
- **Middleware Caching**: Cache middleware results where possible

## Monitoring and Observability

### Metrics Collection
- **Request Metrics**: Request count, duration, status codes
- **Route-Specific Metrics**: Per-route performance metrics
- **Error Metrics**: Error rates and types
- **Resource Metrics**: Memory and CPU usage

### Distributed Tracing
- **Request Tracing**: Trace requests across all routes
- **Service Correlation**: Correlate requests across services
- **Performance Profiling**: Identify performance bottlenecks

## Integration Points

- **Frontend**: Serves as the primary API for the web interface
- **External Clients**: Provides OpenAI-compatible API for external tools
- **Monitoring**: Integrates with monitoring and alerting systems
- **Load Balancers**: Works with external load balancers and proxies

## Future Extensions

The routes_all crate is designed to support:
- **API Versioning**: Multiple API versions simultaneously
- **GraphQL Integration**: GraphQL endpoint alongside REST APIs
- **WebSocket Support**: Real-time WebSocket connections
- **gRPC Integration**: gRPC endpoints for high-performance clients
- **Plugin System**: Dynamic route registration for plugins
