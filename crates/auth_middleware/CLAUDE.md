# CLAUDE.md - auth_middleware

This file provides guidance to Claude Code when working with the `auth_middleware` crate, which provides authentication and authorization middleware for BodhiApp's HTTP server.

## Purpose

The `auth_middleware` crate implements comprehensive authentication and authorization:

- **JWT Token Validation**: Bearer token authentication with proper validation
- **Session Management**: HTTP session handling with secure storage
- **Role-Based Access Control**: Fine-grained authorization with roles and scopes
- **API Authentication**: Middleware for protecting API endpoints
- **Token Services**: JWT token creation, validation, and refresh
- **Security Headers**: Same-origin validation and security controls

## Key Components

### Authentication Middleware (`src/auth_middleware.rs`)
- `auth_middleware` - Main authentication middleware for session-based auth
- Session token validation and refresh token handling
- Same-origin request validation using Sec-Fetch-Site header
- Integration with Tower Sessions for secure session management

### API Authentication Middleware (`src/api_auth_middleware.rs`)
- `api_auth_middleware` - Bearer token authentication for API endpoints
- JWT token extraction and validation from Authorization header
- Resource scope and role-based authorization
- Cache-based token validation for performance

### Token Service (`src/token_service.rs`)
- `DefaultTokenService` - Comprehensive token management service
- JWT token creation, validation, and refresh operations
- Token digest generation for secure storage and lookup
- Integration with database for token status tracking
- Claims extraction and validation with proper leeway handling

### Authentication Utilities (`src/utils.rs`)
- Helper functions for token processing and validation
- Security utilities for request validation
- Error handling and response formatting

## Dependencies

### Core Infrastructure
- `objs` - Domain objects, error types, and validation
- `services` - Business logic services and authentication
- `server_core` - HTTP server infrastructure and routing

### Authentication & Security
- `jsonwebtoken` - JWT token creation and validation
- `sha2` - SHA-256 hashing for token digests
- `base64` - Base64 encoding/decoding for tokens
- `tower-sessions` - HTTP session management with secure storage

### HTTP Framework
- `axum` - Web framework integration with extractors and middleware
- `time` - Time handling for token expiration
- `chrono` - Date/time operations for claims validation

### Development & Testing
- `rstest` - Parameterized testing framework
- `mockall` - Mock service generation for testing

## Architecture Position

The `auth_middleware` crate sits at the HTTP middleware layer:
- **Above**: Server core infrastructure and routing
- **Below**: Route handlers and business logic
- **Integrates**: Authentication services, session management, and security controls
- **Provides**: Security boundaries for all HTTP endpoints

## Usage Patterns

### Session-Based Authentication Middleware
```rust
use auth_middleware::{auth_middleware, AuthError};
use axum::{routing::get, Router};
use tower_sessions::{SessionManagerLayer, MemoryStore};

let session_layer = SessionManagerLayer::new(MemoryStore::default());

let app = Router::new()
    .route("/protected", get(protected_handler))
    .route_layer(axum::middleware::from_fn_with_state(
        app_state.clone(),
        auth_middleware,
    ))
    .layer(session_layer);
```

### API Bearer Token Authentication
```rust
use auth_middleware::{api_auth_middleware, DefaultTokenService};

let app = Router::new()
    .route("/api/chat", post(chat_handler))
    .route_layer(axum::middleware::from_fn_with_state(
        app_state.clone(),
        api_auth_middleware,
    ));
```

### Token Validation
```rust
use auth_middleware::{DefaultTokenService, AuthError};

let token_service = DefaultTokenService::new(
    auth_service,
    secret_service,
    cache_service,
    db_service,
    setting_service,
);

// Validate bearer token from Authorization header
let (token_digest, resource_scope) = token_service
    .validate_bearer_token(&auth_header)
    .await?;

// Check token status in database
let is_active = token_service.is_token_active(&token_digest).await?;
```

### Role and Scope Validation
```rust
use objs::{Role, ResourceScope, TokenScope, UserScope};

// Extract and validate roles from token claims
let roles = extract_roles_from_claims(&claims)?;
if !roles.contains(&Role::User) {
    return Err(AuthError::MissingRoles);
}

// Validate resource scopes
let resource_scope = ResourceScope::new(
    TokenScope::Read,
    UserScope::Own,
);
```

