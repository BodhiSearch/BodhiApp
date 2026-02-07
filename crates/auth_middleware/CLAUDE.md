# CLAUDE.md

This file provides guidance to Claude Code when working with the `auth_middleware` crate.

See [crates/auth_middleware/PACKAGE.md](crates/auth_middleware/PACKAGE.md) for implementation details.

## Purpose

The `auth_middleware` crate serves as BodhiApp's **HTTP authentication and authorization middleware layer**, implementing sophisticated JWT token validation, session management, multi-layered security controls, and type-safe header extraction with comprehensive OAuth2 integration and role-based access control.

## Key Domain Architecture

### Multi-Layer Authentication System
Sophisticated authentication architecture supporting multiple authentication flows:
- **Session-Based Authentication**: HTTP session management with Tower Sessions integration and secure cookie handling
- **Bearer Token Authentication**: JWT token validation with OAuth2 compliance and external client token exchange
- **Dual Authentication Support**: Seamless handling of both session tokens and API bearer tokens with precedence rules
- **Token Exchange Protocol**: RFC 8693 token exchange for external client integration and service-to-service authentication
- **Same-Origin Validation**: Security header validation for CSRF protection and request origin verification

### Type-Safe Header Extraction System
Axum `FromRequestParts` extractors that provide compile-time enforcement of header requirements in route handlers:
- **Design Rationale**: Before extractors, ~25 route handlers each manually extracted internal headers using `BadRequestError` with inconsistent validation logic. The extractor system centralizes this into 7 reusable types with uniform validation behavior.
- **Required vs Optional Pattern**: Required extractors (`ExtractToken`, `ExtractUserId`, `ExtractUsername`, `ExtractRole`, `ExtractScope`) reject requests with 400 Bad Request when headers are missing. Optional extractors (`MaybeToken`, `MaybeRole`) return `None` for missing/empty headers, enabling routes that work with or without authentication context.
- **Typed Parsing**: `ExtractRole` and `MaybeRole` go beyond string extraction -- they parse the header value into `ResourceRole` using `FromStr`, providing domain-validated types directly to handler functions instead of raw strings.
- **Validation Pipeline**: Every extractor follows a consistent three-step validation: header present -> `to_str()` succeeds (valid UTF-8) -> non-empty string. Typed extractors add a fourth step: `FromStr` parsing into the domain type.
- **Error Consolidation**: `HeaderExtractionError` enum (Missing, Invalid, Empty variants) replaces scattered `BadRequestError` usage with structured, descriptive error messages that include the header name and failure reason.

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

### Toolset Authorization Architecture
Specialized middleware for toolset execution endpoints with dual auth-type support:
- **Session vs OAuth Discrimination**: Determines auth type by checking for `X-BodhiApp-Role` header (session) vs `scope_user_` prefix in scope header (OAuth)
- **Multi-Step Authorization Chain**: Toolset ownership -> app-level type enabled -> (OAuth only: app-client registered + toolset scope present) -> toolset available and configured
- **API Token Exclusion**: API tokens (`bodhiapp_*`) are blocked at route level before reaching toolset middleware, simplifying internal logic

### Security Infrastructure Architecture
Multi-layered security controls with comprehensive threat protection:
- **Canonical URL Middleware**: Automatic redirection to canonical URLs with SEO optimization and security benefits
- **Request Origin Validation**: Same-origin request validation using Sec-Fetch-Site headers for CSRF protection
- **Token Digest Security**: SHA-256 token digests for secure database storage and lookup without exposing tokens
- **External Client Security**: Secure external client token validation with issuer and audience verification
- **Internal Header Namespace**: All internal headers use `X-BodhiApp-` prefix via `bodhi_header!` macro, with automatic stripping of user-provided values to prevent header injection

## Architecture Position

The `auth_middleware` crate serves as BodhiApp's **HTTP security orchestration layer**:
- **Above server_core and services**: Coordinates HTTP infrastructure and business services for authentication operations
- **Below route implementations**: Provides authentication, authorization, and header extraction foundation for routes_oai and routes_app
- **Integration with objs**: Uses domain objects (`ResourceRole`, `Role`, `ResourceScope`, `TokenScope`, `UserScope`) for role, scope, and error handling with consistent validation
- **Cross-cutting security**: Implements security boundaries across all HTTP endpoints with comprehensive middleware integration
- **Extractor consumption**: Route handlers in `routes_app` (e.g., `routes_access_request`, `routes_api_token`, `routes_users_list`, `routes_toolsets`) use extractors as handler parameters, replacing manual header extraction with type-safe destructuring

## Cross-Crate Integration Patterns

### Service Layer Authentication Coordination
Complex authentication flows coordinated across BodhiApp's service layer:
- **AuthService Integration**: OAuth2 flows, token exchange, and refresh operations with comprehensive error handling
- **SecretService Coordination**: JWT signing key management, app registration info, and credential encryption
- **DbService Token Management**: API token storage, status tracking, and digest-based lookup with transaction support
- **CacheService Performance**: Token validation caching, exchange result caching, and automatic invalidation strategies
- **SessionService Integration**: HTTP session management with SQLite backend and secure cookie configuration
- **ToolService Integration**: Toolset ownership verification, type enablement checks, and app-client registration validation

