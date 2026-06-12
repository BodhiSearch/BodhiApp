# Multi-Tenant E2E Tests + Add User by Email Feature

> **Created**: 2026-03-11
> **Status**: DRAFT
> **Scope**: Backend "add user" feature, tenant cleanup update, frontend UI, multi-tenant E2E tests
> **Prerequisites**: Keycloak SPI extended with `{ email, role }` support on `POST /resources/assign-role` (separate task)

---

## Context

Multi-tenant stage 2 is functionally complete (auth, tenant lifecycle, session namespacing, RLS isolation). ~35 shared tests pass on the multi-tenant server. What's missing:

1. **New E2E tests** for multi-tenant-specific flows (tenant registration, switching, data isolation, logout)
2. **A "direct add user" feature** — admin/manager can add a user by email to their tenant (needed for the consolidated E2E test where user@email.com adds manager@email.com to their tenant)
3. **Tenant cleanup scoping** — current cleanup truncates ALL tenants; needs to be user-scoped to avoid destroying the pre-seeded test tenant
4. **UI testability** — add semantic `data-test-*` attributes to login/tenant components

The consolidated test validates the complete multi-tenant lifecycle in a single E2E flow.

---

## Key Decisions (from interview)

| Decision | Choice |
|----------|--------|
| Tenant cleanup mechanism | Browser `GET /dev/tenants/cleanup` (session cookie auth) |
| Cleanup filtering | Local DB mirrors SPI response (delete only confirmed client_ids) |
| Pre-seeded tenant name | `[do-not-delete] Test user@email.com tenant` |
| SSO behavior | Auto-completes (no second KC credential prompt) |
| Approach | Browser-first exploration via Claude-in-Chrome, then write tests |
| Test selectors | Add `data-test-*` attributes (clientid, tenant-name, action) |
| Fixture pattern | Shared server + autoResetDb |
| DAG endpoint | Not needed |
| Add user API | `POST /bodhi/v1/users/add` with `{ email, role }` or `{ user_id, role }` |
| SPI response | Returns `{ user_id, username, role, status }` |
| Add user UI | New `/ui/users/add` page, toast + redirect to `/ui/users` on success |
| Role restrictions | Admin/manager only; can assign roles ≤ own role |
| Browser context | Second Playwright BrowserContext for parallel sessions |
| Assigned role in test | `resource_manager` (manager on user's tenant) |
| Isolation data | API model alias |
| Role verification | `GET /bodhi/v1/user` response |
| Cleanup flow | Inline `test.step()` blocks |
| Scenario 3 (auto-login) | Check existing user@email.com tests; update if not covered |

---

## Reuse Existing Patterns

| Pattern | Source | Reuse in |
|---------|--------|----------|
| Role hierarchy check | `users_change_role` in `routes_users.rs:69-119` | `users_add` handler |
| SPI forward | `assign_user_role` in `auth_service.rs:640-667` | `add_user_to_role` method |
| Tenant upsert | `assign_user_role` in `auth_scoped.rs:59-77` | `add_user` method |
| ValidatedJson pattern | Throughout routes_app handlers | `users_add` request validation |
| Card form layout | `setup/tenants/page.tsx` | `/ui/users/add/page.tsx` |
| Page object base | `BasePage.mjs` | All new page objects |
| Dashboard OAuth flow | `LoginPage.mjs:performOAuthLogin` | `MultiTenantLoginPage.performDashboardLogin` |
| `autoResetDb` fixture | `fixtures.mjs` | All new specs |

---

## Phase 0: Browser-First Exploration

> **Agent**: Claude-in-Chrome (manual browser interaction)
> **Gate**: No automated gate — produces exploration notes for subsequent phases
> **Commit**: None

Before writing any code, manually navigate through the multi-tenant flows:

1. Start multi-tenant server: `cd crates/lib_bodhiserver_napi && npm run e2e:server:multi_tenant`
2. Navigate to `http://localhost:41135/ui/login`
3. Walk through: dashboard login → tenant registration → auto OAuth → /ui/chat
4. Verify selectors, redirect timing, SSO auto-completion
5. Note any missing `data-test-*` attributes or UI gaps
6. Document findings to inform subsequent phases

---

## Phase 1: Backend — services crate (add-user + tenant cleanup)

> **Agent**: Specialized sub-agent for `services` crate Rust development
> **Skill**: `/test-services` for test patterns
> **Gate**: `cargo test -p services --lib 2>&1 | grep -E "test result|FAILED|failures:"`
> **Commit**: `feat: add-user service methods and tenant cleanup scoping`

### Scope

| File | Change |
|------|--------|
| `crates/services/src/users/user_objs.rs` | `AddUserRequest`, `AssignRoleResponse` types |
| `crates/services/src/auth/auth_service.rs` | `add_user_to_role` on trait + `KeycloakAuthService` impl + MockAuthService |
| `crates/services/src/users/auth_scoped.rs` | `add_user` method on `AuthScopedUserService` |
| `crates/services/src/tenants/tenant_service.rs` | `delete_tenant_by_client_id` method |
| `crates/services/src/tenants/tenant_repository.rs` | `delete_tenant_by_client_id` impl (tenant + tenant_users) |
| `crates/services/src/users/test_auth_scoped_add_user.rs` | NEW — unit tests |

### Implementation Details

**1a. New types** (`user_objs.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AssignRoleResponse {
  pub user_id: String,
  pub username: String,
  pub role: String,
  pub status: String, // "added" | "updated"
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct AddUserRequest {
  pub email: Option<String>,
  pub user_id: Option<String>,
  pub role: ResourceRole,
}
// Custom validation: exactly one of email or user_id must be present
```

**1b. New trait method** (`auth_service.rs`):
```rust
async fn add_user_to_role(
  &self,
  reviewer_token: &str,
  request: &serde_json::Value, // { email, role } or { user_id, role }
) -> Result<AssignRoleResponse>;
```
Implementation: same SPI endpoint `POST /resources/assign-role`, forwards request body, parses structured `AssignRoleResponse`. Also update `MockAuthService` automock.

**1c. Auth-scoped method** (`auth_scoped.rs`):
```rust
pub async fn add_user(&self, request: &AddUserRequest) -> Result<AssignRoleResponse, AuthScopedUserError> {
  let token = self.require_token()?;
  let tenant_id = self.auth_context.require_tenant_id()?;
  let body = if let Some(email) = &request.email {
    json!({ "email": email, "role": request.role.to_string() })
  } else {
    json!({ "user_id": request.user_id.as_ref().unwrap(), "role": request.role.to_string() })
  };
  let response = self.app_service.auth_service().add_user_to_role(token, &body).await?;
  self.app_service.tenant_service().upsert_tenant_user(tenant_id, &response.user_id).await?;
  Ok(response)
}
```

**1d. Tenant cleanup method** (`tenant_service.rs` + `tenant_repository.rs`):
New `delete_tenant_by_client_id(client_id)` — deletes tenant record AND associated tenant_users records.

**1e. Unit tests** (`test_auth_scoped_add_user.rs`):
- Email path: mock SPI → assert tenant_users upserted
- user_id path: same
- Validation: neither email nor user_id → error
- require_token failure path

---

## Phase 2: Backend — routes_app crate (handler + cleanup update)

> **Agent**: Specialized sub-agent for `routes_app` crate Rust development
> **Skill**: `/test-routes-app` for test patterns
> **Gate**: `cargo test -p routes_app 2>&1 | grep -E "test result|FAILED|failures:"`
> **Commit**: `feat: POST /bodhi/v1/users/add handler and scoped tenant cleanup`

### Scope

| File | Change |
|------|--------|
| `crates/routes_app/src/users/routes_users.rs` | `users_add` handler |
| `crates/routes_app/src/users/mod.rs` | Re-export `users_add` |
| `crates/routes_app/src/routes.rs` | Register route in session_auth router |
| `crates/routes_app/src/shared/openapi.rs` | `AddUserRequest`, `AssignRoleResponse` schemas + `users_add` path |
| `crates/routes_app/src/routes_dev.rs` | Update `dev_tenants_cleanup_handler` |
| `crates/routes_app/src/users/test_*.rs` | Unit tests |

### Implementation Details

**2a. Handler** (`routes_users.rs`):
```rust
pub async fn users_add(
  auth_scope: AuthScope,
  ValidatedJson(request): ValidatedJson<AddUserRequest>,
) -> Result<(StatusCode, Json<AssignRoleResponse>), ApiError> {
  // Validate: caller role >= request.role (reuse pattern from users_change_role:69-119)
  let caller_role = auth_scope.auth_context().resource_role()...;
  if !caller_role.has_access_to(&request.role) { return Err(...) }
  let response = auth_scope.users().add_user(&request).await?;
  let status = if response.status == "added" { StatusCode::CREATED } else { StatusCode::OK };
  Ok((status, Json(response)))
}
```

**2b. Route registration** (`routes.rs`): Add in `session_auth` router (manager minimum):
```rust
.route("/bodhi/v1/users/add", post(users_add))
```

**2c. Cleanup handler update** (`routes_dev.rs`):

**Current**: Calls SPI cleanup → truncates ALL local tenants via `reset_tenants()`.

**New**: Parse SPI response for deleted client_ids → delete only those from local DB:
```rust
// SPI already filters by dashboard-client-id, user-id, and [do-not-delete] prefix
// SPI returns: { deleted: [{ client_id, type }], skipped: [...], errors: [...] }
let deleted: Vec<serde_json::Value> = serde_json::from_value(body["deleted"].clone())?;
for entry in &deleted {
  if let Some(client_id) = entry["client_id"].as_str() {
    auth_scope.tenants().delete_tenant_by_client_id(client_id).await?;
  }
}
```

**2d. Tests**:
- `users_add` happy path → 201
- Role hierarchy violation → error
- Cleanup handler with mocked SPI response

---

## Phase 3: NAPI — Configurable pre-seeded tenant name

> **Agent**: Specialized sub-agent for NAPI/Node.js integration
> **Gate**: `cargo test -p lib_bodhiserver_napi 2>&1 | grep -E "test result|FAILED|failures:"`
> **Commit**: `feat: configurable pre-seeded tenant name with [do-not-delete] prefix`

### Scope

| File | Change |
|------|--------|
| `crates/lib_bodhiserver_napi/src/config.rs` | Add `tenant_name` field + `set_tenant_name` NAPI fn |
| `crates/lib_bodhiserver_napi/tests-js/test-helpers.mjs` | Pass `tenantName` through `createFullTestConfig` |
| `crates/lib_bodhiserver_napi/tests-js/scripts/start-shared-server.mjs` | Set tenant name for multi_tenant |

### Implementation Details

**3a.** Add `tenant_name: Option<String>` to `NapiAppOptions` struct. Add `set_tenant_name` function.

In `try_build_app_options_internal`, change line 135 (`name: "BodhiApp".to_string()`) to:
```rust
name: config.tenant_name.unwrap_or_else(|| "BodhiApp".to_string()),
```

**3b.** In `createFullTestConfig` (`test-helpers.mjs`), accept `tenantName` option:
```javascript
if (tenantName) {
  config = bindings.setTenantName(config, tenantName);
}
```

**3c.** In `start-shared-server.mjs`, for `multi_tenant` deployment:
```javascript
tenantName: '[do-not-delete] Test user@email.com tenant',
```

---

## Phase 4: TypeScript client regeneration

> **Agent**: Specialized sub-agent for build pipeline
> **Gate**: `cd ts-client && npm test 2>&1 | grep -E "Tests:|FAIL"`
> **Commit**: `chore: regenerate TypeScript client with AddUserRequest and AssignRoleResponse`

### Steps
```bash
cargo run --package xtask openapi
cd ts-client && npm run generate && npm test
```

Generates `AddUserRequest` and `AssignRoleResponse` types in `@bodhiapp/ts-client`.

---

## Phase 5: Frontend — data-test attributes + add-user page

> **Agent**: Specialized sub-agent for React/Next.js frontend development
> **Gate**: `cd crates/bodhi && npm test 2>&1 | grep -E "Tests:|FAIL"`
> **Commit**: `feat: add-user page, data-test attributes for multi-tenant login states`

### Scope

| File | Change |
|------|--------|
| `crates/bodhi/src/components/AuthCard.tsx` | Add `data-test-action` on each action button |
| `crates/bodhi/src/app/ui/login/page.tsx` | Add `data-test-state`, `data-test-clientid`, `data-test-tenant-name` |
| `crates/bodhi/src/hooks/useUsers.ts` | Add `useAddUser` hook |
| `crates/bodhi/src/app/ui/users/add/page.tsx` | NEW — add user form page |
| `crates/bodhi/src/app/ui/users/page.tsx` | Add "Add User" navigation button |
| `crates/bodhi/src/app/ui/users/add/__tests__/page.test.tsx` | NEW — Vitest tests |

### Implementation Details

**5a. data-test-* attributes**:

AuthCard (`AuthCard.tsx`) — on each action button:
```tsx
data-test-action={action.label}
```

Login page (`page.tsx`) — add state attribute + tenant metadata:
```tsx
<AuthCard data-test-state="login" ... />        // State A
<AuthCard data-test-state="connect" ... />       // State B1
<AuthCard data-test-state="select" ... />        // State B2
<AuthCard data-test-state="welcome" ... />       // State C
```
On tenant buttons in State B2 and C:
```tsx
data-test-clientid={tenant.client_id}
data-test-tenant-name={tenant.name}
```

**5b. useAddUser hook** (`useUsers.ts`):
```typescript
export function useAddUser(options?: { onSuccess?, onError? }) {
  // POST /bodhi/v1/users/add
  // Invalidates 'users' query on success
}
```

**5c. /ui/users/add page** (NEW):
- `AppInitializer allowedStatus="ready" authenticated={true} minRole="manager"`
- Card-based form (follow `setup/tenants/page.tsx` layout pattern)
- Fields:
  - Email input: `data-testid="add-user-email-input"`
  - Role dropdown: `data-testid="add-user-role-select"` — filtered by current user's role
  - Submit button: `data-testid="add-user-submit-button"`
- Role options: Admin sees all 4 roles, Manager sees manager + power_user + user
- On success: toast "User added successfully" → redirect to `/ui/users`
- On error: inline alert with error message

**5d.** Add "Add User" button on users page linking to `/ui/users/add`.

**5e. Vitest tests** — form rendering, filtered roles, submission, success redirect, error display.

---

## Phase 6: UI Rebuild

> **Agent**: Specialized sub-agent for build pipeline
> **Gate**: `make build.ui-rebuild` exits 0
> **Commit**: `chore: rebuild embedded UI with multi-tenant data-test attributes and add-user page`

```bash
make build.ui-rebuild
```

---

## Phase 7: E2E — Page objects + infrastructure

> **Agent**: Specialized sub-agent for Playwright E2E test infrastructure
> **Skill**: `/playwright` for page object and test patterns
> **Gate**: Syntax check — `cd crates/lib_bodhiserver_napi && node -e "import('./tests-js/pages/MultiTenantLoginPage.mjs')"`
> **Commit**: `feat: multi-tenant page objects and E2E infrastructure`

### Scope

| File | Change |
|------|--------|
| `tests-js/pages/MultiTenantLoginPage.mjs` | NEW — dashboard OAuth + tenant selection page object |
| `tests-js/pages/TenantRegistrationPage.mjs` | NEW — tenant creation form page object |
| `tests-js/pages/AddUserPage.mjs` | NEW — add user form page object |
| `playwright.config.mjs` | Add `**/multi-tenant/**` to standalone testIgnore |

### Implementation Details

**7a. MultiTenantLoginPage** — extends BasePage:
- `performDashboardLogin(credentials)` — clicks "Login to Bodhi Platform" → KC login → callback
- `waitForTenantSelection()` — waits for `[data-test-state="select"]`
- `selectTenant(tenantName)` — clicks `[data-test-tenant-name="${name}"]`
- `switchToTenant(tenantName)` — clicks `button:has-text("Switch to ${name}")`
- `expectStateA()` — asserts `[data-test-state="login"]` visible
- `expectStateC(username)` — asserts `[data-test-state="welcome"]` visible
- `logout()` — clicks `button:has-text("Log Out")`

**7b. TenantRegistrationPage** — extends BasePage:
- `fillTenantForm(name, desc)` — fills `tenant-name-input`, `tenant-description-input`
- `submitTenantForm()` — clicks `create-tenant-button`
- `waitForCreated()` — waits for redirect to `/ui/chat/`

**7c. AddUserPage** — extends BasePage:
- `navigateToAddUser()` — navigates to `/ui/users/add`
- `fillEmail(email)` — fills `add-user-email-input`
- `selectRole(role)` — selects from `add-user-role-select`
- `submitAddUser()` — clicks `add-user-submit-button`
- `expectSuccess()` — waits for toast + redirect to `/ui/users`
- `expectError(msg)` — asserts error alert

**7d. Playwright config** — add to standalone testIgnore:
```javascript
'**/multi-tenant/**',
```

---

## Phase 8: E2E — Consolidated multi-tenant lifecycle test

> **Agent**: Specialized sub-agent for Playwright E2E test writing
> **Skill**: `/playwright` for test conventions
> **Gate**: `cd crates/lib_bodhiserver_napi && npx playwright test --project multi_tenant tests-js/specs/multi-tenant/ 2>&1 | grep -E "passed|failed"`
> **Commit**: `feat: multi-tenant E2E lifecycle test (register, invite, switch, isolate, logout)`

### Scope

| File | Change |
|------|--------|
| `tests-js/specs/multi-tenant/multi-tenant-lifecycle.spec.mjs` | NEW — consolidated lifecycle test |

### Test Flow (single test, `test.step()` blocks)

```javascript
test('multi-tenant lifecycle: register, invite, switch, isolate, logout', async ({ browser, sharedServerUrl }) => {
  // Step 1: Cleanup
  // - Create managerContext (new BrowserContext)
  // - Dashboard login as manager@email.com
  // - Navigate to GET /dev/tenants/cleanup (browser, session cookie auth)

  // Step 2: Register tenant
  // - Navigate to /ui/setup/tenants (redirected after 0 tenants)
  // - Fill "Test Tenant" name, submit
  // - SSO auto-completes tenant OAuth → /ui/chat

  // Step 3: Create API model in manager's tenant
  // - Navigate to API models page, create model alias (for later isolation test)

  // Step 4: Second context — user@email.com
  // - Create userContext (new BrowserContext)
  // - Dashboard login as user@email.com
  // - Single tenant → auto-login → /ui/chat

  // Step 5: Add user
  // - user@email.com navigates to /ui/users/add
  // - Fills manager@email.com email, selects "Manager" role
  // - Submits → toast → redirect to /ui/users

  // Step 6: Tenant switching
  // - Back to managerContext
  // - Navigate to /ui/login → State C with 2 tenants
  // - Click "Switch to [do-not-delete] Test user@email.com tenant"
  // - SSO auto-completes → /ui/chat

  // Step 7: Role verification
  // - GET /bodhi/v1/user → assert role === "resource_manager"

  // Step 8: Data isolation
  // - Verify API model from manager's tenant NOT visible here

  // Step 9: Logout
  // - Navigate to /ui/login → click "Log Out"
  // - Verify State A (login button visible)
});
```

---

## Phase 9: E2E — Dedicated add-user test + auto-login coverage

> **Agent**: Specialized sub-agent for Playwright E2E test writing
> **Skill**: `/playwright` for test conventions
> **Gate**: `cd crates/lib_bodhiserver_napi && npx playwright test --project multi_tenant tests-js/specs/users/add-user.spec.mjs 2>&1 | grep -E "passed|failed"`
> **Commit**: `feat: add-user E2E test and auto-login coverage check`

### Scope

| File | Change |
|------|--------|
| `tests-js/specs/users/add-user.spec.mjs` | NEW — dedicated add-user feature test |
| Existing shared tests | Check/update auto-login (Scenario 3) coverage for user@email.com |

### Add-User Test Cases
- Add user by email (happy path) → toast + redirect
- Role hierarchy restriction (manager can't assign admin) → error
- User not found by email → error display
- User already added → idempotent (200)

### Auto-Login Coverage
Review existing multi_tenant tests for user@email.com. If auto-login (single tenant → auto redirect → /ui/chat) is not explicitly tested, add assertion in existing test.

---

## Phase 10: Full Regression

> **Agent**: Specialized sub-agent for test verification
> **Gate**: All commands below pass
> **Commit**: None (verification only)

```bash
# All multi-tenant E2E tests (existing + new)
cd crates/lib_bodhiserver_napi && npx playwright test --project multi_tenant

# Ensure standalone tests unaffected
cd crates/lib_bodhiserver_napi && npx playwright test --project standalone

# Full backend regression
make test.backend

# Full regression
make test
```

---

## Phase Summary

| Phase | Agent Type | Gate Check | Commit Message |
|-------|-----------|------------|----------------|
| 0 | Claude-in-Chrome | Manual exploration notes | — |
| 1 | services Rust | `cargo test -p services --lib` | `feat: add-user service methods and tenant cleanup scoping` |
| 2 | routes_app Rust | `cargo test -p routes_app` | `feat: POST /bodhi/v1/users/add handler and scoped tenant cleanup` |
| 3 | NAPI integration | `cargo test -p lib_bodhiserver_napi` | `feat: configurable pre-seeded tenant name with [do-not-delete] prefix` |
| 4 | Build pipeline | `cd ts-client && npm test` | `chore: regenerate TypeScript client` |
| 5 | React/Next.js frontend | `cd crates/bodhi && npm test` | `feat: add-user page and data-test attributes` |
| 6 | Build pipeline | `make build.ui-rebuild` | `chore: rebuild embedded UI` |
| 7 | Playwright infra | Syntax check (node import) | `feat: multi-tenant page objects` |
| 8 | Playwright E2E | `npx playwright test --project multi_tenant tests-js/specs/multi-tenant/` | `feat: multi-tenant E2E lifecycle test` |
| 9 | Playwright E2E | `npx playwright test --project multi_tenant tests-js/specs/users/add-user.spec.mjs` | `feat: add-user E2E test and auto-login coverage` |
| 10 | Verification | `make test` | — |

Each phase is **strictly sequential** — the gate check must pass before proceeding to the next phase.
