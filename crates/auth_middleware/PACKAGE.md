# PACKAGE.md - auth_middleware

This document provides detailed technical information for the `auth_middleware` crate, focusing on BodhiApp's HTTP authentication and authorization middleware architecture, extension-based auth context propagation, JWT token management, and multi-layered security implementation patterns.

## Module Structure

The crate is organized into these modules (see `crates/auth_middleware/src/lib.rs`):

- `auth_middleware/` - Core authentication middleware, header constants, session handling, same-origin validation (`middleware.rs`, `tests.rs`)
- `api_auth_middleware` - Role-based API authorization middleware
- `access_request_auth_middleware/` - Generic access request validation middleware with `AccessRequestValidator` trait, `ToolsetAccessRequestValidator`, `McpAccessRequestValidator`, `AccessRequestAuthError` (`middleware.rs`, `tests.rs`)
- `auth_context` - `AuthContext` enum definition, convenience methods
- `token_service/` - JWT token validation, refresh, exchange, and caching via `DefaultTokenService` (`service.rs`, `tests.rs`)
- `canonical_url_middleware` - SEO and security canonical URL redirection
- `utils` - Utility functions (app status, random string generation, error response types)
- `test_utils/` - OAuth2 test client and integration test infrastructure (behind `test-utils` feature): `auth_context.rs`, `auth_server_test_client.rs`

All modules are re-exported publicly from `lib.rs`.

## AuthContext Extension Pattern

Route handlers receive authentication context through `Extension<AuthContext>` from Axum request extensions, set by the auth middleware.

### AuthContext Variants

| Variant | Fields | Authenticated |
|---------|--------|:------------:|
| `Anonymous` | (none) | No |
| `Session` | `user_id`, `username`, `role: Option<ResourceRole>`, `token` | Yes |
| `ApiToken` | `user_id`, `role: TokenScope`, `token` | Yes |
| `ExternalApp` | `user_id`, `role: Option<UserScope>`, `token`, `external_app_token`, `app_client_id`, `access_request_id: Option<String>` | Yes |

The `ExternalApp` variant carries two tokens: `token` is the exchanged (local) access token used for downstream operations; `external_app_token` is the original bearer token from the external client. The `role` is derived from the DB `approved_role`, not from JWT scope claims. When `role` is `None`, `api_auth_middleware` rejects the request.

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

## HTTP Authentication Middleware Architecture

### Dual Authentication Middleware Implementation
Middleware architecture supporting multiple authentication patterns (see `crates/auth_middleware/src/auth_middleware/middleware.rs`):

```rust
pub async fn auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError>;

pub async fn optional_auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError>;
```

**Key Authentication Features**:
- Bearer token authentication takes precedence over session-based authentication
- Same-origin validation using Sec-Fetch-Site headers prevents CSRF attacks on session-based requests
- `remove_app_headers` strips `X-BodhiApp-*` headers from incoming requests to prevent injection
- `optional_auth_middleware` inserts `AuthContext::Anonymous` on any auth failure instead of returning an error
- Session cleanup on token validation failure (refresh not found, expired, auth service error)

### API Authorization Middleware Implementation
Role-based authorization middleware consuming `AuthContext` from extensions (see `crates/auth_middleware/src/api_auth_middleware.rs`):

```rust
pub async fn api_auth_middleware(
  required_role: ResourceRole,
  required_token_scope: Option<TokenScope>,
  required_user_scope: Option<UserScope>,
  State(state): State<Arc<dyn RouterState>>,
  req: Request,
  next: Next,
) -> Result<Response, ApiError>;
```

**Authorization logic by AuthContext variant:**
- `Session { role: Some(role) }` -- checks `role.has_access_to(&required_role)`
- `Session { role: None }` -- returns `MissingAuth`
- `ApiToken { role }` -- checks `role.has_access_to(&required_token_scope)` if token scope provided, otherwise `MissingAuth`
- `ExternalApp { role: Some(role) }` -- checks `role.has_access_to(&required_user_scope)` if user scope provided, otherwise `MissingAuth`
- `ExternalApp { role: None }` -- returns `MissingAuth`
- `Anonymous` -- returns `MissingAuth`

## JWT Token Management Architecture

### DefaultTokenService Implementation
Token management service coordinating multiple authentication flows (see `crates/auth_middleware/src/token_service/service.rs`):

```rust
pub struct DefaultTokenService {
  auth_service: Arc<dyn AuthService>,
  app_instance_service: Arc<dyn AppInstanceService>,
  cache_service: Arc<dyn CacheService>,
  db_service: Arc<dyn DbService>,
  setting_service: Arc<dyn SettingService>,
  concurrency_service: Arc<dyn ConcurrencyService>,
}
```

Key methods:
- `validate_bearer_token()` -- Entry point for bearer token validation. Routes to API token path (`bodhiapp_*` prefix) or external client token path
- `get_valid_session_token()` -- Validates session tokens with automatic refresh on expiration, using distributed lock via `ConcurrencyService` to prevent concurrent refresh storms
- `handle_external_client_token()` -- Validates external token (issuer, audience), looks up access request by scope, validates status/client/user match, performs RFC 8693 token exchange, derives `role` from DB `approved_role`

### CachedExchangeResult
Cached representation of an external token exchange (see `crates/auth_middleware/src/token_service/service.rs`):

