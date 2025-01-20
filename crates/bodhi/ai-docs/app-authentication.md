# Bodhi App Authentication Guide

## Overview

This document outlines the authentication mechanisms and security schemes used in the Bodhi App, along with best practices and implementation details.

## Security Schemes

### 1. HTTP Authentication

#### Basic Auth
- Username/password encoded in base64
- Simple to implement
- Widely supported
- **Limitation**: Credentials sent with every request
- **Use Case**: Legacy systems, simple integrations

#### Bearer Auth
- Token-based authentication (JWT)
- More secure than Basic Auth
- Supports stateless authentication
- **Use Case**: API authentication, modern web services

#### Digest Auth
- Challenge-response mechanism
- More secure than Basic Auth
- **Use Case**: When credentials can't be sent in plain text

### 2. API Key Authentication

- Can be implemented in:
  - Headers (X-API-KEY)
  - Query parameters
  - Cookies
- Simple to implement and use
- Good for machine-to-machine communication
- **Limitations**: 
  - No built-in expiration
  - Less secure than OAuth2
- **Use Case**: Service-to-service communication, simple API access

### 3. OAuth2 Authentication

#### Flow Types

1. **Authorization Code Flow**
   - Best for web applications
   - Secure for mobile apps (with PKCE)
   - Most secure OAuth2 flow
   - Supports refresh tokens

2. **Implicit Flow**
   - Legacy browser-based applications
   - Generally deprecated
   - Less secure than Authorization Code

3. **Resource Owner Password Flow**
   - For trusted clients only
   - Legacy support
   - Not recommended for new applications

4. **Client Credentials Flow**
   - Machine-to-machine communication
   - No user interaction required
   - Good for background processes

#### Benefits
- Industry standard
- Supports scopes for granular permissions
- Token refresh capabilities
- Separation of concerns

#### Limitations
- More complex to implement
- Requires more infrastructure
- Higher learning curve

### 4. OpenID Connect

- Built on top of OAuth2
- Adds standardized identity layer
- JWT tokens with standardized claims
- User info endpoint
- **Best For**: User authentication with identity management

### 5. Mutual TLS (mTLS)

- Client and server certificate authentication
- Very secure for service-to-service communication
- Complex certificate management required
- **Best For**: Internal service mesh, high-security environments

## Common Implementation Patterns

### Web Applications
```
Frontend ─── OAuth2 Auth Code ───> Auth Server
    │                                  │
    │<─── Access Token + Session ──────┘
    │
    │──── Session Cookie ────> Backend API
```

### API Services
```
Client ─── API Key/Bearer Token ───> API Gateway
   │                                    │
   │<─── Rate Limited/Authenticated ────┘
```

### Microservices
```
Service A ─── mTLS + JWT ───> Service B
    │                            │
    │<─── Authenticated ─────────┘
```

## Current Bodhi Implementation

### Authentication Methods

1. **Session-based Authentication**
   - Used for web UI
   - Secure cookie handling
   - CSRF protection

2. **API Key Authentication**
   - For programmatic access
   - Header-based (X-API-KEY)
   - Rate limiting per key

3. **OAuth2 Integration**
   - Used for initial authentication
   - Authorization code flow
   - Secure token handling

### Security Features

1. **Token Management**
   - Short-lived access tokens
   - Secure storage
   - Revocation capabilities

2. **Permission Scopes**
   - User-level access
   - Power user capabilities
   - Admin functions

3. **Rate Limiting**
   - Per-token limits
   - Graduated thresholds
   - Clear headers

## Best Practices

### Token Management
1. Use short-lived access tokens
2. Implement secure token storage
3. Support token revocation
4. Rotate tokens regularly

### Scope Usage
1. Implement granular permissions
2. Follow least privilege principle
3. Document scopes clearly
4. Validate scopes on every request

### Rate Limiting
1. Implement per-token limits
2. Use graduated rate limits
3. Clear limit documentation
4. Proper error responses

### Error Handling
1. Don't leak implementation details
2. Use consistent error formats
3. Document error responses
4. Include rate limit headers

## Future Enhancements

1. **Enhanced API Token System**
   - Scoped API tokens
   - Token analytics
   - Usage monitoring

2. **Service Mesh Security**
   - mTLS implementation
   - Service-to-service auth
   - Certificate management

3. **Advanced Rate Limiting**
   - Dynamic limits
   - User-based quotas
   - Burst handling

4. **Improved Documentation**
   - Interactive API docs
   - Security guidelines
   - Integration examples

## References

1. [OAuth 2.0 Specification](https://oauth.net/2/)
2. [OpenID Connect](https://openid.net/connect/)
3. [JWT Best Practices](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-jwt-bcp)
4. [API Security Checklist](https://github.com/shieldfy/API-Security-Checklist)

## Bodhi App - Authentication/Authorization

### Architecture Overview

Bodhi App implements a hybrid authentication system combining OAuth2 for user authentication and API tokens for programmatic access. The system is designed to work with both cloud-hosted and self-hosted instances.

### Components

1. **OAuth2 Authorization Server**
   - Cloud-hosted authentication service
   - Development: https://dev-id.getbodhi.app
   - Production: https://id.getbodhi.app
   - Manages user authentication and authorization
   - Issues access and refresh tokens

2. **Client Registration**
   - Each Bodhi instance registers with auth server
   - Receives unique client ID and secret
   - Credentials stored encrypted on instance
   - Enables OAuth2 flows for the instance

### Authentication Flows

1. **User Web Authentication**
   ```
   User ──> Bodhi App ──> Auth Server (PKCE flow)
    │         │               │
    │         │<── Tokens ────┘
    │         │
    │<── Session Cookie ──┘
   ```
   - Uses OAuth2 Authorization Code flow with PKCE
   - Implements secure session management
   - Session cookies for persistent authentication

2. **Role-Based Authorization**
   - Roles available in JWT token:
     - `resource_admin`
     - `resource_manager`
     - `resource_power_user`
     - `resource_user`
   - Roles found in: `resource_access.<client_id>.roles`
   - Hierarchical permission structure

3. **API Token System**
   ```
   User ──> Create Token ──> Offline Token
    │                            │
    │                            │
   External App ──> API ────> Bodhi App
   (Bearer Token)        (Token Exchange)
   ```
   - Uses OAuth2 Token Exchange
   - Supports offline_access scope
   - Non-expiring refresh tokens
   - Privilege levels through scopes:
     - `scope_token_user`
     - `scope_token_power_user`

### Security Implementation

1. **Token Management**
   - Access tokens: Short-lived JWT
   - Refresh tokens: Long-lived for API access
   - Token exchange for API authentication
   - Secure token storage

2. **Authorization Controls**
   - Role-based access control (RBAC)
   - Scope-based API token privileges
   - Token privilege cannot exceed user's role
   - Automatic token refresh mechanism

3. **Session Security**
   - Encrypted session cookies
   - Secure cookie attributes
   - PKCE for authorization code flow
   - XSS/CSRF protections

### Future Improvements

1. **Token Authorization**
   - Migrate from scope-based to role-based
   - Implement token-specific roles
   - Enhanced privilege management
   - Granular permission controls

2. **Security Enhancements**
   - Token usage analytics
   - Advanced rate limiting
   - Token lifecycle management
   - Enhanced audit logging 