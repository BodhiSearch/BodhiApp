# auth_middleware -- CLAUDE.md

**Companion docs** (load as needed):
- `PACKAGE.md` -- Implementation details, error enum reference, module index

## Purpose

HTTP authentication and authorization middleware. Validates JWT tokens and sessions, injects `AuthContext` into request extensions, enforces role-based access control.

## Architecture Position

```
services (AuthContext, AuthService, TenantService, etc.)
                  |
          auth_middleware      <-- this crate
            /     |      \
     routes_app  server_app  lib_bodhiserver
```

State type: `State<Arc<dyn AppService>>` (not `RouterState` -- that was removed).

## AuthContext Enum

Defined in `services::auth::auth_context`, re-exported via `src/auth_context.rs`.

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
3. Falls back to session token (same-origin only) -> `AuthContext::Session`
4. Returns `AuthError::InvalidAccess` if no valid auth
5. Inserts `AuthContext` into `req.extensions_mut()`

### `optional_auth_middleware` (permissive)
Same logic but inserts `AuthContext::Anonymous` on any auth failure. Cleans up invalid session data on token validation failure.

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

`src/middleware_error.rs` -- captures `AppError` metadata, implements `IntoResponse`. Has blanket `From<T: AppError + 'static>` impl. Replaces `ApiError` as the middleware return type.

No `"param": null` in JSON -- only adds `param` key when args is non-empty.

## DefaultTokenService

`src/token_service/token_service.rs` -- coordinates token validation, refresh, and exchange.

Dependencies: `AuthService`, `TenantService`, `CacheService`, `DbService`, `SettingService`, `ConcurrencyService`, `TimeService`.

Key methods:
- `validate_bearer_token()` -- routes to API token (`bodhiapp_*` prefix) or external token path
- `get_valid_session_token()` -- validates with auto-refresh, distributed lock via `ConcurrencyService`
- `handle_external_client_token()` -- validates issuer/audience, looks up access request, performs RFC 8693 exchange, derives `role` from DB `approved_role`

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

## Test Utilities (`test-utils` feature)

`AuthContext` factory methods defined in `services/test_utils/auth_context.rs`, re-exported from `auth_middleware::test_utils`:
- `test_session(user_id, username, role)`, `test_session_with_token(...)`, `test_session_no_role(...)`
- `test_api_token(user_id, role)`
- `test_external_app(user_id, role, app_client_id, access_request_id)`, `test_external_app_no_role(...)`

`RequestAuthContextExt::with_auth_context(ctx)` -- inserts `AuthContext` into request extensions for tests.

`AuthServerTestClient` (`src/test_utils/auth_server_test_client.rs`) -- OAuth2 integration test client with dynamic client creation.

## Commands

- `cargo test -p auth_middleware` -- all tests
- `cargo test -p auth_middleware test_live_auth_middleware` -- live OAuth2 tests (requires running OAuth2 server)
