# Remove AppStatus::TenantSelection Variant

## Context

`AppStatus` currently has 4 variants: `Setup`, `Ready`, `ResourceAdmin`, `TenantSelection`. The `TenantSelection` variant exists solely for multi-tenant deployments and is never stored in the database — it's computed at request time in `setup_show()`. This mixes deployment-mode concerns into what should be a standalone app lifecycle enum.

**Goal**: Remove `TenantSelection` from `AppStatus`. Multi-tenant detection should use `deployment == multi_tenant` (from `AppInfo`) + `client_id` presence instead of a dedicated status variant.

**New status model**:
- **Standalone**: `Setup` → `ResourceAdmin` → `Ready` (unchanged)
- **Multi-tenant**: `Setup` (no memberships) | `Ready` (has memberships or anonymous)

## Status Mapping (Backend)

Current `setup_show()` auth context → status mapping, and what changes:

| Auth Context | Current Status | New Status | Change? |
|---|---|---|---|
| `Anonymous { MultiTenant }` | `TenantSelection` | `Ready` | YES |
| `MultiTenantSession { client_id: None } + has_memberships` | `TenantSelection` | `Ready` | YES |
| `MultiTenantSession { client_id: None } + !has_memberships` | `Setup` | `Setup` | no |
| `MultiTenantSession { client_id: Some }` | `Ready` | `Ready` | no |
| `Anonymous { Standalone }` | reads from DB | reads from DB | no |
| `Session { .. }` | `Ready` | `Ready` | no |
| `ApiToken / ExternalApp` | `Ready` | `Ready` | no |

## Frontend Routing (AppInitializer)

Current routing in `AppInitializer.tsx` switch statement, and what changes:

**Current**:
- `setup` + multi_tenant → `/ui/setup/tenants`
- `setup` + standalone → `/ui/setup`
- `ready` → `/ui/chat`
- `resource_admin` → `/ui/setup/resource-admin`
- `tenant_selection` → `/ui/login`

**New**:
- `setup` + multi_tenant → `/ui/setup/tenants` (unchanged)
- `setup` + standalone → `/ui/setup` (unchanged)
- `ready` + multi_tenant + !client_id → `/ui/login` (NEW)
- `ready` (all other) → `/ui/chat` (unchanged)
- `resource_admin` → `/ui/setup/resource-admin` (unchanged)
- Remove `tenant_selection` case entirely

## Parallel Changes (Already Staged)

The following changes are already staged (`git diff --cached`) and must be accounted for:

1. **`UserInfoEnvelope`** changed: `has_dashboard_session: bool` → `dashboard: Option<DashboardUser>`
   - `DashboardUser` is a new struct: `{ user_id, username, first_name, last_name }`
   - File: `crates/routes_app/src/users/users_api_schemas.rs`

2. **`MultiTenantSession { client_id: None }` user response**: Now returns `UserResponse::LoggedOut` (was `LoggedIn`) with `dashboard: Some(DashboardUser)` populated from the dashboard token
   - File: `crates/routes_app/src/users/routes_users_info.rs`

3. **Login page**: `needsTenantSelection` condition changed from `isAuthenticated && !!userInfo?.has_dashboard_session && !appInfo?.client_id` to `!!userInfo?.dashboard && !appInfo?.client_id`
   - File: `crates/bodhi/src/app/ui/login/page.tsx` (lines 96, 125)

4. **Frontend mock helpers**: `mockUserLoggedOut()` and `mockUserLoggedIn()` now accept `dashboard` property
   - File: `crates/bodhi/src/test-utils/msw-v2/handlers/user.ts`

5. **Staged tests already use new field**: `mockUserLoggedOut({ dashboard: { user_id: ..., username: ... } })` pattern
   - File: `crates/bodhi/src/app/ui/login/page.test.tsx` (lines 368-370)

