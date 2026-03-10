# Multi-Tenant E2E Tests — Comprehensive Kickoff

> **Created**: 2026-03-10
> **Status**: TODO
> **Scope**: New Playwright E2E tests for multi-tenant flows + existing test adaptations
> **Context**: `SUMMARY.md` (architecture), `decisions.md` (D21-D106)

---

## Context

Multi-tenant stage 2 is implemented at HEAD. The multi-tenant shared server runs on port 41135 (PostgreSQL) with 30 shared tests passing. This kickoff covers all remaining E2E test work, ordered by lowest-hanging fruit first:
1. **testIgnore fixes** — config-only, immediate wins
2. **Existing test adaptations** — unlock more tests on multi_tenant project
3. **New multi-tenant test scenarios** — page objects and fixtures created inline as needed

### Current E2E State

- **Standalone project**: Port 51135, SQLite, all tests passing
- **Multi-tenant project**: Port 41135, PostgreSQL, 30 shared tests passing
- **testIgnore (multi_tenant)**: `setup/**`, `chat/chat.spec.mjs`, `chat/chat-agentic.spec.mjs`, `models/**`, `tokens/**`, `oauth/oauth-chat-streaming.spec.mjs`
- **Pre-seeded tenant**: `user@email.com` with `INTEG_TEST_MT_TENANT_ID` via `ensure_tenant()` in server startup

### Test User Separation

| User | Purpose | Env Vars |
|------|---------|----------|
| `user@email.com` | Shared feature tests (pre-seeded tenant) | `INTEG_TEST_USERNAME`, `_ID`, `_PASSWORD` |
| `manager@email.com` | Multi-tenant setup tests (tenants cleaned between tests) | `INTEG_TEST_USER_MANAGER`, `_ID`, `_PASSWORD` |

---

## testIgnore Updates (Phase 1-2)

Target state after Phases 1-2:

```javascript
// multi_tenant project:
testIgnore: [
  '**/setup/**',                              // Setup wizard is standalone-only
  '**/chat/chat.spec.mjs',                   // Local GGUF model only
  '**/chat/chat-agentic.spec.mjs',           // Local GGUF model only
  '**/models/**',                             // Local model operations
  '**/oauth/oauth-chat-streaming.spec.mjs',  // Covered separately
  '**/request-access/**',                     // Uses createServerManager() — standalone-specific (Phase 1)
  // REMOVED: '**/tokens/**' — adapted to use API models (Phase 2)
],

// standalone project (add):
testIgnore: [
  '**/multi-tenant-setup/**',                // MT-specific scenarios (Phase 1)
],
```

---

## Existing Test Adaptations (Phases 2-3)

### Adapt api-tokens.spec.mjs for multi_tenant (Phase 2)

**File**: `tests-js/specs/tokens/api-tokens.spec.mjs`

Current state: Excluded from multi_tenant via `**/tokens/**` in testIgnore. Uses `selectModelQwen()` for chat integration tests (GGUF-dependent).

Adaptation:
1. In `beforeAll`: Register an API model via UI (same pattern as `api-models.spec.mjs`)
2. Replace 4x `chatSettings.selectModelQwen()` with `chatSettings.selectModel(registeredModelName)`
3. Add cleanup in final test step (delete registered API model)
4. Remove `**/tokens/**` from multi_tenant testIgnore

Verify: `cd crates/lib_bodhiserver_napi && npx playwright test --project multi_tenant tests-js/specs/tokens/`

### Enhance api-models chat testing (Phase 3)

**File**: `tests-js/specs/api-models/api-models.spec.mjs`

Current state: Has basic chat (send Q&A, verify response). Runs on both projects.

Enhancement:
- Add multi-chat management tests (create/switch/delete conversations)
- Add edge cases: empty message handling, long messages, network error recovery
- Same depth as `chat.spec.mjs:67-115` but using API models instead of local GGUF
- This covers "same chat test for API models" requirement
- `chat.spec.mjs` stays standalone-only (uses `selectModelQwen`)

