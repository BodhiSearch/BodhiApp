# Multi-Tenant Stage 2 — Summary

> **Created**: 2026-03-06 | **Completed**: 2026-03-10
> **Commits**: `239a3395d` through `8aa9acae2` (HEAD)
> **Scope**: Backend + Frontend + Integration tests for multi-tenant SaaS deployment
> **Companion docs**:
> - `multi-tenant-flow-ctx.md` — Functional spec: how multi-tenant works, co-existence with standalone
> - `auth-context-deployment-coexistence.md` — AuthContext domain model, deployment mode coexistence, middleware architecture
> - `two-phase-login-session-architecture.md` — Two-phase login flow, session namespacing, SSO reuse, frontend state machine
> - `tenant-lifecycle-spi-database.md` — Tenant registration, SPI proxy pattern, trust model, RLS, database evolution
> - `decisions.md` (D21-D106) — All architectural decisions
> - `TECHDEBT.md` — Deferred items and open findings
> - `decision-organization-feature-deferred.md` — Keycloak Organizations deferral research
> - `kickoff-e2e-multi-tenant-coverage.md` — E2E test implementation plan (TODO)

---

## Scope

Stage 2 adds multi-tenant SaaS support to BodhiApp. A single BodhiApp instance serves multiple tenants, each with its own Keycloak resource-client. Key additions:

- **Two-phase login**: Dashboard OAuth (platform) + Resource-client OAuth (tenant)
- **Tenant management**: Register, list, switch, activate tenants
- **Session namespacing**: All session keys prefixed by `{client_id}:`
- **RLS enforcement**: All mutating DB ops use `begin_tenant_txn(tenant_id)`
- **`MultiTenantSession` AuthContext variant**: Dashboard-only sessions without active tenant
- **Frontend**: Login page refactor, tenant registration, dashboard callback

**Milestones**: M1 SPI (external), M2 Backend, M3 Frontend, M4 Integration Tests, M5 Pre-E2E fixes, M6 Tenant membership + review fixes — all complete.

---

## Architecture Overview

### Deployment Modes

| Mode | `BODHI_DEPLOYMENT` | Description |
|------|-------------------|-------------|
| **Standalone** | `standalone` (default) | Single tenant. Desktop/Docker. One resource-client. |
| **Multi-Tenant** | `multi_tenant` | SaaS. Dashboard client + N resource-clients per user. |

Standalone is effectively multi-tenant with exactly one tenant and no dashboard phase. Unified middleware code path — no deployment-mode branching.

### Two-Phase Login Flow

```
Phase 1: Dashboard Authentication
1. User visits cloud.getbodhi.app
2. GET /info -> status: "tenant_selection"
3. Frontend shows "Login to Bodhi Platform" button
4. POST /auth/dashboard/initiate -> Keycloak OAuth URL (BODHI_MULTITENANT_CLIENT_ID)
5. User authenticates -> redirected back
6. POST /auth/dashboard/callback -> stores dashboard:access_token, dashboard:refresh_token
7. Redirects to /ui/login

Phase 2: Tenant Selection + Resource-Client OAuth
8. /ui/login checks /user/info (has_dashboard_session)
9. GET /tenants -> list of user's resource-clients (enriched with is_active, logged_in)
   0 clients -> /ui/setup/tenants/ (registration form)
   1 client  -> auto-redirect (SSO, no re-authentication)
   N clients -> tenant selector dropdown
10. POST /auth/initiate { client_id } -> Resource-client OAuth
11. Keycloak SSO -> instant redirect (no password entry)
12. POST /auth/callback -> stores {client_id}:access_token, sets active_client_id
13. GET /info -> status: "ready"
```

**Key insight**: Keycloak SSO session from Phase 1 is reused in Phase 2, so the resource-client OAuth is instant.

### Session Key Schema