**Impact on this plan**: Our test mocks must use `dashboard: { ... }` (not `has_dashboard_session: true`). The staged tests at lines 331, 347, 371 still use `status: 'tenant_selection'` — those are what we change to `status: 'ready'`.

## Implementation Steps

### Layer 1: Backend — Remove `TenantSelection` from enum

**File**: `crates/services/src/tenants/tenant_objs.rs`
- Remove `TenantSelection` variant and its `#[schema]` annotation from `AppStatus` enum
- Verify: `cargo check -p services`

### Layer 2: Backend — Update `setup_show()` handler

**File**: `crates/routes_app/src/setup/routes_setup.rs`

Update the match arms in `setup_show()`:

1. `Anonymous { MultiTenant }` → change return from `(AppStatus::TenantSelection, None)` to `(AppStatus::Ready, None)`
2. `MultiTenantSession { client_id: None }` + has_memberships → change from `(AppStatus::TenantSelection, None)` to `(AppStatus::Ready, None)`
3. Merge arms that now return the same `(Ready, None)`:
   - `Anonymous { MultiTenant }` and `MultiTenantSession { client_id: None, has_memberships }` both return `(Ready, None)`
   - Consider merging with `MultiTenantSession { client_id: Some }` which also returns `(Ready, _)`
4. `MultiTenantSession { client_id: None } + !has_memberships` stays `(Setup, None)`

- Verify: `cargo check -p routes_app`

### Layer 3: Backend tests — Update existing

**File**: `crates/routes_app/src/setup/test_setup.rs`
- No `TenantSelection` references exist here (only tests standalone flow with `Setup`, `Ready`)
- No changes needed

**File**: `crates/routes_app/tests/test_live_multi_tenant.rs`
- Line 230: Update comment `tenant_selection` → `ready`
- Line 251: Change `assert_eq!(AppStatus::TenantSelection, body.status)` → `assert_eq!(AppStatus::Ready, body.status)`

**File**: `crates/server_app/tests/test_live_multi_tenant.rs`
- Line 82: Update comment
- Line 102: Update comment
- Line 109: Change `assert_eq!("tenant_selection", ...)` → `assert_eq!("ready", ...)`
- Line 262: Update comment
- Line 274: Update comment
- Line 281: Change assertion → `"ready"`
- Lines 307-308: Change `status == "setup" || status == "tenant_selection"` → `status == "setup" || status == "ready"`

### Layer 3b: Backend tests — Add missing coverage

**File**: `crates/routes_app/src/setup/test_setup.rs`

The existing `test_app_info_handler` only tests standalone scenarios. Add unit test cases for each multi-tenant auth context path through `setup_show()`:

1. **Anonymous { MultiTenant } → Ready**
   - Mock `AuthContext::Anonymous { deployment: MultiTenant }` via request extension
   - Assert `status == Ready`, `client_id == None`, `deployment == MultiTenant`

2. **MultiTenantSession { client_id: None } + has_memberships → Ready**
   - Mock `MultiTenantSession` with `client_id: None`
   - Mock `TenantService::has_memberships()` returning `true`
   - Assert `status == Ready`, `client_id == None`

3. **MultiTenantSession { client_id: None } + !has_memberships → Setup**
   - Mock `MultiTenantSession` with `client_id: None`
   - Mock `TenantService::has_memberships()` returning `false`
   - Assert `status == Setup`, `client_id == None`

4. **MultiTenantSession { client_id: Some } → Ready**
   - Mock `MultiTenantSession` with `client_id: Some("test-client")`
   - Assert `status == Ready`, `client_id == Some("test-client")`

Use the existing test pattern in `test_setup.rs`: `AppServiceStubBuilder` + `RequestAuthContextExt` + `tower::oneshot()`.

- Verify: `cargo test -p routes_app -p server_app 2>&1 | grep -E "test result|FAILED"`

### Layer 4: Regenerate OpenAPI + ts-client

```bash
cargo run --package xtask openapi
make build.ts-client
```

This removes `tenant_selection` from the generated `AppStatus` type in `ts-client/src/types/types.gen.ts`.

