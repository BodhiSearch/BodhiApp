# PACKAGE.md - auth_middleware

This document provides detailed technical information for the `auth_middleware` crate, focusing on BodhiApp's HTTP authentication and authorization middleware architecture, type-safe header extraction, sophisticated JWT token management, and multi-layered security implementation patterns.

## Module Structure

The crate is organized into these modules (see `crates/auth_middleware/src/lib.rs`):

- `auth_middleware` - Core authentication middleware, header constants, session handling, same-origin validation
- `api_auth_middleware` - Role and scope-based API authorization middleware
- `access_request_auth_middleware` - Generic access request validation middleware with `AccessRequestValidator` trait, `ToolsetAccessRequestValidator`, `McpAccessRequestValidator`, `AccessRequestAuthError`
- `toolset_auth_middleware` - Specialized authorization for toolset execution endpoints
- `auth_context` - `AuthContext` enum definition, convenience methods, test factory methods
- `token_service` - JWT token validation, refresh, exchange, and caching via `DefaultTokenService`
- `canonical_url_middleware` - SEO and security canonical URL redirection
- `utils` - Utility functions (app status, random string generation, error response types)
- `test_utils` - OAuth2 test client and integration test infrastructure (behind `test-utils` feature)

All modules are re-exported publicly from `lib.rs`.

## AuthContext Extension Pattern

Route handlers receive authentication context through `Extension<AuthContext>` from Axum request extensions, set by the auth middleware. This replaced the previous header-based extractor approach (`ExtractUserId`, `ExtractRole`, `ExtractToken`, etc.) which has been removed.

### AuthContext Variants

| Variant | Fields | Authenticated |
|---------|--------|:------------:|
| `Anonymous` | (none) | No |
| `Session` | `user_id`, `username`, `role: Option<ResourceRole>`, `token` | Yes |
| `ApiToken` | `user_id`, `scope: TokenScope`, `token` | Yes |
| `ExternalApp` | `user_id`, `scope: UserScope`, `token`, `azp`, `access_request_id: Option<String>` | Yes |

### Handler Pattern

```rust
async fn handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<..., ApiError> {
  let user_id = auth_context.user_id().expect("behind auth middleware");
}
```

### Test Injection

```rust
use auth_middleware::RequestAuthContextExt;
let request = Request::builder()
  .uri("/endpoint")
  .body(Body::empty())
  .unwrap()
  .with_auth_context(AuthContext::test_session("user", "name", ResourceRole::Admin));
```

## Internal Header Constants

All internal headers use `X-BodhiApp-` prefix via `bodhi_header!` macro (see `crates/auth_middleware/src/auth_middleware.rs`):

| Constant | Header Value | Injected By |
|----------|-------------|-------------|
| `KEY_HEADER_BODHIAPP_TOKEN` | `X-BodhiApp-Token` | auth_middleware |
| `KEY_HEADER_BODHIAPP_USERNAME` | `X-BodhiApp-Username` | auth_middleware |
| `KEY_HEADER_BODHIAPP_ROLE` | `X-BodhiApp-Role` | auth_middleware (session auth) |
| `KEY_HEADER_BODHIAPP_SCOPE` | `X-BodhiApp-Scope` | auth_middleware (bearer auth) |
| `KEY_HEADER_BODHIAPP_USER_ID` | `X-BodhiApp-User-Id` | auth_middleware |
| `KEY_HEADER_BODHIAPP_TOOL_SCOPES` | `X-BodhiApp-Tool-Scopes` | auth_middleware (bearer auth) |
| `KEY_HEADER_BODHIAPP_AZP` | `X-BodhiApp-Azp` | auth_middleware (bearer auth) |

The `remove_app_headers()` function strips all `X-BodhiApp-*` headers from incoming requests before re-injecting validated values, preventing header injection attacks.

## HTTP Authentication Middleware Architecture

### Dual Authentication Middleware Implementation
Sophisticated middleware architecture supporting multiple authentication patterns (see `crates/auth_middleware/src/auth_middleware.rs`):

```rust
// Strict authentication - rejects unauthenticated requests
pub async fn auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  _headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError>;

// Optional authentication - continues on auth failure
pub async fn optional_auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError>;
```

**Key Authentication Features**:
- Bearer token authentication takes precedence over session-based authentication for API compatibility
- Same-origin validation using Sec-Fetch-Site headers prevents CSRF attacks on session-based requests
- Internal header management prevents header injection attacks while providing validated information to routes
- Dual middleware approach supports both strict authentication and optional authentication patterns

### API Authorization Middleware Implementation
Fine-grained authorization middleware with configurable role and scope requirements (see `crates/auth_middleware/src/api_auth_middleware.rs`):

```rust
pub async fn api_auth_middleware(
  required_role: Role,
  required_token_scope: Option<TokenScope>,
  required_user_scope: Option<UserScope>,
  State(state): State<Arc<dyn RouterState>>,
  req: Request,
  next: Next,
) -> Result<Response, ApiError>;
```

**Authorization Architecture Features**:
- Role-based authorization follows hierarchical ordering with `has_access_to()` validation from objs crate
- ResourceScope union type seamlessly handles both TokenScope and UserScope authorization contexts
- Configurable authorization requirements allow different endpoints to specify different access levels
- Authorization precedence rules ensure role-based authorization takes precedence over scope-based when both are present