```
Multi-tenant:
  dashboard:access_token        -- Dashboard JWT
  dashboard:refresh_token
  active_client_id              -- Currently selected resource-client
  {client_id_A}:access_token    -- Resource-client A JWT
  {client_id_A}:refresh_token
  {client_id_B}:access_token    -- Resource-client B JWT (if switched)
  {client_id_B}:refresh_token
  user_id

Standalone (same pattern, just one client):
  active_client_id
  {client_id}:access_token
  {client_id}:refresh_token
  user_id
```

### AuthContext Variants

```rust
AuthContext::MultiTenantSession {
    client_id: Option<String>,     // Some if resource-authenticated
    tenant_id: Option<String>,     // Some if resource-authenticated
    user_id: String,               // From dashboard token
    username: String,
    role: Option<ResourceRole>,    // From resource JWT, if present
    token: Option<String>,         // Resource JWT, if present
    dashboard_token: String,       // Dashboard JWT
}
```

Created by `optional_auth_middleware` when dashboard session exists. Enables dev endpoints (cleanup, DAG) and tenant management routes to access the dashboard token.

### Middleware Token Lookup

```
optional_auth_middleware / auth_middleware:
1. Check deployment mode -> DeploymentMode::MultiTenant?
2. Read session["dashboard:access_token"] -> if present, build MultiTenantSession base
3. Read session["active_client_id"]
4. Read session["{active_client_id}:access_token"]
5. Decode JWT -> resolve tenant via get_tenant_by_client_id(azp)
6. Validate/refresh token
7. Populate full AuthContext with tenant + role + dashboard token
```

---

## Key Endpoints

### New (multi-tenant only)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/auth/dashboard/initiate` | POST | Start dashboard OAuth2 |
| `/auth/dashboard/callback` | POST | Complete dashboard OAuth2, redirect to /ui/login |
| `/tenants` | GET | List user's tenants (SPI proxy, enriched with is_active, logged_in) |
| `/tenants` | POST | Register new tenant (SPI proxy + local tenant row) |
| `/tenants/{client_id}/activate` | POST | Instant tenant switch (if cached token valid) |

All return "not supported" error in standalone mode.

### Enhanced existing

| Endpoint | Changes |
|----------|---------|
| `POST /auth/initiate` | `client_id` always required in body (D68). Stores `auth_client_id` in session. |
| `POST /auth/callback` | Reads `auth_client_id` from session. Namespaced session keys. |
| `GET /info` | Returns `deployment: "standalone"\|"multi_tenant"`, `client_id`. Session-aware for MT status. |
| `GET /user/info` | Returns `has_dashboard_session: bool` (skip_serializing_if false). |

### Dev-only (non-production)

| Endpoint | Purpose |
|----------|---------|
| `POST /dev/clients/{client_id}/dag` | Enable Direct Access Grants on KC client |
| `GET\|DELETE /dev/tenants/cleanup` | Clean up KC tenants + truncate local tenants table |

---

## Implementation Record

| Milestone | Scope | Status | Test Counts |
|-----------|-------|--------|-------------|
| M1: Keycloak SPI | JPA entities, tenant endpoints, dual-write | Complete (external repo) | — |
| M2: Backend | Schema, session namespacing, dashboard auth, tenant CRUD, /info + /user/info | Complete | services=832, routes_app=649, server_app=8 |
| M3: Frontend | Login refactor, tenant registration, dashboard callback, hooks, AppInitializer | Complete | — |
| M4: Integration Tests | Dev endpoints, routes_app oneshot (7), server_app live TCP (3) | Complete | routes_app=656, server_app=11 |
| M5: Pre-E2E Fixes | Playwright config, multi-tenant server, testIgnore | Complete | 30 shared tests passing on multi_tenant |
| M6: Review + Membership | tenants_users table, auth middleware, stage 2 review findings | Complete | — |

---

## Frontend Summary

### New Pages
- `/ui/auth/dashboard/callback/page.tsx` — Dashboard OAuth callback
- `/ui/setup/tenants/page.tsx` — Tenant registration form (name, description)

