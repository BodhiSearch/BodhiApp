# auth_middleware - Authentication and Authorization

## Overview

The `auth_middleware` crate provides HTTP middleware for authentication and authorization in BodhiApp. It implements JWT-based authentication, role-based access control, and API token validation to secure HTTP endpoints.

## Purpose

- **Authentication Middleware**: HTTP middleware for request authentication
- **Authorization Control**: Role-based access control for protected endpoints
- **Token Validation**: JWT and API token validation and verification
- **Security Enforcement**: Consistent security policy enforcement across all routes
- **Session Management**: User session handling and validation

## Key Components

### Core Middleware

#### Auth Middleware (`auth_middleware.rs`)
- Primary authentication middleware for web requests
- JWT token validation and parsing
- User session management
- Role extraction and validation
- Request context enrichment with user information

#### API Auth Middleware (`api_auth_middleware.rs`)
- Authentication middleware specifically for API endpoints
- API token validation
- Rate limiting and throttling
- API-specific security policies
- OpenAI-compatible authentication

### Token Management

#### Token Service (`token_service.rs`)
- JWT token creation and validation
- Token expiration and renewal
- Token scope and permission management
- Secure token storage and retrieval
- Token revocation and blacklisting

### Utilities

#### Auth Utilities (`utils.rs`)
- Authentication helper functions
- Token parsing and validation utilities
- Role checking and permission utilities
- Security-related utility functions

## Directory Structure

```
src/
├── lib.rs                    # Main module exports
├── auth_middleware.rs        # Web authentication middleware
├── api_auth_middleware.rs    # API authentication middleware
├── token_service.rs          # Token management service
├── utils.rs                  # Authentication utilities
├── resources/                # Localization resources
│   └── en-US/
└── test_utils/               # Testing utilities
    └── mod.rs
```

## Key Features

### JWT Authentication
- Secure JWT token validation
- Configurable token expiration
- Token refresh mechanisms
- Secure token signing and verification

### Role-Based Access Control (RBAC)
- User role management (Admin, PowerUser, BasicUser)
- Endpoint-level permission checking
- Hierarchical role inheritance
- Fine-grained access control

### API Token Support
- API key authentication for programmatic access
- Token scope management
- Rate limiting per token
- Token usage tracking

### Security Features
- Secure token storage
- Protection against common attacks (CSRF, XSS)
- Request validation and sanitization
- Audit logging for security events

## Authentication Flow

### Web Authentication
1. User provides credentials (OAuth2 or direct login)
2. System validates credentials and creates JWT
3. JWT is stored in secure HTTP-only cookie
4. Middleware validates JWT on each request
5. User information is extracted and added to request context

### API Authentication
1. Client provides API token in Authorization header
2. Middleware validates token and checks permissions
3. Request is allowed or denied based on token scope
4. Usage is tracked for rate limiting

## Middleware Integration

### Axum Integration
The middleware integrates seamlessly with Axum:

```rust
let app = Router::new()
    .route("/protected", get(protected_handler))
    .layer(AuthMiddleware::new(auth_service))
    .route("/api/v1/chat", post(api_handler))
    .layer(ApiAuthMiddleware::new(token_service));
```

### Request Context
Authenticated requests have user information available:

```rust
async fn protected_handler(
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Response>, ApiError> {
    // Access user.id, user.role, user.permissions
}
```

## Dependencies

### Core Dependencies
- **objs**: Domain objects and error types
- **services**: Authentication and token services
- **server_core**: HTTP server infrastructure
- **axum**: HTTP framework and middleware support

### Security Dependencies
- **jsonwebtoken**: JWT token handling
- **tower**: Middleware abstractions
- **cookie**: Secure cookie management

## Usage Patterns

### Protecting Routes
Routes can be protected by applying the appropriate middleware:

```rust
// Web routes with session authentication
let web_routes = Router::new()
    .route("/dashboard", get(dashboard))
    .layer(AuthMiddleware::new(auth_service));

// API routes with token authentication
let api_routes = Router::new()
    .route("/api/chat", post(chat_endpoint))
    .layer(ApiAuthMiddleware::new(token_service));
```

### Role-Based Protection
Different endpoints can require different roles:

```rust
async fn admin_handler(
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Response>, ApiError> {
    user.require_role(Role::Admin)?;
    // Admin-only logic
}

async fn user_handler(
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Response>, ApiError> {
    user.require_role(Role::BasicUser)?;
    // User logic
}
```

### Token Validation
API tokens are validated with scope checking:

```rust
async fn api_handler(
    Extension(token): Extension<ValidatedToken>,
) -> Result<Json<Response>, ApiError> {
    token.require_scope(TokenScope::ChatAccess)?;
    // API logic
}
```

## Security Considerations

### Token Security
- JWT tokens are signed with secure keys
- Tokens have appropriate expiration times
- Refresh tokens are used for long-lived sessions
- Token revocation is supported

### Cookie Security
- HTTP-only cookies prevent XSS attacks
- Secure flag ensures HTTPS-only transmission
- SameSite attribute prevents CSRF attacks
- Proper cookie expiration handling

### Rate Limiting
- API tokens have rate limits
- Brute force protection on authentication endpoints
- Configurable rate limiting policies

## Error Handling

### Authentication Errors
- Clear error messages for authentication failures
- Proper HTTP status codes (401, 403)
- Localized error messages
- Security event logging

### Authorization Errors
- Role-based error messages
- Permission denied responses
- Audit trail for authorization failures

## Testing Support

The auth_middleware crate includes testing utilities:
- Mock authentication services
- Test token generation
- Authentication flow testing
- Permission testing helpers

## Integration Points

- **Routes**: All protected routes use auth middleware
- **Services**: Integrates with auth and token services
- **Frontend**: Provides authentication state to UI
- **Logging**: Security events are logged for audit

## Configuration

### JWT Configuration
- Configurable signing keys
- Token expiration settings
- Refresh token policies
- Issuer and audience validation

### API Token Configuration
- Token scope definitions
- Rate limiting settings
- Token expiration policies
- Usage tracking configuration

## Future Extensions

The auth middleware is designed to support:
- Multi-factor authentication (MFA)
- OAuth2 provider integration
- SAML authentication
- Advanced rate limiting
- Audit logging enhancements
- Custom authentication schemes