### Toolset Authorization Middleware
Specialized middleware for toolset execution endpoints (see `crates/auth_middleware/src/toolset_auth_middleware.rs`):

```rust
pub async fn toolset_auth_middleware(
  State(state): State<Arc<dyn RouterState>>,
  Path((id, method)): Path<(String, String)>,
  req: Request,
  next: Next,
) -> Result<Response, ApiError>;
```

Authorization checks vary by auth type:
- **Session auth** (has Role header): toolset ownership + app-level type enabled + toolset available
- **OAuth auth** (has `scope_user_` scope): above checks + app-client registered + toolset scope present in `X-BodhiApp-Tool-Scopes`

## JWT Token Management Architecture

### DefaultTokenService Implementation
Comprehensive token management service coordinating multiple authentication flows (see `crates/auth_middleware/src/token_service.rs`):

```rust
pub struct DefaultTokenService {
  auth_service: Arc<dyn AuthService>,
  secret_service: Arc<dyn SecretService>,
  cache_service: Arc<dyn CacheService>,
  db_service: Arc<dyn DbService>,
  setting_service: Arc<dyn SettingService>,
}
```

Key methods:
- `validate_bearer_token()` - Validates bearer tokens, checking database for API tokens first, then handling external client tokens
- `get_valid_session_token()` - Validates session tokens with automatic refresh on expiration
- `handle_internal_api_token()` - Cache-based validation for internal API tokens with refresh fallback
- `handle_external_client_token()` - External token validation with issuer/audience checks and RFC 8693 exchange

### Token Digest Security
Secure token storage using SHA-256 digests (see `crates/auth_middleware/src/token_service.rs`):

```rust
pub fn create_token_digest(bearer_token: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(bearer_token.as_bytes());
  format!("{:x}", hasher.finalize())[0..12].to_string()
}
```

## Security Infrastructure Implementation

### Same-Origin Validation
CSRF protection through security header validation (see `crates/auth_middleware/src/auth_middleware.rs`):
- `is_same_origin()` checks `Sec-Fetch-Site` header against `HOST` header
- Localhost requests require `same-origin` value; non-localhost requests are allowed

### Canonical URL Middleware
SEO and security URL normalization (see `crates/auth_middleware/src/canonical_url_middleware.rs`):
- Only redirects GET and HEAD requests to avoid breaking forms and APIs
- Skips health check and exempt paths
- Returns 301 Moved Permanently to configured canonical URL

## Error Architecture

### Error Types
Multiple error enums for different middleware concerns:

- `AuthError` (see `crates/auth_middleware/src/auth_middleware.rs`) - Authentication failures: InvalidAccess, RefreshTokenNotFound, TokenInactive, TowerSession
- `ApiAuthError` (see `crates/auth_middleware/src/api_auth_middleware.rs`) - Authorization failures: Forbidden, MissingAuth
- `AccessRequestAuthError` (see `crates/auth_middleware/src/access_request_auth_middleware.rs`) - Access request authorization: MissingAuth, EntityNotFound, EntityNotApproved, AccessRequestNotFound, AccessRequestNotApproved, AppClientMismatch, UserMismatch, AccessRequestInvalid, InvalidApprovedJson
- `ToolsetAuthError` (see `crates/auth_middleware/src/toolset_auth_middleware.rs`) - Toolset authorization: MissingUserId, MissingAuth, AppClientNotRegistered, MissingToolsetScope, ToolsetNotFound

All error types use `thiserror` for message formatting and `errmeta_derive::ErrorMeta` with `AppError` trait implementation for consistent HTTP response generation.

## Utility Functions

Core utility functions (see `crates/auth_middleware/src/utils.rs`):
- `app_status_or_default()` - Retrieves app status with graceful fallback to Setup status
- `generate_random_string()` - Cryptographically secure random string generation for tokens
- `ApiErrorResponse` - Consistent error response struct for HTTP error handling

## Testing Infrastructure

### Unit Tests
- Auth middleware tests (see `crates/auth_middleware/src/auth_middleware.rs`) - Session and bearer token flows with mock services
- API auth middleware tests (see `crates/auth_middleware/src/api_auth_middleware.rs`) - Role/scope authorization combinations
- Toolset auth middleware tests (see `crates/auth_middleware/src/toolset_auth_middleware.rs`) - Session and OAuth auth paths with `MockToolService`

### OAuth2 Test Infrastructure
The `test_utils` module provides OAuth2 integration testing (behind `test-utils` feature):
- `AuthServerConfig` / `AuthServerTestClient` (see `crates/auth_middleware/src/test_utils/auth_server_test_client.rs`) - Complete OAuth2 client setup workflow with dynamic client creation
- Live integration tests (see `crates/auth_middleware/tests/test_live_auth_middleware.rs`) - Requires live OAuth2 server

## Commands

**Testing**: `cargo test -p auth_middleware` (includes authentication flow and middleware testing)
**Integration Testing**: `cargo test -p auth_middleware --features test-utils` (includes OAuth2 integration test client)
**Live Testing**: `cargo test -p auth_middleware test_live_auth_middleware` (requires live OAuth2 server configuration)