### Modified Pages
- `/ui/login/page.tsx` — Conditional: `MultiTenantLoginContent` (4 states) vs `LoginContent` (standalone)
  - State A: No dashboard session → "Login to Bodhi Platform"
  - State B1: Single tenant, auto-login → seamless redirect
  - State B2: Multiple tenants → selector dropdown
  - State C: Fully authenticated → welcome + switch + logout

### New Hooks
- `useDashboardOAuthInitiate()`, `useDashboardOAuthCallback()` — Dashboard auth
- `useTenants()`, `useCreateTenant()`, `useTenantActivate()` — Tenant management
- `useOAuthInitiate()` updated to accept `{ client_id: string }`

### AppInitializer Changes
- `allowedStatus` accepts `AppStatus | AppStatus[]`
- Routes `tenant_selection` → `/ui/login`, `setup` in multi-tenant → `/ui/setup/tenants`

### TypeScript Types (regenerated)
`AppInfo.deployment`, `AppInfo.client_id`, `AuthInitiateRequest.client_id`, `TenantListResponse`, `TenantListItem`, `CreateTenantRequest`, `CreateTenantResponse`, `UserInfoEnvelope.has_dashboard_session`

---

## Architecture Refactor (Stage 2)

### `MultiTenantSession` AuthContext Variant
Added to `services::auth::AuthContext`. Created by middleware when dashboard session exists. Contains `dashboard_token: String` for SPI proxy calls. Optional `client_id`/`tenant_id`/`token`/`role` fields — populated when resource-authenticated.

### `tenants_users` Table (Keycloak SPI)
`bodhi_clients_users` table tracks membership only (no role column). Keycloak groups are sole role source of truth. Enables fast `GET /tenants` queries without group enumeration.

### `DeploymentMode` Enum
`services::DeploymentMode` enum (`Standalone`, `MultiTenant`). Added to `AuthContext::Anonymous { deployment }`. Read from `BODHI_DEPLOYMENT` setting via `SettingService`.

### Session Key Breaking Change (D56)
Flat keys → namespaced keys. Standalone users re-login once. No legacy compatibility layer.

---

## Testing Infrastructure

### Integration Test Structure
- **routes_app oneshot** (`test_live_multi_tenant.rs`): 7 tests with real Keycloak tokens, `tower::oneshot()`
- **server_app live TCP** (`test_live_multi_tenant.rs`): 3 tests on port 51135, serial execution
- **Dev endpoints**: `POST /dev/clients/{id}/dag`, `GET|DELETE /dev/tenants/cleanup`

### E2E Infrastructure
- Dual Playwright projects: `standalone` (SQLite, port 51135) + `multi_tenant` (PostgreSQL, port 41135)
- Multi-tenant server: `start-shared-server.mjs --deployment multi_tenant --db-type postgres --port 41135`
- Pre-seeded tenant for `user@email.com` via `ensure_tenant()` in server startup
- 30 shared tests passing on multi_tenant project (setup, chat/GGUF, models, tokens excluded)

### Test Clients
- Dashboard: `INTEG_TEST_MT_DASHBOARD_CLIENT_ID` / `SECRET` (env vars)
- Tenant: `INTEG_TEST_MT_TENANT_ID` / `SECRET` (pre-seeded resource-client)
- Test users: `user@email.com` (resource admin), `manager@email.com` (resource manager)

---

## Open Findings

See `TECHDEBT.md` for full list. Key items:
- `get_standalone_app()` production usages (4 files) — will error with >1 tenant
- Navigation item visibility — hide LLM features in multi-tenant mode
- Service construction — conditional route registration per deployment mode
- Frontend unit tests — no tests for MT-specific components
- E2E tests — multi-tenant-specific scenarios not yet covered (see `kickoff-e2e-multi-tenant-coverage.md`)
- Integration test CI — needs Keycloak access + `.env.test` credentials
