# Multi-Tenant BodhiApp Backend -- Kickoff

> **Created**: 2026-03-06
> **Updated**: 2026-03-09
> **Status**: ✅ COMPLETED (commits `6a7d879..04788eb`)
> **Scope**: Backend Rust crates: `services` -> `server_core` -> `routes_app` -> `server_app` -> `lib_bodhiserver`
> **Prior work**:
> - `ai-docs/claude-plans/20260303-multi-tenant/` (Phase 1-5: RLS, isolation tests, CRUD uniformity)
> - `ai-docs/claude-plans/20260306-multi-tenant-2/` (middleware JWT-based tenant resolution — completed)
> - Keycloak SPI implementation: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/keycloak-bodhi-ext/ai-docs/claude-plans/20260307-multi-tenant/`
> **SPI integration doc**: `keycloak-bodhi-ext/ai-docs/claude-plans/20260307-multi-tenant/20260307-tenants-integration-doc.md`
> **Context doc**: `ai-docs/claude-plans/20260306-multi-tenant-2/multi-tenant-flow-ctx.md`

---

## Context

The auth middleware now resolves tenants from JWT claims (azp/aud) instead of `get_standalone_app()`. The Keycloak SPI is implemented and deployed to dev. The next step is implementing the multi-tenant login flow in the BodhiApp backend: dashboard authentication, tenant listing, tenant registration, tenant selection, and resource-client login.

### Current Standalone Auth Flow

```
1. GET /bodhi/v1/info -> status: "setup" (no tenants)
2. POST /bodhi/v1/setup -> registers client with SPI POST /resources, creates tenant row (status: resource_admin)
3. POST /bodhi/v1/auth/initiate -> constructs Keycloak login URL using get_standalone_app()
4. GET /ui/auth/callback -> POST /bodhi/v1/auth/callback -> exchanges code, stores tokens in session
5. If ResourceAdmin: calls SPI /make-resource-admin, transitions to Ready, refreshes token
6. Subsequent requests: middleware reads session token, resolves tenant from JWT azp
```

### Target Multi-Tenant Auth Flow

```
1. GET /bodhi/v1/info -> status: "tenant_selection" (multi-tenant mode, no active tenant)
2. POST /bodhi/v1/auth/dashboard/initiate -> constructs Keycloak login URL for client-bodhi-multi-tenant
3. POST /bodhi/v1/auth/dashboard/callback -> exchanges code, stores dashboard token in session
   -> Redirects to /ui/login
4. /ui/login calls GET /bodhi/v1/user/info -> returns dashboard session state
5. Frontend calls GET /bodhi/v1/tenants -> list user's clients (proxy to SPI, enriched with session metadata)
6a. If 0 clients -> redirect to /ui/setup/tenants/ -> POST /bodhi/v1/tenants -> SPI registers client
    -> BodhiApp creates tenant row (status: Ready, created_by: user_id)
6b. If 1 client -> auto-select
6c. If N clients -> user picks from dropdown
7. POST /bodhi/v1/auth/initiate { client_id } -> constructs Keycloak login URL for selected resource-client
8. POST /bodhi/v1/auth/callback -> exchanges code, stores resource-client token in session
   (keyed by azp from JWT) -> Redirects to /ui/chat
