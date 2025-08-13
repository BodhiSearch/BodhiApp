# CLAUDE.md - server_app

This file provides guidance to Claude Code when working with the `server_app` crate, which provides the main HTTP server executable for BodhiApp.

## Purpose

The `server_app` crate implements the standalone HTTP server application:

- **HTTP Server**: Complete HTTP server implementation with Axum
- **Service Initialization**: Bootstrap all application services and dependencies
- **Configuration Management**: Load and validate application configuration
- **Graceful Shutdown**: Proper resource cleanup and shutdown handling
- **Production Deployment**: Optimized server setup for production environments

## Key Components

### Server Application
- Main server entry point with complete service initialization
- HTTP server configuration with proper middleware stack
- Database connection management and migrations
- Session storage setup and configuration

### Service Bootstrap
- Dependency injection container setup
- Service registration and configuration
- Database initialization and migration management
- Authentication and security service setup

### Configuration
- Environment-based configuration loading
- Database connection string management
- Authentication provider configuration
- Logging and tracing setup

## Dependencies

### Route Composition
- `routes_all` - Complete HTTP routing and middleware stack
- `auth_middleware` - Authentication and authorization
- `server_core` - HTTP server infrastructure

### Business Logic
- `services` - All application services and business logic
- `objs` - Domain objects and error handling

### Infrastructure
- `axum` - HTTP web framework
- `tokio` - Async runtime
- `tower-sessions` - Session management
- `sqlx` - Database connectivity

## Architecture Position

The `server_app` crate sits at the application entry point:
- **Coordinates**: Complete application bootstrap and initialization
- **Manages**: HTTP server lifecycle and resource management
- **Provides**: Production-ready server executable
- **Integrates**: All application components into cohesive service

## Usage Patterns

### Server Startup
```rust
use server_app::ServerApp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_app = ServerApp::new().await?;
    server_app.run().await?;
    Ok(())
}
```

### Service Configuration
```rust
let app_service = DefaultAppService::new(
    setting_service,
    hub_service,
    data_service,
    auth_service,
    db_service,
    session_service,
    secret_service,
    cache_service,
    localization_service,
    time_service,
);
```

### Server Setup
```rust
let router_state = DefaultRouterState::new(shared_context, app_service);
let app = create_router(router_state).await?;
let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
axum::serve(listener, app).await?;
```

## Service Initialization

### Database Setup
1. Load database configuration from environment
2. Create connection pool with appropriate settings
3. Run database migrations to latest schema
4. Verify database connectivity and schema version

### Session Management
1. Initialize session storage backend (SQLite)
2. Configure session layer with security settings
3. Set up session cleanup and maintenance tasks

### Authentication Services
1. Load OAuth provider configuration
2. Initialize JWT signing keys and validation
3. Set up keyring service for secure credential storage
4. Configure authentication middleware

### Business Services
1. Initialize data service with proper paths
2. Set up hub service for model management
3. Configure cache service with appropriate limits
4. Initialize secret service with encryption keys

## Configuration Management

### Environment Variables
- `DATABASE_URL` - Database connection string
- `OAUTH_CLIENT_ID`, `OAUTH_CLIENT_SECRET` - OAuth provider credentials
- `JWT_SECRET` - JWT signing key
- `LOG_LEVEL` - Logging configuration
- `BIND_ADDRESS`, `PORT` - Server binding configuration

### Configuration Loading
```rust
let config = ServerConfig::from_env()?;
let database_url = config.database_url();
let bind_address = config.bind_address();
```

### Validation
- Validate all required configuration is present
- Test database connectivity during startup
- Verify authentication provider configuration
- Validate file system permissions and paths

## Production Considerations

### Performance Optimization
- Connection pooling for database and HTTP clients
- Efficient session storage with cleanup
- Proper async runtime configuration
- Memory usage optimization

### Security Configuration
- Secure session configuration with appropriate flags
- JWT token security with proper signing
- Database connection security
- File system permission validation

### Monitoring Setup
- Health check endpoint configuration
- Metrics collection and reporting
- Structured logging with correlation IDs
- Error tracking and alerting

## Error Handling

### Startup Errors
- Configuration validation failures
- Database connection errors
- Service initialization failures
- Port binding conflicts

### Runtime Errors
- Database connectivity issues
- Authentication service failures
- File system permission errors
- Memory or resource exhaustion

### Graceful Degradation
- Fallback for non-critical service failures
- Proper error responses for client requests
- Service health monitoring and recovery

## Deployment

### Docker Deployment
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin server_app

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/server_app /usr/local/bin/
EXPOSE 8080
CMD ["server_app"]
```

### Environment Configuration
```bash
export DATABASE_URL="sqlite:///data/bodhi.db"
export OAUTH_CLIENT_ID="your-oauth-client-id"
export OAUTH_CLIENT_SECRET="your-oauth-client-secret"
export JWT_SECRET="your-jwt-signing-secret"
export LOG_LEVEL="info"
export BIND_ADDRESS="0.0.0.0:8080"
```

### Health Checks
- `/health` endpoint for load balancer health checks
- Database connectivity validation
- Critical service availability checks
- Resource usage monitoring

## Development Guidelines

### Adding New Services
1. Define service trait and implementation
2. Add service to dependency injection container
3. Update service initialization in bootstrap code
4. Add configuration parameters as needed
5. Include health check validation

### Configuration Changes
1. Add new environment variables with defaults
2. Update configuration validation logic
3. Document configuration requirements
4. Test configuration loading and validation

### Error Handling
- Use structured error types with context
- Provide clear error messages for common issues
- Log errors appropriately for debugging
- Handle startup failures gracefully

## Testing Strategy

### Integration Testing
- Complete server startup and shutdown
- Database migration and connectivity
- Authentication flow end-to-end
- Health check endpoint validation

### Load Testing
- Server performance under concurrent load
- Database connection pool behavior
- Memory usage patterns
- Error handling under stress

## Monitoring and Observability

### Metrics Collection
- HTTP request metrics (count, latency, errors)
- Database connection pool metrics
- Authentication success/failure rates
- Resource usage (CPU, memory, disk)

### Logging
- Structured logging with JSON format
- Request correlation IDs
- Error stack traces with context
- Performance profiling data

### Health Monitoring
- Application health status
- Database connectivity
- External service dependencies
- Resource utilization thresholds

## Security Considerations

### Server Security
- Secure default configuration
- Input validation on all endpoints
- Rate limiting for API endpoints
- Security headers for web responses

### Data Protection
- Encrypted database connections
- Secure session management
- Proper secret handling and rotation
- Audit logging for sensitive operations

### Network Security
- TLS configuration for HTTPS
- CORS policy configuration
- Request size limits
- DDoS protection considerations

## Future Extensions

The server_app crate can be extended with:
- Clustering and load balancing support
- Advanced monitoring and metrics
- Configuration hot reloading
- Plugin system for extensions
- Multi-tenancy support