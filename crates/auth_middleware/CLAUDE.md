# CLAUDE.md

This file provides guidance to Claude Code when working with the `auth_middleware` crate.

See [crates/auth_middleware/PACKAGE.md](crates/auth_middleware/PACKAGE.md) for implementation details.

## Purpose

The `auth_middleware` crate serves as BodhiApp's HTTP authentication and authorization middleware layer, implementing JWT token validation, session management, multi-layered security controls, and extension-based auth context propagation with OAuth2 integration and role-based access control.

## Key Domain Architecture

### AuthContext Enum

The central type for authentication state propagation. Auth middleware validates credentials and injects `Extension<AuthContext>` into request extensions. Route handlers consume it via `Extension<AuthContext>` with pattern matching.

**Variants:**
- `Anonymous` -- No authentication present (used by `inject_optional_auth_info` when no valid credentials found)
- `Session { user_id, username, role: Option<ResourceRole>, token }` -- Browser session with JWT token from HTTP session storage
- `ApiToken { user_id, scope: TokenScope, token }` -- Database-backed API token (`bodhiapp_*` prefix) with scope-based access
- `ExternalApp { user_id, scope: UserScope, token, azp, access_request_id: Option<String> }` -- External OAuth client token after RFC 8693 token exchange

**Convenience methods:**
- `user_id() -> Option<&str>` -- Returns user ID for all authenticated variants, None for Anonymous
- `token() -> Option<&str>` -- Returns the token string for all authenticated variants, None for Anonymous
- `app_role() -> Option<AppRole>` -- Converts variant-specific role/scope into unified `AppRole` enum
- `is_authenticated() -> bool` -- Returns true for all variants except Anonymous

### Multi-Layer Authentication System

- **Session-Based Authentication**: HTTP session management with Tower Sessions integration and secure cookie handling
- **Bearer Token Authentication**: JWT token validation with OAuth2 compliance and external client token exchange
- **Dual Authentication Support**: Bearer token takes precedence over session-based authentication
- **Token Exchange Protocol**: RFC 8693 token exchange for external client integration
- **Same-Origin Validation**: Sec-Fetch-Site header validation for CSRF protection on session-based requests

### Authentication Middleware Functions

**`auth_middleware`** -- Strict authentication, rejects unauthenticated requests:
1. Strips user-sent `X-BodhiApp-*` headers (defense-in-depth)
2. Checks for bearer token in Authorization header, validates and constructs `AuthContext::ApiToken` or `AuthContext::ExternalApp`
3. Falls back to session token validation for same-origin requests, constructs `AuthContext::Session`
4. Returns `AuthError::InvalidAccess` if no valid authentication found
5. Inserts `AuthContext` into request extensions via `req.extensions_mut().insert(auth_context)`

**`inject_optional_auth_info`** -- Permissive authentication, allows unauthenticated requests:
1. Same validation logic as `auth_middleware`
2. On any failure, inserts `AuthContext::Anonymous` instead of returning an error
3. Cleans up invalid session data when token validation fails

### Header Security

The `remove_app_headers` function strips any incoming `X-BodhiApp-*` headers from requests to prevent header injection attacks. Uses `KEY_PREFIX_HEADER_BODHIAPP` constant for prefix matching. This is defense-in-depth since the old header-based transport has been replaced by extension-based `AuthContext`, but the stripping remains as a safety measure.

### JWT Token Management Architecture

- **Token Service Coordination**: `DefaultTokenService` handles token validation, refresh, and exchange operations
- **Multi-Token Type Support**: Session tokens, API tokens (`bodhiapp_*`), and external client tokens with different validation rules
- **Cache-Based Performance**: Token validation caching with expiration handling and automatic invalidation
- **Database Token Tracking**: API token status management with active/inactive state tracking and SHA-256 digest-based lookup
- **Claims Validation**: JWT claims validation with leeway handling, issuer verification, and audience validation

### Role-Based Authorization System

**`api_auth_middleware`** -- Route-level authorization consuming `AuthContext` from extensions:
- Reads `AuthContext` from request extensions (set by upstream auth middleware)
- Pattern matches on variant to apply appropriate authorization:
  - `Session`: Checks `role.has_access_to(&required_role)` using role hierarchy
  - `ApiToken`: Checks `scope.has_access_to(&required_token_scope)` if token scope is allowed
  - `ExternalApp`: Checks `scope.has_access_to(&required_user_scope)` if user scope is allowed
  - `Anonymous` or `Session { role: None }`: Returns `MissingAuth`
- Role hierarchy: Admin > Manager > PowerUser > User

### Toolset Authorization Architecture