9. Session: active_client_id set, subsequent requests use that token
10. GET /bodhi/v1/info -> status: "ready" (active tenant selected)
```

---

## Key Files to Explore

### Auth routes and setup
- `crates/routes_app/src/auth/routes_auth.rs` — current `auth_initiate`, `auth_callback`, `auth_logout`
- `crates/routes_app/src/setup/routes_setup.rs` — `setup_show` (GET /info), `setup_create` (POST /setup)
- `crates/routes_app/src/middleware/auth/auth_middleware.rs` — session token reading (now JWT-based)
- `crates/routes_app/src/middleware/token_service/token_service.rs` — `get_valid_session_token` (now takes `&Tenant`)

### Services and config
- `crates/services/src/auth/auth_service.rs` — `AuthService` trait (register_client, exchange_auth_code, etc.)
- `crates/services/src/tenants/` — `TenantService`, `Tenant` object, `AppStatus` enum
- `crates/services/src/settings/setting_service.rs` — `multitenant_client_id()`, `deployment()` methods
- `crates/services/src/settings/constants.rs` — `BODHI_MULTITENANT_CLIENT_ID`, `BODHI_DEPLOYMENT`

### Session infrastructure
- `crates/routes_app/src/middleware/auth/auth_middleware.rs` — `SESSION_KEY_ACCESS_TOKEN`, `SESSION_KEY_REFRESH_TOKEN`
- Session store is SQLite-backed via `tower_sessions`

### Test patterns
- `crates/routes_app/src/auth/test_login_initiate.rs` — existing auth test patterns
- `crates/routes_app/src/middleware/auth/test_auth_middleware_isolation.rs` — multi-tenant middleware tests
- `crates/routes_app/src/tokens/test_tokens_isolation.rs` — isolation test pattern with `sea_context`

---

## Changes Required

### 1. Schema: Add `created_by` to tenants table

**Migration** (modify existing, per D14 convention):
- Add `created_by VARCHAR(255)` column to `tenants` table (nullable)
- In standalone mode: Set during `auth_callback` ResourceAdmin -> Ready transition via `set_client_ready(client_id, user_id)` which handles both status update and `created_by`
- In multi-tenant mode: set during `POST /bodhi/v1/tenants` (user_id from dashboard token's sub claim)

### 2. New AppStatus variant: `TenantSelection`

Add `TenantSelection` variant to the `AppStatus` enum in `crates/services/src/tenants/tenant_objs.rs`.

**`/info` endpoint behavior** (behind `optional_auth_middleware`):
- Standalone mode: existing behavior (Setup -> ResourceAdmin -> Ready)
- Multi-tenant mode:
  - No dashboard token in session -> return `TenantSelection` (frontend shows login button)
  - Dashboard token but no active_client_id -> return `TenantSelection` (frontend shows tenant selector)
  - active_client_id set with valid resource-client token -> return `Ready`

**Response additions**:
- `deployment: "standalone" | "multi_tenant"` — frontend uses for feature visibility (e.g., hiding LLM features in multi-tenant)
- `client_id` — the active/standalone tenant's client_id. Frontend reads this for `POST /auth/initiate`

**Implementation note**: `setup_show` currently calls `app_status_or_default(&tenant_svc)`. In multi-tenant mode, status depends on session state (dashboard token, active_client_id), not just DB state. The `/info` endpoint needs access to the session or auth context (via `optional_auth_middleware`) to determine the correct status.

### 3. New auth routes for dashboard login

**`POST /bodhi/v1/auth/dashboard/initiate`**:
- Only available in multi-tenant mode (check `BODHI_DEPLOYMENT`); return error "not supported" in standalone
- Construct Keycloak login URL using `BODHI_MULTITENANT_CLIENT_ID` (= `client-bodhi-multi-tenant`) from settings
- Dashboard client secret from `BODHI_MULTITENANT_CLIENT_SECRET`
- Generate PKCE challenge, store state in session
- Return redirect URL

**`POST /bodhi/v1/auth/dashboard/callback`**:
- Exchange code for tokens using dashboard client credentials
- Store tokens in session: `dashboard:access_token`, `dashboard:refresh_token`
- Redirect to `/ui/login`

### 4. Tenant listing proxy endpoint

**`GET /bodhi/v1/tenants`**:
- Requires dashboard token in session
- **Dashboard token refresh**: Before proxying, check dashboard token expiry. If expired, use `dashboard:refresh_token` to get a new dashboard token from Keycloak. Update session. If refresh fails, redirect to dashboard re-login.
- Calls Keycloak SPI: `GET /realms/{realm}/bodhi/tenants` with dashboard token as Authorization header
- **HTTP client**: Use existing `reqwest` client in `AuthService` (already makes HTTP calls to Keycloak)

**SPI response**:
```json
{
  "tenants": [
    { "client_id": "bodhi-tenant-abc123", "name": "My Workspace", "description": "A team workspace" }
  ]
}
```
- Response wrapper field is `"tenants"`
- **No role field** — roles live in Keycloak groups; visible after tenant login via JWT claims
- `description` may be null

**Enrichment** (BodhiApp adds per client):
- `is_active`: true if client_id matches session's `active_client_id`
- `logged_in`: true if `{client_id}:access_token` exists in session and is not expired

**SPI error handling**: Transform SPI error responses (`{ "error": "message" }`) into OpenAI-compatible error format. SPI returns 401 for all auth failures. If SPI is unreachable, return 500.

**SPI auth error codes**:
| SPI Status | SPI Message | Meaning |
|------------|-------------|---------|
| 401 | `"invalid session"` | No or invalid bearer token |
| 401 | `"service account tokens not allowed"` | Service account token used |
| 401 | `"dashboard client not found"` | Token's azp client doesn't exist |
| 401 | `"token is not from a valid dashboard client"` | azp doesn't have `bodhi.client_type=multi-tenant` |

### 5. Tenant registration endpoint

**`POST /bodhi/v1/tenants`**:
- Requires dashboard token in session
- Request body: `{ name: string, description: string }` — user sends name + description only
- Backend adds `redirect_uris` internally: `["{BODHI_APP_URL}/ui/auth/callback"]` — resource callback only, reconstructed each time from config
- Calls Keycloak SPI: `POST /realms/{realm}/bodhi/tenants` with dashboard token in Authorization header

**SPI request body** (backend constructs from user input + config):
```json
{
  "name": "My Bodhi Instance",
  "description": "Optional description",
  "redirect_uris": ["https://cloud.getbodhi.app/ui/auth/callback"]
}
```

**SPI response** (201 Created):
```json
{ "client_id": "bodhi-tenant-{uuid}", "client_secret": "generated-secret" }
```

**SPI side effects**:
- Creates full resource client (roles, groups, service account) with `bodhi.client_type=resource`
- Inserts into `bodhi_clients` with `multi_tenant_client_id` = dashboard's client_id, `created_by_user_id` = token's `sub`
- Auto-makes creating user `resource_admin` (joins admins group + inserts `bodhi_clients_users` membership)

**SPI error codes**:
| Status | Message | Cause |
|--------|---------|-------|
| 400 | `"name is required"` | Missing name |
| 401 | (auth errors) | Same as GET /tenants |
| 409 | `"user already has a tenant for this dashboard"` | One-per-user constraint |

**BodhiApp post-SPI**:
- Creates tenant row: `{ client_id, encrypted_client_secret, status: Ready, created_by: user_id }` — same schema as standalone, secret encrypted same way
- **Atomicity**: If SPI succeeds but local tenant row creation fails, accept orphan. Log at critical/warning for manual Keycloak cleanup. No compensating delete.
- Tenant created with status `Ready` immediately — no setup wizard. API key configuration accessible from settings.
- Returns success with the new client_id (frontend auto-initiates resource-client login)

### 6. Enhanced existing auth routes

**`POST /bodhi/v1/auth/initiate`** (existing):
- `client_id` always required in request body (both modes)
- Standalone: frontend sends the single tenant's `client_id` (obtained from `/info` response)
- Multi-tenant: frontend sends the selected tenant's `client_id`
- Look up tenant by `get_tenant_by_client_id(client_id)`, construct Keycloak login URL
- **Redirect URI**: resource-client uses `/ui/auth/callback`, dashboard uses `/ui/auth/dashboard/callback`

**`POST /bodhi/v1/auth/callback`** (existing):
- **Breaking change**: Switch to namespaced keys immediately. Existing sessions won't have `active_client_id`, so middleware treats them as unauthenticated. Users re-login once.
- Uses `instance.client_id` (from `auth_client_id` session key lookup) for session key namespacing
- Store tokens: `{client_id}:access_token`, `{client_id}:refresh_token`
- Set `active_client_id` in session
- Redirect to `/ui/chat`
- Standalone `created_by` update: handled via `set_client_ready(client_id, user_id)` which sets both status=Ready and created_by in one call
- **Shared code exchange**: DEFERRED (D80 TECHDEBT) — code is duplicated between `routes_auth.rs` and `routes_dashboard_auth.rs`

### 7. Enhanced `/user/info` endpoint

**`GET /bodhi/v1/user/info`** (existing):
- Currently returns user info from resource-client token
- **Extend**: also return dashboard session info when dashboard token exists in session
- Frontend uses this to distinguish sub-states of `tenant_selection`:
  - No dashboard session -> show login button
  - Dashboard session exists -> call GET /tenants and show selector
- Exact response shape for dashboard state deferred — decide during implementation

**Implementation**: Handler reads dashboard token directly from session (independent of AuthContext). No AuthContext extension needed — dashboard tokens are a session concern, not a per-request auth concern.

### 8. Session key changes and middleware updates

**Session keys** (namespaced for both modes):
- `dashboard:access_token` — dashboard client token (multi-tenant only)
- `dashboard:refresh_token` — dashboard client refresh token
- `active_client_id` — currently selected resource-client
- `{client_id}:access_token` — resource-client tokens
- `{client_id}:refresh_token` — resource-client refresh tokens
- `user_id` — keep as-is (same user across all tenants)

**Middleware update**:
- `auth_middleware` and `optional_auth_middleware`: currently read `session[SESSION_KEY_ACCESS_TOKEN]`
- Change to two-step: read `session["active_client_id"]`, then `session[f"{active_client_id}:access_token"]`
- If no `active_client_id` in session, treat as no auth (Anonymous in optional, error in strict)
- `get_valid_session_token` already takes `&Tenant` — the middleware resolves tenant from `active_client_id` via `get_tenant_by_client_id`
- Migrate standalone to namespaced keys for consistency (no branching in middleware)

### 9. Tenant switching

**`POST /bodhi/v1/tenants/{client_id}/activate`** (new):
- Validates that `{client_id}:access_token` exists in session and is not expired
- Sets `active_client_id` in session
- Returns 200

**Switching flow**:
- Frontend shows tenant dropdown (from enriched `GET /tenants` with `is_active`, `logged_in`)
- If target client has `logged_in: true` -> call `POST /tenants/{client_id}/activate` (instant switch)
- If `logged_in: false` -> frontend initiates OAuth2 login for the new resource-client
- Frontend indicates switching to logged-out client may trigger login flow
- Keycloak SSO session reused -> instant redirect, no password entry

---

## Decision Summary

| # | Decision |
|---|----------|
| D29 | Keycloak SPI is source of truth for user's client list (not tenants table) |
| D30 | Auto-redirect for single client, dropdown for multiple |
| D31 | Standard OAuth2 auth code flow for resource-client login (not token exchange) |
| D32 | Keep all tokens in session, namespaced by client_id |
| D33 | Two-step middleware token lookup: read active_client_id -> read {client_id}:access_token |
| D35 | Separate dashboard auth endpoints. Existing /auth/initiate reused for resource-client |
| D36 | Tenant created with status Ready in multi-tenant mode |
| D37 | `created_by` column on tenants (nullable) |
| D39 | /ui/login page reused for tenant selection and switching |
| D41 | `POST /bodhi/v1/tenants` — same path for GET (list) and POST (create) |
| D47 | Dashboard callback redirects to /ui/login |
| D49 | `/user/info` extended for dashboard session detection |
| D50 | `GET /tenants` enriched with `is_active`, `logged_in` per client |
| D51 | Tenant row created during POST /tenants, pre-exists before callback |
| D52 | Accept orphans on tenant creation failure |
| D53 | Transparent dashboard token refresh before SPI proxy calls |
| D54 | `/info` behind `optional_auth_middleware` for session access |
| D55 | `POST /bodhi/v1/tenants/{client_id}/activate` for instant tenant switching |
| D56 | Breaking session key migration — namespaced keys immediately |
| D57 | Redirect URIs passed from backend in SPI request body |
| D58 | Deployment mode injection — deferred |
| D60 | Tenant registration API: name + description only, backend adds redirect_uris |
| D64 | SPI proxy errors -> OpenAI-compatible error format |
| D66 | `created_by` is Keycloak user ID (JWT `sub` claim) |
| D67 | `/info` returns `deployment: "standalone" \| "multi_tenant"` |
| D68 | `client_id` always required in `POST /auth/initiate` (both modes) |
| D69 | Tenant row: same schema, encrypted secret |
| D70 | `/info` includes `client_id` |
| D74 | Auth callback extracts `client_id` from JWT `azp` for session namespacing |
| D77 | Separate frontend callback routes: `/ui/auth/dashboard/callback` vs `/ui/auth/callback` |
| D79 | Tenant created Ready immediately, no setup wizard for multi-tenant |
| D80 | Shared parameterized code exchange utility (DEFERRED — TECHDEBT) |
| D81 | `/user/info` dashboard state response shape — deferred |
| D86 | Use existing reqwest in AuthService for SPI proxy calls |
| D88 | Redirect URI reconstructed from `public_server_url()` config each time |
| D89 | Four milestones: M1=SPI, M2=Backend, M3=Frontend+Backend, M4=Integration tests |
| D90 | ~~`BODHI_APP_URL` env var~~ — SUPERSEDED: uses `public_server_url()` |
| D91 | Single Keycloak realm — all tenants share one realm |
| D93 | Redirect URIs for tenant resource-clients: `{BODHI_APP_URL}/ui/auth/callback` only |

---

## Testing Infrastructure

### Dashboard Client for Local Testing

A dashboard client (type `multi-tenant`) pre-configured in the dev Keycloak:
- **Test Client ID**: `test-dashboard-client` — configurable via `BODHI_MULTITENANT_CLIENT_ID` env var
- **Test Client Secret**: `dashboard-secret` — configurable via `BODHI_MULTITENANT_CLIENT_SECRET` env var
- **Production Client ID**: configurable (e.g., `bodhi-dashboard`) — set by Keycloak admin
- **Direct Grant enabled**: programmatic token creation for tests
- **Client attribute**: `bodhi.client_type=multi-tenant`

This enables:
- **Rust unit tests**: Use direct grant to create dashboard tokens for test users. Call Keycloak token endpoint with `grant_type=password`, `client_id`, `client_secret`, `username`, `password`.
- **Isolation tests**: Create dashboard tokens for different test users, register resource-clients, verify tenant isolation.
- **All tests use real Keycloak**: No mocking of SPI HTTP calls. CI must have access to the dev Keycloak instance.

### Test utility pattern

```rust
async fn get_dashboard_token(username: &str, password: &str) -> String {
  // POST to Keycloak token endpoint with direct grant
  // client_id = test-dashboard-client (or from BODHI_MULTITENANT_CLIENT_ID)
  // client_secret = dashboard-secret (or from BODHI_MULTITENANT_CLIENT_SECRET)
  // grant_type = password
  // Returns access_token
}
```

### Test matrix

- `#[values("sqlite", "postgres")]` for DB tests
- Standalone vs multi-tenant mode where applicable

