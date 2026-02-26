# Authentication Architecture

## Overview

Bodhi App implements a secure authentication system based on OAuth2/OpenID Connect standards with Keycloak as the identity provider. The system requires authentication for all access, supporting both single-user and multi-user scenarios with role-based access control.

## Authentication Architecture

### OAuth2 Authentication (Required)
**Use Case**: All deployments - single-user, multi-user, team collaboration, enterprise

**Characteristics**:
- External identity provider (Keycloak) required
- Role-based access control
- Secure token management
- Multi-user support with single-user capability
- Centralized user management
- "Dumb frontend" architecture - all validation on backend

**Security Model**: Zero-trust with token-based authentication

## Authentication Flow Design

### Initial Setup Flow
```
Application Start
    ↓
Check Configuration
    ↓
OAuth2 Setup Required
    ↓
External Auth Server Configuration
    ↓
Client Registration
    ↓
Role Assignment (First User = Admin)
    ↓
System Ready
```

### User Authentication Flow
```
User Access Request
    ↓
Authentication Check
    ↓
┌─────────────┐    ┌─────────────┐
│ Valid Token │    │ No/Invalid  │
│ - Continue  │    │ - Redirect  │
└─────────────┘    └─────────────┘
                       ↓
                OAuth2 Login Flow
                       ↓
                Frontend sends ALL
                query params to backend
                       ↓
                Backend validates and
                determines redirect
```

### Backend-Driven Authentication Logic
```
Frontend Request
    ↓
Send all data to backend
    ↓
Backend validates OAuth params
    ↓
┌─────────────────┐    ┌──────────────────┐
│ Valid OAuth     │    │ Invalid OAuth    │
│ - Create session│    │ - Return error   │
│ - HTTP 303      │    │ - Error message  │
│ - Location hdr  │    │ - HTTP 4xx       │
└─────────────────┘    └──────────────────┘
    ↓                      ↓
Frontend redirects     Frontend shows error
to Location header     and redirects to login
```

## Role-Based Access Control

### Session Role Hierarchy (ResourceRole)
Session-authenticated users have a `ResourceRole` from Keycloak `resource_access` claims:
```
Admin > Manager > PowerUser > User
```
Used for session-based authorization (user management, system config).

### External App / API Token Roles (UserScope / TokenScope)
Two-tier enums with only `User` and `PowerUser` variants:
- **UserScope**: `scope_user_user`, `scope_user_power_user` -- used for external app access requests
- **TokenScope**: `scope_token_user`, `scope_token_power_user` -- used for API token creation

### Permission Model
- **Hierarchical**: Higher roles inherit all permissions of lower roles
- **Additive**: Permissions are cumulative, not restrictive

## External App Access Request Workflow

3rd-party apps access BodhiApp via OAuth2 token exchange. The flow requires explicit user approval:

```
3rd-party App → POST /bodhi/v1/apps/request-access
  (with app_client_id, requested_role, flow_type, redirect_url)
    ↓
  Server creates draft access request
  Returns { status: "draft", review_url: "/apps/access-requests/review/{id}" }
    ↓
  App redirects user to review_url on BodhiApp
    ↓
  User reviews and approves (with approved_role, toolset/MCP selections)
    ↓
  Server redirects to app callback with scope_access_request:<uuid>
    ↓
  App combines scopes → Keycloak authorization → token exchange
    ↓
  Exchanged token cached with role from DB approved_role
```

### Key Design Decisions
- **No auto-approve**: All access requests require explicit user review
- **Role from DB, not JWT**: External app role derived from `approved_role` on DB access request record, not from JWT scope parsing
- **Two role columns**: `requested_role` (NOT NULL, what app requested) + `approved_role` (nullable, what user approved)

## Token Management

### AppRegInfo Structure
**OAuth Client Registration Information**:
```rust
pub struct AppRegInfo {
  pub client_id: String,     // OAuth2 client identifier
  pub client_secret: String, // OAuth2 client secret
}
```

**Security Design**: AppRegInfo contains only OAuth2 client credentials. JWT validation fields (public_key, alg, kid, issuer) are intentionally excluded because:

1. **Server-Side Token Storage**: All tokens are created and stored server-side, preventing external interception
2. **Database-Backed Integrity**: Token validation uses cryptographic hash verification against database records
3. **OAuth2 Backend Validation**: Token authenticity verified through Keycloak backend during refresh operations
4. **Simplified Security Model**: Fewer cryptographic operations and key management requirements

### Session-Based Authentication
**Use Case**: Web UI interactions, browser-based access

**Characteristics**:
- Session cookies for state management
- Automatic token refresh
- Browser-integrated security
- CSRF protection

### API Token Authentication
**Use Case**: Programmatic access, integrations, automation

**Characteristics**:
- DB-backed tokens with SHA-256 hash verification
- Two-tier scope: `scope_token_user` / `scope_token_power_user`
- Revocable access
- Machine-to-machine communication

### External App Authentication (OAuth2 Token Exchange)
**Use Case**: 3rd-party applications accessing BodhiApp resources

**Characteristics**:
- OAuth2 token exchange via Keycloak
- Access request workflow with user approval
- Role derived from DB `approved_role`, not JWT scopes
- `AuthContext::ExternalApp` with `role: Option<UserScope>`

