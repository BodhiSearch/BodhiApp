# PACKAGE.md -- routes_app::middleware

Implementation details and module index. For architecture and design, see `CLAUDE.md` in this directory.

Previously the standalone `auth_middleware` crate. Merged into `routes_app` as an internal module.

## Module Structure

Entry point: `mod.rs` -- re-exports all public types.

| Module | Files | Purpose |
|--------|-------|---------|
| `auth/` | `auth_middleware.rs` | `auth_middleware`, `optional_auth_middleware`, `remove_app_headers`, `AuthError` |
| `apis/` | `api_middleware.rs` | `api_auth_middleware`, `ApiAuthError` |
| `access_requests/` | `access_request_middleware.rs` | `access_request_auth_middleware`, `AccessRequestAuthError`, `AccessRequestValidator` trait, `ToolsetAccessRequestValidator`, `McpAccessRequestValidator` |
| `error.rs` | `error.rs` | `MiddlewareError` struct with blanket `From<T: AppError>` |
| `token_service/` | `token_service.rs` | `DefaultTokenService`, `CachedExchangeResult` |
| `redirects/` | `canonical_url_middleware.rs` | URL normalization (301 redirect for GET/HEAD) |
| `utils.rs` | `utils.rs` | `app_status_or_default(TenantService)`, `generate_random_string()`, `ApiErrorResponse` |

Test utilities are in `crates/routes_app/src/test_utils/` (not in this module):
- `auth_context.rs` -- `RequestAuthContextExt`, AuthContext factory re-exports
- `auth_server_test_client.rs` -- OAuth2 integration test client

## Error Enums

All use `errmeta_derive::ErrorMeta` with `AppError` trait.

### AuthError (`auth/auth_middleware.rs`)
Token, Role, TokenScope, UserScope, MissingRoles, InvalidAccess, TokenInactive, TokenNotFound, AuthService, Tenant, DbError, RefreshTokenNotFound, TowerSession, InvalidToken

### ApiAuthError (`apis/api_middleware.rs`)
Forbidden, MissingAuth, InvalidRole, InvalidScope, InvalidUserScope

### AccessRequestAuthError (`access_requests/access_request_middleware.rs`)
MissingAuth, EntityNotFound, AccessRequestNotFound, AccessRequestNotApproved, AccessRequestInvalid, EntityNotApproved

## Exported Constants

- `SESSION_KEY_ACCESS_TOKEN`, `SESSION_KEY_REFRESH_TOKEN`, `SESSION_KEY_USER_ID`
- `KEY_PREFIX_HEADER_BODHIAPP` (`"X-BodhiApp-"`)

## Commands

- `cargo test -p routes_app` -- all tests (includes middleware tests)
- `cargo test -p routes_app -- middleware` -- middleware-specific tests
- `cargo test -p routes_app -- test_live_auth_middleware` -- live OAuth2 tests