### Access-request tests — defer (Phase 7)

**File**: `tests-js/specs/request-access/multi-user-request-approval-flow.spec.mjs`

Uses `createServerManager()` (standalone-specific). Added to multi_tenant testIgnore in Phase 1. Full rework to use shared MT server is a separate task (Phase 7).

### oauth-chat-streaming.spec.mjs — keep excluded

Already excluded. Both OAuth token exchange and API model chat are tested separately. Low incremental value for combined test.

---

## New Multi-Tenant Test Scenarios (Phases 4-8)

All tests use `manager@email.com` with cleanup via browser-based `GET /dev/tenants/cleanup`.

### Cleanup Mechanism

The cleanup endpoint requires a `MultiTenantSession` AuthContext (populated by `optional_auth_middleware` from dashboard session cookie). Flow:

1. Dashboard login as `manager@email.com` (browser OAuth flow → session cookie)
2. Navigate to `GET /dev/tenants/cleanup` via `page.goto()` — session cookie authenticates
3. Handler calls SPI `DELETE /test/tenants/cleanup` with dashboard token + truncates local tenants table
4. Returns JSON of deleted tenants
5. Navigate to `/ui/login` — 0 tenants → redirect to `/ui/setup/tenants/`

**Registration endpoint**: `routes_app/src/routes.rs:122-129` registers both GET and DELETE methods, in `optional_auth` router (line 130).

### Scenario 1 (HIGH): Dashboard Login + Tenant Registration — Phase 4

```
Fresh browser context (no session)
1. Navigate to /ui/login
2. Verify State A: "Login to Bodhi Platform" button visible
   - AuthCard title: "Login"
   - Button text: "Login to Bodhi Platform"
3. Click login → redirect to Keycloak
4. Fill credentials (manager@email.com) → submit
5. Dashboard callback processes → redirects to /ui/login
6. 0 tenants → redirect to /ui/setup/tenants/
7. Fill tenant name (data-testid="tenant-name-input")
8. Optionally fill description (data-testid="tenant-description-input")
9. Click "Create Workspace" (data-testid="create-tenant-button")
10. Auto-initiates resource OAuth → Keycloak SSO (instant) → /ui/chat/
11. Verify: GET /bodhi/v1/info returns deployment: "multi_tenant", client_id: <new>
```

### Scenario 2 (HIGH): Tenant Switching — Phase 5

```
Prerequisite: 2 tenants registered for manager@email.com
1. Login, land on /ui/chat/ with Tenant A active
2. Navigate to /ui/login
3. Verify State C: "Welcome" card with active workspace name
4. Verify "Switch to <Tenant B>" button visible
5. Click switch → OAuth or activate → /ui/chat/
6. Verify /info shows client_id changed to Tenant B
7. Navigate to /ui/login → verify Tenant B is now active
```

### Scenario 3 (MEDIUM): Auto-Login with Single Tenant — Phase 5

```
Prerequisite: 1 tenant registered for manager@email.com
1. Logout (or fresh context with dashboard session)
2. Dashboard login → /ui/login
3. Verify auto-redirect (useRef guard): single tenant → resource OAuth → /ui/chat/
4. No tenant selector shown
```

### Scenario 4 (MEDIUM): Cross-Tenant Data Isolation — Phase 6

```
Prerequisite: 2 tenants
1. Login to Tenant A
2. Create API model (via UI) + API token
3. Switch to Tenant B
4. Verify: API model NOT visible, API token NOT visible
5. Switch back to Tenant A
6. Verify: API model and token ARE visible
```

### Scenario 5 (LOW): Multi-Tenant Logout — Phase 8

```
1. Login (fully authenticated with tenant)
2. Click "Log Out" on /ui/login State C
3. Verify: returns to State A ("Login to Bodhi Platform" button, NOT tenant selector)
```

