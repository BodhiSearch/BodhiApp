# CLAUDE.md - routes_all

This file provides guidance to Claude Code when working with the `routes_all` crate, which composes and unifies all HTTP routes for BodhiApp.

## Purpose

The `routes_all` crate serves as the central router composition point:

- **Route Composition**: Combines OpenAI API routes and application routes
- **Middleware Integration**: Applies authentication, logging, and security middleware
- **Router Configuration**: Configures complete HTTP routing for the application
- **OpenAPI Integration**: Provides unified OpenAPI documentation
- **State Management**: Manages shared state across all route handlers

## Key Components

### Router Composition
- Combines `routes_oai` (OpenAI compatibility) and `routes_app` (application-specific) routes
- Applies appropriate middleware layers for authentication and security
- Configures route-specific middleware and error handling
- Manages shared router state injection

### Middleware Stack
- Authentication middleware for protected endpoints
- API authentication for bearer token validation
- Session management for web interface
- Logging and tracing middleware
- Error handling and response formatting

### OpenAPI Documentation
- Unified OpenAPI specification combining all endpoint documentation
- Interactive documentation interface
- Schema definitions and examples
- Authentication flow documentation

## Dependencies

### Route Implementations
- `routes_oai` - OpenAI-compatible API endpoints
- `routes_app` - Application-specific endpoints
- `auth_middleware` - Authentication and authorization middleware
- `server_core` - HTTP server infrastructure and state management

### Core Infrastructure
- `objs` - Domain objects and error handling
- `services` - Business logic services

## Architecture Position

The `routes_all` crate sits at the HTTP composition layer:
- **Coordinates**: All HTTP routing and middleware application
- **Integrates**: OpenAI API routes, application routes, and authentication
- **Provides**: Complete HTTP service configuration
- **Manages**: Request routing, middleware stacks, and response handling

## Usage Patterns

### Router Creation
```rust
use routes_all::create_router;
use server_core::RouterState;

let router_state = // ... initialize router state
let app = create_router(router_state)
    .await
    .expect("Failed to create router");

// Router is ready to serve HTTP requests
```

### Route Organization
```rust
// OpenAI API routes (protected with API auth)
Router::new()
    .route("/v1/chat/completions", post(create_chat_completion))
    .route("/v1/models", get(list_models))
    .route_layer(axum::middleware::from_fn_with_state(
        state.clone(),
        api_auth_middleware,
    ))

// Application routes (protected with session auth)
Router::new()
    .route("/app/models", get(app_list_models))
    .route("/app/auth/login", get(login))
    .route_layer(axum::middleware::from_fn_with_state(
        state.clone(),
        auth_middleware,
    ))
```

### Middleware Application
```rust
let protected_routes = Router::new()
    .merge(oai_routes)
    .merge(app_routes)
    .layer(TraceLayer::new_for_http())
    .layer(session_layer)
    .with_state(router_state);
```

## Route Categories

### OpenAI API Routes (`/v1/*`)
- `/v1/chat/completions` - Chat completion endpoint
- `/v1/models` - Model listing
- Protected with API bearer token authentication
- Full OpenAI API compatibility

### Application Routes (`/app/*`)
- `/app/models/*` - Model management
- `/app/auth/*` - Authentication and user management
- `/app/tokens/*` - API token management
- `/app/settings/*` - Application configuration
- Protected with session-based authentication

### Public Routes
- `/` - Application root (typically redirects to UI)
- `/health` - Health check endpoint
- `/docs` - OpenAPI documentation
- No authentication required

### Development Routes (`/dev/*`)
- Development and debugging endpoints
- Enabled only in development builds
- Administrative access required

## Authentication Strategy

### Dual Authentication System
1. **API Authentication**: Bearer tokens for programmatic access
   - Applied to `/v1/*` routes
   - JWT token validation
   - Scope-based authorization

2. **Session Authentication**: Web interface authentication
   - Applied to `/app/*` routes  
   - Session-based with secure cookies
   - OAuth2 integration

### Route Protection
```rust
// API routes with bearer token auth
let api_routes = Router::new()
    .merge(oai_routes)
    .route_layer(axum::middleware::from_fn_with_state(
        state.clone(),
        api_auth_middleware,
    ));

// App routes with session auth
let app_routes = Router::new()
    .merge(app_specific_routes)
    .route_layer(axum::middleware::from_fn_with_state(
        state.clone(),
        auth_middleware,
    ));
```

## Middleware Stack

### Applied Middleware (in order)
1. **Tracing Layer**: Request logging and distributed tracing
2. **Session Layer**: HTTP session management with secure storage
3. **Authentication**: Route-specific authentication middleware
4. **Error Handling**: Centralized error processing and response formatting
5. **State Injection**: Router state availability in handlers

### Error Handling
- Centralized error mapping to HTTP responses
- Consistent error response format across all endpoints
- Proper HTTP status codes for different error types
- Security-conscious error messages

## OpenAPI Documentation

### Unified Specification
- Combines documentation from all route crates
- Comprehensive endpoint documentation with examples
- Schema definitions for all request/response types
- Authentication flow documentation

### Documentation Features
- Interactive Swagger UI interface
- Request/response examples
- Schema validation
- Authentication requirements per endpoint

## Development Guidelines

### Adding New Route Groups
1. Import the new route crate as dependency
2. Add route group to router composition
3. Apply appropriate middleware layers
4. Update OpenAPI documentation integration
5. Add integration tests for new routes

### Middleware Configuration
- Apply middleware in correct order for proper functionality
- Use route-specific middleware for targeted protection
- Ensure proper error handling throughout middleware stack
- Test middleware interactions thoroughly

### State Management
- Ensure router state contains all required services
- Use Arc for shared state to enable cloning
- Validate state configuration during router creation
- Handle state initialization errors gracefully

## Performance Considerations

### Router Efficiency
- Efficient route matching with optimized routing table
- Minimal middleware overhead for high-frequency endpoints
- Connection pooling and reuse through shared state

### Middleware Performance
- Authentication middleware with caching for token validation
- Session middleware with optimized storage backend
- Tracing middleware with appropriate sampling rates

## Security Considerations

### Route Security
- All sensitive endpoints protected with appropriate authentication
- Public endpoints carefully reviewed for security implications
- Rate limiting on authentication and API endpoints

### Middleware Security
- Secure session configuration with appropriate flags
- JWT token validation with proper signature verification
- CORS configuration for cross-origin requests

## Testing Strategy

### Integration Testing
- End-to-end testing of complete routing configuration
- Authentication flow testing across different route types
- Middleware interaction testing
- Error handling validation

### Performance Testing
- Load testing of complete router configuration
- Authentication middleware performance validation
- Database connection pooling under load

## Monitoring and Observability

### Request Metrics
- Request count and latency by route and method
- Authentication success/failure rates
- Error rates by endpoint and error type
- Session management metrics

### Tracing Integration
- Distributed tracing across all requests
- Correlation IDs for request tracking
- Performance profiling and bottleneck identification

## Deployment Considerations

### Configuration
- Environment-specific middleware configuration
- Authentication provider configuration
- Database and cache connection settings
- Logging and tracing configuration

### Health Checks
- Router health validation
- Dependency service health checks
- Database connectivity validation
- Cache service availability

## Future Extensions

The routes_all crate is designed to accommodate:
- Additional API versions and compatibility layers
- Enhanced security middleware (rate limiting, DDoS protection)
- Metrics and monitoring endpoints
- WebSocket support for real-time features
- GraphQL endpoint integration