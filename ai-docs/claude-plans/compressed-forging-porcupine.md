# Invite Link Flow + Remove Add-User-by-Email

## Context

The "add user by email" feature poses an **email enumeration risk** in a shared Keycloak realm. The Keycloak SPI's `POST /resources/assign-role` only accepts `user_id` (UUID), not email. The replacement is a **shareable invite link** pattern: admin shares `https://<host>/ui/login/?invite=<client_id>`, user authenticates, then goes through access-request flow if they don't have a role.

**Key architectural findings:**
- `role: None` → `/ui/request-access/` redirect is already implemented in `AppInitializer` (`crates/bodhi/src/components/AppInitializer.tsx:91-93`)
- Dashboard OAuth callback always returns `location: "/ui/login"` (`crates/routes_app/src/tenants/routes_dashboard_auth.rs:229-231`)
- Resource OAuth callback always returns `location: "/ui/chat"` (`crates/routes_app/src/auth/routes_auth.rs:303-325`)
- Auth callback page checks `sessionStorage('bodhi-return-url')` and overrides redirect target (`crates/bodhi/src/app/ui/auth/callback/page.tsx:24-28`)
- `public_server_url()` already exists in `SettingService` (`crates/services/src/settings/setting_service.rs:401-409`)
- User-facing access-request endpoints are in `public_apis` group (no `api_auth_middleware`), accessible with `role: None`
- Session is cleared on access request approval, forcing re-auth with updated role

---

## Execution Plan

### Batch 1 — Sub-agent: Backend Rust Changes
**Type:** `general-purpose` | **Sequential phases, single agent**

#### Phase 1: Remove Add-User-by-Email from Rust Crates

Work upstream-to-downstream per layered methodology.

**Step 1a — `services` crate:**

| File | Remove |
|------|--------|
| `crates/services/src/users/user_objs.rs` | `AddUserRequest` struct, its `Validate` impl, `AssignRoleResponse` struct |
| `crates/services/src/users/auth_scoped.rs` | `add_user()` method + `AddUserRequest`/`AssignRoleResponse` imports |
| `crates/services/src/auth/auth_service.rs` | `add_user_to_role()` trait method + `KeycloakAuthService` impl (dead code) |

**Gate 1a:** `cargo test -p services --lib`

**Step 1b — `routes_app` crate:**

| File | Remove |
|------|--------|
| `crates/routes_app/src/users/routes_users.rs` | `users_add()` handler (with `#[utoipa::path]` attribute) |
| `crates/routes_app/src/users/error.rs` | `AddFailed(String)` variant from `UsersRouteError` |
| `crates/routes_app/src/routes.rs` | `users_add` import, route registration at `{ENDPOINT_USERS}/add` in `manager_session_apis` |
| `crates/routes_app/src/shared/openapi.rs` | `AddUserRequest`, `AssignRoleResponse` schema registrations, `__path_users_add` path, related imports |

**Gate 1b:** `cargo test -p routes_app`

#### Phase 2: Add `url: String` to AppInfo Response

| File | Change |
|------|--------|
| `crates/routes_app/src/setup/setup_api_schemas.rs` | Add `pub url: String` field to `AppInfo` struct with `#[schema(example = "https://example.com")]` |
| `crates/routes_app/src/setup/routes_setup.rs` | In `setup_show()`, add `url: settings.public_server_url().await` to `AppInfo` construction (line ~80-86) |

**Gate 2:** `cargo test -p routes_app -- setup`

#### Phase 2b: Regenerate OpenAPI + TypeScript types

```bash
cargo run --package xtask openapi
make build.ts-client
```

**Gate 2b:** `make ci.ts-client-check` (or verify `ts-client` compiles and tests pass)

---

### Batch 2 — Sub-agent: Frontend Removal + Invite Link UI
**Type:** `general-purpose` | **Sequential phases, single agent**

#### Phase 3: Remove Add-User-by-Email from Frontend

**Files to DELETE:**
- `crates/bodhi/src/app/ui/users/add/page.tsx`

**Files to EDIT:**

| File | Remove |
|------|--------|
| `crates/bodhi/src/hooks/useUsers.ts` | `useAddUser` hook, `ENDPOINT_USERS_ADD` constant, `AddUserRequest`/`AssignRoleResponse` imports from `@bodhiapp/ts-client` |
| `crates/bodhi/src/app/ui/users/page.tsx` | "Add User" button/Link to `/ui/users/add` (line ~81-86) |

**Gate 3:** `cd crates/bodhi && npm test`

#### Phase 4: Invite Link UI on Users Page (Multi-Tenant Only)

**File:** `crates/bodhi/src/app/ui/users/page.tsx`