### Scenario 6 (LOW): Resource Admin Flow — Phase 8

```
1. Create tenant → login
2. Verify admin role (users page accessible, can manage settings)
```

---

## Page Objects and Fixtures Reference

Created inline during Phases 4-5. Reference for all page object methods and selectors.

### LoginPage.mjs extensions (Phase 4-5)

**File**: `tests-js/pages/LoginPage.mjs`

```javascript
// Phase 4: Dashboard login + logout
async performDashboardLogin(username, password) {
  // Click "Login to Bodhi Platform" → Keycloak → fill creds → callback → /ui/login
}
async clickLogout() {
  // Click "Log Out" button on State C
}

// Phase 5: Tenant selection
async waitForTenantSelector() {
  // Wait for AuthCard with title "Select Workspace"
}
async selectTenant(tenantName) {
  // Click button with tenant name text
}
async waitForAutoLogin() {
  // Wait for redirect chain: /ui/login → Keycloak SSO → /auth/callback → /ui/chat/
}
```

**Frontend selectors** (from `crates/bodhi/src/app/ui/login/page.tsx`):
- Login page container: `data-testid="login-page"`
- State A: AuthCard title "Login", button text "Login to Bodhi Platform"
- State B1: AuthCard title "Connect to Workspace"
- State B2: AuthCard title "Select Workspace", buttons with tenant names
- State C: AuthCard title "Welcome", "Go to Home", "Switch to <name>", "Log Out"

### TenantRegistrationPage.mjs (Phase 4)

**File**: `tests-js/pages/TenantRegistrationPage.mjs`

**Frontend selectors** (from `crates/bodhi/src/app/ui/setup/tenants/page.tsx`):
- Name input: `data-testid="tenant-name-input"`
- Description textarea: `data-testid="tenant-description-input"`
- Submit button: `data-testid="create-tenant-button"` (text: "Create Workspace" / "Creating...")

```javascript
export class TenantRegistrationPage extends BasePage {
  async fillTenantName(name) { await this.page.getByTestId('tenant-name-input').fill(name); }
  async fillDescription(desc) { await this.page.getByTestId('tenant-description-input').fill(desc); }
  async submitRegistration() { await this.page.getByTestId('create-tenant-button').click(); }
  async waitForRegistrationComplete() {
    await this.page.waitForURL(url => url.origin === this.baseUrl && url.pathname === '/ui/chat/');
  }
}
```

### multiTenantFixtures.mjs (Phase 4)

**File**: `tests-js/fixtures/multiTenantFixtures.mjs`

```javascript
export class MultiTenantFixtures {
  static getManagerCredentials() {
    return {
      username: process.env.INTEG_TEST_USER_MANAGER,
      userId: process.env.INTEG_TEST_USER_MANAGER_ID,
      password: process.env.INTEG_TEST_PASSWORD,
    };
  }
  static createTenantData(index = 1) {
    return { name: `Test Workspace ${index}`, description: `Test workspace ${index} for E2E` };
  }
}
```

---

## Test Infrastructure

### Environment Requirements

- Real Keycloak at `main-id.getbodhi.app` (dev env)
- PostgreSQL containers: `docker compose -f docker/docker-compose.test.yml up -d`
- Real API keys (`INTEG_TEST_OPENAI_API_KEY`) for API model tests
- `.env.test` with all `INTEG_TEST_*` vars

### Key Env Vars for Multi-Tenant Tests

```
INTEG_TEST_MT_DASHBOARD_CLIENT_ID    # Dashboard client ID
INTEG_TEST_MT_DASHBOARD_CLIENT_SECRET # Dashboard client secret
INTEG_TEST_MT_TENANT_ID              # Pre-seeded tenant client ID
INTEG_TEST_MT_TENANT_SECRET          # Pre-seeded tenant client secret
INTEG_TEST_USER_MANAGER              # manager@email.com
INTEG_TEST_USER_MANAGER_ID           # Keycloak user ID for manager
INTEG_TEST_PASSWORD                  # Shared password
```