---

## Gate Checks

### Per-crate validation
```bash
cargo test -p services --lib 2>&1 | grep -E "test result|FAILED|failures:"
cargo test -p routes_app 2>&1 | grep -E "test result|FAILED|failures:"
cargo test -p server_app --lib 2>&1 | grep -E "test result|FAILED|failures:"
```

### Cross-crate regression
```bash
cargo test -p services -p server_core -p routes_app -p server_app --lib 2>&1 | grep -E "test result|FAILED|failures:"
```

### New tests required
- **services**: TenantService `created_by` field tests, AppStatus::TenantSelection serialization
- **routes_app**: Dashboard auth initiate/callback tests, tenant listing proxy tests (with session enrichment), tenant registration tests, enhanced auth_initiate with deployment branching, namespaced session key tests in middleware, /user/info with dashboard session
- **Isolation tests**: Multi-tenant login flow with sqlite/postgres x standalone/multi-tenant matrix

### Final test counts (post-M2 implementation)
- services: 832 tests passing
- routes_app: 649 tests passing (647 lib + 2 integration)
- server_app: 8 tests passing
- Total: 1489 tests, 0 failures, 0 warnings

### Updated test counts (post-M4 integration tests)
- services: 832 tests passing
- routes_app: 656 tests passing (647 unit + 2 live auth + 7 live multi-tenant)
- server_app: 8 unit + 3 integration (compile-verified, need real Keycloak)
- Total: 1496+ tests, 0 failures

