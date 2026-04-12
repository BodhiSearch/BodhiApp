# PACKAGE.md -- routes_app::middleware

Implementation details and module index. For architecture and design, see `CLAUDE.md` in this directory.

Previously the standalone `auth_middleware` crate. Merged into `routes_app` as an internal module.

## Module Structure

Entry point: `mod.rs` -- re-exports all public types.

| Module | Files | Purpose |
|--------|-------|---------|
| `auth/` | `auth_middleware.rs` | `auth_middleware`, `optional_auth_middleware`, `remove_app_headers`, `AuthError` |
| `anthropic_auth_middleware.rs` | `anthropic_auth_middleware.rs` | `anthropic_auth_middleware`, `strip_sentinel_headers` — `/anthropic/*` and `/v1/messages` path-scoped; strips `SENTINEL_API_KEY` then rewrites `x-api-key` → `Authorization: Bearer` |
| `openai_auth_middleware.rs` | `openai_auth_middleware.rs` | `openai_auth_middleware` — `/v1/*` path-scoped; strips `SENTINEL_API_KEY` from `Authorization` / `x-api-key` |
| `apis/` | `api_middleware.rs` | `api_auth_middleware`, `ApiAuthError` |
| `access_requests/` | `access_request_middleware.rs` | `access_request_auth_middleware`, `AccessRequestAuthError`, `AccessRequestValidator` trait, `McpAccessRequestValidator` |
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
- `SENTINEL_API_KEY` (`"bodhiapp_sentinel_api_key_ignored"`) — placeholder the chat UI sends to pi-ai SDKs; stripped by `anthropic_auth_middleware` / `openai_auth_middleware`

## Commands

- `cargo test -p routes_app` -- all tests (includes middleware tests)
- `cargo test -p routes_app -- middleware` -- middleware-specific tests
- `cargo test -p routes_app -- test_live_auth_middleware` -- live OAuth2 tests

## AuthContext Enum

Defined in `services::auth::auth_context`.

**Variants**:
- `Anonymous { client_id: Option<String>, tenant_id: Option<String>, deployment: DeploymentMode }`
- `Session { client_id, tenant_id, user_id, username, role: Option<ResourceRole>, token }` — single-tenant session
- `MultiTenantSession { client_id: Option<String>, tenant_id: Option<String>, user_id, username, role: Option<ResourceRole>, token: Option<String>, dashboard_token: String }` — multi-tenant session (with or without active resource tenant)
- `ApiToken { client_id, tenant_id, user_id, role: TokenScope, token }`
- `ExternalApp { client_id, tenant_id, user_id, role: Option<UserScope>, token, external_app_token, app_client_id, access_request_id: Option<String> }`

`Session`, `ApiToken`, `ExternalApp` have required `client_id: String`, `tenant_id: String`. `MultiTenantSession` has both as `Option` (dashboard-only sessions have no active tenant).

**Convenience methods**: `user_id()`, `require_user_id()`, `client_id()`, `require_client_id()`, `tenant_id()`, `require_tenant_id()`, `token()`, `external_app_token()`, `app_role()`, `is_authenticated()`, `is_multi_tenant()`, `dashboard_token()`, `require_dashboard_token()`.

`require_user_id()` returns `Result<&str, AuthContextError>` (not `ApiError`).

**Multi-tenant helpers**: `is_multi_tenant()` returns `true` for `MultiTenantSession` and `Anonymous { deployment: MultiTenant }`. `require_dashboard_token()` returns `Result<&str, AuthContextError>` -- only `MultiTenantSession` carries a dashboard token. Route handlers use these instead of querying `SettingService`.

## Middleware Step Details

### `auth_middleware` (strict) — Step List
1. Strips `X-BodhiApp-*` headers (defense-in-depth)
2. Resolves `deployment_mode` from `SettingService`
3. Checks bearer token -> `AuthContext::ApiToken` or `AuthContext::ExternalApp`
4. Falls back to session token (same-origin only) via **two-step lookup**:
   a. Read `active_client_id` from session
   b. Read `access_token:{client_id}` using namespaced key
   c. Resolve tenant from JWT `azp` claim via `get_tenant_by_client_id()`
   d. Call `get_valid_session_token()` for validation/refresh
   e. **Multi-tenant mode**: also resolves dashboard token via `try_resolve_dashboard_token()` -> `AuthContext::MultiTenantSession`
   f. **Standalone mode**: -> `AuthContext::Session`
5. Returns `AuthError::InvalidAccess` if no valid auth
6. Inserts `AuthContext` into `req.extensions_mut()`

**No setup check**: Middleware does authentication only. Setup routes gate via `app_status_or_default()`.

### `optional_auth_middleware` (permissive)
Same logic but inserts `AuthContext::Anonymous { client_id: None, tenant_id: None, deployment }` on any auth failure. Cleans up invalid session data on token validation failure.

