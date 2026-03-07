# Multi-Tenant E2E Tests — Coverage (Phase 3)

> **Created**: 2026-03-09
> **Status**: TODO
> **Scope**: New Playwright E2E tests for multi-tenant-specific flows
> **Depends on**: `kickoff-e2e-multi-tenant.md` (Phase 1+2 — **COMPLETED**)
> **Context doc**: `multi-tenant-flow-ctx.md`

---

## Context

Phase 1+2 (completed) configured the multi-tenant server and verified existing shared tests pass. This phase adds **new** E2E tests for multi-tenant-specific flows that don't exist in standalone mode.

### What's already working

The multi-tenant shared server on port 41135 is configured with:
- `BODHI_DEPLOYMENT=multi_tenant` (env var)
- Dashboard client credentials (`BODHI_MULTITENANT_CLIENT_ID`, `BODHI_MULTITENANT_CLIENT_SECRET`)
- Pre-seeded tenant for `user@email.com` via `ensure_tenant()`
- Login flow working: dashboard OAuth → auto-login (single tenant) → resource OAuth → `/ui/chat/`
- 30 shared tests passing on multi_tenant project

### What's NOT tested yet

- Dashboard login flow (explicit button click → Keycloak → callback)
- Tenant registration flow (0 tenants → setup wizard → register → auto-login)
- Tenant switching flow (N tenants → select → switch)
- Multi-tenant info endpoint behavior at each state

---

## Task

Add Playwright E2E tests in `specs/multi-tenant-setup/` that exercise multi-tenant-specific UI flows using `manager@email.com` (separate from `user@email.com` used by shared tests).

---

## Test User Separation

- **`user@email.com`**: Used by shared feature tests. Has a pre-seeded tenant on the shared server. Do NOT modify this user's tenants.
- **`manager@email.com`**: Used by setup flow tests below. Tenants are cleaned up before each test so the setup flow can be exercised from scratch.

---

## Cleanup Mechanism

E2E tests are **black-box** — all interactions go through the browser UI. The cleanup mechanism:

1. **Login as `manager@email.com`**: Navigate to `/ui/login`, click dashboard login button, authenticate via Keycloak OAuth flow. After login, the browser has a dashboard session cookie.

2. **Navigate to cleanup URL**: Go to `GET /dev/tenants/cleanup` via `page.goto()`. This endpoint is registered for both GET and DELETE — GET for browser-based E2E tests. The dashboard session cookie authenticates the request. Returns JSON of deleted tenants.

3. **Navigate to login page**: Go to `/ui/login`. Since manager has no tenants, the app redirects to `/ui/setup/tenants/` — the tenant registration form.

4. **Test proceeds**: Exercise tenant creation, auto-login, etc. from a clean state.

---

## Critical Test Scenarios

### 1. Dashboard login flow

```
State: No session (fresh browser context)
1. Navigate to /ui/login → see "Login to Bodhi Platform" button
2. Click login → redirect to Keycloak → authenticate as manager@email.com
3. Dashboard callback processes code → redirects based on tenant state:
   - 0 tenants: → /ui/setup/tenants/ (registration form)
   - 1 tenant: → auto-login via resource OAuth → /ui/chat/
   - N tenants: → /ui/login (tenant selector)
```

### 2. Tenant registration flow

```
State: Dashboard session, 0 tenants (after cleanup)
1. On /ui/setup/tenants/ — see registration form
2. Fill in name (required), description (optional) → submit
3. Auto-initiates resource OAuth → Keycloak SSO → redirect back
4. Now authenticated with active tenant → redirected to /ui/chat/
5. Verify /info shows: deployment: "multi_tenant", client_id: <new_tenant_id>
```

### 3. Tenant switching (requires 2+ tenants)

```
State: Dashboard session, N tenants, currently on tenant A
1. Navigate to /ui/login → see tenant selector
2. Select tenant B (logged_in) → instant switch via activate
3. Verify /info shows: client_id: <tenant_B_id>
4. Select tenant C (not logged_in) → OAuth flow → Keycloak SSO → redirect
5. Verify /info shows: client_id: <tenant_C_id>
```

### 4. Info endpoint behavior at each state