### Session Token Management
```rust
use tower_sessions::Session;
use auth_middleware::{SESSION_KEY_ACCESS_TOKEN, SESSION_KEY_REFRESH_TOKEN};

// Store tokens in session
session.insert(SESSION_KEY_ACCESS_TOKEN, access_token).await?;
session.insert(SESSION_KEY_REFRESH_TOKEN, refresh_token).await?;

// Retrieve tokens from session
let access_token: String = session
    .get(SESSION_KEY_ACCESS_TOKEN)
    .await?
    .ok_or(AuthError::TokenNotFound)?;
```

## Integration Points

### With HTTP Routes
- Middleware applied to route layers for authentication
- Request extension with authenticated user information
- Error handling with appropriate HTTP status codes

### With Services Layer
- Authentication service for OAuth flows and token exchange
- Secret service for JWT signing key management
- Database service for token status tracking
- Cache service for performance optimization

### With Session Management
- Tower Sessions integration for secure session storage
- Session-based authentication for web interfaces
- Secure cookie handling with appropriate flags

## Authentication Flows

### Session-Based Flow
1. **Request Validation**: Check Sec-Fetch-Site for same-origin requests
2. **Session Lookup**: Retrieve access token from session storage
3. **Token Validation**: Validate JWT token with proper claims checking
4. **Token Refresh**: Automatically refresh expired tokens using refresh token
5. **User Context**: Inject authenticated user information into request

### API Bearer Token Flow
1. **Header Extraction**: Extract bearer token from Authorization header
2. **Token Validation**: Validate JWT signature and claims
3. **Database Check**: Verify token status in database (active/revoked)
4. **Scope Validation**: Check resource scopes and user permissions
5. **Request Processing**: Allow authenticated request to proceed

## Security Features

### Token Security
- JWT tokens with proper signing and validation
- Token digest generation for secure database storage
- Automatic token refresh with secure refresh tokens
- Token revocation support through database status

### Request Validation
- Same-origin request validation using security headers
- Bearer token format validation and sanitization
- Proper error handling without information disclosure

### Session Security
- Secure session management with Tower Sessions
- HTTP-only and secure cookie flags
- Session timeout and cleanup mechanisms

## Error Handling

### Authentication Errors
- `TokenNotFound` - Missing or invalid authentication token
- `TokenInactive` - Token has been revoked or deactivated
- `InvalidToken` - Malformed or invalid JWT token
- `MissingRoles` - Insufficient permissions for resource access

### Authorization Errors
- `InvalidAccess` - Access denied for requested resource
- `RoleError` - Role validation failures
- `TokenScopeError` - Token scope validation failures
- `UserScopeError` - User scope validation failures

### Service Integration Errors
- `AuthServiceError` - Authentication service communication failures
- `SecretServiceError` - JWT signing key retrieval failures
- `TowerSession` - Session management errors

## Performance Considerations

### Caching Strategy
- Token validation results cached for performance
- Database lookups minimized through intelligent caching
- JWT validation with configurable leeway for clock skew

### Database Optimization
- Token status queries optimized with proper indexing
- Batch operations for token management
- Connection pooling for database access

### Memory Management
- Efficient session storage with appropriate cleanup
- JWT claims processing with minimal allocations
- Token digest computation with secure hashing

## Development Guidelines

### Adding New Authentication Methods
1. Implement authentication logic in appropriate middleware
2. Add error types with proper `AppError` integration
3. Include comprehensive validation and security checks
4. Add unit tests with mock services
5. Update integration tests for new flows

### Security Best Practices
- Always validate token signatures and claims
- Use secure random generation for token secrets
- Implement proper token expiration and refresh
- Log security events for monitoring and analysis
- Validate all input and sanitize error messages

### Testing Strategy
- Unit tests for token validation logic
- Integration tests for complete authentication flows
- Security tests for edge cases and attack scenarios
- Performance tests for high-load scenarios

## Monitoring and Logging

### Security Events
- Failed authentication attempts with context
- Token validation failures and reasons
- Suspicious request patterns and anomalies
- Session management events and errors

### Performance Metrics
- Authentication middleware response times
- Token validation cache hit rates
- Database query performance for token operations
- Session storage utilization and cleanup

## Configuration

### JWT Configuration
- Configurable token expiration times
- JWT signing algorithm and key rotation
- Claims validation rules and requirements
- Token refresh policies and intervals

### Security Settings
- Same-origin validation enforcement
- Session timeout and cleanup intervals
- Rate limiting for authentication attempts
- Security header validation rules