```rust
pub struct CachedExchangeResult {
  pub token: String,
  pub app_client_id: String,
  pub role: Option<String>,
  pub access_request_id: Option<String>,
}
```

Cached under key `exchanged_token:{token_digest}` where digest is first 12 chars of SHA-256 hex. On cache hit, the exchanged token's expiry is validated before reuse; expired entries trigger a fresh exchange.

### API Token Validation
API tokens (`bodhiapp_*` prefix) are validated by:
1. Extracting prefix (first 8 chars after `bodhiapp_`) for DB lookup
2. Checking token status is `Active`
3. Computing full SHA-256 hash and comparing with constant-time equality
4. Parsing `scopes` field into `TokenScope`

## Security Infrastructure Implementation

### Same-Origin Validation
CSRF protection through security header validation (see `crates/auth_middleware/src/auth_middleware/middleware.rs`):
- `is_same_origin()` checks `Sec-Fetch-Site` header against `HOST` header
- Localhost requests require `same-origin` value; non-localhost requests are allowed

### Header Security
- `remove_app_headers()` strips all headers with `X-BodhiApp-` prefix using case-insensitive matching
- `KEY_PREFIX_HEADER_BODHIAPP` constant defines the prefix

### Canonical URL Middleware
SEO and security URL normalization (see `crates/auth_middleware/src/canonical_url_middleware.rs`):
- Only redirects GET and HEAD requests to avoid breaking forms and APIs
- Skips health check and exempt paths
- Returns 301 Moved Permanently to configured canonical URL

## Error Architecture

### Error Types
Multiple error enums for different middleware concerns:

- `AuthError` (see `crates/auth_middleware/src/auth_middleware/middleware.rs`) - Authentication failures: Token, Role, TokenScope, UserScope, MissingRoles, InvalidAccess, TokenInactive, TokenNotFound, AuthService, AppInstance, DbError, RefreshTokenNotFound, TowerSession, InvalidToken, AppStatusInvalid
- `ApiAuthError` (see `crates/auth_middleware/src/api_auth_middleware.rs`) - Authorization failures: Forbidden, MissingAuth, InvalidRole, InvalidScope, InvalidUserScope
- `AccessRequestAuthError` (see `crates/auth_middleware/src/access_request_auth_middleware/middleware.rs`) - Access request authorization: MissingAuth, EntityNotFound, EntityNotApproved, AccessRequestNotFound, AccessRequestNotApproved, AppClientMismatch, UserMismatch, AccessRequestInvalid, InvalidApprovedJson

All error types use `thiserror` for message formatting and `errmeta_derive::ErrorMeta` with `AppError` trait implementation for consistent HTTP response generation.

## Utility Functions

Core utility functions (see `crates/auth_middleware/src/utils.rs`):
- `app_status_or_default()` - Retrieves app status with graceful fallback to Setup status
- `generate_random_string()` - Cryptographically secure random string generation for tokens
- `ApiErrorResponse` - Consistent error response struct for HTTP error handling

## Testing Infrastructure

### Unit Tests
- Auth middleware tests (see `crates/auth_middleware/src/auth_middleware/tests.rs`) - Session and bearer token flows
- API auth middleware tests (see `crates/auth_middleware/src/api_auth_middleware.rs`) - Role/scope authorization combinations including `ExternalApp { role: None }` rejection
- Access request auth middleware tests (see `crates/auth_middleware/src/access_request_auth_middleware/tests.rs`) - Session and OAuth auth paths

### Test Factory Methods
Behind `test-utils` feature flag (see `crates/auth_middleware/src/test_utils/auth_context.rs`):
- `AuthContext::test_session(user_id, username, role)` -- Session with `role: Some(role)`, default "test-token"
- `AuthContext::test_session_no_role(user_id, username)` -- Session with `role: None`
- `AuthContext::test_session_with_token(user_id, username, role, token)` -- Session with custom token
- `AuthContext::test_api_token(user_id, role)` -- ApiToken with default "test-api-token"
- `AuthContext::test_external_app(user_id, role, app_client_id, access_request_id)` -- ExternalApp with `role: Some(role)`, default tokens
- `AuthContext::test_external_app_no_role(user_id, app_client_id, access_request_id)` -- ExternalApp with `role: None`, default tokens
- `RequestAuthContextExt::with_auth_context(ctx)` -- Inserts `AuthContext` into request extensions

### OAuth2 Test Infrastructure
The `test_utils` module provides OAuth2 integration testing (behind `test-utils` feature):
- `AuthServerConfig` / `AuthServerTestClient` (see `crates/auth_middleware/src/test_utils/auth_server_test_client.rs`) - Complete OAuth2 client setup workflow with dynamic client creation
- `ClientInfo` -- Holds `client_id` and `client_secret: Option<String>`
- `DynamicClients` -- Holds `app_client` and `resource_client` as `ClientInfo`
- `setup_dynamic_clients()` -- Creates app client, resource client, and makes test user admin
- Live integration tests (see `crates/auth_middleware/tests/test_live_auth_middleware.rs`) - Requires live OAuth2 server

## Commands

**Testing**: `cargo test -p auth_middleware` (includes authentication flow and middleware testing)
**Integration Testing**: `cargo test -p auth_middleware --features test-utils` (includes OAuth2 integration test client)
**Live Testing**: `cargo test -p auth_middleware test_live_auth_middleware` (requires live OAuth2 server configuration)
