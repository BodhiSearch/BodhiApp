# Auth, Sessions, and Middleware

## Overview

BodhiApp's authentication layer resolves user identity and tenant context from incoming requests via a middleware chain, then encodes the result as an `AuthContext` enum variant injected into request extensions. The deployment mode (see [01-deployment-modes-and-status.md](01-deployment-modes-and-status.md)) determines whether a `Session` (standalone) or `MultiTenantSession` (multi-tenant) variant is constructed. Session state uses namespaced keys scoped by tenant `client_id`, enabling multiple tenant tokens to coexist in a single HTTP session. The middleware handles JWT validation, token refresh with distributed locking, and dashboard token lifecycle -- all without deployment-mode branching in the core resolution logic.

## Functional Behavior

### Auth Endpoints

| Endpoint | Method | Middleware | Purpose |
|----------|--------|------------|---------|
| `/bodhi/v1/auth/initiate` | POST | `optional_auth` | Start resource-client OAuth2 flow |
| `/bodhi/v1/auth/callback` | POST | `optional_auth` | Complete resource-client OAuth2, store tokens, redirect to `/ui/chat` |
| `/bodhi/v1/auth/logout` | POST | `public` (no auth) | Destroy entire session, redirect to `/ui/login` |
| `/bodhi/v1/auth/dashboard/initiate` | POST | `optional_auth` | Start dashboard OAuth2 flow (multi-tenant only) |
| `/bodhi/v1/auth/dashboard/callback` | POST | `optional_auth` | Complete dashboard OAuth2, store tokens, redirect to `/ui/login` |
| `/bodhi/v1/info` | GET | `optional_auth` | Returns `AppInfo` with `status`, `deployment`, `client_id` |
| `/bodhi/v1/user/info` | GET | `optional_auth` | Returns user info + `has_dashboard_session` flag |

### Request Authentication Flow

Every request follows this resolution order:

1. **Bearer token** (Authorization header) -- checked first regardless of deployment mode
   - `bodhiapp_*` prefix: API token path (hash verification + tenant from `client_id` suffix)
   - Other: External app JWT (issuer validation + tenant from `aud` claim + RFC 8693 exchange)
2. **Session token** (same-origin requests only) -- two-step namespaced lookup
   - Read `active_client_id` from session
   - Read `{active_client_id}:access_token` from session
   - Extract `azp` from JWT, resolve tenant via `get_tenant_by_client_id(azp)`
   - Validate/refresh token via `get_valid_session_token(session, token, &tenant)`
   - **Multi-tenant addition**: Also resolve dashboard token via `try_resolve_dashboard_token()`
3. **Dashboard-only** (multi-tenant, optional_auth only) -- when no resource token exists
   - Read and validate `dashboard:access_token` from session
   - Extract user info from dashboard JWT claims
   - Construct `MultiTenantSession { client_id: None, token: None, dashboard_token }`
4. **No auth** -- `Anonymous { deployment }` (optional_auth) or 401 (strict auth)

### Same-Origin Check

Session-based auth is only attempted for same-origin requests. The `sec-fetch-site` header determines origin:
- `same-origin` / `same-site` / absent (non-browser clients): allowed
- `cross-site` / `none` / unknown values: rejected (bearer token still works)

### Standalone Auth Flow

```
1. GET /info -> status: "setup" (no tenants in DB)
2. POST /setup -> registers client via Keycloak SPI POST /resources, creates tenant (status: ResourceAdmin)
3. POST /auth/initiate { client_id } -> Keycloak OAuth2 login URL
   - client_id obtained from /info response (D68, D70)
4. POST /auth/callback { code, state }
   -> Exchanges code for tokens using tenant's client credentials
   -> If ResourceAdmin: calls make_resource_admin, sets tenant Ready + created_by via set_tenant_ready()
   -> Stores tokens: {client_id}:access_token, {client_id}:refresh_token
   -> Sets active_client_id in session
   -> Redirects to /ui/chat
5. Subsequent requests: middleware reads session, resolves tenant from JWT azp
```

### Multi-Tenant Auth Flow (Two-Phase)