**Changes:**
- Fetch `useAppInfo()` to get `deployment`, `url`, and `client_id`
- In multi-tenant mode (`appInfo.deployment === 'multi_tenant'`), add invite link section where "Add User" button was:
  - Read-only text input: `${appInfo.url}/ui/login/?invite=${appInfo.client_id}`
  - Copy-to-clipboard button (use `navigator.clipboard.writeText()`)
  - Helper text: "Share this link to invite users to your workspace"
- In standalone mode: no invite link section (no replacement for removed button)
- `data-testid` attributes: `invite-url-input`, `invite-copy-button`

**Gate 4:** `cd crates/bodhi && npm test`

---

### Batch 3 — Sub-agent: Login Page Invite Parameter Handling
**Type:** `general-purpose` | **Single phase, single agent**

#### Phase 5: Login Page `?invite=` Handling

**File:** `crates/bodhi/src/app/ui/login/page.tsx`

##### Changes to `MultiTenantLoginContent`:

**5a. Read invite parameter on mount:**
- Import `useSearchParams` from `next/navigation`
- Read `?invite=` query param
- If present, store as `sessionStorage.setItem('login_to_tenant', clientId)`
- Clear the query param from URL via `router.replace('/ui/login/')`

**5b. Process invite (takes priority over auto-login):**
- On mount / when data loads, check `sessionStorage.getItem('login_to_tenant')`
- If set, execute invite flow; clear from sessionStorage after consuming
- **Suppress auto-login** (`useEffect` at line 107-123) when `login_to_tenant` is in sessionStorage

**5c. Invite flow decision table:**

| Condition | Action |
|-----------|--------|
| No dashboard session | Keep `login_to_tenant` in sessionStorage (survive OAuth redirect), trigger `initiateDashboardOAuth()` |
| Dashboard session + target tenant in `tenantsData` with `logged_in: true` | `activateTenant()`, stay on `/ui/login/`, show toast "Already a member of this workspace" |
| Dashboard session + target tenant in `tenantsData` with `logged_in: false` | Set `sessionStorage('bodhi-return-url', '/ui/login/')`, call `initiateOAuth({ client_id })` |
| Dashboard session + target NOT in `tenantsData` (or tenants not loaded yet) | Set `sessionStorage('bodhi-return-url', '/ui/login/')`, call `initiateOAuth({ client_id })` |

**5d. role: None guard on login page:**
- Add check **before** State C render (before line 130):
  ```
  if (userInfo?.auth_status === 'logged_in' && appInfo?.client_id && !userInfo.role) {
    router.push(ROUTE_REQUEST_ACCESS);
    return null;
  }
  ```
- Handles: user returns to `/ui/login/` after invite-triggered OAuth with no role assigned

##### Changes to `LoginContent` (standalone):
- Read `?invite=` param but **ignore it** — proceed with normal fixed-client_id login

##### Implementation notes:
- Reuse `bodhi-return-url` mechanism from auth callback page (`crates/bodhi/src/app/ui/auth/callback/page.tsx:24-28`)
- Follow `mcpFormStore.ts` sessionStorage pattern: save before redirect, restore after callback, auto-cleanup
- Use `useRef` guard (like existing `hasAutoLoginTriggered`) to prevent double-processing in StrictMode

**Gate 5:** `cd crates/bodhi && npm test`

---

### Batch 4 — Sub-agent: Manual Browser Verification
**Type:** `general-purpose` | **Rebuild + manual testing via claude-in-chrome**

#### Phase 6: Build & Manual Verification

**6a. Rebuild:**
```bash
make build.ui-rebuild
```

**6b. Launch multi-tenant server and verify these flows:**

1. **New user invite flow**: Navigate to `/ui/login/?invite=<client_id>` → dashboard login → tenant OAuth → returns to `/ui/login/` → redirects to `/ui/request-access/`
2. **Already-member flow**: Navigate to invite link when already logged into that tenant → toast "Already a member of this workspace" + stay on login page
3. **Standalone ignore**: Navigate to `/ui/login/?invite=<anything>` in standalone mode → normal login (invite ignored)
4. **Normal login (no invite)**: Verify existing multi-tenant login flow unchanged (no regression)
5. **Invite URL copy**: Navigate to `/ui/users/` in multi-tenant mode → invite link visible → copy button works

**Note:** claude-in-chrome has a single browser context. For approval step, logout and login as the other user to test access-request approval.

**Gate 6:** All 5 flows verified manually

---

### Batch 5 — Sub-agent: E2E Test Updates
**Type:** `general-purpose` | **Sequential steps, single agent**

#### Phase 7: Delete Obsolete E2E Files