**`toolset_auth_middleware`** -- Specialized middleware for toolset execution endpoints:
- Reads `AuthContext` from request extensions
- **Session flow**: Toolset ownership check + app-level type enabled + instance configured
- **OAuth flow** (`ExternalApp` with `access_request_id`): Access request validation + status check + app client match + user match + instance in approved list + type enabled + instance configured
- API tokens are blocked at route level before reaching this middleware

## Architecture Position

- **Above server_core and services**: Coordinates HTTP infrastructure and business services for authentication
- **Below route implementations**: Provides `AuthContext` extension for routes_app and routes_oai handlers
- **Integration with objs**: Uses domain types (`ResourceRole`, `TokenScope`, `UserScope`, `AppRole`, `ResourceScope`)
- **Handler consumption**: Route handlers extract `Extension<AuthContext>` and pattern match on variants instead of reading individual headers

## Cross-Crate Integration Patterns

### Service Layer Coordination
- **AuthService**: OAuth2 flows, token exchange, and refresh operations
- **SecretService**: JWT signing key management, app registration info, credential encryption
- **DbService**: API token storage, status tracking, digest-based lookup
- **CacheService**: Token validation and exchange result caching
- **SessionService**: HTTP session management with SQLite backend
- **ToolService**: Toolset ownership verification, type enablement checks

### Handler Integration Pattern
Route handlers consume auth context via Axum's extension extraction:
```rust
async fn my_handler(Extension(auth_context): Extension<AuthContext>) -> Response {
    match &auth_context {
        AuthContext::Session { user_id, role, .. } => { /* session logic */ }
        AuthContext::ApiToken { scope, .. } => { /* api token logic */ }
        AuthContext::ExternalApp { azp, .. } => { /* external app logic */ }
        AuthContext::Anonymous => { /* unauthenticated logic */ }
    }
}
```

## Test Utilities

Behind the `test-utils` feature flag in `auth_context.rs`:

**Factory methods on `AuthContext`:**
- `test_session(user_id, username, role)` -- Creates Session variant with default "test-token"
- `test_session_with_token(user_id, username, role, token)` -- Session with custom token
- `test_session_no_role(user_id, username)` -- Session with `role: None`
- `test_api_token(user_id, scope)` -- ApiToken with default "test-api-token"
- `test_external_app(user_id, scope, azp, access_request_id)` -- ExternalApp with default "test-external-token"

**`RequestAuthContextExt` trait** (on `Request<Body>`):
- `.with_auth_context(ctx)` -- Inserts `AuthContext` into request extensions for test setup

## Important Constraints

### Authentication Security Requirements
- JWT signatures and claims must be validated with proper leeway handling for clock skew
- Session-based authentication requires same-origin validation using Sec-Fetch-Site headers
- Bearer token authentication supports both internal API tokens and external client tokens
- Token refresh operations must atomically update session storage
- External client token exchange must validate issuer and audience claims

### Authorization Consistency Standards
- Role hierarchy (Admin > Manager > PowerUser > User) enforced via `has_access_to()`
- Resource scope authorization supports both `TokenScope` and `UserScope` with proper precedence
- Authorization middleware reads `AuthContext` from extensions -- it must run after auth middleware

### Token Management Security Rules
- Token digests use SHA-256 for secure database storage
- Token validation results are cached with appropriate TTL
- Database token status checked for all API tokens to support revocation
- External client tokens validated against configured issuer and audience

### HTTP Security Requirements
- `X-BodhiApp-*` headers stripped from incoming requests via `remove_app_headers` (defense-in-depth)
- Auth middleware inserts `AuthContext` as request extension, not headers
- Error responses must not leak sensitive authentication information
- Session management uses secure cookie configuration with SameSite::Strict

## Module Structure

- `auth_context.rs` -- `AuthContext` enum definition, convenience methods, test factories
- `auth_middleware.rs` -- `auth_middleware`, `inject_optional_auth_info`, `remove_app_headers`, `AuthError`, session key constants
- `api_auth_middleware.rs` -- `api_auth_middleware`, `ApiAuthError`, role/scope authorization logic
- `toolset_auth_middleware.rs` -- `toolset_auth_middleware`, `ToolsetAuthError`, toolset access validation
- `token_service.rs` -- `DefaultTokenService`, token validation/refresh/exchange orchestration
- `canonical_url_middleware.rs` -- Canonical URL redirection middleware
- `utils.rs` -- `app_status_or_default`, `generate_random_string`, `ApiErrorResponse`
- `test_utils/` -- Test infrastructure (behind `test-utils` feature)

## Exported Constants

- `SESSION_KEY_ACCESS_TOKEN`, `SESSION_KEY_REFRESH_TOKEN`, `SESSION_KEY_USER_ID` -- Session storage keys
- `KEY_PREFIX_HEADER_BODHIAPP` -- Prefix for internal header stripping ("X-BodhiApp-")