---

## Implementation Summary

### What was implemented (6 phases)

| Phase | Scope | Commits |
|-------|-------|---------|
| 1 | Foundation: `created_by` column, `AppStatus::TenantSelection`, SPI types, AuthService proxy methods, `multitenant_client_secret()` | `6a7d879` |
| 2 | Session key namespacing (`{client_id}:access_token`, `active_client_id`), middleware two-step lookup, token refresh lock per-tenant | `53a2890` |
| 3 | Dashboard auth routes (`/auth/dashboard/initiate`, `/auth/dashboard/callback`), `ensure_valid_dashboard_token()` helper | `1113926` |
| 4 | Tenant management routes: `GET /tenants` (SPI proxy + enrichment), `POST /tenants` (SPI + local row), `POST /tenants/{client_id}/activate` | `215f679` |
| 5 | `/info` returns `deployment` + `client_id`; `/user/info` returns `has_dashboard_session` via `UserInfoEnvelope` | `880fc14` |
| 6 | TECHDEBT.md (ConcurrencyService advisory locks, session token lifecycle), CLAUDE.md updates | `c9e9dd6` |

### What was deferred from M2 (now resolved in M3)

| Item | Resolution |
|------|------------|
| D68: `client_id` required in `POST /auth/initiate` | **RESOLVED in M3**: `AuthInitiateRequest { client_id }`, uses `get_tenant_by_client_id()` |
| D77: Dashboard redirect URI mismatch | **RESOLVED in M3**: `dashboard_callback_url()` added to `SettingService` |
| `/info` session-aware status for multi-tenant | **RESOLVED in M3**: `resolve_multi_tenant_status()` checks session for dashboard token + SPI |

