# Authentication Architecture

## Overview

Bodhi App implements a flexible authentication system that supports both single-user and multi-user scenarios. The system is designed around OAuth2/OpenID Connect standards with Keycloak as the identity provider, while also supporting a no-authentication mode for local development and personal use.

## Authentication Modes

### 1. No Authentication Mode
**Use Case**: Personal use, local development, single-user scenarios

**Characteristics**:
- No login required
- All features immediately accessible
- No user management overhead
- Suitable for desktop applications
- Local configuration only

**Security Model**: Trust-based (local environment assumed secure)

### 2. OAuth2 Authentication Mode
**Use Case**: Multi-user environments, team collaboration, enterprise deployments

**Characteristics**:
- External identity provider (Keycloak)
- Role-based access control
- Secure token management
- Multi-user support
- Centralized user management

**Security Model**: Zero-trust with token-based authentication

## Authentication Flow Design

### Initial Setup Flow
```
Application Start
    ↓
Check Configuration
    ↓
┌─────────────────┐    ┌──────────────────┐
│ No Auth Mode    │    │ OAuth2 Mode      │
│ - Direct Access │    │ - Setup Required │
│ - Local Config  │    │ - External Auth  │
└─────────────────┘    └──────────────────┘
    ↓                      ↓
Ready to Use          Registration Flow
```

### OAuth2 Setup Flow
```
Setup Initiation
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
┌─────────────────┐    ┌──────────────────┐
│ No Auth Mode    │    │ OAuth2 Mode      │
│ - Allow Access  │    │ - Check Token    │
└─────────────────┘    └──────────────────┘
                           ↓
                    ┌─────────────┐    ┌─────────────┐
                    │ Valid Token │    │ No/Invalid  │
                    │ - Continue  │    │ - Redirect  │
                    └─────────────┘    └─────────────┘
                                           ↓
                                    OAuth2 Login Flow
```

## Role-Based Access Control

### Role Hierarchy
```
Administrator
├── Full system access
├── User management
├── System configuration
└── All lower role permissions
    ↓
Manager
├── User management
├── Content management
└── All lower role permissions
    ↓
Power User
├── Advanced features
├── API access
└── All lower role permissions
    ↓
User
└── Basic application features
```

### Permission Model
- **Hierarchical**: Higher roles inherit all permissions of lower roles
- **Additive**: Permissions are cumulative, not restrictive
- **Contextual**: Some permissions may be resource-specific

## Token Management

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
- Long-lived offline tokens
- Scope-based permissions
- Revocable access
- Machine-to-machine communication

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
- **JWT Tokens**: Stateless, verifiable tokens
- **Token Rotation**: Automatic refresh for active sessions
- **Secure Storage**: Encrypted session management

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
- **Desktop Application**: No authentication overhead
- **Local Development**: Simplified setup
- **Personal Use**: Direct feature access
- **Offline Capability**: No external dependencies

### Multi-User Deployment
- **Server Environment**: Centralized authentication
- **Team Collaboration**: Shared resources with access control
- **Enterprise Integration**: Existing identity infrastructure
- **Scalability**: Support for growing user base

### Hybrid Deployment
- **Development to Production**: Smooth transition path
- **Feature Parity**: Same features regardless of auth mode
- **Configuration Driven**: Runtime authentication mode selection
- **Migration Support**: Convert between authentication modes

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

### Flexibility
- **Mode Selection**: Choose appropriate authentication for use case
- **Progressive Enhancement**: Start simple, add security as needed
- **Configuration Driven**: Runtime behavior modification

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
- **[Backend Integration](backend-integration.md)** - API integration patterns
- **[Frontend Architecture](frontend-architecture.md)** - UI authentication integration
