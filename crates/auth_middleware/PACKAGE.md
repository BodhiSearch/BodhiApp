# PACKAGE.md -- auth_middleware

Implementation details and module index. For architecture and design, see `CLAUDE.md`.

## Module Structure

Entry point: `src/lib.rs` -- re-exports all public types.

| Module | Files | Purpose |
|--------|-------|---------|
| `auth_middleware/` | `auth_middleware.rs` | `auth_middleware`, `optional_auth_middleware`, `remove_app_headers`, `AuthError` |
| `api_auth_middleware` | `api_auth_middleware.rs` | `api_auth_middleware`, `ApiAuthError` |
| `access_request/` | `access_request_middleware.rs` | `access_request_auth_middleware`, `AccessRequestAuthError`, `AccessRequestValidator` trait, `ToolsetAccessRequestValidator`, `McpAccessRequestValidator` |
| `auth_context` | `auth_context.rs` | Re-export shim: `pub use services::AuthContext;` |
| `middleware_error` | `middleware_error.rs` | `MiddlewareError` struct with blanket `From<T: AppError>` |
| `token_service/` | `token_service.rs` | `DefaultTokenService`, `CachedExchangeResult` |
| `canonical_url_middleware` | `canonical_url_middleware.rs` | URL normalization (301 redirect for GET/HEAD) |
| `utils` | `utils.rs` | `app_status_or_default(TenantService)`, `generate_random_string()`, `ApiErrorResponse` |
| `test_utils/` | `auth_context.rs`, `auth_server_test_client.rs` | Test factories, `RequestAuthContextExt`, OAuth2 test client |

## Error Enums

All use `errmeta_derive::ErrorMeta` with `AppError` trait.

### AuthError (`src/auth_middleware/auth_middleware.rs`)
Token, Role, TokenScope, UserScope, MissingRoles, InvalidAccess, TokenInactive, TokenNotFound, AuthService, Tenant, DbError, RefreshTokenNotFound, TowerSession, InvalidToken, AppStatusInvalid

### ApiAuthError (`src/api_auth_middleware.rs`)
Forbidden, MissingAuth, InvalidRole, InvalidScope, InvalidUserScope

### AccessRequestAuthError (`src/access_request/access_request_middleware.rs`)
MissingAuth, EntityNotFound, AccessRequestNotFound, AccessRequestNotApproved, AccessRequestInvalid, EntityNotApproved

## Exported Constants

- `SESSION_KEY_ACCESS_TOKEN`, `SESSION_KEY_REFRESH_TOKEN`, `SESSION_KEY_USER_ID`
- `KEY_PREFIX_HEADER_BODHIAPP` (`"X-BodhiApp-"`)

## Commands

- `cargo test -p auth_middleware` -- all tests
- `cargo test -p auth_middleware test_live_auth_middleware` -- live OAuth2 tests