### Still deferred

| Item | Reason | Impact |
|------|--------|--------|
| D80: Shared code exchange utility | Duplicated between `routes_auth.rs` and `routes_dashboard_auth.rs` | Low priority, no functional impact |
| Multi-tenant-aware logout | `session.delete()` clears ALL tokens | Frontend logout in multi-tenant mode logs out of all tenants |

### Key implementation details for frontend

**New endpoints** (all behind `optional_auth_middleware`):
- `POST /bodhi/v1/auth/dashboard/initiate` — returns `{ location: "keycloak_auth_url" }` (201)
- `POST /bodhi/v1/auth/dashboard/callback` — body: `{ code, state }`, stores dashboard tokens, returns redirect
- `GET /bodhi/v1/tenants` — returns `{ tenants: [{ client_id, name, description, is_active, logged_in }] }`
- `POST /bodhi/v1/tenants` — body: `{ name, description }` (Validate), returns `{ client_id }`
- `POST /bodhi/v1/tenants/{client_id}/activate` — returns 200

**All return error `dashboard_auth_route_error-not_multi_tenant` when `BODHI_DEPLOYMENT != "multi_tenant"`.**

**Redirect URIs**:
- Resource clients: `{public_server_url()}/ui/auth/callback` (via `login_callback_url()`)
- Dashboard client: `{public_server_url()}/ui/auth/dashboard/callback` (via `dashboard_callback_url()`)
- No `BODHI_APP_URL` env var — uses `public_server_url()` (computed from `BODHI_PUBLIC_SCHEME/HOST/PORT`)
- Dashboard callback URL stored in session as `dashboard_callback_url`

**`BODHI_MULTITENANT_CLIENT_SECRET`**: Read from env var only (never DB). Must be set in deployment config.

**Dashboard test client** (in `server_app/tests/resources/.env.test`):
- `INTEG_TEST_MULTI_TENANT_CLIENT_ID=test-client-bodhi-multi-tenant`
- `INTEG_TEST_MULTI_TENANT_CLIENT_SECRET=<secret>`
- Redirect URI: `http://localhost:51135/ui/auth/dashboard/callback`
