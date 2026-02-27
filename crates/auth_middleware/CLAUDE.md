# CLAUDE.md

This file provides guidance to Claude Code when working with the `auth_middleware` crate.

See [crates/auth_middleware/PACKAGE.md](crates/auth_middleware/PACKAGE.md) for implementation details.

## Purpose

The `auth_middleware` crate serves as BodhiApp's HTTP authentication and authorization middleware layer, implementing JWT token validation, session management, multi-layered security controls, and extension-based auth context propagation with OAuth2 integration and role-based access control.

## Key Domain Architecture

### AuthContext Enum

The central type for authentication state propagation. Auth middleware validates credentials and injects `Extension<AuthContext>` into request extensions. Route handlers consume it via `Extension<AuthContext>` with pattern matching.

**Variants:**
- `Anonymous` -- No authentication present (used by `optional_auth_middleware` when no valid credentials found)
- `Session { user_id, username, role: Option<ResourceRole>, token }` -- Browser session with JWT token from HTTP session storage
- `ApiToken { user_id, role: TokenScope, token }` -- Database-backed API token (`bodhiapp_*` prefix) with scope-based access
- `ExternalApp { user_id, role: Option<UserScope>, token, external_app_token, app_client_id, access_request_id: Option<String> }` -- External OAuth client token after RFC 8693 token exchange. `token` is the exchanged (local) token; `external_app_token` is the original token from the external client. `role` is derived from the DB `approved_role` on the access request, not from JWT scope claims.

**Convenience methods:**
- `user_id() -> Option<&str>` -- Returns user ID for all authenticated variants, None for Anonymous
- `token() -> Option<&str>` -- Returns the token string for all authenticated variants, None for Anonymous
- `external_app_token() -> Option<&str>` -- Returns the original external client token for ExternalApp, None for other variants
- `app_role() -> Option<AppRole>` -- Converts variant-specific role into unified `AppRole` enum; returns None for Anonymous, Session with no role, or ExternalApp with no role
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

**`optional_auth_middleware`** -- Permissive authentication, allows unauthenticated requests:
1. Same validation logic as `auth_middleware`
2. On any failure, inserts `AuthContext::Anonymous` instead of returning an error
3. Cleans up invalid session data when token validation fails

### Header Security

The `remove_app_headers` function strips any incoming `X-BodhiApp-*` headers from requests to prevent header injection attacks. Uses `KEY_PREFIX_HEADER_BODHIAPP` constant for prefix matching. This is defense-in-depth since the old header-based transport has been replaced by extension-based `AuthContext`, but the stripping remains as a safety measure.

### JWT Token Management Architecture

- **Token Service Coordination**: `DefaultTokenService` handles token validation, refresh, and exchange operations. Constructor accepts `Arc<dyn TimeService>` for testable timestamp operations -- no direct `Utc::now()` calls.
- **Multi-Token Type Support**: Session tokens, API tokens (`bodhiapp_*`), and external client tokens with different validation rules
- **Cache-Based Performance**: Token exchange results cached via `CachedExchangeResult` with `role: Option<String>` and `access_request_id: Option<String>` for quick lookup
- **Database Token Tracking**: API token status management with active/inactive state tracking and SHA-256 hash-based lookup
- **Claims Validation**: JWT claims validation with leeway handling, issuer verification, and audience validation
- **Access Request Validation**: External token exchange validates access request scope, status, app client match, user match, and access request ID consistency before building `AuthContext::ExternalApp`

### Role Derivation for External Apps

The `role` field on `AuthContext::ExternalApp` is derived from the database's `approved_role` column on the access request record (via `CachedExchangeResult.role`), not from JWT scope claims. When no approved access request is found or the `approved_role` is absent, `role` is `None`, and the `api_auth_middleware` rejects the request with `MissingAuth`.

### Role-Based Authorization System

**`api_auth_middleware`** -- Route-level authorization consuming `AuthContext` from extensions:
- Reads `AuthContext` from request extensions (set by upstream auth middleware)
- Pattern matches on variant to apply appropriate authorization:
  - `Session { role: Some(role) }`: Checks `role.has_access_to(&required_role)` using role hierarchy
  - `ApiToken { role }`: Checks `role.has_access_to(&required_token_scope)` if token scope parameter is provided
  - `ExternalApp { role: Some(role) }`: Checks `role.has_access_to(&required_user_scope)` if user scope parameter is provided
  - `Anonymous`, `Session { role: None }`, or `ExternalApp { role: None }`: Returns `MissingAuth`
- Role hierarchy: Admin > Manager > PowerUser > User

### Access Request Authorization Architecture

**`access_request_auth_middleware`** -- Generic middleware for validating entity access against approved resources in access requests (see `access_request_auth_middleware/middleware.rs`):
- `AccessRequestValidator` trait with two methods: `extract_entity_id(path)` and `validate_approved(approved_json, entity_id)`
- Two implementations: `ToolsetAccessRequestValidator` (validates toolset instance in `approved.toolsets`), `McpAccessRequestValidator` (validates MCP instance in `approved.mcps`)
- Deserializes `approved` JSON from access request into `objs::ApprovedResources` struct
- `AccessRequestAuthError` enum with variants like `EntityNotApproved`, `AccessRequestNotFound`, `AppClientMismatch` etc.
- **Session flow**: passes through (session users have direct access)
- **OAuth flow** (`ExternalApp` with `access_request_id`): validates access request status, app client ID match, user match, then delegates to validator for entity-level approval check

## Architecture Position

