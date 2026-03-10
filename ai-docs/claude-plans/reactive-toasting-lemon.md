# Plan: Simplify multi-tenant-2 folder + E2E kickoff prompt

## Context

Multi-tenant stage 2 is implemented at HEAD commit (111 files, +4492/-1003). The `ai-docs/claude-plans/20260306-multi-tenant-2/` folder has ~28 intermediate files accumulated during implementation. One task remains: multi-tenant E2E tests. This plan covers:
1. Trimming the folder to final reference docs (code is source of truth)
2. Creating a comprehensive E2E kickoff prompt for the remaining task

---

## Part 1: Folder Simplification

### Execution Strategy

Launch 3 parallel sub-agents, each given a batch of HEAD commit files to analyze against the plan docs:

**Agent 1 — services crate** (~30 files): auth_context, tenant_repository/service, access_request, db migrations, RLS, test_utils, settings, toolsets, models
- Cross-reference with: `20260306-services-multi-tenant-isolation-test.md`, `20260309-create-tenant-membership.md`, `20260309-mt-repo-test-audit.md`
- Output: summary of implemented changes, TECHDEBT items to update

**Agent 2 — routes_app crate** (~25 files): auth routes, middleware, setup, tenants, tokens, users, dev routes, shared
- Cross-reference with: `20260306-middleware-multi-tenant.md`, `20260307-backend-impl.md`, `20260309-mt-arch-refactor.md`, `20260309-mt-review.md`
- Output: summary of implemented changes, decisions to verify

**Agent 3 — UI + E2E + NAPI** (~20 files): login page, LoginMenu, hooks, playwright config, fixtures, test specs, start-shared-server
- Cross-reference with: `20260308-frontend-impl.md`, `20260308-pre-e2e-fixes.md`, `20260307-e2e-test-failure.md`
- Output: summary of implemented changes, current E2E test state

### Files to DELETE (21 files)

All completed kickoff and implementation files:

| File | Reason |
|------|--------|
| `kickoff.md` | Completed middleware kickoff |
| `kickoff-bodhi-backend.md` | Completed backend kickoff |
| `kickoff-bodhi-frontend.md` | Completed frontend kickoff |
| `kickoff-e2e-multi-tenant.md` | Completed Phase 1+2 E2E infra |
| `kickoff-e2e-standalone-fixes.md` | Stale — failures are fixed |
| `kickoff-integ-test-multi-tenant.md` | Completed integration test kickoff |
| `kickoff-keycloak-spi.md` | Completed SPI kickoff |
| `kickoff-multi-tenant-routes-app-test.md` | Completed routes_app test kickoff |
| `kickoff-pre-e2e-fixes.md` | Completed pre-E2E fixes |
| `kickoff-ui-test-multi-tenant.md` | Completed UI test kickoff |
| `20260306-middleware-multi-tenant.md` | Completed. Decisions in decisions.md |
| `20260306-multi-tenant-e2e-pg.md` | Completed. Trivial fix |
| `20260306-services-multi-tenant-isolation-test.md` | Completed. Code is truth |
| `20260307-backend-impl.md` | Completed M2 |
| `20260307-e2e-test-failure.md` | Completed E2E infra |
| `20260308-frontend-impl.md` | Completed M3 |
| `20260308-pre-e2e-fixes.md` | Completed M5 |
| `20260308-rs-integration-test.md` | Completed M4 |
| `20260309-create-tenant-membership.md` | Completed |
| `20260309-mt-arch-refactor.md` | Completed. Key decisions absorbed into SUMMARY |
| `20260309-mt-repo-test-audit.md` | Completed gap analysis |
| `20260309-mt-review.md` | Completed review |
| `multi-tenant-flow-ctx.md` | Merged into SUMMARY.md |
| `routes_app-isolation-test.md` | Completed |

### Files to KEEP (modified)

| File | Action |
|------|--------|
| `SUMMARY.md` | **CREATE** — merge `multi-tenant-flow-ctx.md` (679 lines) into ~300-line summary |
| `decisions.md` | **TRIM** — remove verbose code examples, mark superseded decisions |
| `TECHDEBT.md` | **UPDATE** — cross-reference HEAD commit, mark resolved items, add new findings |
| `decision-organization-feature-deferred.md` | **KEEP as-is** — standalone research artifact for enterprise upgrade path |
| `kickoff-e2e-multi-tenant-coverage.md` | **REPLACE** with new comprehensive E2E kickoff (Part 2 below) |

### SUMMARY.md Content Outline (~300 lines)