### Token Lifecycle
```
Token Creation
    ↓
Active Usage
    ↓
┌─────────────────┐    ┌──────────────────┐
│ Session Token   │    │ API Token        │
│ - Auto Refresh  │    │ - Long Lived     │
│ - Browser Only  │    │ - Revocable      │
└─────────────────┘    └──────────────────┘
    ↓                      ↓
Expiration/Logout      Manual Revocation
```

## Security Architecture

### Authentication Security
- **OAuth2/OpenID Connect**: Industry standard protocols
- **JWT Claims Parsing**: Token claims validation without signature verification
- **Database Token Integrity**: SHA-256 hash verification prevents token tampering
- **Token Rotation**: Automatic refresh for active sessions
- **Secure Storage**: Encrypted session management

### JWT Validation Architecture
**Current Implementation**: Database-backed token integrity validation instead of JWT signature validation

**Security Rationale**:
- **No External Token Exposure**: Tokens created and stored server-side only
- **Hash-Based Integrity**: SHA-256 hash verification prevents tampering
- **OAuth2 Backend Validation**: Keycloak validates token authenticity during operations
- **Reduced Attack Surface**: No local JWT signature validation code to exploit

**Token Validation Flow**:
1. Parse JWT claims (no signature verification)
2. Database lookup by user_id and token_id (jti claim)
3. Cryptographic hash verification for integrity
4. Business logic validation (iat, typ, azp, exp)

### Authorization Security
- **Principle of Least Privilege**: Users get minimum required access
- **Role-Based Control**: Permissions tied to roles, not individuals
- **Token Scoping**: API tokens limited to specific capabilities
- **Audit Trail**: Authentication and authorization events logged

### Transport Security
- **HTTPS Only**: All authentication traffic encrypted
- **Secure Cookies**: HttpOnly, Secure, SameSite attributes
- **Token Validation**: Cryptographic signature verification
- **Session Protection**: CSRF and XSS mitigation

## Integration Points

### Frontend Integration
- **Route Protection**: Authentication-aware routing
- **State Management**: User authentication state
- **Error Handling**: Authentication failure recovery
- **User Experience**: Seamless login/logout flows

### Backend Integration
- **Middleware Architecture**: Request-level authentication
- **Service Integration**: Authentication context propagation
- **API Protection**: Endpoint-level authorization
- **Database Security**: User context in data operations

### External Integration
- **Identity Provider**: Keycloak OAuth2 server
- **Token Exchange**: Standard OAuth2 flows
- **User Synchronization**: Identity data management
- **Federation Support**: Multiple identity sources

## Deployment Considerations

### Single-User Deployment
- **Desktop Application**: OAuth2 required but simplified setup
- **Local Development**: Keycloak instance required
- **Personal Use**: Single user with admin role
- **Secure by Default**: No authentication bypass options

### Multi-User Deployment
- **Server Environment**: Centralized authentication
- **Team Collaboration**: Shared resources with access control
- **Enterprise Integration**: Existing identity infrastructure
- **Scalability**: Support for growing user base

### Deployment Consistency
- **Unified Architecture**: Same authentication flow for all deployments
- **Security First**: No authentication bypass modes
- **Configuration Driven**: OAuth2 server configuration only
- **Simplified Maintenance**: Single authentication path to support

## Operational Aspects

### User Management
- **Self-Service**: Users can manage their own tokens
- **Administrative Control**: Admins can manage all users
- **Role Assignment**: Flexible role-based permissions
- **Access Revocation**: Immediate access termination capability

### Monitoring and Auditing
- **Authentication Events**: Login, logout, token creation
- **Authorization Events**: Access grants and denials
- **Security Events**: Failed authentication attempts
- **Compliance Support**: Audit trail for regulatory requirements

### Maintenance and Support
- **Token Rotation**: Automated security maintenance
- **Configuration Management**: Authentication settings
- **Troubleshooting**: Authentication issue diagnosis
- **Performance Monitoring**: Authentication system health

## Design Principles

### Consistency
- **Single Authentication Path**: OAuth2 required for all deployments
- **Secure by Default**: No authentication bypass options
- **Configuration Driven**: OAuth2 server configuration only

### Security
- **Defense in Depth**: Multiple security layers
- **Standard Protocols**: Industry-proven authentication methods
- **Minimal Attack Surface**: Reduce security vulnerabilities

### User Experience
- **Transparent Operation**: Authentication doesn't impede workflow
- **Consistent Interface**: Same experience across authentication modes
- **Error Recovery**: Clear guidance for authentication issues

### Maintainability
- **Standard Implementation**: OAuth2/OpenID Connect compliance
- **Modular Design**: Pluggable authentication components
- **Clear Separation**: Authentication vs. authorization concerns

## Future Considerations

### Enhanced Authentication
- **Multi-Factor Authentication**: Additional security layers
- **Biometric Integration**: Modern authentication methods
- **Social Login**: Popular identity provider integration

### Advanced Authorization
- **Attribute-Based Access Control**: Fine-grained permissions
- **Dynamic Permissions**: Context-aware access control
- **Resource-Level Security**: Granular access management

### Integration Expansion
- **Enterprise SSO**: SAML and other enterprise protocols
- **API Gateway Integration**: Centralized authentication
- **Microservices Security**: Service-to-service authentication

## Related Documentation

- **[Implementation Details](../02-features/implemented/authentication.md)** - Technical implementation
- **[App Status System](app-status.md)** - Application state management
- **[API Integration](api-integration.md)** - "Dumb frontend" patterns and OAuth examples
- **[Frontend Next.js](frontend-react.md)** - Frontend authentication patterns and backend-driven validation
