# CLAUDE.md - routes_app

This file provides guidance to Claude Code when working with the `routes_app` crate, which implements application-specific API endpoints for BodhiApp.

## Purpose

The `routes_app` crate provides application-specific HTTP endpoints:

- **Model Management**: Create, pull, and manage model aliases
- **Authentication**: OAuth login, token exchange, and user management
- **API Token Management**: Generate and manage API tokens for programmatic access
- **Application Settings**: Configuration management and system settings
- **Development Endpoints**: Developer tools and debugging interfaces
- **Setup and Onboarding**: Application initialization and configuration
- **OpenAPI Documentation**: Comprehensive API documentation with Utoipa

## Key Components

### Model Management Routes
- `routes_create.rs` - Model alias creation with configuration
- `routes_pull.rs` - Model download and alias management
- `routes_models.rs` - Model listing and metadata endpoints

### Authentication Routes
- `routes_login.rs` - OAuth login flow and session management
- `routes_api_token.rs` - API token generation and management
- `routes_user.rs` - User profile and account management

### Application Routes
- `routes_settings.rs` - Application configuration and preferences
- `routes_setup.rs` - Initial setup and onboarding flows
- `routes_dev.rs` - Development tools and debugging endpoints
- `routes_ui.rs` - UI-specific endpoints and static content

### Infrastructure
- `error.rs` - Application-specific error types
- `objs.rs` - Request/response objects and validation
- `openapi.rs` - OpenAPI specification generation

## Dependencies

### Core Infrastructure
- `objs` - Domain objects and validation
- `services` - Business logic services
- `commands` - CLI command implementations
- `server_core` - HTTP server infrastructure
- `auth_middleware` - Authentication and authorization

### HTTP Framework
- `axum` - Web framework with routing and extractors
- `axum-extra` - Additional extractors and utilities
- `tower-sessions` - Session management

### Authentication
- `oauth2` - OAuth2 client implementation
- `jsonwebtoken` - JWT token handling
- `base64` - Encoding/decoding utilities

### API Documentation
- `utoipa` - OpenAPI specification generation
- Comprehensive API documentation with examples

## Architecture Position

The `routes_app` crate sits at the application API layer:
- **Above**: Server core and middleware infrastructure
- **Below**: Frontend applications and API clients
- **Coordinates**: Authentication, model management, and application configuration
- **Integrates**: All major application services and business logic

## Usage Patterns

### Route Registration
```rust
use routes_app::*;
use axum::{routing::{get, post}, Router};

let app = Router::new()
    .route("/app/models", get(list_models))
    .route("/app/models/create", post(create_model))
    .route("/app/auth/login", get(login))
    .route("/app/auth/token", post(create_token))
    .with_state(router_state);
```

### Model Management
```rust
use routes_app::{create_model, CreateModelRequest};

// Create new model alias
let request = CreateModelRequest {
    alias: "my-model".to_string(),
    repo: "microsoft/DialoGPT-medium".to_string(),
    filename: "pytorch_model.bin".to_string(),
    auto_download: true,
    // ... other fields
};

let response = create_model(State(router_state), Json(request)).await?;
```

### Authentication Flow
```rust
use routes_app::{login, oauth_callback};

// Start OAuth login
let auth_url = login(State(router_state)).await?;

// Handle OAuth callback
let tokens = oauth_callback(
    State(router_state),
    Query(callback_params),
    session,
).await?;
```

### API Token Management
```rust
use routes_app::{create_token, TokenRequest};

let token_request = TokenRequest {
    name: "My API Token".to_string(),
    scopes: vec!["read".to_string(), "write".to_string()],
    expires_in: Some(3600), // 1 hour
};

let token_response = create_token(
    State(router_state),
    Json(token_request),
).await?;
```

## Integration Points

### With Commands Layer
- Model creation and pulling use command implementations
- Direct integration with `CreateCommand` and `PullCommand`
- Async execution of CLI operations through HTTP endpoints

### With Services Layer
- Authentication service for OAuth flows
- Data service for model management
- Hub service for model discovery and download
- Settings service for configuration management

### With Auth Middleware
- Protected endpoints use authentication middleware
- Session management for web interface
- API token validation for programmatic access

## API Endpoints