```
1. No session → { auth_status: "logged_out", deployment: "multi_tenant" }
2. Dashboard session only → { auth_status: "logged_out", has_dashboard_session: true }
3. Dashboard + active tenant → { auth_status: "logged_in", deployment: "multi_tenant", client_id: "..." }
```

---

## Page Objects Needed

Explore existing page objects and extend or create new ones:

### Existing (may need extension)
- `LoginPage.mjs` — needs methods for:
  - `performDashboardLogin()` — click "Login to Bodhi Platform", fill Keycloak creds, wait for callback
  - `performTenantSelection(tenantName)` — select a specific tenant from the list
  - `waitForAutoLogin()` — wait for single-tenant auto-login redirect chain

### New page objects
- `TenantRegistrationPage.mjs` — for `/ui/setup/tenants/`:
  - `fillTenantName(name)`, `fillDescription(desc)`, `submitRegistration()`
  - `waitForRegistrationComplete()` — wait for OAuth redirect chain to complete
- `TenantSelectorPage.mjs` — for the multi-tenant login page tenant list:
  - `selectTenant(name)`, `waitForTenantSwitch()`
  - `getTenantList()` — returns visible tenant names

### Frontend references (for page object design)
- `crates/bodhi/src/app/ui/login/page.tsx` — `MultiTenantLoginContent` component
- `crates/bodhi/src/app/ui/setup/tenants/page.tsx` — tenant registration form
- `crates/bodhi/src/app/ui/auth/dashboard/callback/page.tsx` — dashboard OAuth callback
- `crates/bodhi/src/components/AppInitializer.tsx` — deployment-aware routing

---

## Test Structure

```
tests-js/specs/multi-tenant-setup/
├── tenant-registration.spec.mjs     # Scenario 2: register tenant from scratch
├── dashboard-login.spec.mjs         # Scenario 1: dashboard login states
└── tenant-switching.spec.mjs        # Scenario 3: switch between tenants
```

Add to `playwright.config.mjs`:
- `standalone` project: `testIgnore` should include `'**/multi-tenant-setup/**'`
- `multi_tenant` project: these specs run normally

---

## Build & Run

```bash
# Build UI first
make build.ui-rebuild

# Run only multi-tenant coverage tests
cd crates/lib_bodhiserver_napi && npx playwright test --project multi_tenant specs/multi-tenant-setup/

# Run one at a time during development
cd crates/lib_bodhiserver_napi && npx playwright test --project multi_tenant specs/multi-tenant-setup/tenant-registration.spec.mjs
```

---

## Files to Explore

### E2E infrastructure (existing — read-only reference)
- `playwright.config.mjs` — project definitions, web servers
- `tests-js/scripts/start-shared-server.mjs` — multi-tenant server startup
- `tests-js/fixtures.mjs` — test fixtures (autoResetDb, sharedServerUrl)
- `tests-js/utils/auth-server-client.mjs` — `getMultiTenantConfig()`, Keycloak helpers

### Page objects (extend)
- `tests-js/pages/LoginPage.mjs` — login page interactions
- `tests-js/pages/BasePage.mjs` — common methods, OAuth helpers

### Frontend (the UI being tested)
- `crates/bodhi/src/app/ui/login/page.tsx` — `MultiTenantLoginContent` component
- `crates/bodhi/src/app/ui/setup/tenants/page.tsx` — tenant registration form
- `crates/bodhi/src/app/ui/auth/dashboard/callback/page.tsx` — dashboard OAuth callback
- `crates/bodhi/src/components/AppInitializer.tsx` — deployment-aware routing

### Existing test patterns (reference)
- `tests-js/specs/setup/setup-flow.spec.mjs` — setup wizard E2E (standalone reference)
- `tests-js/specs/auth/token-refresh-integration.spec.mjs` — complex auth flow reference
- `tests-js/specs/oauth/oauth2-token-exchange.spec.mjs` — OAuth flow reference

### Rust integration tests (reference for flow patterns)
- `crates/server_app/tests/test_live_multi_tenant.rs` — full flow, state progression
- `crates/server_app/tests/utils/live_server_utils.rs` — session helpers
