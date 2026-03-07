# middleware -- CLAUDE.md

**Location**: `crates/routes_app/src/middleware/`
**Companion docs**: `PACKAGE.md` in this directory for implementation details and module index.

Previously the standalone `auth_middleware` crate. Merged into `routes_app` as an internal module.

## Purpose

HTTP authentication and authorization middleware. Validates JWT tokens and sessions, injects `AuthContext` into request extensions, enforces role-based access control.

## Architecture Position

```
services (AuthContext, AuthService, TenantService, etc.)
                  |
     routes_app::middleware      <-- this module
                  |
     routes_app route handlers (via AuthScope extractor)
```

State type: `State<Arc<dyn AppService>>`.

## AuthContext Enum

Defined in `services::auth::auth_context`.

**Variants** (all non-Anonymous variants have `client_id: String`, `tenant_id: String`):
- `Anonymous { client_id: Option<String>, tenant_id: Option<String> }`
- `Session { client_id, tenant_id, user_id, username, role: Option<ResourceRole>, token }`
- `ApiToken { client_id, tenant_id, user_id, role: TokenScope, token }`
- `ExternalApp { client_id, tenant_id, user_id, role: Option<UserScope>, token, external_app_token, app_client_id, access_request_id: Option<String> }`

**Convenience methods**: `user_id()`, `require_user_id()`, `client_id()`, `require_client_id()`, `tenant_id()`, `require_tenant_id()`, `token()`, `external_app_token()`, `app_role()`, `is_authenticated()`.

`require_user_id()` returns `Result<&str, AuthContextError>` (not `ApiError`).

## Middleware Functions

All return `Result<Response, MiddlewareError>`.

### `auth_middleware` (strict)
1. Strips `X-BodhiApp-*` headers (defense-in-depth)
2. Checks bearer token -> `AuthContext::ApiToken` or `AuthContext::ExternalApp`
3. Falls back to session token (same-origin only) via **two-step lookup**:
   a. Read `active_client_id` from session
   b. Read `access_token:{client_id}` using namespaced key
   c. Resolve tenant from JWT `azp` claim via `get_tenant_by_client_id()`
   d. Call `get_valid_session_token()` for validation/refresh -> `AuthContext::Session`
4. Returns `AuthError::InvalidAccess` if no valid auth
5. Inserts `AuthContext` into `req.extensions_mut()`

**No setup check**: Middleware does authentication only. Setup routes gate via `app_status_or_default()`.

### Session Key Format (Multi-Tenant)
Session keys are namespaced by `client_id` to support multiple tenants per session. Constants and helper functions are defined in `services::session_keys` and re-exported from the `services` crate:
- `access_token_key(client_id)` -> `"{client_id}:access_token"` (helper function)
- `refresh_token_key(client_id)` -> `"{client_id}:refresh_token"`
- `SESSION_KEY_ACTIVE_CLIENT_ID` = `"active_client_id"` (marks which tenant is active)
- `DASHBOARD_ACCESS_TOKEN_KEY` = `"dashboard:access_token"`
- `DASHBOARD_REFRESH_TOKEN_KEY` = `"dashboard:refresh_token"`
- Token refresh lock: `{client_id}:{session_id}:refresh_token` (per-tenant per-session)

### `optional_auth_middleware` (permissive)
Same logic but inserts `AuthContext::Anonymous { client_id: None, tenant_id: None }` on any auth failure. Cleans up invalid session data on token validation failure.

### `api_auth_middleware` (authorization)
Reads `AuthContext` from extensions, pattern-matches to enforce role hierarchy:
- `Session { role: Some(role) }` -> checks `role.has_access_to(&required_role)`
- `ApiToken { role }` -> checks against `required_token_scope`
- `ExternalApp { role: Some(role) }` -> checks against `required_user_scope`
- `Anonymous`, `Session { role: None }`, `ExternalApp { role: None }` -> `MissingAuth`

Role hierarchy: Admin > Manager > PowerUser > User

### `access_request_auth_middleware` (entity-level)
Validates entity access against approved resources in access requests.
- `AccessRequestValidator` trait: `extract_entity_id(path)` + `validate_approved(approved_json, entity_id)`
- Implementations: `ToolsetAccessRequestValidator`, `McpAccessRequestValidator`
- Session users pass through; `ExternalApp` with `access_request_id` validated against DB

## MiddlewareError

`error.rs` -- captures `AppError` metadata, implements `IntoResponse`. Has blanket `From<T: AppError + 'static>` impl.

No `"param": null` in JSON -- only adds `param` key when args is non-empty.

## DefaultTokenService

`token_service/token_service.rs` -- coordinates token validation, refresh, and exchange.

Dependencies: `AuthService`, `TenantService`, `CacheService`, `DbService`, `SettingService`, `ConcurrencyService`, `TimeService`.

Key methods:
- `validate_bearer_token()` -- routes to API token (`bodhiapp_*` prefix) or external token path
- `get_valid_session_token(session, access_token, &Tenant)` -- validates with auto-refresh, distributed lock via `ConcurrencyService`. Caller resolves tenant from JWT `azp` and passes it in.
- `handle_external_client_token()` -- resolves tenant from JWT `aud` claim via `get_tenant_by_client_id()`, validates issuer, looks up access request, performs RFC 8693 exchange, derives `role` from DB `approved_role`

### Tenant Resolution Strategy
- **Session tokens**: Middleware extracts `azp` from JWT, calls `get_tenant_by_client_id(azp)` to resolve tenant
- **External tokens**: `handle_external_client_token` extracts `aud` from JWT, calls `get_tenant_by_client_id(aud)` to resolve tenant
- **API tokens**: `validate_bearer_token` extracts `client_id` suffix from token format `bodhiapp_<random>.<client_id>`
- No `get_standalone_app()` calls in middleware — works identically for standalone and multi-tenant deployments

### CachedExchangeResult
Fields: `token`, `client_id`, `tenant_id`, `app_client_id`, `role: Option<String>`, `access_request_id: Option<String>`.
Cached under `exchanged_token:{token_digest}` (first 12 chars of SHA-256 hex).

### API Token Validation
1. Extract prefix (first 8 chars after `bodhiapp_`) for DB lookup
2. Check status is `Active`
3. Full SHA-256 hash + constant-time comparison
4. Parse `scopes` into `TokenScope`

## ExternalApp Role Derivation

`role` on `AuthContext::ExternalApp` comes from DB `approved_role` column (via `CachedExchangeResult.role`), NOT from JWT scope claims. When `role` is `None`, `api_auth_middleware` rejects with `MissingAuth`.

## Test Utilities

`AuthContext` factory methods defined in `services/test_utils/auth_context.rs`, re-exported from `routes_app::test_utils`:
- `test_session(user_id, username, role)`, `test_session_with_token(...)`, `test_session_no_role(...)`
- `test_api_token(user_id, role)`
- `test_external_app(user_id, role, app_client_id, access_request_id)`, `test_external_app_no_role(...)`

`RequestAuthContextExt::with_auth_context(ctx)` -- inserts `AuthContext` into request extensions for tests.

`AuthServerTestClient` (`test_utils/auth_server_test_client.rs`) -- OAuth2 integration test client with dynamic client creation.

## Commands

- `cargo test -p routes_app` -- all tests (includes middleware tests)
- `cargo test -p routes_app -- middleware` -- middleware-specific tests
- `cargo test -p routes_app -- test_live_auth_middleware` -- live OAuth2 tests (requires running OAuth2 server)