### HTTP Infrastructure Integration
Authentication middleware coordinates with HTTP infrastructure for request processing:
- **RouterState Integration**: Dependency injection providing access to AppService registry for authentication operations
- **Request Header Management**: Automatic injection and removal of internal authentication headers for security
- **Error Translation**: Service errors converted to appropriate HTTP status codes with user-friendly messages
- **Middleware Composition**: Layered middleware architecture with auth_middleware, inject_session_auth_info, and canonical_url_middleware
- **Extractor-to-Handler Flow**: Auth middleware injects `X-BodhiApp-*` headers into requests; extractors in route handlers read and validate those headers, creating a clean separation between authentication (middleware) and consumption (handler parameters)
- **OpenAI API Security**: Bearer token authentication for OpenAI-compatible endpoints through routes_oai integration
- **Application API Security**: Session-based and bearer token authentication for application endpoints through routes_app integration
- **API Token Management**: Database-backed API token validation for external client access to OpenAI endpoints
- **Route Composition Integration**: Coordinated with routes_all for comprehensive middleware orchestration with hierarchical authorization and multi-layer security

### Domain Object Integration
Extensive use of objs crate for authentication and authorization:
- **Role and Scope Types**: Hierarchical role validation and resource scope authorization with consistent domain logic
- **Error System Integration**: Authentication errors implement AppError trait for consistent HTTP response generation
- **ResourceRole in Extractors**: `ExtractRole` and `MaybeRole` parse header strings directly into `ResourceRole` domain type, eliminating raw string handling in route handlers
- **ResourceScope Union**: Seamless handling of both TokenScope and UserScope authorization contexts
- **Validation Consistency**: Domain object validation rules applied consistently across authentication flows and extractor parsing

## Authentication Flow Orchestration

### Multi-Flow Authentication Architecture
Sophisticated authentication coordination supporting multiple authentication patterns:

**Session-Based Authentication Flow**:
1. **Same-Origin Validation**: Sec-Fetch-Site header validation for CSRF protection and request origin verification
2. **Session Token Retrieval**: Access token extraction from HTTP session storage with Tower Sessions integration
3. **Token Validation**: JWT token validation with claims verification and expiration checking
4. **Automatic Token Refresh**: Expired token refresh using refresh tokens with seamless session updates
5. **Header Injection**: Role, username, user ID, and token injected as `X-BodhiApp-*` headers for downstream extractors

**Bearer Token Authentication Flow**:
1. **Token Extraction**: Bearer token extraction from Authorization header with format validation
2. **Database Token Lookup**: API token status verification with digest-based lookup and active/inactive checking
3. **External Client Handling**: External client token validation with issuer and audience verification
4. **Token Exchange**: RFC 8693 token exchange for external clients with scope validation and caching
5. **Header Injection**: Scope, user ID, azp, and tool scopes injected as `X-BodhiApp-*` headers for downstream extractors

**Header Extraction in Route Handlers** (downstream of middleware):
- Required extractors cause 400 Bad Request if the middleware did not inject the expected header
- Optional extractors gracefully handle absent headers for routes that support unauthenticated access
- Typed extractors provide parsed domain objects directly in handler function signatures

**Dual Authentication Support**:
- **Precedence Rules**: Bearer token authentication takes precedence over session-based authentication
- **Header Management**: Automatic injection of X-BodhiApp-Token, X-BodhiApp-Role, X-BodhiApp-Scope, X-BodhiApp-Username, X-BodhiApp-User-Id headers
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

### Header Extraction Consistency
- All route handlers must use extractor types from `extractors.rs` instead of manual `req.headers().get()` calls with ad-hoc error handling
- Required extractors must be used when the authentication middleware guarantees the header will be present (e.g., behind `auth_middleware`)
- Optional extractors must be used for routes behind `inject_optional_auth_info` where authentication may not be present
- Typed extractors (`ExtractRole`, `MaybeRole`) must be preferred over string extractors when the header value represents a domain type
- New internal headers should follow the `X-BodhiApp-` prefix convention via `bodhi_header!` macro and get corresponding extractors

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
- Internal authentication headers (X-BodhiApp-*) must be automatically removed from incoming requests to prevent injection
- Authentication middleware must inject appropriate headers for downstream route handlers with validated information
- Canonical URL middleware must redirect to configured canonical URLs for SEO and security benefits
- Error responses must not leak sensitive authentication information while providing actionable error messages
- Session management must use secure cookie configuration with SameSite::Strict and appropriate security flags

## Authentication Extension Patterns

### Adding New Extractors
When adding extractors for new internal headers:

1. **Define the header constant** using `bodhi_header!("HeaderName")` in `auth_middleware.rs`
2. **Choose required vs optional** based on whether the header is guaranteed by upstream middleware
3. **Implement `FromRequestParts`** using `extract_required_header` or `extract_optional_header` helper functions
4. **Add typed parsing** if the header value represents a domain type with `FromStr` implementation
5. **Add unit tests** following the existing pattern: test present, test missing, test invalid (for typed extractors)
6. **Ensure middleware injection**: Verify that `auth_middleware` or `inject_optional_auth_info` injects the header before the extractor will consume it

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
