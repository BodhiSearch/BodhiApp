# Multi-Tenant Flow — Comprehensive Context

> **Created**: 2026-03-06
> **Updated**: 2026-03-09
> **Purpose**: Preserve all design context from the multi-tenant interview/discussion so nothing is lost across sessions.
> **Companion kickoffs**: `kickoff-keycloak-spi.md` (completed), `kickoff-bodhi-backend.md`, `kickoff-bodhi-frontend.md`, `kickoff-integ-test-multi-tenant.md` (completed)
> **SPI integration doc**: `keycloak-bodhi-ext/ai-docs/claude-plans/20260307-multi-tenant/20260307-tenants-integration-doc.md`

---

## Table of Contents

1. [Deployment Modes](#deployment-modes)
2. [Two-Phase Login Flow (Multi-Tenant)](#two-phase-login-flow-multi-tenant)
3. [Standalone Flow Adaptation](#standalone-flow-adaptation)
4. [Session Architecture](#session-architecture)
5. [Middleware Token Lookup](#middleware-token-lookup)
6. [AppStatus State Machine](#appstatus-state-machine)
7. [Keycloak SPI (Implemented)](#keycloak-spi-implemented)
8. [Tenant Registration Flow](#tenant-registration-flow)
9. [Tenant Selection & Switching](#tenant-selection--switching)
10. [Auth Endpoint Design](#auth-endpoint-design)
11. [Service Construction by Deployment Mode](#service-construction-by-deployment-mode)
12. [Frontend Flow](#frontend-flow)
13. [Testing Infrastructure](#testing-infrastructure)
14. [Decision Index](#decision-index)

---

## Deployment Modes

BodhiApp supports two deployment modes controlled by `BODHI_DEPLOYMENT` env var:

| Mode | Value | Description |
|------|-------|-------------|
| **Standalone** | `standalone` (default) | Single-tenant, single user or small team. Desktop (Tauri) or self-hosted Docker. One Keycloak resource-client. |
| **Multi-Tenant** | `multi_tenant` | SaaS deployment (e.g., cloud.getbodhi.app). Multiple tenants sharing one BodhiApp instance. Dashboard client + N resource-clients. |

The system is designed so that multi-tenant code paths **do not conflict** with standalone — standalone is effectively multi-tenant with exactly one tenant and no dashboard phase.

---

## Two-Phase Login Flow (Multi-Tenant)

```
Phase 1: Dashboard Authentication
----------------------------------
1. User visits cloud.getbodhi.app
2. GET /bodhi/v1/info -> status: "tenant_selection"
   (for bodhi_deployment='multi-tenant'; for bodhi_deployment='standalone',
    it gets the app status for the standalone app and shows)
3. Frontend shows "Login to Bodhi Platform" button
4. POST /bodhi/v1/auth/dashboard/initiate
   -> Constructs Keycloak OAuth2 URL for BODHI_MULTITENANT_CLIENT_ID (=client-bodhi-multi-tenant)
   -> (if bodhi_deployment!='multi-tenant', /auth/dashboard/* returns error not supported)
   -> Generates PKCE challenge, stores state in session
   -> Returns redirect URL to Keycloak login page
5. User authenticates at Keycloak -> redirected back with auth code
6. POST /bodhi/v1/auth/dashboard/callback
   -> Exchanges code for dashboard tokens (azp = client-bodhi-multi-tenant)
   -> Stores tokens: session["dashboard:access_token"],
      session["dashboard:refresh_token"]
   -> Redirects to /ui/login

Phase 2: Tenant Selection + Resource-Client Authentication
----------------------------------------------------------
7. /ui/login calls GET /bodhi/v1/user/info to check dashboard session state
   -> Returns dashboard user info if dashboard token exists
8. Frontend calls GET /bodhi/v1/tenants
   -> BodhiApp proxies to Keycloak SPI: GET /realms/{realm}/bodhi/tenants
   -> Uses dashboard access_token as Authorization header
      (validates for azp=BODHI_MULTITENANT_CLIENT_ID)
   -> BodhiApp enriches response with session metadata (is_active, logged_in per client)
   -> Returns list of user's resource-clients

9a. If 0 clients -> Frontend redirects to /ui/setup/tenants/
    -> User fills registration form (name, description)
    -> POST /bodhi/v1/tenants { name, description }
    -> BodhiApp proxies to SPI with dashboard token
    -> SPI creates resource-client, returns client_id + client_secret
    -> BodhiApp creates tenant row (status: Ready, created_by: user_id from dashboard token)
    -> Frontend auto-initiates resource-client login

9b. If 1 client -> Auto-redirect to resource-client OAuth2 (no user interaction needed)

9c. If N clients -> Show tenant selector dropdown -> user picks one

10. POST /bodhi/v1/auth/initiate { client_id }
    -> Uses client_id from request body (selected tenant)
    -> Constructs Keycloak OAuth2 URL for the selected resource-client
    -> PKCE + state stored in session

11. Keycloak SSO session already exists from Phase 1 -> user gets resource-client token
    WITHOUT re-entering password (instant redirect back)

12. POST /bodhi/v1/auth/callback
    -> Exchanges code for resource-client tokens
    -> Uses instance.client_id from session's auth_client_id lookup
    -> Stores: session["{client_id}:access_token"],
       session["{client_id}:refresh_token"]
    -> Sets session["active_client_id"] = client_id
    -> Redirects to /ui/chat

13. GET /bodhi/v1/info -> status: "ready" (active tenant selected, valid token)
```

**Key insight**: The Keycloak SSO session from Phase 1 is reused in Phase 2, so the second OAuth2 redirect is instant — the user never sees the Keycloak login page again. This makes the two-phase flow feel seamless.

---

## Standalone Flow Adaptation

Standalone mode adapts to the same code paths without conflicts:

### What stays the same
- **JWT-based tenant resolution**: Middleware resolves tenant from JWT `azp` claim — works identically for standalone's single tenant
- **Session token storage**: Migrated to namespaced keys (`{client_id}:access_token`) — breaking change, users re-login once
- **`begin_tenant_txn(tenant_id)`**: Works the same — standalone just always uses the one tenant_id
- **`POST /auth/initiate`**: Always requires `client_id` in body. Standalone frontend reads client_id from `/info` response

### What differs
| Aspect | Standalone | Multi-Tenant |
|--------|-----------|--------------|
| Dashboard phase | Skipped entirely | Required first step |
| `/info` status flow | Setup -> ResourceAdmin -> Ready | TenantSelection -> Ready |
| `/info` response | Includes `deployment: "standalone"`, `client_id` | Includes `deployment: "multi_tenant"`, `client_id` if active |
| Tenant count | Always 1 | 0..N |
| `created_by` on tenant | Keycloak user ID set during ResourceAdmin -> Ready (via `set_client_ready`) | Keycloak user ID set during `POST /tenants` |
| SPI registration | `POST /resources` (anonymous). Client ID: `bodhi-resource-<UUID>` | `POST /tenants` (dashboard token). Client ID: `bodhi-tenant-<UUID>` |
| Setup steps | Full: Login -> Models -> API Keys -> Toolsets -> Extension -> Complete | No mandatory setup wizard — tenant Ready immediately |
| `/auth/dashboard/*` routes | Return error "not supported in standalone mode" | Fully functional |
| Session keys | Namespaced: `{client_id}:access_token`, `{client_id}:refresh_token`, `active_client_id` | Same namespaced pattern + `dashboard:access_token`, `dashboard:refresh_token` |

### SPI endpoint separation
- **`POST /realms/{realm}/bodhi/resources`** — Standalone only. Anonymous registration during setup wizard. Client ID: `bodhi-resource-<UUID>`. Now dual-writes to `bodhi_clients` table (null `multi_tenant_client_id`, null `created_by_user_id`). Test env uses `test-resource-<UUID>`.
- **`POST /realms/{realm}/bodhi/tenants`** — Multi-tenant only. Requires dashboard token. Creates resource-client with ID `bodhi-tenant-<UUID>` + auto-assigns creator as admin + writes to `bodhi_clients` and `bodhi_clients_users`. Test env uses `test-tenant-<UUID>`.

### `created_by` column
Added to `tenants` table (nullable VARCHAR(255)):
- **Value**: Keycloak user ID (JWT `sub` claim) in both modes
- **Standalone**: Set during `auth_callback` ResourceAdmin -> Ready transition via `set_client_ready(client_id, user_id)` which handles both status update and `created_by` in one call
- **Multi-tenant**: Set during `POST /bodhi/v1/tenants` (from dashboard token's `sub` claim)
- **Existing tenants**: NULL (migration adds nullable column)

---

## Session Architecture

### Session key schema

```
Multi-tenant session layout:
+-- dashboard:access_token     -- Dashboard client JWT (only in multi-tenant mode)
+-- dashboard:refresh_token    -- Dashboard client refresh token
+-- active_client_id           -- Currently selected resource-client ID
+-- {client_id_A}:access_token -- Resource-client A's JWT
+-- {client_id_A}:refresh_token
+-- {client_id_B}:access_token -- Resource-client B's JWT (if user switched)
+-- {client_id_B}:refresh_token
+-- user_id                    -- Keycloak user ID (same across all tenants)

Standalone session layout:
+-- active_client_id           -- Always the single tenant's client_id
+-- {client_id}:access_token   -- The single resource-client JWT
+-- {client_id}:refresh_token
+-- user_id

Session key constants and helper functions are defined in `services::session_keys`
and re-exported from the `services` crate.
```

### Why namespaced keys
- **Tenant switching without re-login**: When a user switches from tenant A to tenant B, both tokens coexist in the session. If tenant B's token is still valid, the switch is instant via `POST /bodhi/v1/tenants/{client_id}/activate`.
- **Dashboard + resource tokens coexist**: The dashboard token and resource-client tokens live side by side, enabling the two-phase flow.
- **Consistent code path**: Middleware always does `session["active_client_id"]` -> `session["{active_client_id}:access_token"]`, regardless of deployment mode.
- **Breaking migration**: Standalone switches from flat keys to namespaced keys immediately. Users re-login once.

### Session store
SQLite-backed via `tower_sessions`. Same store for both deployment modes.

### Dashboard token refresh
Before proxying to the SPI, the backend transparently refreshes expired dashboard tokens using `dashboard:refresh_token`. If refresh fails, redirects to dashboard re-login.

---

## Middleware Token Lookup

The middleware (already refactored to JWT-based tenant resolution) follows a two-step lookup:

```
auth_middleware / optional_auth_middleware:
1. Read session["active_client_id"]
   -> If missing: Anonymous (optional) or 401 (strict)
2. Read session["{active_client_id}:access_token"]
   -> If missing: Anonymous (optional) or 401 (strict)
3. Decode JWT, extract azp claim
4. Resolve tenant: get_tenant_by_client_id(azp)
   -> If not found: 500 (TenantError::NotFound has ErrorType::InternalServer)
5. Validate token expiry via get_valid_session_token(&tenant)
   -> If expired: attempt refresh using session["{active_client_id}:refresh_token"]
   -> If refresh succeeds: update session tokens, continue
   -> If refresh fails: Anonymous (optional) or 401 (strict)
6. Build AuthContext::Session { user_id, tenant_id, ... }
```

For **API tokens** (Bearer header with `bodhiapp_` prefix):
- Extract token, look up by prefix hash in DB (cross-tenant by design)
- Verify SHA-256 hash of full token string
- Resolve tenant from `client_id` suffix: `bodhiapp_<random>.<client_id>`

For **external app tokens** (Bearer header, non-`bodhiapp_` prefix):
- Decode JWT, extract `aud` claim (not `azp` — external tokens use audience)
- Resolve tenant: `get_tenant_by_client_id(aud)`

---

## AppStatus State Machine

### Standalone mode
```
[No tenants in DB]
    -> Setup
        | POST /bodhi/v1/setup (registers client via SPI /resources, creates tenant)
    -> ResourceAdmin
        | First login callback (calls SPI /make-resource-admin, transitions status)
    -> Ready
```

### Multi-tenant mode
```
[No dashboard token in session]
    -> TenantSelection (frontend shows login button)
        | Dashboard OAuth2 login -> /ui/login
[Dashboard token, no active_client_id]
    -> TenantSelection (frontend shows tenant selector or registration)
        | User selects/creates tenant + resource-client OAuth2
[active_client_id set, valid resource-client token]
    -> Ready
```

### `/info` endpoint logic (as implemented)
```rust
// In setup_show handler:
let (status, client_id) = if settings.is_multi_tenant().await {
    resolve_multi_tenant_status(&auth_scope, &session).await?
} else {
    let status = app_status_or_default(&tenant_svc).await;
    let client_id = auth_scope.auth_context().client_id().map(|s| s.to_string());
    (status, client_id)
};

// resolve_multi_tenant_status:
// 1. Has active_client_id + {client_id}:access_token? -> (Ready, Some(client_id))
// 2. Has dashboard:access_token?
//    a. ensure_valid_dashboard_token() + list_tenants()
//    b. 0 tenants -> (Setup, None)
//    c. 1+ tenants -> (TenantSelection, None)
//    d. SPI error -> (TenantSelection, None)
//    e. Token refresh failed -> (TenantSelection, None)
// 3. No dashboard token -> (TenantSelection, None)
```

**Important**: In multi-tenant mode, `/info` status depends on **session state**, not just DB state. The handler accepts `Session` parameter for multi-tenant status resolution. `/info` also returns `deployment: "standalone"|"multi_tenant"` and the active tenant's `client_id`.

---

## Keycloak SPI (Implemented)

> Full details: `kickoff-keycloak-spi.md` and `keycloak-bodhi-ext/ai-docs/claude-plans/20260307-multi-tenant/20260307-tenants-integration-doc.md`

### Client type: `multi-tenant`
Dashboard clients have attribute `bodhi.client_type=multi-tenant`. Provisioned by Keycloak admin, NOT via SPI. The SPI validates dashboard tokens by checking this attribute.

### SPI Tables

**`bodhi_clients`** — One row per resource client (both standalone and multi-tenant):
- `id` VARCHAR(36) PK, `realm_id`, `client_id` (string, UNIQUE), `multi_tenant_client_id` (nullable — dashboard's client_id, NULL for standalone), `created_by_user_id` (nullable), `created_at`, `updated_at`

**`bodhi_clients_users`** — Membership proxy (no role column):
- `id` VARCHAR(36) PK, `realm_id`, `client_id` (string), `user_id` (UUID), `created_at`, `updated_at`, UNIQUE(client_id, user_id)

**Role source of truth**: Keycloak groups (`users-{clientId}` with subgroups: users, power-users, managers, admins). The `bodhi_clients_users` table tracks membership only — for fast `GET /tenants` queries.

### SPI Endpoints

| Operation | Method + Path | Auth | Response |
|-----------|--------------|------|----------|
| Create tenant | `POST /bodhi/tenants` | User token (azp=dashboard) | 201: `{client_id, client_secret}` |
| List tenants | `GET /bodhi/tenants` | User token (azp=dashboard) | 200: `{tenants: [{client_id, name, description}]}` |

- No role field in GET /tenants response — role visible after tenant login (from JWT claims)
- All auth errors return 401 (not 403)
- All errors: `{ "error": "descriptive message" }`

### Dual-Write Behavior

| Endpoint | bodhi_clients | bodhi_clients_users |
|----------|--------------|---------------------|
| POST /resources | INSERT (null multi_tenant, null created_by) | — |
| POST /tenants | INSERT (with multi_tenant + created_by) | INSERT membership |
| make-resource-admin | SET created_by_user_id | INSERT membership |
| assign-role | — | INSERT membership |
| remove-user | — | DELETE membership |

---

## Tenant Registration Flow

### Multi-tenant registration
```
1. User has dashboard token, 0 existing clients
2. Frontend redirects to /ui/setup/tenants/
3. Shows registration form: name (required, min 1, max 255), description (required, min 1, max 1000)
4. POST /bodhi/v1/tenants { name, description }
5. Backend:
   a. Extract user_id from dashboard token's sub claim
   b. Add redirect_uris from BODHI_APP_URL config: ["{BODHI_APP_URL}/ui/auth/callback"]
   c. Proxy to SPI: POST /realms/{realm}/bodhi/tenants
      - Authorization: Bearer <dashboard_access_token>
      - Body: { name, description, redirect_uris }
   d. SPI creates resource-client (201) -> returns { client_id, client_secret }
   e. BodhiApp creates tenant row:
      - client_id, encrypted client_secret
      - status: Ready (no setup wizard for multi-tenant)
      - created_by: user_id from dashboard token
   f. Return { client_id } to frontend
6. Frontend auto-initiates resource-client OAuth2 with new client_id
```

### Standalone registration (existing setup flow)
```
1. GET /info -> status: "setup"
2. POST /bodhi/v1/setup
   - Calls SPI: POST /realms/{realm}/bodhi/resources (NO Authorization header)
   - SPI dual-writes to bodhi_clients (null multi_tenant, null created_by)
   - Creates tenant with status: ResourceAdmin
   - created_by: NULL (set later during first login)
3. POST /bodhi/v1/auth/initiate { client_id } -> OAuth2 login
4. POST /bodhi/v1/auth/callback -> exchanges code
   - Calls SPI /make-resource-admin -> assigns admin (dual-writes to bodhi_clients + bodhi_clients_users)
   - Calls `set_client_ready(client_id, user_id)` — transitions ResourceAdmin -> Ready AND sets `created_by` in one call
```

---

## Tenant Selection & Switching

### Initial selection (after dashboard login)
```
/ui/login with dashboard token:
1. Call GET /bodhi/v1/user/info -> check dashboard session state
2. Call GET /bodhi/v1/tenants -> enriched with is_active, logged_in
   0 clients -> redirect to /ui/setup/tenants/ (registration)
   1 client  -> auto-redirect to resource-client OAuth2
   N clients -> show dropdown with current active client selected
```

### Switching to another tenant
```
1. User navigates to /ui/login (doubles as tenant switch page)
2. Page shows:
   - Dropdown of available tenants (from GET /tenants, enriched)
   - Current active tenant highlighted
   - Login status per tenant (logged_in)
3. User selects tenant B:
   a. If logged_in for tenant B:
      -> POST /bodhi/v1/tenants/{client_id}/activate (instant switch)
   b. If not logged_in for tenant B:
      -> POST /auth/initiate with client_id_B
      -> Keycloak SSO -> instant redirect (no password)
      -> POST /auth/callback -> stores token, sets active_client_id
```

### Logout behavior
- **Logout from resource-client**: Clear `{active_client_id}:access_token`, clear `active_client_id`. Dashboard token preserved -> user returns to tenant selector on /ui/login.
- **Full logout**: Clear all session data including dashboard tokens -> user returns to login page.

---

## Auth Endpoint Design

### New endpoints (multi-tenant only)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/bodhi/v1/auth/dashboard/initiate` | POST | Start dashboard OAuth2 flow |
| `/bodhi/v1/auth/dashboard/callback` | POST | Complete dashboard OAuth2, store dashboard tokens, redirect to /ui/login |
| `/bodhi/v1/tenants` | GET | List user's resource-clients (proxy to SPI, enriched with `is_active`, `logged_in`) |
| `/bodhi/v1/tenants` | POST | Register new resource-client (proxy to SPI + create tenant row). Body: `{ name, description }` |
| `/bodhi/v1/tenants/{client_id}/activate` | POST | Instant tenant switch — set active_client_id if cached token is valid |

These endpoints return error "not supported" in standalone mode.

### Enhanced existing endpoints

**`POST /bodhi/v1/auth/initiate`** — ✅ IMPLEMENTED:
- `client_id` always required in request body via `AuthInitiateRequest { client_id: String }` (D68)
- Uses `get_tenant_by_client_id(&request.client_id)` — no more `get_standalone_app()`
- Stores `auth_client_id` in session for callback retrieval
- Standalone frontend sends `client_id` obtained from `/info` response

**`POST /bodhi/v1/auth/callback`** — ✅ IMPLEMENTED:
- Reads `auth_client_id` from session to look up tenant via `get_tenant_by_client_id()`
- Uses `instance.client_id` (from tenant lookup) for session key namespacing (NOT JWT `azp` — the tenant's `client_id` is used directly)
- Stores tokens under namespaced keys: `{client_id}:access_token`, `{client_id}:refresh_token`
- Sets `active_client_id` in session
- Cleans up `auth_client_id` and `callback_url` from session after exchange
- Code exchange NOT shared with dashboard callback (duplicated, low-priority TECHDEBT)
- Redirects to `/ui/chat`
- Standalone extra: during ResourceAdmin -> Ready transition, calls `make_resource_admin` + `set_client_ready(client_id, user_id)` (sets both status=Ready and created_by in one call)

**`GET /bodhi/v1/info`** — ✅ IMPLEMENTED:
- Behind `optional_auth_middleware` (D54 implemented). `AuthScope` populates `AuthContext::Session` for authenticated users, falls back to `Anonymous` for unauthenticated requests.
- Accepts `Session` parameter for multi-tenant status resolution
- Returns `deployment: String` (from `settings.deployment_mode()`) and `client_id: Option<String>`
- Standalone: existing DB-based status logic via `app_status_or_default()`, `client_id` from `AuthContext` (populated for authenticated users via optional_auth_middleware)
- Multi-tenant: `resolve_multi_tenant_status()` checks session state:
  1. Has `active_client_id` with valid resource token -> `Ready` + `client_id`
  2. Has dashboard token -> calls `ensure_valid_dashboard_token()` + SPI `list_tenants()`:
     - 0 tenants -> `Setup`
     - 1+ tenants -> `TenantSelection`
     - SPI error -> `TenantSelection` (user can retry)
  3. No dashboard token -> `TenantSelection`

**`GET /bodhi/v1/user/info`** — ✅ IMPLEMENTED:
- Extended with `has_dashboard_session: bool` (only serialized when true, via `skip_serializing_if`)
- Reads `dashboard:access_token` key from session (independent of AuthContext)
- Returns `UserInfoEnvelope { #[serde(flatten)] user: UserResponse, has_dashboard_session: bool }`

### Dashboard client credentials
- `BODHI_MULTITENANT_CLIENT_ID` — setting (DB or env), read via `settings.multitenant_client_id()`
- `BODHI_MULTITENANT_CLIENT_SECRET` — env var only (never in DB), read via `settings.multitenant_client_secret()` which calls `get_env()`
- Redirect URIs constructed from `settings.public_server_url()` — no `BODHI_APP_URL` env var exists
- Dashboard client uses standard confidential client with PKCE (not public client)

---

## Service Construction by Deployment Mode

In multi-tenant SaaS mode, certain services are unnecessary or different:

| Service | Standalone | Multi-Tenant |
|---------|-----------|--------------|
| InferenceService | Active (local LLM) | Disabled (no local models) |
| HubService | Active (model downloads) | Disabled (no model management) |
| DataService | Active (model aliases) | Reduced (API models only) |
| LLM routes | Registered | Skipped |
| Setup routes | Registered | Skipped (replaced by tenant registration) |

The `BODHI_DEPLOYMENT` setting influences `AppServiceBuilder` in `lib_bodhiserver` to conditionally initialize services and register routes.

### Setup flow per mode
- **Standalone**: Login -> Download Models -> Local Models/LLM Engine -> API Keys -> Toolsets -> Browser Extension -> Complete (7 steps)
- **Multi-tenant**: No mandatory setup wizard — tenant Ready immediately. API key config accessible via settings.

---

## Frontend Flow — IMPLEMENTED

### AppInitializer changes (`components/AppInitializer.tsx`)
- `allowedStatus` accepts `AppStatus | AppStatus[]`
- Handles: `setup` (standalone -> setup wizard, multi-tenant -> `/ui/setup/tenants`), `resource_admin` -> resource admin page, `ready` -> app, `tenant_selection` -> `/ui/login`

### Login page (`/ui/login/page.tsx`) — multi-purpose
Two main components: `StandaloneLoginContent` and `MultiTenantLoginContent`.

**Standalone mode**:
- Not logged in: "Login" button with `client_id` from `/info`
- Logged in: user info + home/logout buttons

**Multi-tenant mode** (`MultiTenantLoginContent`):
- **No dashboard session**: "Login to Bodhi Platform" button -> dashboard OAuth
- **Dashboard session, 1 tenant**: Auto-initiate resource OAuth (seamless with Keycloak SSO)
- **Dashboard session, N tenants**: Tenant selector with `is_active`/`logged_in` status. Connect button triggers activate (if `logged_in`) or OAuth (if not)
- **Fully authenticated**: Current tenant info + switch dropdown + logout

### Frontend callback routes
- **Dashboard OAuth**: Redirects to `/ui/auth/dashboard/callback/page.tsx` -> POSTs to `/bodhi/v1/auth/dashboard/callback` -> redirects to `/ui/login`
- **Resource-client OAuth**: Redirects to `/ui/auth/callback/page.tsx` -> POSTs to `/bodhi/v1/auth/callback` -> redirects to `/ui/chat`
- Separate routes, separate redirect_uris registered in Keycloak

### Registration page (`/ui/setup/tenants/page.tsx`)
- Form: name (required, min 1 char, max 255), description (required, min 1 char, max 1000)
- Uses `useCreateTenant()` mutation -> POST `/bodhi/v1/tenants`
- On success, auto-initiates resource OAuth with returned `client_id`
- Tenant created with status Ready immediately — no setup wizard

### New hooks (`hooks/useAuth.ts`, `hooks/useTenants.ts`)
- `useDashboardOAuthInitiate()` — POST `/bodhi/v1/auth/dashboard/initiate`
- `useDashboardOAuthCallback()` — POST `/bodhi/v1/auth/dashboard/callback`, body: `{ code, state }`
- `useTenants(options?)` — GET `/bodhi/v1/tenants`, query key: `'tenants'`, respects `enabled` flag
- `useCreateTenant()` — POST `/bodhi/v1/tenants`, invalidates `'tenants'` on success
- `useTenantActivate()` — POST `/bodhi/v1/tenants/{client_id}/activate`, invalidates `'tenants'` and `'info'` on success
- `useOAuthInitiate()` — updated to accept `{ client_id: string }` variable

### TypeScript types (regenerated)
Types available in `@bodhiapp/ts-client`:
- `AppStatus` enum — includes `tenant_selection`
- `AppInfo` — `deployment: string`, `client_id?: string | null`
- `AuthInitiateRequest` — `{ client_id: string }`
- `TenantListResponse` — `{ tenants: TenantListItem[] }`
- `TenantListItem` — `{ client_id, name, description?, is_active, logged_in }`
- `CreateTenantRequest` — `{ name: string, description: string }` (both required, with validation)
- `CreateTenantResponse` — `{ client_id }`
- ~~`DashboardAuthCallbackRequest`~~ — does NOT exist as a separate type; dashboard callback reuses `AuthCallbackRequest` (`{ code, state, error, error_description }`)
- `UserInfoEnvelope` — extends UserResponse with `has_dashboard_session?: boolean`

Regenerate after API changes: `cargo run --package xtask openapi && cd ts-client && npm run generate`

---

## Testing Infrastructure

### Dashboard client for testing
A dashboard client (type `multi-tenant`) pre-configured in the dev Keycloak at `main-id.getbodhi.app`:
- **Test Client ID**: `test-dashboard-client` — configurable via `BODHI_MULTITENANT_CLIENT_ID` env var
- **Test Client Secret**: `dashboard-secret` — configurable via `BODHI_MULTITENANT_CLIENT_SECRET` env var
- **Direct Grant enabled**: programmatic token creation for tests

### Integration testing strategy
- SPI deployed to `main-id.getbodhi.app` dev Keycloak
- All backend tests use real Keycloak — no SPI mocking
- CI must have network access to dev Keycloak
- SPI tests use existing patterns: unit tests + Testcontainers integration

### M4 Integration Tests (implemented)

Two levels of integration tests exercise the multi-tenant HTTP endpoints against real Keycloak:

**routes_app oneshot tests** (`crates/routes_app/tests/test_live_multi_tenant.rs`) — 7 tests:
- Use `tower::oneshot()` with real Keycloak tokens (no TCP server)
- Helpers: `create_multi_tenant_state()`, `create_standalone_state()`, `inject_session()`, `get_dashboard_token()`, `get_resource_token()`
- Tests: `/info` states, dashboard auth rejection in standalone, tenant activate success/failure, `/user/info` dashboard session detection

**server_app live TCP tests** (`crates/server_app/tests/test_live_multi_tenant.rs`) — 3 tests:
- Real TCP server on port 51135, real Keycloak, serial execution
- Helpers in `crates/server_app/tests/utils/live_server_utils.rs`: `setup_multitenant_app_service()`, `start_multitenant_live_server()`, `create_dashboard_session()`, `add_resource_token_to_session()`, `get_dashboard_token_via_password_grant()`, `get_resource_token_via_password_grant()`
- Tests: full 12-step e2e flow, state progression, standalone rejection

**Dev-only test support endpoints** (D106, not exposed in production):
- `POST /dev/clients/{client_id}/dag` — enables Direct Access Grants on a KC client via SPI proxy, returns `{client_id, client_secret}`
- `DELETE /dev/tenants/cleanup` — cleans up KC tenants via SPI + truncates local tenants table

### Test utility
```rust
async fn get_dashboard_token(username: &str, password: &str) -> String {
    // POST to Keycloak token endpoint at main-id.getbodhi.app
    // grant_type=password, client_id=test-dashboard-client, client_secret=dashboard-secret
    // Returns access_token
}
```

### Test matrix
- `#[values("sqlite", "postgres")]` for DB tests
- Standalone vs multi-tenant mode where applicable
- Isolation tests: cross-tenant visibility checks

### SPI HTTP proxy testing
BodhiApp uses existing `reqwest` client in `AuthService` for SPI proxy calls. Tests call real SPI endpoints on dev Keycloak.

---

## Decision Index

| # | Decision |
|---|----------|
| D29 | Keycloak SPI is source of truth for user's client list (not BodhiApp tenants table) |
| D30 | Auto-redirect for single client, dropdown for multiple |
| D31 | Standard OAuth2 auth code flow for resource-client login (not token exchange) |
| D32 | Keep all tokens in session, namespaced by client_id. Dashboard + resource-client tokens coexist |
| D33 | Two-step middleware token lookup: read active_client_id -> read {client_id}:access_token |
| D35 | Separate dashboard auth endpoints. Existing /auth/initiate reused with deployment-mode branching |
| D36 | Tenant created with status Ready in multi-tenant mode (skip ResourceAdmin dance) |
| D37 | `created_by` column on tenants (nullable). Standalone: NOT yet set during auth_callback (TECHDEBT). Multi-tenant: set during POST /tenants |
| D38 | Multi-tenant setup steps: no mandatory wizard — tenant Ready immediately |
| D39 | /ui/login page reused for tenant selection and switching |
| D41 | `POST /bodhi/v1/tenants` — same path for GET (list) and POST (create), standard REST |
| D42 | SPI: `GET/POST /realms/{realm}/bodhi/tenants` — separate from `/resources`, no trailing slash |
| D43 | Reuse `/resources/*` for role management, centralized dual-write to `bodhi_clients_users` |
| D44 | 4-level role hierarchy: resource_admin > resource_manager > resource_power_user > resource_user (in Keycloak groups) |
| D45 | `/make-resource-admin` dual-writes to `bodhi_clients` (created_by) + `bodhi_clients_users` (membership) |
| D47 | Dashboard callback redirects to `/ui/login` |
| D48 | Registration UI at `/ui/setup/tenants/`, calls `POST /bodhi/v1/tenants` |
| D49 | `/user/info` extended for dashboard session detection — exact shape deferred |
| D50 | `GET /tenants` enriched with `is_active`, `logged_in` per client |
| D51 | Tenant row created during `POST /bodhi/v1/tenants`, pre-exists before resource-client callback |
| D52 | Accept orphans on tenant creation failure — log for manual cleanup |
| D53 | Transparent dashboard token refresh before SPI proxy calls |
| D54 | `/info` behind `optional_auth_middleware` for session access |
| D55 | `POST /bodhi/v1/tenants/{client_id}/activate` for instant tenant switch |
| D56 | Breaking session key change — namespaced keys immediately, users re-login |
| D57 | BodhiApp backend passes redirect_uris to SPI from `BODHI_APP_URL` config |
| D58 | Deployment mode injection into handlers — deferred |
| D59 | SPI uses JPA with custom entities + Liquibase changelog |
| D60 | `POST /tenants` user API: `{ name, description }` only, backend adds redirect_uris |
| D62 | Client type validation via `bodhi.client_type` Keycloak attribute |
| D63 | Logout scope semantics — deferred |
| D64 | SPI proxy errors -> OpenAI-compatible error body. SPI returns `{ "error": "message" }` |
| D65 | One-client-per-user hard limit, defer expansion |
| D66 | `created_by` = Keycloak user ID (sub claim) |
| D67 | `/info` returns `deployment: standalone\|multi_tenant` |
| D68 | `client_id` always required in `POST /auth/initiate` |
| D69 | Tenant row same schema, encrypted secret |
| D70 | `/info` includes `client_id` for active/standalone tenant |
| D71 | SPI tables via Liquibase changelog in `META-INF/bodhi-changelog.xml` |
| D72 | SPI deployed to `main-id.getbodhi.app` dev env first |
| D73 | All backend tests use real Keycloak, no SPI mocking |
| D74 | Auth callback uses tenant's `client_id` (from `auth_client_id` session lookup) for session namespacing (NOT JWT `azp`) |
| D75 | No trailing slash on SPI endpoints |
| D76 | Sequential dev: SPI -> Backend -> Frontend |
| D77 | Separate frontend callback routes: `/ui/auth/dashboard/callback`, `/ui/auth/callback` |
| D78 | SPI is source of truth for login-able clients, BodhiApp tenants table is local state |
| D79 | Tenant Ready immediately, no mandatory setup wizard |
| D80 | Shared parameterized code exchange utility for dashboard + resource callbacks |
| D81 | `/user/info` dashboard state — implemented as `UserInfoEnvelope` with `has_dashboard_session: bool` |
| D82 | Client naming: `bodhi-tenant-<UUID>` (multi-tenant), `bodhi-resource-<UUID>` (standalone) |
| D83 | Both ID renames, existing clients keep old IDs. Test: `test-resource-<UUID>`, `test-tenant-<UUID>` |
| D84 | Keycloak groups = source of truth for roles. `bodhi_clients_users` tracks membership only for fast queries |
| D85 | SPI tests follow existing patterns (unit + Testcontainers) |
| D86 | Use existing reqwest in AuthService for SPI proxy calls |
| D88 | Redirect URI reconstructed from `BODHI_APP_URL` config, not stored |
| D89 | Four milestones: M1=SPI (done), M2=Backend (done), M3=Frontend+Backend prerequisites (done), M4=Integration tests (done) |
| D90 | ~~`BODHI_APP_URL` env var~~ — SUPERSEDED: uses `public_server_url()` (from `BODHI_PUBLIC_SCHEME/HOST/PORT`) |
| D91 | Single Keycloak realm — all tenants share one realm |
| D93 | Redirect URIs for tenant resource-clients: `{public_server_url()}/ui/auth/callback` only |
| D103 | `forward_spi_request` uses owned String params for mockall compatibility |
| D104 | `DefaultDbService` uses builder pattern `.with_env_type()` for env_type injection |
| D105 | D99 superseded — `ensure_valid_dashboard_token` uses TimeService (not SystemTime) |
| D106 | Dev-only test endpoints (`/dev/clients/{id}/dag`, `/dev/tenants/cleanup`) — not exposed in production |

---

## Implementation Order

Four milestones:

**M1: Keycloak SPI** (`keycloak-bodhi-ext` repo) — **COMPLETED**:
- JPA entities + Liquibase tables (`bodhi_clients`, `bodhi_clients_users`)
- `GET/POST /tenants` endpoints with dashboard token auth
- Dual-write on existing endpoints (assign-role, remove-user, make-resource-admin, POST /resources)
- Client ID prefix renames (`bodhi-resource-`, `bodhi-tenant-`, `bodhi-app-`)
- Deployed to `main-id.getbodhi.app` dev env

**M2: BodhiApp Backend** — **COMPLETED** (commits `6a7d879..04788eb`):
- Phase 1: Foundation — schema, types, SPI proxy methods on AuthService
- Phase 2: Session key namespacing (`{client_id}:access_token`, `active_client_id`)
- Phase 3: Dashboard auth routes (`/auth/dashboard/initiate`, `/auth/dashboard/callback`)
- Phase 4: Tenant management routes (list, create, activate)
- Phase 5: Enhanced `/info` (returns `deployment` + `client_id`) and `/user/info` (returns `has_dashboard_session`)
- Phase 6: TECHDEBT + documentation updates
- **Test counts**: services=832, routes_app=649, server_app=8 — all passing, 0 warnings

**M3: Frontend + Backend Prerequisites** — **COMPLETE** (staged, pending commit):
- D68 implemented: `client_id` always required in `POST /auth/initiate` body via `AuthInitiateRequest`
- D77 implemented: `dashboard_callback_url()` on `SettingService`, dashboard auth uses correct callback URL
- `/info` session-aware: `resolve_multi_tenant_status()` checks dashboard token + SPI for multi-tenant status
- `auth_callback` reads `auth_client_id` from session (no more `get_standalone_app()`)
- Next.js UI: dashboard callback page, login page refactor, tenant registration page
- New hooks: dashboard auth, tenant management, modified OAuth initiate
- AppInitializer: array `allowedStatus`, `tenant_selection` routing, deployment-aware setup routing
- OpenAPI spec + TypeScript client regenerated

**M4: Integration Tests** — **COMPLETED** (staged, pending commit):
- Phase 1: Infrastructure — `DbService` production guard + `reset_tenants()`, `AuthService.forward_spi_request()`, `ensure_valid_dashboard_token` TimeService fix
- Phase 2: Dev endpoints — `POST /dev/clients/{client_id}/dag`, `DELETE /dev/tenants/cleanup` (dev-only)
- Phase 3: routes_app integration tests — 7 oneshot tests with real Keycloak tokens
- Phase 4: server_app integration tests — 3 live TCP tests (compile-verified)
- **Test counts**: services=832, routes_app=656 (647 unit + 2 live auth + 7 live multi-tenant), server_app=8 unit + 3 compile-verified

**Still deferred**:
- D80: Shared parameterized code exchange utility — code duplicated between `routes_auth.rs` and `routes_dashboard_auth.rs` (low priority)
- Multi-tenant-aware logout — `session.delete()` clears all tokens, not selective per tenant
- E2E/Playwright tests for multi-tenant flows
- Service construction: conditional route registration for deployment modes
- Navigation item visibility: hide LLM-specific items in multi-tenant
- CI pipeline for integration tests (needs real Keycloak + `.env.test` credentials)

See individual kickoff documents for detailed implementation plans per milestone.