### Server Configuration

Multi-tenant server started by `start-shared-server.mjs` with:
- `--deployment multi_tenant --db-type postgres --port 41135`
- Sets `BODHI_DEPLOYMENT=multi_tenant`, `BODHI_MULTITENANT_CLIENT_ID`, `BODHI_MULTITENANT_CLIENT_SECRET`
- Pre-seeds tenant for `user@email.com` (shared tests)
- `manager@email.com` tenants: created/cleaned by E2E tests themselves

---

## Implementation Phases

Ordered by lowest-hanging fruit first. Page objects and fixtures are created inline in the phase that first needs them.

### Phase 1 — testIgnore Config Fixes (config-only, zero test changes)

1. Update `playwright.config.mjs` multi_tenant testIgnore:
   - Add `'**/request-access/**'` (uses `createServerManager()` — standalone-specific)
   - Add `'**/multi-tenant-setup/**'` to standalone testIgnore (prep for future MT specs)
2. Verify: `npx playwright test --project multi_tenant` (30 shared tests still pass, request-access no longer attempted)
3. Verify: `npx playwright test --project standalone` (all standalone tests still pass)

### Phase 2 — Adapt api-tokens for multi_tenant (unlocks tokens/* on MT)

1. Modify `api-tokens.spec.mjs`:
   - In `beforeAll`: Register an API model via UI (same pattern as `api-models.spec.mjs`)
   - Replace 4x `chatSettings.selectModelQwen()` with `chatSettings.selectModel(registeredModelName)`
   - Add cleanup in final test step (delete registered API model)
2. Remove `'**/tokens/**'` from multi_tenant testIgnore
3. Verify: `npx playwright test --project multi_tenant tests-js/specs/tokens/`

### Phase 3 — Enhance api-models chat depth (both projects)

1. Add multi-chat management tests to `api-models.spec.mjs` (or new `api-models-chat.spec.mjs`)
2. Port edge cases from `chat.spec.mjs:67-115` pattern: multi-chat management, network error recovery
3. `chat.spec.mjs` stays standalone-only (uses `selectModelQwen`)
4. Verify: `npx playwright test --project multi_tenant tests-js/specs/api-models/`

### Phase 4 — Dashboard Login + Tenant Registration (Scenario 1)

Creates page objects and fixtures inline as this is the first MT-specific test.

1. Create `multiTenantFixtures.mjs` with `getManagerCredentials()`, `createTenantData()`
2. Extend `LoginPage.mjs` with `performDashboardLogin()`, `clickLogout()`
3. Create `TenantRegistrationPage.mjs` with `fillTenantName()`, `fillDescription()`, `submitRegistration()`, `waitForRegistrationComplete()`
4. Write `specs/multi-tenant-setup/tenant-registration.spec.mjs` (Scenario 1)
5. Verify: `npx playwright test --project multi_tenant tests-js/specs/multi-tenant-setup/tenant-registration.spec.mjs`

### Phase 5 — Auto-Login + Tenant Switching (Scenarios 2, 3)

Page objects from Phase 4 already available.

1. Extend `LoginPage.mjs` with `waitForAutoLogin()`, `waitForTenantSelector()`, `selectTenant()`
2. Write `specs/multi-tenant-setup/auto-login.spec.mjs` (Scenario 3 — single tenant auto-redirect)
3. Write `specs/multi-tenant-setup/tenant-switching.spec.mjs` (Scenario 2 — multi-tenant switch)
4. Verify: `npx playwright test --project multi_tenant tests-js/specs/multi-tenant-setup/`

### Phase 6 — Cross-Tenant Data Isolation (Scenario 4)

1. Write `specs/multi-tenant-setup/cross-tenant-isolation.spec.mjs`
   - Login Tenant A → create API model + API token → switch Tenant B → verify not visible → switch back → verify present