### Model Management
- `GET /app/models` - List available models
- `POST /app/models/create` - Create new model alias
- `POST /app/models/pull` - Pull model from repository
- `DELETE /app/models/{alias}` - Delete model alias

### Authentication
- `GET /app/auth/login` - Initiate OAuth login
- `GET /app/auth/callback` - OAuth callback handler
- `POST /app/auth/logout` - End user session
- `GET /app/auth/user` - Get current user info

### API Tokens
- `GET /app/tokens` - List user's API tokens
- `POST /app/tokens` - Create new API token
- `DELETE /app/tokens/{id}` - Revoke API token

### Settings
- `GET /app/settings` - Get application settings
- `POST /app/settings` - Update application settings
- `GET /app/setup` - Check setup status
- `POST /app/setup` - Complete initial setup

## Request/Response Objects

### Model Management
- `CreateModelRequest` - Model creation parameters
- `PullModelRequest` - Model pull parameters
- `ModelResponse` - Model information and status

### Authentication
- `LoginResponse` - OAuth authorization URL
- `TokenRequest` - API token creation parameters
- `TokenResponse` - Created token information
- `UserResponse` - User profile information

### Settings
- `SettingsRequest` - Configuration updates
- `SettingsResponse` - Current application settings
- `SetupRequest` - Initial setup parameters

## Error Handling

### Application Errors
- Model not found errors
- Authentication failures
- Permission denied errors
- Validation failures

### HTTP Status Mapping
- 200 OK - Successful operations
- 201 Created - Resource creation
- 400 Bad Request - Validation errors
- 401 Unauthorized - Authentication required
- 403 Forbidden - Insufficient permissions
- 404 Not Found - Resource not found
- 500 Internal Server Error - System errors

## Authentication Flow

### OAuth2 Flow
1. User initiates login via `/app/auth/login`
2. Redirect to OAuth provider with PKCE challenge
3. Provider redirects to `/app/auth/callback` with code
4. Exchange code for tokens and create session
5. Store tokens securely in session storage

### API Token Flow
1. User requests token via `/app/tokens` endpoint
2. Generate JWT with specified scopes and expiration
3. Store token hash in database for revocation
4. Return token to user for API access

## Performance Considerations

### Async Operations
- All model operations are asynchronous
- Long-running operations provide progress feedback
- Non-blocking request processing

### Caching
- Model list caching for performance
- Settings caching with appropriate invalidation
- Token validation caching

### Database Operations
- Efficient queries with proper indexing
- Transaction management for data consistency
- Connection pooling for scalability

## Development Guidelines

### Adding New Endpoints
1. Define request/response objects with validation
2. Implement handler with proper error handling
3. Add authentication/authorization as needed
4. Include OpenAPI documentation
5. Add comprehensive tests

### Error Handling Best Practices
- Use appropriate HTTP status codes
- Provide clear error messages with context
- Log errors appropriately for debugging
- Handle edge cases and validation failures

### Testing Strategy
- Unit tests for individual handlers
- Integration tests for complete flows
- Authentication and authorization testing
- Error condition validation

## Security Considerations

### Input Validation
- Comprehensive validation of all inputs
- Sanitization of file paths and names
- Parameter validation against allowed ranges

### Authentication Security
- Secure OAuth2 implementation with PKCE
- JWT tokens with proper expiration
- Session security with appropriate flags
- API token revocation capabilities

### Data Protection
- Secure handling of sensitive information
- Proper error messages without information leakage
- Audit logging for security events

## OpenAPI Documentation

### Specification Generation
- Automatic OpenAPI 3.0 specification generation
- Comprehensive endpoint documentation
- Request/response schema definitions
- Example requests and responses

### Documentation Features
- Interactive API documentation
- Schema validation and examples
- Authentication flow documentation
- Error response documentation

## Monitoring and Observability

### Request Metrics
- Endpoint usage statistics
- Response time and error rates
- Authentication success/failure rates
- Model operation metrics

### Business Metrics
- Model creation and usage patterns
- User authentication patterns
- API token usage statistics
- System configuration changes

## Future Extensions

The routes_app crate is designed for extensibility:
- Additional model management operations
- Enhanced user management features
- Advanced settings and configuration options
- Webhook and notification endpoints
- Batch operations and bulk management