```
Phase 1: Dashboard Authentication
1. GET /info -> status: "tenant_selection" (Anonymous + MultiTenant deployment)
2. POST /auth/dashboard/initiate
   -> Guards: is_multi_tenant() check (returns error in standalone)
   -> Constructs Keycloak OAuth2 URL for BODHI_MULTITENANT_CLIENT_ID
   -> PKCE + state stored under dashboard-prefixed session keys
3. POST /auth/dashboard/callback { code, state }
   -> Exchanges code using multitenant_client_id + multitenant_client_secret
   -> Stores: dashboard:access_token, dashboard:refresh_token
   -> Redirects to /ui/login

Phase 2: Tenant Selection + Resource-Client Auth
4. GET /info -> status: "tenant_selection" or "setup" (based on has_memberships())
   - MultiTenantSession with client_id=None, dashboard token present
5. GET /tenants -> list user's tenants from local tenants_users table (enriched with is_active, logged_in)
6. POST /auth/initiate { client_id } -> same endpoint as standalone
   - client_id from selected tenant
   - Keycloak SSO reuses Phase 1 session (no re-authentication)
7. POST /auth/callback { code, state }
   -> Stores: {client_id}:access_token, {client_id}:refresh_token, active_client_id
   -> Redirects to /ui/chat
8. GET /info -> status: "ready"
```