**Files to DELETE:**
- `crates/lib_bodhiserver_napi/tests-js/pages/AddUserPage.mjs`
- `crates/lib_bodhiserver_napi/tests-js/specs/users/add-user.spec.mjs`

#### Phase 8: Create New Page Objects

**New file: `crates/lib_bodhiserver_napi/tests-js/pages/RequestAccessPage.mjs`**
- Navigate to `/ui/request-access/`
- Submit access request
- Verify pending state
- Follow existing page object patterns (see `AddUserPage.mjs` before deletion, `MultiTenantLoginPage.mjs`)

**New file: `crates/lib_bodhiserver_napi/tests-js/pages/AccessRequestsPage.mjs`**
- Navigate to access requests management page
- Find pending request by username
- Approve/reject request
- Follow existing page object patterns

#### Phase 9: Update Lifecycle Test

**File:** `crates/lib_bodhiserver_napi/tests-js/specs/multi-tenant/multi-tenant-lifecycle.spec.mjs`

**Old Steps 5-8:** User adds manager by email → manager switches tenant → verify role → data isolation

**New flow replacing Steps 5-8:**

| Step | Actor | Action |
|------|-------|--------|
| 5 | Manager (existing context) | Navigate to invite URL: `${baseUrl}/ui/login/?invite=${userClientId}` (client_id from `.env.test`) |
| 6 | Manager | Dashboard OAuth → tenant OAuth → lands on `/ui/request-access/` |
| 7 | Manager | Submit access request |
| 8 | **User (new BrowserContext)** | Login → navigate to access request management → approve Manager's request |
| 9 | Manager (back to original context) | Session was cleared on approval → re-auth triggers |
| 10 | Manager | Now has 2 tenants → switch to User's tenant |
| 11 | Manager | Verify role assignment |
| 12 | Manager | Data isolation verification (Manager's API model NOT visible in User's tenant) |
| 13 | Both | Logout |

**Infrastructure:**
- Use separate Playwright `browser.newContext()` for User approval (isolated cookies/session)
- User client_id from `.env.test` (fixed, pre-seeded)

**Gate 9:** `make build.ui-rebuild && make test.napi` (multi-tenant tests pass)

---

### Batch 6 — Sub-agent: Final Regression
**Type:** `general-purpose` | **Final regeneration + full test suite**

#### Phase 10: Final Regeneration & Full Test

```bash
cargo run --package xtask openapi
make build.ts-client
make build.ui-rebuild
```

**Gate 10 — Full regression:**
1. `cargo test -p services --lib` — no references to removed types
2. `cargo test -p routes_app` — all pass including openapi tests
3. `cd crates/bodhi && npm test` — component tests pass
4. `make test.napi` — E2E tests pass
5. `make test` — full regression

---

## Files Summary

### Delete (3):
- `crates/bodhi/src/app/ui/users/add/page.tsx`
- `crates/lib_bodhiserver_napi/tests-js/pages/AddUserPage.mjs`
- `crates/lib_bodhiserver_napi/tests-js/specs/users/add-user.spec.mjs`

### Modify — Backend (9):
- `crates/services/src/users/user_objs.rs` — remove `AddUserRequest`, `AssignRoleResponse`
- `crates/services/src/users/auth_scoped.rs` — remove `add_user()` method
- `crates/services/src/auth/auth_service.rs` — remove `add_user_to_role()` trait + impl
- `crates/routes_app/src/users/routes_users.rs` — remove `users_add()` handler
- `crates/routes_app/src/users/error.rs` — remove `AddFailed` variant
- `crates/routes_app/src/routes.rs` — remove `users_add` route registration
- `crates/routes_app/src/shared/openapi.rs` — remove schemas/paths for add-user
- `crates/routes_app/src/setup/setup_api_schemas.rs` — add `url: String` field
- `crates/routes_app/src/setup/routes_setup.rs` — add `url` to response

### Modify — Frontend (3):
- `crates/bodhi/src/hooks/useUsers.ts` — remove `useAddUser` hook
- `crates/bodhi/src/app/ui/users/page.tsx` — remove add button, add invite link UI
- `crates/bodhi/src/app/ui/login/page.tsx` — add invite parameter handling

### Modify — E2E (1):
- `crates/lib_bodhiserver_napi/tests-js/specs/multi-tenant/multi-tenant-lifecycle.spec.mjs`

### New — E2E page objects (2):
- `crates/lib_bodhiserver_napi/tests-js/pages/RequestAccessPage.mjs`
- `crates/lib_bodhiserver_napi/tests-js/pages/AccessRequestsPage.mjs`

### Auto-generated (3):
- `openapi.json`
- `ts-client/src/openapi-typescript/openapi-schema.ts`
- `ts-client/src/types/types.gen.ts`