2. Verify: `npx playwright test --project multi_tenant tests-js/specs/multi-tenant-setup/cross-tenant-isolation.spec.mjs`

### Phase 7 — Access Request Adaptation (separate task)

1. Rework `multi-user-request-approval-flow.spec.mjs` for shared multi-tenant server
2. Remove `'**/request-access/**'` from multi_tenant testIgnore
3. Verify: `npx playwright test --project multi_tenant tests-js/specs/request-access/`

### Phase 8 — Low Priority

1. `specs/multi-tenant-setup/mt-logout.spec.mjs` (Scenario 5)
2. `specs/multi-tenant-setup/mt-resource-admin.spec.mjs` (Scenario 6)

---

## Test Structure

```
tests-js/specs/multi-tenant-setup/        # NEW directory
├── tenant-registration.spec.mjs          # Scenario 1: register + login
├── auto-login.spec.mjs                   # Scenario 3: single-tenant auto
├── tenant-switching.spec.mjs             # Scenario 2: switch between tenants
├── cross-tenant-isolation.spec.mjs       # Scenario 4: data isolation
├── mt-logout.spec.mjs                    # Scenario 5: logout
└── mt-resource-admin.spec.mjs            # Scenario 6: admin flow

tests-js/pages/
├── LoginPage.mjs                         # EXTENDED with dashboard methods
└── TenantRegistrationPage.mjs            # NEW

tests-js/fixtures/
└── multiTenantFixtures.mjs               # NEW
```

---

## Build & Run

```bash
# Prerequisites
docker compose -f docker/docker-compose.test.yml up -d  # PostgreSQL containers
make build.ui-rebuild                                     # Rebuild embedded UI

# Run all multi-tenant tests
cd crates/lib_bodhiserver_napi && npx playwright test --project multi_tenant

# Run specific phase
cd crates/lib_bodhiserver_napi && npx playwright test --project multi_tenant tests-js/specs/multi-tenant-setup/
cd crates/lib_bodhiserver_napi && npx playwright test --project multi_tenant tests-js/specs/tokens/

# Debug single test
cd crates/lib_bodhiserver_napi && npx playwright test --project multi_tenant --headed tests-js/specs/multi-tenant-setup/tenant-registration.spec.mjs
```

---

## Key Reference Files

### E2E Infrastructure
- `playwright.config.mjs` — project config, testIgnore, webServer
- `tests-js/fixtures.mjs` — autoResetDb fixture, sharedServerUrl
- `tests-js/scripts/start-shared-server.mjs` — multi-tenant server startup
- `tests-js/utils/auth-server-client.mjs` — `getMultiTenantConfig()`, Keycloak helpers
- `tests-js/utils/db-config.mjs` — port mapping (standalone=51135, multi_tenant=41135)

### Frontend (UI being tested)
- `crates/bodhi/src/app/ui/login/page.tsx` — `MultiTenantLoginContent` (4 states)
- `crates/bodhi/src/app/ui/setup/tenants/page.tsx` — registration form (3 data-testid selectors)
- `crates/bodhi/src/app/ui/auth/dashboard/callback/page.tsx` — dashboard OAuth callback
- `crates/bodhi/src/components/AppInitializer.tsx` — deployment-aware routing

### Backend (cleanup + dev endpoints)
- `crates/routes_app/src/routes_dev.rs` — `dev_tenants_cleanup_handler`, `dev_clients_dag_handler`
- `crates/routes_app/src/routes.rs:115-131` — dev route registration (optional_auth, non-production)

### Existing Test Patterns
- `tests-js/specs/api-models/api-models.spec.mjs` — API model chat pattern
- `tests-js/specs/chat/chat.spec.mjs` — chat depth reference (standalone-only)
- `tests-js/specs/setup/setup-flow.spec.mjs` — setup wizard E2E reference