See [03-tenant-management-and-spi.md](03-tenant-management-and-spi.md#tenant-selection-and-switching-flow) for the tenant management perspective on this flow.

### Token Refresh Lifecycle

**Resource token refresh** (in `get_valid_session_token`):
- Check JWT `exp` claim against `TimeService.utc_now()`
- If expired: acquire distributed lock `{client_id}:{session_id}:refresh_token`
- Double-check pattern: re-read token from session (another request may have refreshed)
- Read `{client_id}:refresh_token`, call `auth_service.refresh_token()`
- Update session with new tokens, save session
- Lock prevents concurrent refreshes for the same tenant+session

**Dashboard token refresh** (in `get_valid_dashboard_token`):
- Same pattern with 30-second pre-expiry buffer (`exp - 30`)
- Lock key: `dashboard:{session_id}:refresh`
- Uses `multitenant_client_id` and `multitenant_client_secret` from settings
- On failure in `optional_auth`: falls back to Anonymous (dashboard-only sessions lost)
- On failure in `auth_middleware`: returns 401

### Session Invalidation

- **Logout**: `session.delete()` destroys the entire session (all tenant tokens, dashboard tokens)
- **Token refresh failure**: `clear_session_auth_data()` removes `active_client_id` + that tenant's tokens + `user_id`
- **Session clear triggers**: `RefreshTokenNotFound`, `Token(*)`, `AuthService(*)`, `InvalidToken(*)`
- **Multi-tenant-aware logout**: Not yet implemented (D63 deferred) -- logout clears everything, not selective per tenant

### Auth Endpoint Guards

- Dashboard endpoints (`/auth/dashboard/*`, `/tenants`, `/tenants/*/activate`) check `auth_context.is_multi_tenant()` and return `DashboardAuthRouteError::NotMultiTenant` in standalone mode
- `POST /setup` checks `settings.is_multi_tenant()` and returns `SetupRouteError::NotStandalone` in multi-tenant mode
- `POST /auth/callback` checks tenant status -- rejects if `AppStatus::Setup`

### External App (Third-Party) Token Flow

1. Bearer JWT with non-`bodhiapp_` prefix
2. Validate JWT `exp` against `TimeService`
3. Check cache (`exchanged_token:{token_digest}`) for previous exchange result (TTL: 5 minutes)
4. If uncached: `handle_external_client_token()`
   - Extract `aud` from JWT claims
   - Validate `iss` matches configured `auth_issuer`
   - Resolve tenant via `get_tenant_by_client_id(aud)` (D24: trust `aud` after issuer check)
   - Validate access_request scope and status
   - Perform RFC 8693 token exchange with tenant's credentials
   - Verify `access_request_id` claim from exchanged token matches DB record
   - Verify `approved_role` does not exceed user's `resource_access` role (privilege escalation check)
5. Cache result, construct `AuthContext::ExternalApp`

### API Token Format and Resolution

Format: `bodhiapp_<base64url_random>.<client_id>`

1. Extract prefix (first 8 chars of random part) for DB lookup
2. Check `status == Active`
3. SHA-256 full token hash + constant-time comparison
4. Resolve tenant from `client_id` suffix via `get_tenant_by_client_id()`
5. Construct `AuthContext::ApiToken`

## Architecture & Data Model

### AuthContext Enum

```rust
// crates/services/src/auth/auth_context.rs
pub enum AuthContext {
  Anonymous {
    client_id: Option<String>,
    tenant_id: Option<String>,
    deployment: DeploymentMode,          // enables sync is_multi_tenant() check
  },
  Session {                               // STANDALONE ONLY
    client_id: String,                    // always present (non-optional)
    tenant_id: String,
    user_id: String,
    username: String,
    role: Option<ResourceRole>,
    token: String,
  },
  MultiTenantSession {                    // MULTI-TENANT ONLY
    client_id: Option<String>,            // None = dashboard-only
    tenant_id: Option<String>,            // None = dashboard-only
    user_id: String,                      // from dashboard JWT sub
    username: String,                     // from dashboard JWT preferred_username
    role: Option<ResourceRole>,           // None when dashboard-only or no tenant role
    token: Option<String>,                // None when dashboard-only
    dashboard_token: String,              // always present, validated/refreshed by middleware
  },
  ApiToken { client_id, tenant_id, user_id, role: TokenScope, token },
  ExternalApp { client_id, tenant_id, user_id, role: Option<UserScope>, token,
                external_app_token, app_client_id, access_request_id: Option<String> },
}
```

**Key design insight**: The variant type encodes deployment mode. `Session` implies standalone; `MultiTenantSession` implies multi-tenant. Route handlers use `auth_context.is_multi_tenant()` (sync) instead of `settings.is_multi_tenant().await` (async). This was a deliberate refactoring that eliminated 9 async settings calls in route handlers (D58).

**Convenience methods**:

| Method | Returns | Notes |
|--------|---------|-------|
| `is_multi_tenant()` | `bool` | `true` for `MultiTenantSession`; checks `deployment` field for `Anonymous` |
| `is_authenticated()` | `bool` | `true` for all non-`Anonymous` (including dashboard-only `MultiTenantSession`) |
| `client_id()` | `Option<&str>` | `None` for `Anonymous` and dashboard-only `MultiTenantSession` |
| `tenant_id()` | `Option<&str>` | `None` for `Anonymous` and dashboard-only `MultiTenantSession` |
| `user_id()` | `Option<&str>` | `Some` for all authenticated variants (including dashboard-only) |
| `token()` | `Option<&str>` | Resource token; `None` for `Anonymous` and dashboard-only |
| `resource_role()` | `Option<&ResourceRole>` | Works for both `Session` and `MultiTenantSession` |
| `dashboard_token()` | `Option<&str>` | `Some` only for `MultiTenantSession` |
| `require_dashboard_token()` | `Result<&str, AuthContextError>` | 401 error if not `MultiTenantSession` |
| `app_role()` | `Option<AppRole>` | Unified role across all authenticated variants |

### AuthContextError

```rust
// crates/services/src/auth/auth_context.rs
pub enum AuthContextError {
  AnonymousNotAllowed,        // 403 -- require_user_id() on Anonymous
  MissingClientId,            // 403
  MissingToken,               // 401
  MissingTenantId,            // 500 (internal -- should never happen in authenticated context)
  MissingDashboardToken,      // 401
}
```

### Session Key Schema

Defined in `crates/services/src/session_keys.rs`, re-exported from `services` crate.

```
Global keys:
  user_id                              -- Keycloak user ID (same across all tenants)
  active_client_id                     -- currently selected tenant's client_id

Dashboard keys (multi-tenant only):
  dashboard:access_token               -- dashboard client JWT
  dashboard:refresh_token              -- dashboard client refresh token

Tenant-namespaced keys (format: {client_id}:<type>):
  {client_id}:access_token             -- resource-client JWT
  {client_id}:refresh_token            -- resource-client refresh token

OAuth flow transient keys (cleaned up after callback):
  oauth_state / dashboard_oauth_state  -- CSRF state
  pkce_verifier / dashboard_pkce_verifier
  callback_url / dashboard_callback_url
  auth_client_id                       -- tenant being authenticated against

Lock keys (format: {client_id}:{session_id}:<type>):
  {client_id}:{session_id}:refresh_token   -- resource token refresh lock
  dashboard:{session_id}:refresh           -- dashboard token refresh lock
```

**Why namespaced keys**: Enables tenant switching without re-login. When a user switches from tenant A to tenant B, both tokens coexist. If tenant B's token is still valid, the switch is instant (via `POST /tenants/{client_id}/activate` which just updates `active_client_id`). Breaking migration (D56) -- no legacy flat-key compatibility.

### Middleware Chain

```
Request
  |
  v
[Session layer - tower_sessions, SQLite-backed, global]
  |
  v
[public_apis] -----> handler (no auth context, no AuthScope fallback to Anonymous)
  |
  v
[optional_auth_middleware] -----> handler (AuthContext always set: real or Anonymous)
  |
  v
[auth_middleware] -----> [api_auth_middleware(role)] -----> handler (AuthContext guaranteed non-Anonymous)
                              |
                              v
                         [access_request_auth_middleware] -----> handler (entity-level access check)
```

**Route group membership**:

| Group | Middleware | Key Routes |
|-------|-----------|------------|
| `public_apis` | None | `/ping`, `/health`, `/setup`, `/logout`, app access requests |
| `optional_auth` | `optional_auth_middleware` | `/info`, `/user/info`, `/auth/initiate`, `/auth/callback`, `/auth/dashboard/*`, `/tenants`, dev endpoints |
| Role-gated | `auth_middleware` + `api_auth_middleware(role)` | All CRUD endpoints (tokens, models, settings, toolsets, MCPs, users) |

### DefaultTokenService

Coordinates token validation, refresh, and exchange. Constructed per-request by middleware (not a singleton).

**Dependencies**: `AuthService`, `TenantService`, `CacheService`, `DbService`, `SettingService`, `ConcurrencyService`, `TimeService`

**Key methods**:

| Method | Signature | Purpose |
|--------|-----------|---------|
| `validate_bearer_token(header)` | `-> Result<AuthContext, AuthError>` | Routes to API token or external token path |
| `get_valid_session_token(session, token, &Tenant)` | `-> Result<(String, Option<ResourceRole>), AuthError>` | Validates + refreshes resource token with lock |
| `get_valid_dashboard_token(session, token)` | `-> Result<String, AuthError>` | Validates + refreshes dashboard token with lock |
| `handle_external_client_token(token)` | `-> Result<(AuthContext, CachedExchangeResult), AuthError>` | RFC 8693 exchange for external JWTs |

### AuthError Enum

```rust
// crates/routes_app/src/middleware/auth/error.rs
pub enum AuthError {
  Token(TokenError),              // transparent, from JWT parsing / validation
  Role(RoleError),                // 401
  TokenScope(TokenScopeError),    // 401
  UserScope(UserScopeError),      // 401
  MissingRoles,                   // 401
  InvalidAccess,                  // 401 -- no valid auth present (strict middleware)
  TokenInactive,                  // 401
  TokenNotFound,                  // 401
  AuthService(AuthServiceError),  // transparent, from Keycloak calls
  Tenant(TenantError),            // transparent, from tenant lookup
  DbError(DbError),               // transparent
  RefreshTokenNotFound,           // 401 -- session expired
  TowerSession(session::Error),   // from session store errors
  InvalidToken(String),           // 401 -- malformed or invalid token
}
```

### Tenant Resolution Strategy (Summary)

| Token Type | Claim Used | Lookup Method | Rationale |
|-----------|------------|---------------|-----------|
| Session JWT | `azp` (authorized party) | `get_tenant_by_client_id(azp)` | D22: azp identifies which tenant's OAuth client the user logged into |
| External JWT | `aud` (audience) | `get_tenant_by_client_id(aud)` | D24: audience is the target tenant after issuer validation |
| API token | suffix after last `.` | `get_tenant_by_client_id(suffix)` | Format: `bodhiapp_<random>.<client_id>` |
| No token | N/A | N/A | `Anonymous { client_id: None, tenant_id: None }` |

**Unified code path (D23)**: No `if standalone / if multi_tenant` branching in tenant resolution. Works identically for 1 tenant or N tenants. The only deployment-mode branching is at AuthContext construction time (choosing `Session` vs `MultiTenantSession`) and dashboard token resolution.

### JWT Claims Types

All defined in `crates/services/src/shared_objs/token.rs`:

| Type | Fields | Used For |
|------|--------|----------|
| `Claims` | `sub`, `preferred_username`, `azp`, `exp`, `resource_access` | Session token validation, role extraction |
| `UserIdClaims` | `sub`, `preferred_username` | User info extraction from refreshed tokens |
| `ScopeClaims` | `sub`, `azp`, `aud`, `iss`, `scope`, `exp`, `resource_access`, `access_request_id` | External app token validation |
| `ExpClaims` | `exp` | Quick expiry check before full validation |

`extract_claims::<T>(token)` performs raw base64 decode without expiry check (D25: expired JWT claims are safe for tenant resolution since they are still cryptographically signed).

### Dashboard Client Configuration

| Setting | Source | Access Method |
|---------|--------|---------------|
| `BODHI_MULTITENANT_CLIENT_ID` | DB or env var | `settings.multitenant_client_id()` |
| `BODHI_MULTITENANT_CLIENT_SECRET` | Env var only (D98) | `settings.multitenant_client_secret()` (calls `get_env()`) |

See [01-deployment-modes-and-status.md](01-deployment-modes-and-status.md#configuration-differences) for deployment-mode guards on these settings.

### ConcurrencyService Lock Patterns

Two lock methods used for token refresh:
- `with_lock_auth(key, closure)` -- used for resource token refresh (returns `(String, Option<ResourceRole>)`)
- `with_lock_str(key, closure)` -- used for dashboard token refresh (returns `String`)

Both use double-checked locking: after acquiring the lock, re-read the token from session to check if another request already refreshed it.

## Technical Implementation

### Key Files

| File | Purpose |
|------|---------|
| `crates/services/src/auth/auth_context.rs` | `AuthContext` enum, `AuthContextError`, convenience methods |
| `crates/services/src/session_keys.rs` | Session key constants + namespaced key format functions |
| `crates/routes_app/src/middleware/auth/auth_middleware.rs` | `auth_middleware`, `optional_auth_middleware`, `try_resolve_dashboard_token`, `clear_session_auth_data`, `is_same_origin` |
| `crates/routes_app/src/middleware/auth/error.rs` | `AuthError` enum |
| `crates/routes_app/src/middleware/token_service/token_service.rs` | `DefaultTokenService`: `validate_bearer_token`, `get_valid_session_token`, `get_valid_dashboard_token`, `handle_external_client_token` |
| `crates/routes_app/src/auth/routes_auth.rs` | `auth_initiate`, `auth_callback`, `auth_logout` |
| `crates/routes_app/src/tenants/routes_dashboard_auth.rs` | `dashboard_auth_initiate`, `dashboard_auth_callback` |
| `crates/routes_app/src/setup/routes_setup.rs` | `setup_show` (`/info` handler with AuthContext match) |
| `crates/routes_app/src/routes.rs` | Route group assembly: `public_apis`, `optional_auth`, role-gated groups |
| `crates/services/src/settings/setting_service.rs` | `deployment_mode()`, `is_multi_tenant()`, `multitenant_client_id()`, `multitenant_client_secret()`, `dashboard_callback_url()`, `login_callback_url()` |
| `crates/services/src/shared_objs/token.rs` | JWT claim structs (`Claims`, `ScopeClaims`, etc.) and `extract_claims()` |
| `crates/routes_app/src/middleware/auth/test_auth_middleware.rs` | Middleware unit tests |
| `crates/routes_app/src/middleware/auth/test_auth_middleware_isolation.rs` | Multi-tenant middleware isolation tests (sqlite/postgres x standalone/multi-tenant) |
| `crates/routes_app/src/middleware/token_service/test_token_service.rs` | Token service unit tests |

### Refactoring History

1. **Pre-MT**: Middleware called `get_standalone_app()` to find "the one tenant." Session used flat keys (`access_token`, `refresh_token`). `AppStatusInvalid` check in middleware rejected Setup-status requests before auth.

2. **Stage 2 Phase 1 (JWT-based resolution)**: Removed `get_standalone_app()` from middleware. Tenant resolved from JWT `azp`/`aud` claims. `get_valid_session_token` takes `&Tenant` parameter. `AppStatusInvalid` removed from middleware (setup routes gate independently). Session migrated to namespaced keys (`{client_id}:access_token`).

3. **Stage 2 M2 (Backend)**: Dashboard auth routes added. Session key namespacing formalized. `ensure_valid_dashboard_token()` helper created. `resolve_multi_tenant_status()` for `/info` called Keycloak SPI (remote calls on every `/info` request).

4. **Architecture refactor**: `MultiTenantSession` AuthContext variant introduced. `tenants_users` local table replaced SPI calls for `/info`. Middleware validates/refreshes dashboard tokens. `resolve_multi_tenant_status()` deleted. `ensure_valid_dashboard_token()` deleted. `/info` became fully local (match on AuthContext + `has_memberships()` DB query). `list_tenants` removed from `AuthService`. Session key constants moved to `services::session_keys`. Deployment mode encoded in AuthContext variant type. 9 async `settings.is_multi_tenant()` calls replaced with sync `auth_context.is_multi_tenant()`.

## Decisions

Decisions are referenced by ID. See [08-decisions-index.md](08-decisions-index.md) for the canonical decision table with full descriptions.

| ID | Title | Status |
|----|-------|--------|
| D21 | JWT-only tenant resolution | Implemented |
| D22 | Ignore BODHI_MULTITENANT_CLIENT_ID in middleware | Implemented |
| D23 | Unified middleware code path | Implemented |
| D24 | Trust JWT aud after issuer check | Implemented |
| D25 | Expired JWT claims safe for tenant resolution | Implemented |
| D26 | Anonymous = None/None | Implemented |
| D28 | Middleware-only scope for get_standalone_app removal | Implemented |
| D32 | Namespaced session keys | Implemented |
| D33 | Two-step middleware token lookup | Implemented |
| D35 | Separate dashboard auth endpoints | Implemented |
| D53 | Transparent dashboard token refresh | Implemented |
| D54 | /info behind optional_auth_middleware | Implemented |
| D56 | Breaking session key migration | Implemented |
| D58 | Deployment mode in AuthContext | Implemented |
| D63 | Logout scope semantics | Deferred |
| D68 | client_id required in POST /auth/initiate | Implemented |
| D74 | Auth callback uses tenant's client_id for namespacing | Implemented |
| D77 | Separate frontend callback routes | Implemented |
| D80 | Shared code exchange utility | Deferred (TECHDEBT) |
| D81 | /user/info dashboard state | Implemented |
| D98 | Dashboard client secret from env only | Implemented |
| D99 | Dashboard token expiry via TimeService | Implemented (D105 superseded D99) |
| D101 | MT endpoints error in standalone | Implemented |

## Known Gaps & TECHDEBT

1. **D80 code exchange duplication**: `routes_auth.rs` (resource callback) and `routes_dashboard_auth.rs` (dashboard callback) duplicate the OAuth2 code exchange logic. Low priority -- no functional impact. A shared parameterized utility was planned but deferred.

2. **D63 multi-tenant-aware logout**: `session.delete()` destroys all session data including all tenant tokens and dashboard tokens. There is no selective resource-client logout (clear one tenant's tokens while preserving dashboard and other tenants). Users must fully re-authenticate after logout in multi-tenant mode.

3. **DefaultTokenService constructed per-request**: Middleware creates a new `DefaultTokenService` for each request by cloning `Arc` references to all 7 dependencies. This is architecturally clean but allocates a struct per request. Not a performance concern in practice since the `Arc` clones are cheap.

4. **Dashboard token 30-second buffer**: `get_valid_dashboard_token` refreshes tokens 30 seconds before expiry (`exp - 30`). Resource token refresh has no such buffer (refreshes only after expiry). This inconsistency is harmless but could be unified.

5. **Session store is SQLite-backed**: Both standalone and multi-tenant use the same SQLite-backed `tower_sessions` store. In high-concurrency multi-tenant deployments, this could become a bottleneck. No plans to change for the current scale.

6. **Exchange cache TTL is hardcoded**: `EXCHANGE_CACHE_TTL_SECS = 300` (5 minutes) for external app token exchange results. Not configurable via settings.

7. **Dashboard initiate does not check token expiry**: `dashboard_auth_initiate` checks if a dashboard token exists and can be decoded (`extract_claims` succeeds), but does not check if it is expired. An expired but decodable token would cause a 200 response directing to `/ui/login`, where the middleware would then fail to resolve the dashboard token and fall back to Anonymous.
