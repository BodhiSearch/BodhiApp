# CLAUDE.md

This file provides guidance to Claude Code when working with the `auth_middleware` crate.

See [crates/auth_middleware/PACKAGE.md](crates/auth_middleware/PACKAGE.md) for implementation details.

## Purpose

The `auth_middleware` crate serves as BodhiApp's **HTTP authentication and authorization middleware layer**, implementing sophisticated JWT token validation, session management, and multi-layered security controls with comprehensive OAuth2 integration and role-based access control.

## Key Domain Architecture

### Multi-Layer Authentication System
Sophisticated authentication architecture supporting multiple authentication flows:
- **Session-Based Authentication**: HTTP session management with Tower Sessions integration and secure cookie handling
- **Bearer Token Authentication**: JWT token validation with OAuth2 compliance and external client token exchange
- **Dual Authentication Support**: Seamless handling of both session tokens and API bearer tokens with precedence rules
- **Token Exchange Protocol**: RFC 8693 token exchange for external client integration and service-to-service authentication
- **Same-Origin Validation**: Security header validation for CSRF protection and request origin verification

### JWT Token Management Architecture
Comprehensive token lifecycle management with sophisticated validation:
- **Token Service Coordination**: Centralized token validation, refresh, and exchange operations through `DefaultTokenService`
- **Multi-Token Type Support**: Offline tokens, access tokens, and external client tokens with different validation rules
- **Cache-Based Performance**: Token validation caching with expiration handling and automatic cache invalidation
- **Database Token Tracking**: API token status management with active/inactive state tracking and digest-based lookup
- **Claims Validation**: Comprehensive JWT claims validation with leeway handling, issuer verification, and audience validation

### Role-Based Authorization System
Fine-grained access control with hierarchical role management:
- **Role Hierarchy Enforcement**: Admin > Manager > PowerUser > User ordering with `has_access_to()` validation
- **Resource Scope Authorization**: Token-based and user-based scope validation with `ResourceScope` union type
- **API Authorization Middleware**: Route-level authorization with configurable role and scope requirements
- **Authorization Precedence**: Role-based authorization takes precedence over scope-based when both are present
- **Cross-Service Authorization**: Integration with objs crate role and scope types for consistent authorization

### Security Infrastructure Architecture
Multi-layered security controls with comprehensive threat protection:
- **Canonical URL Middleware**: Automatic redirection to canonical URLs with SEO optimization and security benefits
- **Request Origin Validation**: Same-origin request validation using Sec-Fetch-Site headers for CSRF protection
- **Token Digest Security**: SHA-256 token digests for secure database storage and lookup without exposing tokens
- **External Client Security**: Secure external client token validation with issuer and audience verification
- **Security Header Management**: Automatic injection and removal of internal security headers for request processing

## Architecture Position

The `auth_middleware` crate serves as BodhiApp's **HTTP security orchestration layer**:
- **Above server_core and services**: Coordinates HTTP infrastructure and business services for authentication operations
- **Below route implementations**: Provides authentication and authorization foundation for routes_oai and routes_app
- **Integration with objs**: Uses domain objects for role, scope, and error handling with consistent validation
- **Cross-cutting security**: Implements security boundaries across all HTTP endpoints with comprehensive middleware integration

## Cross-Crate Integration Patterns

### Service Layer Authentication Coordination
Complex authentication flows coordinated across BodhiApp's service layer:
- **AuthService Integration**: OAuth2 flows, token exchange, and refresh operations with comprehensive error handling
- **SecretService Coordination**: JWT signing key management, app registration info, and credential encryption
- **DbService Token Management**: API token storage, status tracking, and digest-based lookup with transaction support
- **CacheService Performance**: Token validation caching, exchange result caching, and automatic invalidation strategies
- **SessionService Integration**: HTTP session management with SQLite backend and secure cookie configuration

### HTTP Infrastructure Integration
Authentication middleware coordinates with HTTP infrastructure for request processing:
- **RouterState Integration**: Dependency injection providing access to AppService registry for authentication operations
- **Request Header Management**: Automatic injection and removal of internal authentication headers for security
- **Error Translation**: Service errors converted to appropriate HTTP status codes with user-friendly messages
- **Middleware Composition**: Layered middleware architecture with auth_middleware, inject_session_auth_info, and canonical_url_middleware
- **OpenAI API Security**: Bearer token authentication for OpenAI-compatible endpoints through routes_oai integration
- **Application API Security**: Session-based and bearer token authentication for application endpoints through routes_app integration
- **API Token Management**: Database-backed API token validation for external client access to OpenAI endpoints
- **Route Composition Integration**: Coordinated with routes_all for comprehensive middleware orchestration with hierarchical authorization and multi-layer security

### Domain Object Integration
Extensive use of objs crate for authentication and authorization:
- **Role and Scope Types**: Hierarchical role validation and resource scope authorization with consistent domain logic
- **Error System Integration**: Authentication errors implement AppError trait for consistent HTTP response generation
- **ResourceScope Union**: Seamless handling of both TokenScope and UserScope authorization contexts
- **Validation Consistency**: Domain object validation rules applied consistently across authentication flows

## Authentication Flow Orchestration

### Multi-Flow Authentication Architecture
Sophisticated authentication coordination supporting multiple authentication patterns:

**Session-Based Authentication Flow**:
1. **Same-Origin Validation**: Sec-Fetch-Site header validation for CSRF protection and request origin verification
2. **Session Token Retrieval**: Access token extraction from HTTP session storage with Tower Sessions integration
3. **Token Validation**: JWT token validation with claims verification and expiration checking
4. **Automatic Token Refresh**: Expired token refresh using refresh tokens with seamless session updates
5. **Role Extraction**: Role extraction from JWT claims with hierarchical authorization validation

**Bearer Token Authentication Flow**:
1. **Token Extraction**: Bearer token extraction from Authorization header with format validation
2. **Database Token Lookup**: API token status verification with digest-based lookup and active/inactive checking
3. **External Client Handling**: External client token validation with issuer and audience verification
4. **Token Exchange**: RFC 8693 token exchange for external clients with scope validation and caching
5. **Resource Scope Authorization**: Token-based or user-based scope validation with ResourceScope union type

**Dual Authentication Support**:
- **Precedence Rules**: Bearer token authentication takes precedence over session-based authentication
- **Header Management**: Automatic injection of X-Resource-Token, X-Resource-Role, and X-Resource-Scope headers
- **Security Isolation**: Removal of user-provided internal headers to prevent header injection attacks
- **Error Coordination**: Consistent error handling across both authentication flows with user-friendly messages

### Token Service Orchestration Workflows
Complex token management workflows coordinated across multiple services:

**Token Validation Workflow**:
1. **Cache Check**: Token validation result lookup in cache for performance optimization
2. **Database Verification**: API token status verification with digest-based lookup
3. **JWT Validation**: Comprehensive JWT claims validation with leeway handling and signature verification
4. **Service Coordination**: AuthService token refresh or exchange operations with error handling
5. **Cache Update**: Validation result caching with appropriate TTL and invalidation strategies

**External Client Token Exchange**:
1. **Issuer Validation**: External token issuer verification against configured auth issuer
2. **Audience Verification**: Token audience validation to ensure it targets the current client
3. **Scope Extraction**: User scope extraction from external token for exchange operation
4. **Token Exchange**: AuthService token exchange with proper scope mapping and error handling
5. **Result Caching**: Exchange result caching with token digest-based keys for performance

## Important Constraints

### Authentication Security Requirements
- All authentication operations must validate JWT signatures and claims with proper leeway handling for clock skew
- Session-based authentication requires same-origin validation using Sec-Fetch-Site headers for CSRF protection
- Bearer token authentication must support both internal API tokens and external client tokens with different validation rules
- Token refresh operations must be atomic with proper session updates and error recovery mechanisms
- External client token exchange must validate issuer and audience claims to prevent token substitution attacks

### Authorization Consistency Standards
- Role-based authorization must follow hierarchical ordering (Admin > Manager > PowerUser > User) consistently
- Resource scope authorization must support both TokenScope and UserScope with proper precedence rules
- Authorization middleware must handle missing authentication gracefully with appropriate HTTP status codes
- Cross-service authorization decisions must use consistent domain objects from objs crate for validation
- API authorization middleware must support configurable role and scope requirements for different endpoints

### Token Management Security Rules
- Token digests must use SHA-256 hashing for secure database storage without exposing actual tokens
- Token validation results must be cached with appropriate TTL and automatic invalidation on token changes
- Database token status must be checked for all API tokens to support token revocation and lifecycle management
- External client tokens must be validated against configured issuer and audience to prevent cross-client attacks
- Token exchange operations must follow RFC 8693 standards with proper scope mapping and validation

### HTTP Security Integration Requirements
- Internal authentication headers (X-Resource-*) must be automatically removed from incoming requests to prevent injection
- Authentication middleware must inject appropriate headers for downstream route handlers with validated information
- Canonical URL middleware must redirect to configured canonical URLs for SEO and security benefits
- Error responses must not leak sensitive authentication information while providing actionable error messages
- Session management must use secure cookie configuration with SameSite::Strict and appropriate security flags

## Authentication Extension Patterns

### Adding New Authentication Middleware
When creating new authentication middleware for specific use cases:

1. **Service Coordination Design**: Use RouterState dependency injection for consistent AppService access
2. **Error Handling Integration**: Create middleware-specific errors that implement AppError trait for consistent HTTP responses
3. **Header Management**: Follow established patterns for internal header injection and removal for security
4. **Token Validation**: Leverage DefaultTokenService for consistent token validation and caching strategies
5. **Testing Infrastructure**: Use comprehensive service mocking for isolated middleware testing scenarios

### Extending Authorization Patterns
For new authorization requirements and access control patterns:

1. **Role Hierarchy Integration**: Ensure new authorization logic follows established role hierarchy with has_access_to() validation
2. **Resource Scope Extensions**: Extend ResourceScope union type for new authorization contexts while maintaining precedence rules
3. **Cross-Service Consistency**: Coordinate with objs crate for new role and scope types to maintain domain consistency
4. **API Middleware Integration**: Design configurable authorization middleware that supports different role and scope requirements
5. **Authorization Testing**: Create comprehensive authorization testing scenarios with different role and scope combinations

### Token Service Extensions
For new token management capabilities and validation patterns:

1. **Token Type Support**: Extend token validation for new JWT token types while maintaining security standards
2. **Cache Strategy**: Design appropriate caching strategies for new token validation patterns with proper TTL and invalidation
3. **Database Integration**: Coordinate with DbService for new token storage and status tracking requirements
4. **External Integration**: Support new external client token patterns while maintaining issuer and audience validation
5. **Performance Optimization**: Implement efficient token validation patterns that minimize database and service calls