### Layer 5: Frontend — AppInitializer

**File**: `crates/bodhi/src/components/AppInitializer.tsx`

1. Remove `case 'tenant_selection'` (lines 68-69)
2. Update `case 'ready'` to handle multi-tenant routing:
   ```typescript
   case 'ready':
     if (appInfo.deployment === 'multi_tenant' && !appInfo.client_id) {
       router.push(ROUTE_LOGIN);
     } else {
       router.push(ROUTE_DEFAULT);
     }
     break;
   ```
3. Update validation list (line 136): remove `'tenant_selection'` from the array
   ```typescript
   if (!['setup', 'ready', 'resource_admin'].includes(appInfo.status)) {
   ```

### Layer 6: Frontend — Login page

**File**: `crates/bodhi/src/app/ui/login/page.tsx`

1. Update `allowedStatuses` (line 456-457):
   ```typescript
   const allowedStatuses: AppStatus[] =
     isMultiTenant && hasInviteFlow ? ['ready', 'setup'] : ['ready'];
   ```
2. Remove `AppStatus` import if no longer needed (check usage — it's used in type annotation, likely still needed)

### Layer 7: Frontend tests — Update existing

**File**: `crates/bodhi/src/app/ui/login/page.test.tsx`

Update multi-tenant mock statuses (3 occurrences in staged code):
- Line 331: `mockAppInfo({ status: 'tenant_selection', deployment: 'multi_tenant' })` → `mockAppInfo({ status: 'ready', deployment: 'multi_tenant' })`
- Line 347: same change
- Line 371: same change

These staged tests already use the new `dashboard` field pattern (e.g. `mockUserLoggedOut({ dashboard: { user_id: ..., username: ... } })`). We only change the `status` values.

Tests verified:
- "does not call /tenants when no dashboard session present" — status changes, same logic
- "State A: shows login button when no dashboard session" — status changes, same logic
- "State B: shows tenant selection when dashboard session present" — status changes, same logic (already uses `mockUserLoggedOut({ dashboard: {...} })`)
- "State C: welcome when fully authenticated" — already uses `status: 'ready'`, no change

**File**: `crates/bodhi/src/test-utils/msw-v2/handlers/info.ts`
- No `mockAppInfoTenantSelection` helper exists — no changes needed
- After ts-client regeneration, `'tenant_selection'` will be a TypeScript type error (catches leftover references)

### Layer 7b: Frontend tests — Add missing coverage

**File**: `crates/bodhi/src/components/AppInitializer.test.tsx`

Currently MISSING multi-tenant routing tests. The existing parameterized tests (lines 95-104) only test standalone scenarios (default `deployment: 'standalone'`). Add to the "routing based on currentStatus and allowedStatus" describe block:

1. **`ready + multi_tenant + !client_id → /ui/login`** (NEW routing behavior)
   - Mock: `mockAppInfo({ status: 'ready', deployment: 'multi_tenant' })` (client_id defaults to undefined)
   - Assert: `pushMock` called with `'/ui/login'`

2. **`ready + multi_tenant + client_id → /ui/chat`** (verify active tenant routes to chat)
   - Mock: `mockAppInfo({ status: 'ready', deployment: 'multi_tenant', client_id: 'test-client' })`
   - Assert: `pushMock` called with `ROUTE_DEFAULT`

3. **`setup + multi_tenant → /ui/setup/tenants`** (existing behavior, never tested)
   - Mock: `mockAppInfo({ status: 'setup', deployment: 'multi_tenant' })`
   - Assert: `pushMock` called with `'/ui/setup/tenants'`

4. **`ready + multi_tenant + !client_id stays on page when allowedStatus='ready'`** (guard behavior)
   - Mock: `mockAppInfo({ status: 'ready', deployment: 'multi_tenant' })`
   - Render with `<AppInitializer allowedStatus="ready" />`
   - Assert: `pushMock` NOT called (page renders children)

- Verify: `cd crates/bodhi && npm test`

### Layer 8: E2E tests

**File**: `crates/lib_bodhiserver_napi/tests-js/pages/MultiTenantLoginPage.mjs`
- Line 85: `waitForTenantSelection()` waits for `[data-test-state="select"]` CSS selector — this is a UI element check, NOT an API status check
- **No changes needed** — the multi-tenant login page still renders `data-test-state="select"` when showing tenant selection; only the backend status value changed

**File**: `crates/lib_bodhiserver_napi/tests-js/specs/multi-tenant/multi-tenant-lifecycle.spec.mjs`
- Line 165: `await userLogin.waitForTenantSelection()` — uses the page object above
- **No changes needed** — the E2E flow still works because:
  1. Anonymous user hits app → backend returns `status: 'ready'` (was `tenant_selection`)
  2. AppInitializer sees `ready + multi_tenant + !client_id` → routes to `/ui/login` (same destination)
  3. Login page accepts `ready` status → renders MultiTenantLoginContent → shows `data-test-state="select"` for 2+ tenants
  4. Page object finds the selector → test passes

**E2E impact summary**: Zero E2E test file changes required. The tests verify UI behavior (selectors, navigation), not API status strings. The routing destination (`/ui/login`) and UI state (`data-test-state="select"`) remain identical.

### Layer 9: Documentation updates

**Files to update** (remove `TenantSelection` references):
- `crates/services/CLAUDE.md` — update `tenant_objs.rs` description
- `crates/bodhi/src/CLAUDE.md` — update AppInitializer routing description

**Files to leave as-is** (historical planning docs):
- `ai-docs/claude-plans/20260306-multi-tenant-2/*.md` — historical, no need to update
- `ai-docs/claude-plans/20260320-*.md` — historical

## Verification

1. `cargo check -p services -p routes_app` — compilation
2. `cargo test -p services -p routes_app -p server_app` — backend tests
3. `cargo run --package xtask openapi && make build.ts-client` — regenerate types
4. `cd crates/bodhi && npm test` — frontend tests
5. `make build.ui-rebuild` — rebuild embedded UI
6. Manual: start multi-tenant server, verify anonymous → login page, dashboard+no-tenants → setup, dashboard+tenants → login with selection, fully authenticated → chat

## Files Modified (Summary)

| File | Change |
|---|---|
| **Backend** | |
| `crates/services/src/tenants/tenant_objs.rs` | Remove `TenantSelection` variant |
| `crates/routes_app/src/setup/routes_setup.rs` | Update `setup_show()` match arms, merge arms |
| **Backend tests** | |
| `crates/routes_app/src/setup/test_setup.rs` | ADD 4 multi-tenant unit tests for `setup_show()` |
| `crates/routes_app/tests/test_live_multi_tenant.rs` | Update 1 assertion + 1 comment |
| `crates/server_app/tests/test_live_multi_tenant.rs` | Update 3 assertions + 5 comments |
| **Generated** | |
| `openapi.json` | Regenerated (auto) |
| `ts-client/src/types/types.gen.ts` | Regenerated (auto) |
| **Frontend** | |
| `crates/bodhi/src/components/AppInitializer.tsx` | New routing logic for ready+multi_tenant, remove tenant_selection case |
| `crates/bodhi/src/app/ui/login/page.tsx` | Remove `tenant_selection` from allowedStatuses |
| **Frontend tests** | |
| `crates/bodhi/src/app/ui/login/page.test.tsx` | Update 3 mock statuses from tenant_selection → ready |
| `crates/bodhi/src/components/AppInitializer.test.tsx` | ADD 4 multi-tenant routing tests |
| **E2E tests** | |
| (no changes) | E2E tests check UI selectors, not API status strings |
| **Docs** | |
| `crates/services/CLAUDE.md` | Update AppStatus description |
| `crates/bodhi/src/CLAUDE.md` | Update AppInitializer routing docs |