**Dashboard-only sessions** (multi-tenant, no resource token): If no active resource token exists but a valid dashboard token is found in session, constructs `AuthContext::MultiTenantSession { client_id: None, token: None, dashboard_token, ... }` with user info extracted from the dashboard JWT. This enables routes like `/tenants` and `/user/info` that need only dashboard-level auth.

### `try_resolve_dashboard_token()` (helper)
Reads `DASHBOARD_ACCESS_TOKEN_KEY` from session, validates/refreshes via `DefaultTokenService::get_valid_dashboard_token()`. Returns `Option<String>` -- `None` if no dashboard token in session or refresh fails. Used by both `auth_middleware` and `optional_auth_middleware` in multi-tenant mode.

### `access_request_auth_middleware` (entity-level)
Validates entity access against approved resources in access requests.
- `AccessRequestValidator` trait: `extract_entity_id(path)` + `validate_approved(approved_json, entity_id)`
- Implementations: `McpAccessRequestValidator`
- Session users pass through; `ExternalApp` with `access_request_id` validated against DB

## Session Key Format (Multi-Tenant)

Session keys are namespaced by `client_id` to support multiple tenants per session. Constants and helper functions are defined in `services::session_keys` and re-exported from the `services` crate:
- `access_token_key(client_id)` -> `"{client_id}:access_token"` (helper function)
- `refresh_token_key(client_id)` -> `"{client_id}:refresh_token"`
- `SESSION_KEY_ACTIVE_CLIENT_ID` = `"active_client_id"` (marks which tenant is active)
- `DASHBOARD_ACCESS_TOKEN_KEY` = `"dashboard:access_token"`
- `DASHBOARD_REFRESH_TOKEN_KEY` = `"dashboard:refresh_token"`
- Token refresh lock: `{client_id}:{session_id}:refresh_token` (per-tenant per-session)

## DefaultTokenService Details

`token_service/token_service.rs` -- coordinates token validation, refresh, and exchange.

**Dependencies**: `AuthService`, `TenantService`, `CacheService`, `DbService`, `SettingService`, `ConcurrencyService`, `TimeService`.

**Key methods**:
- `validate_bearer_token()` -- routes to API token (`bodhiapp_*` prefix) or external token path
- `get_valid_session_token(session, access_token, &Tenant)` -- validates with auto-refresh, distributed lock via `ConcurrencyService`. Caller resolves tenant from JWT `azp` and passes it in.
- `get_valid_dashboard_token(session, dashboard_token) -> Result<String, AuthError>` -- validates JWT expiry, refreshes with distributed lock if expired. Dashboard token refresh uses `DASHBOARD_REFRESH_TOKEN_KEY` from session. Previously, dashboard token validation was done in route handlers via a now-deleted `ensure_valid_dashboard_token()` function; it now lives in the middleware layer via `try_resolve_dashboard_token()`.
- `handle_external_client_token()` -- resolves tenant from JWT `aud` claim via `get_tenant_by_client_id()`, validates issuer, looks up access request, performs RFC 8693 exchange, derives `role` from DB `approved_role`

### CachedExchangeResult
Fields: `token`, `client_id`, `tenant_id`, `app_client_id`, `role: Option<String>`, `access_request_id: Option<String>`.
Cached under `exchanged_token:{token_digest}` (first 12 chars of SHA-256 hex).

## Tenant Resolution Strategy

- **Session tokens**: Middleware extracts `azp` from JWT, calls `get_tenant_by_client_id(azp)` to resolve tenant
- **External tokens**: `handle_external_client_token` extracts `aud` from JWT, calls `get_tenant_by_client_id(aud)` to resolve tenant
- **API tokens**: `validate_bearer_token` extracts `client_id` suffix from token format `bodhiapp_<random>.<client_id>`
- No `get_standalone_app()` calls in middleware — works identically for standalone and multi-tenant deployments

## ExternalApp Role Derivation

`role` on `AuthContext::ExternalApp` comes from DB `approved_role` column (via `CachedExchangeResult.role`), NOT from JWT scope claims. When `role` is `None`, `api_auth_middleware` rejects with `MissingAuth`.

## Test Utilities

`AuthContext` factory methods defined in `services/test_utils/auth_context.rs`, re-exported from `routes_app::test_utils`:
- `test_session(user_id, username, role)`, `test_session_with_token(...)`, `test_session_no_role(...)`
- `test_api_token(user_id, role)`
- `test_external_app(user_id, role, app_client_id, access_request_id)`, `test_external_app_no_role(...)`

`RequestAuthContextExt::with_auth_context(ctx)` -- inserts `AuthContext` into request extensions for tests.

`AuthServerTestClient` (`test_utils/auth_server_test_client.rs`) -- OAuth2 integration test client with dynamic client creation.