```
# Multi-Tenant Stage 2 — Summary

## Scope (10 lines)
What stage 2 covered, milestone list, commit reference.

## Architecture Overview (60 lines)
- Deployment modes table (standalone vs multi-tenant)
- Two-phase login flow (condensed)
- Session key schema
- Middleware token lookup summary

## Key Endpoints (40 lines)
Table: dashboard auth, tenant CRUD, activate, enhanced /info, /user/info

## Implementation Record (40 lines)
- Milestone table (M1-M6) with status
- Test count summary per crate

## Frontend Summary (30 lines)
- New pages, hooks, AppInitializer changes
- TypeScript type updates

## Architecture Refactor (30 lines)
- MultiTenantSession AuthContext variant
- tenants_users table
- DeploymentMode enum

## Testing Infrastructure (30 lines)
- Integration test structure
- E2E infra (project renaming, multi-tenant server config)
- Dev-only endpoints (D106)

## Open Findings (30 lines)
Cross-reference TECHDEBT.md
```

### TECHDEBT.md Updates

Based on HEAD commit analysis:
- **Verify** `get_standalone_app()` usages (grep shows 32 matches, some in test/doc files — verify production usages)
- **Mark resolved**: Any TECHDEBT items fixed in the review findings commit (F1-F13, F16)
- **Keep as-is**: Navigation visibility, service construction, shared code exchange, MT-aware logout, integration test CI, frontend unit tests

### Final Folder Structure

```
20260306-multi-tenant-2/
├── SUMMARY.md                              (~300 lines, NEW)
├── decisions.md                            (~500 lines, trimmed)
├── TECHDEBT.md                             (~90 lines, updated)
├── decision-organization-feature-deferred.md  (139 lines, as-is)
└── kickoff-e2e-multi-tenant-coverage.md    (rewritten — Part 2)
```

5 files, down from 28.

---

## Part 2: E2E Kickoff Prompt

The kickoff file (`kickoff-e2e-multi-tenant-coverage.md`) will be rewritten to cover:

### A. New Multi-Tenant-Specific Test Scenarios

All tests use `manager@email.com`, cleanup via `GET /dev/tenants/cleanup` (browser-based, uses dashboard session cookie).

**Scenario 1 (HIGH): Dashboard Login + Tenant Registration**
- Fresh context → platform login → Keycloak auth → 0 tenants → redirect to `/ui/setup/tenants/` → create tenant → OAuth SSO → land on `/ui/chat/` → verify `/info` shows `deployment: "multi_tenant"`

**Scenario 2 (HIGH): Tenant Switching**
- Create 2 tenants → login to Tenant A → navigate to `/ui/login` → see tenant selector → switch to Tenant B → verify `client_id` changes

**Scenario 3 (MEDIUM): Auto-Login with Single Tenant**
- Create 1 tenant → logout → re-login via dashboard → SSO auto-redirect → single tenant auto-login (useRef guard) → land on `/ui/chat/` without seeing selector

**Scenario 4 (MEDIUM): Cross-Tenant Data Isolation**
- 2 tenants → login Tenant A → create API model (via UI, real OpenAI key) + API token → switch Tenant B → verify model and token NOT visible → switch back → verify present

**Scenario 5 (LOW): Multi-Tenant Logout**
- Login → logout → verify State A (platform login button, not tenant selector)

**Scenario 6 (LOW): Resource Admin Flow**
- Create tenant → verify admin role → verify users page accessible

### B. Existing Test Adaptations

**B1. Enhance api-models chat testing** (`api-models/*.spec.mjs`)
- Already has basic chat (send Q&A, verify response) — runs on both projects
- Enhance with: multi-chat management, edge cases, network error recovery (same depth as `chat.spec.mjs:67-115`)
- This covers the "same chat test for API models" requirement
- chat.spec.mjs stays standalone-only (uses local GGUF via `selectModelQwen`)

**B2. Adapt api-tokens.spec.mjs for multi_tenant**
- Remove from `testIgnore`
- Replace `selectModelQwen()` calls with API model selection
- Add `beforeAll`: register API model via UI (same pattern as api-models.spec.mjs)
- Replace 4x `chatSettings.selectModelQwen()` with `chatSettings.selectModel(registeredModelName)`
- Add cleanup in final test step

**B3. Adapt access-request tests for multi_tenant**
- `multi-user-request-approval-flow.spec.mjs` currently uses `createServerManager()` (standalone-specific)
- Needs rework to use shared multi-tenant server with `manager@email.com`
- Fundamentally different flow in multi-tenant (tenant-scoped access requests)

**B4. Review oauth-chat-streaming.spec.mjs**
- Currently excluded because no models available in multi-tenant
- Keep in testIgnore — both OAuth token exchange and API model chat are covered separately
- Low incremental value for combined test

### C. testIgnore Updates

```javascript
// multi_tenant project:
testIgnore: [
  '**/setup/**',                          // Setup wizard is standalone-only
  '**/chat/chat.spec.mjs',               // Local GGUF model only
  '**/chat/chat-agentic.spec.mjs',       // Local GGUF model only
  '**/models/**',                         // Local model operations
  '**/oauth/oauth-chat-streaming.spec.mjs', // Covered separately
  // REMOVED: '**/tokens/**' — adapted to use API models
  // ADD multi-tenant-specific scenarios to standalone testIgnore
],

// standalone project:
testIgnore: [
  '**/multi-tenant-setup/**',            // MT-specific scenarios
],
```