**Upstream dependencies** (crates this depends on):
- [`objs`](../objs/CLAUDE.md) -- domain types (`ResourceRole`, `TokenScope`, `UserScope`, `AppRole`, `ErrorType`)
- [`services`](../services/CLAUDE.md) -- `AuthService`, `DbService`, `CacheService`, `SessionService`, `AppInstanceService`, `ConcurrencyService`, `SettingService`
- [`server_core`](../server_core/CLAUDE.md) -- `RouterState` for middleware integration

**Downstream consumers** (crates that depend on this):
- [`routes_app`](../routes_app/CLAUDE.md) -- route handlers consume `Extension<AuthContext>` from middleware
- [`server_app`](../server_app/CLAUDE.md) -- composes auth middleware into the server's router
- [`lib_bodhiserver`](../lib_bodhiserver/CLAUDE.md) -- composes auth middleware into embedded server

## Cross-Crate Integration Patterns

### Service Layer Coordination
- **AuthService**: OAuth2 flows, token exchange, and refresh operations
- **AppInstanceService**: App registration info (client_id, client_secret) for token validation
- **DbService**: API token storage, status tracking, prefix-based lookup, access request scope lookup
- **CacheService**: Token exchange result caching
- **SessionService**: HTTP session management with SQLite backend
- **SettingService**: Auth issuer configuration for external token validation
- **ConcurrencyService**: Distributed lock for session token refresh (prevents concurrent refresh storms)

### Handler Integration Pattern
Route handlers consume auth context via Axum's extension extraction:
```rust
async fn my_handler(Extension(auth_context): Extension<AuthContext>) -> Response {
    match &auth_context {
        AuthContext::Session { user_id, role, .. } => { /* session logic */ }
        AuthContext::ApiToken { role, .. } => { /* api token logic */ }
        AuthContext::ExternalApp { app_client_id, role, .. } => { /* external app logic */ }
        AuthContext::Anonymous => { /* unauthenticated logic */ }
    }
}
```

## Test Utilities

Behind the `test-utils` feature flag in `test_utils/auth_context.rs`:

**Factory methods on `AuthContext`:**
- `test_session(user_id, username, role)` -- Creates Session variant with default "test-token"
- `test_session_with_token(user_id, username, role, token)` -- Session with custom token
- `test_session_no_role(user_id, username)` -- Session with `role: None`
- `test_api_token(user_id, role)` -- ApiToken with default "test-api-token"
- `test_external_app(user_id, role, app_client_id, access_request_id)` -- ExternalApp with `role: Some(role)` and default tokens
- `test_external_app_no_role(user_id, app_client_id, access_request_id)` -- ExternalApp with `role: None` and default tokens

**`RequestAuthContextExt` trait** (on `Request<Body>`):
- `.with_auth_context(ctx)` -- Inserts `AuthContext` into request extensions for test setup

## Important Constraints

### Authentication Security Requirements
- JWT signatures and claims must be validated with proper leeway handling for clock skew
- Session-based authentication requires same-origin validation using Sec-Fetch-Site headers
- Bearer token authentication supports both internal API tokens and external client tokens
- Token refresh operations must atomically update session storage with distributed lock protection
- External client token exchange must validate issuer and audience claims

### Authorization Consistency Standards
- Role hierarchy (Admin > Manager > PowerUser > User) enforced via `has_access_to()`
- Authorization middleware reads `AuthContext` from extensions -- it must run after auth middleware
- `ExternalApp` with `role: None` is treated as unauthenticated by `api_auth_middleware`

### Token Management Security Rules
- API tokens use SHA-256 full hash for database storage and constant-time comparison
- Token exchange results are cached with the exchanged token digest as key
- Database token status checked for all API tokens to support revocation
- External client tokens validated against configured issuer and audience

### HTTP Security Requirements
- `X-BodhiApp-*` headers stripped from incoming requests via `remove_app_headers` (defense-in-depth)
- Auth middleware inserts `AuthContext` as request extension, not headers
- Error responses must not leak sensitive authentication information
- Session management uses secure cookie configuration with SameSite::Strict

## Module Structure

- `auth_context.rs` -- `AuthContext` enum definition, convenience methods
- `auth_middleware/` -- `auth_middleware`, `optional_auth_middleware`, `remove_app_headers`, `AuthError`, session key constants
- `api_auth_middleware.rs` -- `api_auth_middleware`, `ApiAuthError`, role-based authorization logic
- `access_request_auth_middleware/` -- `access_request_auth_middleware`, `AccessRequestAuthError`, `AccessRequestValidator` trait, `ToolsetAccessRequestValidator`, `McpAccessRequestValidator`
- `token_service/` -- `DefaultTokenService` (takes `Arc<dyn TimeService>` in constructor), `CachedExchangeResult`, token validation/refresh/exchange orchestration
- `canonical_url_middleware.rs` -- Canonical URL redirection middleware
- `utils.rs` -- `app_status_or_default`, `generate_random_string`, `ApiErrorResponse`
- `test_utils/` -- Test infrastructure (behind `test-utils` feature): `auth_context.rs` (factory methods, `RequestAuthContextExt`), `auth_server_test_client.rs` (OAuth2 integration test client)

## Exported Constants

- `SESSION_KEY_ACCESS_TOKEN`, `SESSION_KEY_REFRESH_TOKEN`, `SESSION_KEY_USER_ID` -- Session storage keys
- `KEY_PREFIX_HEADER_BODHIAPP` -- Prefix for internal header stripping ("X-BodhiApp-")