### D. Page Object Changes

**Extend `LoginPage.mjs`** with:
- `performDashboardLogin(credentials)` — click "Login to Bodhi Platform", Keycloak flow
- `waitForTenantSelector()` — wait for State B2
- `selectTenant(tenantName)` — click tenant button
- `waitForAutoLogin()` — wait for single-tenant auto-redirect
- `clickLogout()` — click "Log Out"

**New `TenantRegistrationPage.mjs`**:
- `fillTenantName(name)`, `fillDescription(desc)`, `submitRegistration()`
- Selectors from `crates/bodhi/src/app/ui/setup/tenants/page.tsx`

**New `multiTenantFixtures.mjs`**:
- `createTenantData(index)`, `getManagerCredentials()`

### E. Test Infrastructure

- Real Keycloak at `main-id.getbodhi.app`
- Real API keys (`INTEG_TEST_OPENAI_API_KEY`)
- `manager@email.com` env vars: `INTEG_TEST_USER_MANAGER`, `INTEG_TEST_USER_MANAGER_ID`, `INTEG_TEST_PASSWORD`
- Cleanup: browser-based `GET /dev/tenants/cleanup` with dashboard session cookie
- API model registration as UI test steps (not pre-seeded)

### F. Implementation Phases

**Phase A — Tokens Test Adaptation** (standalone, unblocks testIgnore update):
1. Modify `api-tokens.spec.mjs` — register API model, replace selectModelQwen
2. Update `playwright.config.mjs` — remove `'**/tokens/**'` from multi_tenant testIgnore
3. Verify: `npx playwright test --project multi_tenant tests-js/specs/tokens/`

**Phase B — API Models Chat Enhancement** (standalone):
1. Add multi-chat management test to `api-models.spec.mjs` (or new `api-models-chat.spec.mjs`)
2. Port edge cases, network error recovery from `chat.spec.mjs` pattern
3. Verify: `npx playwright test --project multi_tenant tests-js/specs/api-models/`

**Phase C — Page Objects and Fixtures** (foundation for MT tests):
1. Extend `LoginPage.mjs` with dashboard login methods
2. Create `TenantRegistrationPage.mjs`
3. Create `multiTenantFixtures.mjs`

**Phase D — Core MT Test Scenarios** (depends on Phase C):
1. `specs/multi-tenant-setup/tenant-registration.spec.mjs` (Scenario 1)
2. `specs/multi-tenant-setup/dashboard-login.spec.mjs` (Scenario 3)
3. `specs/multi-tenant-setup/tenant-switching.spec.mjs` (Scenario 2)
4. `specs/multi-tenant-setup/cross-tenant-isolation.spec.mjs` (Scenario 4)
5. Update both project testIgnore lists

**Phase E — Access Request Adaptation** (depends on MT infrastructure):
1. Rework `multi-user-request-approval-flow.spec.mjs` for shared multi-tenant server
2. Verify: `npx playwright test --project multi_tenant tests-js/specs/request-access/`

**Phase F — Optional Low Priority**:
1. `mt-logout.spec.mjs` (Scenario 5)
2. `mt-resource-admin.spec.mjs` (Scenario 6)

---

## Critical Files

### Plan docs to modify
- `ai-docs/claude-plans/20260306-multi-tenant-2/` — all files listed in DELETE/KEEP sections

### Key reference files for E2E kickoff
- `crates/lib_bodhiserver_napi/playwright.config.mjs` — project config, testIgnore
- `crates/lib_bodhiserver_napi/tests-js/fixtures.mjs` — test fixtures
- `crates/lib_bodhiserver_napi/tests-js/pages/LoginPage.mjs` — extend with dashboard methods
- `crates/lib_bodhiserver_napi/tests-js/specs/api-models/api-models.spec.mjs` — enhance chat
- `crates/lib_bodhiserver_napi/tests-js/specs/tokens/api-tokens.spec.mjs` — adapt for API models
- `crates/lib_bodhiserver_napi/tests-js/specs/chat/chat.spec.mjs` — reference for chat depth
- `crates/bodhi/src/app/ui/login/page.tsx` — MultiTenantLoginContent states
- `crates/bodhi/src/app/ui/setup/tenants/page.tsx` — registration form selectors
- `crates/routes_app/src/routes_dev.rs` — cleanup endpoints

---

## Verification

### Part 1 (Folder simplification)
- Verify final folder has exactly 5 files
- Verify SUMMARY.md covers key architectural patterns from multi-tenant-flow-ctx.md
- Verify TECHDEBT.md items are accurate against HEAD commit code
- Verify decisions.md has no references to deleted files

### Part 2 (E2E kickoff)
- Kickoff prompt is self-contained and actionable
- All file paths verified to exist
- All data-testid selectors verified against current frontend code
- Build & run commands verified
