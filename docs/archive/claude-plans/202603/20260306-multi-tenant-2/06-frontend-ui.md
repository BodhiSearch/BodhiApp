# Frontend UI for Multi-Tenant

## Overview

The BodhiApp frontend adapts between standalone and multi-tenant modes using the `deployment` field from the `/bodhi/v1/info` endpoint (see [01-deployment-modes-and-status.md](01-deployment-modes-and-status.md#get-bodhiv1info-endpoint)). In standalone mode, the UI follows the existing single-tenant OAuth flow (setup wizard, single login button). In multi-tenant mode, the UI implements a two-phase login experience: dashboard OAuth for platform identity, then tenant selection/creation and resource-client OAuth for workspace access. The `AppInitializer` component orchestrates routing based on `AppStatus` and deployment mode, while the login page (`/ui/login`) serves as a multi-purpose hub for platform login, tenant selection, tenant switching, and authenticated state display.

For the backend auth flows that these UI components interact with, see [02-auth-sessions-middleware.md](02-auth-sessions-middleware.md#multi-tenant-auth-flow-two-phase). For tenant management endpoints, see [03-tenant-management-and-spi.md](03-tenant-management-and-spi.md#tenant-crud-endpoints).

## Functional Behavior

### AppInitializer Routing Logic

`AppInitializer` reads `GET /bodhi/v1/info` and routes based on `status` and `deployment`:

| AppStatus | Deployment | Route Target |
|-----------|-----------|--------------|
| `setup` | `standalone` | `/ui/setup` (setup wizard) |
| `setup` | `multi_tenant` | `/ui/setup/tenants` (tenant registration) |
| `resource_admin` | any | `/ui/setup/resource-admin` |
| `tenant_selection` | any | `/ui/login` |
| `ready` | any | `/ui/chat` (default) |

The `allowedStatus` prop accepts `AppStatus | AppStatus[]`. When the current status matches an allowed value, the component renders its children. Otherwise it redirects to the status-appropriate route.

### Login Page State Machine (Multi-Tenant)

The login page (`/ui/login`) renders `MultiTenantLoginContent` when `appInfo.deployment === 'multi_tenant'`, otherwise `LoginContent` (standalone). The multi-tenant component has four visual states:

**State A -- No dashboard session** (`has_dashboard_session` is falsy or user not loaded):
- Shows "Login to Bodhi Platform" button
- Clicking triggers `POST /bodhi/v1/auth/dashboard/initiate`
- On success, redirects to Keycloak OAuth login page

**State B1 -- Dashboard session, 1 tenant, auto-login triggered**:
- `useEffect` with `useRef` guard auto-initiates resource OAuth or tenant activation
- If the single tenant has `logged_in: true`, calls `POST /tenants/{client_id}/activate`
- If `logged_in: false`, calls `POST /auth/initiate { client_id }`
- On auto-login failure, shows manual "Connect to [name]" button with error message

**State B2 -- Dashboard session, N tenants**:
- Shows "Select Workspace" card with one button per tenant
- Each button calls `activateTenant` (if `logged_in`) or `initiateOAuth` (if not)
- Displays connecting state per selected tenant

**State C -- Fully authenticated** (resource token active, `appInfo.client_id` present):
- Shows "Welcome" card with username and active workspace name
- Lists "Switch to [name]" buttons for other tenants
- Shows "Log Out" button (destroys entire session)

### Login Page (Standalone)

`LoginContent` renders a simpler flow:
- **Not logged in**: "Login" button that reads `client_id` from `/info` response, calls `POST /auth/initiate { client_id }`
- **Logged in**: Welcome message with "Go to Home" and "Log Out" buttons
- Validates `client_id` is present before initiating OAuth; shows error if missing

### Dashboard Callback Page

Route: `/ui/auth/dashboard/callback`

- Wrapped in `<Suspense>` for Next.js App Router `useSearchParams` compatibility
- `useRef` guard prevents duplicate submissions in React StrictMode
- Extracts `code` and `state` from URL search params
- Calls `POST /bodhi/v1/auth/dashboard/callback { code, state }`
- On success: redirects to `/ui/login` (via `router.push` or `window.location.href`)
- On error: shows error alert with "Try Again" button linking to `/ui/login`

### Resource Callback Page

Route: `/ui/auth/callback` (unchanged from standalone, shared by both modes)

- Same `useRef` guard and `<Suspense>` pattern
- Extracts all URL search params and passes to `POST /bodhi/v1/auth/callback`
- On success: checks `sessionStorage['bodhi-return-url']` first, otherwise redirects to location from response (typically `/ui/chat`)

### Tenant Registration Page

Route: `/ui/setup/tenants`

- Wrapped in `<AppInitializer allowedStatus="setup">` -- only reachable when `/info` returns `setup` status (user has dashboard session but zero tenants)
- Form: Workspace Name (required, min 3 chars) and Description (optional)
- Calls `POST /bodhi/v1/tenants { name, description }` via `useCreateTenant`
- On success: auto-initiates resource OAuth with returned `client_id` via `useOAuthInitiate`
- Shows "Setting up your workspace..." loading state during redirect

### LoginMenu Component

Standalone-only sidebar login/logout widget. Reads `client_id` from `/info` response for OAuth initiation. Not used in multi-tenant mode (multi-tenant login is handled entirely on the `/ui/login` page).

### Invite Link on Users Page (Multi-Tenant Only)

In multi-tenant mode, the users management page (`/ui/users/page.tsx`) displays a shareable invite link section:
- Read-only text input showing: `${appInfo.url}/ui/login/?invite=${appInfo.client_id}`
- Copy-to-clipboard button (`navigator.clipboard.writeText()`)
- Helper text: "Share this link to invite users to your workspace"
- `data-testid` attributes: `invite-url-input`, `invite-copy-button`
- Uses `useAppInfo()` to get `url`, `client_id`, and `deployment`
- In standalone mode: no invite link (users go through access-request flow directly)

### Invite Link Handling on Login Page

The login page handles `?invite=<client_id>` query parameter for invited users.

**On mount**: If `?invite=` is present, store `client_id` in `sessionStorage` as `login_to_tenant`, clear the query param from URL via `router.replace('/ui/login/')`.

**Invite flow decision table** (takes priority over auto-login):

| Condition | Action |
|-----------|--------|
| No dashboard session | Keep `login_to_tenant` in sessionStorage (survives OAuth redirect), trigger `initiateDashboardOAuth()` |
| Dashboard session + target in tenantsData with `logged_in: true` | `activateTenant()`, show toast "Already a member of this workspace" |
| Dashboard session + target in tenantsData with `logged_in: false` | Set `sessionStorage('bodhi-return-url', '/ui/login/')`, call `initiateOAuth({ client_id })` |
| Dashboard session + target NOT in tenantsData | Set `sessionStorage('bodhi-return-url', '/ui/login/')`, call `initiateOAuth({ client_id })` |

**role:None guard**: After resource OAuth returns the user to `/ui/login`, if `userInfo.auth_status === 'logged_in'` and `!userInfo.role`, redirect to `/ui/request-access`. This handles invited users who have no role yet.

**Standalone**: `LoginContent` reads `?invite=` param but ignores it -- normal fixed-client_id login proceeds.

**sessionStorage pattern**: Follows `mcpFormStore.ts` pattern -- save before redirect, restore after callback, auto-cleanup. Uses `useRef` guard (like existing `hasAutoLoginTriggered`) to prevent double-processing in StrictMode.

## Architecture & Data Model

### Component Tree

```
LoginPage
  AppInitializer(allowedStatus=['ready', 'tenant_selection'], authenticated=false)
    if deployment === 'multi_tenant':
      MultiTenantLoginContent
        useAppInfo()          -- GET /info
        useUser()             -- GET /user/info (has_dashboard_session)
        useTenants()          -- GET /tenants (when needsTenantSelection)
        useSearchParams()     -- reads ?invite= query param
        sessionStorage        -- login_to_tenant persistence across OAuth redirects
        useDashboardOAuthInitiate()
        useOAuthInitiate()
        useTenantActivate()
        useLogoutHandler()
    else:
      LoginContent
        useAppInfo()          -- GET /info (for client_id)
        useUser()
        useOAuthInitiate()
        useLogoutHandler()

DashboardCallbackPage
  Suspense
    DashboardCallbackContent
      useDashboardOAuthCallback()

TenantRegistrationPage
  AppInitializer(allowedStatus='setup', authenticated=false)
    TenantRegistrationContent
      useCreateTenant()
      useOAuthInitiate()
```

### TypeScript Types (from `@bodhiapp/ts-client`)

| Type | Fields | Usage |
|------|--------|-------|
| `AppInfo` | `status: AppStatus`, `deployment: string`, `client_id?: string`, `url: string`, `version`, `commit_sha` | Boot status and deployment mode detection. `url` comes from `settings.public_server_url()` |
| `AppStatus` | `'setup' \| 'ready' \| 'resource_admin' \| 'tenant_selection'` | Routing decisions in AppInitializer |
| `TenantListResponse` | `{ tenants: TenantListItem[] }` | Tenant selector rendering |
| `TenantListItem` | `{ client_id, name, description?, is_active, logged_in }` | Per-tenant state in selector UI |
| `CreateTenantRequest` | `{ name: string, description: string }` | Tenant registration form |
| `CreateTenantResponse` | `{ client_id: string }` | Auto-OAuth after creation |
| `AuthInitiateRequest` | `{ client_id: string }` | Unified resource-client OAuth initiation |
| `AuthCallbackRequest` | `{ code?, state?, error?, error_description? }` | Shared by both resource and dashboard callbacks |
| `RedirectResponse` | `{ location: string }` | All auth endpoints return redirect location |
| `UserInfo` | `{ auth_status, username, role, has_dashboard_session?, ... }` | Dashboard session detection |

### React Hooks

| Hook | File | API | Query Key | Notes |
|------|------|-----|-----------|-------|
| `useAppInfo()` | `hooks/useInfo.ts` | `GET /bodhi/v1/info` | `appInfo` | Returns `AppInfo` with `deployment` and `client_id` |
| `useOAuthInitiate()` | `hooks/useAuth.ts` | `POST /bodhi/v1/auth/initiate` | mutation | Accepts `{ client_id }` variable |
| `useOAuthCallback()` | `hooks/useAuth.ts` | `POST /bodhi/v1/auth/callback` | mutation | Reused for resource-client callback |
| `useDashboardOAuthInitiate()` | `hooks/useAuth.ts` | `POST /bodhi/v1/auth/dashboard/initiate` | mutation | No request body (void) |
| `useDashboardOAuthCallback()` | `hooks/useAuth.ts` | `POST /bodhi/v1/auth/dashboard/callback` | mutation | Sends `{ code, state }` |
| `useTenants()` | `hooks/useTenants.ts` | `GET /bodhi/v1/tenants` | `tenants` | Accepts `{ enabled? }` to defer fetching |
| `useCreateTenant()` | `hooks/useTenants.ts` | `POST /bodhi/v1/tenants` | mutation | Invalidates `tenants` on success |
| `useTenantActivate()` | `hooks/useTenants.ts` | `POST /tenants/{client_id}/activate` | mutation | Dynamic URL via function. Invalidates `tenants`, `appInfo`, `user` on success |
| `useLogoutHandler()` | `hooks/useAuth.ts` | `POST /bodhi/v1/logout` | mutation | Invalidates all queries. Wraps `useLogout()` |

### Data Flow: Multi-Tenant Boot Sequence

```
1. AppInitializer mounts
   GET /info -> { status: 'tenant_selection', deployment: 'multi_tenant', client_id: null }
   -> allowedStatus includes 'tenant_selection'? Yes on login page -> render children

2. LoginPage checks deployment
   appInfo.deployment === 'multi_tenant' -> render MultiTenantLoginContent

3. MultiTenantLoginContent mounts
   GET /user/info -> { auth_status: 'logged_out' } or { has_dashboard_session: false }
   -> State A: show "Login to Bodhi Platform" button

4. User clicks -> POST /auth/dashboard/initiate
   -> RedirectResponse { location: keycloak_url } -> window.location.href = keycloak_url

5. Keycloak login -> redirect to /ui/auth/dashboard/callback?code=X&state=Y

6. DashboardCallbackContent
   POST /auth/dashboard/callback { code, state }
   -> RedirectResponse { location: '/ui/login' } -> router.push('/ui/login')

7. LoginPage re-mounts
   GET /user/info -> { has_dashboard_session: true }
   needsTenantSelection = true
   GET /tenants -> { tenants: [...] }

8a. 0 tenants: /info returns status=setup -> AppInitializer redirects to /ui/setup/tenants
8b. 1 tenant: auto-login useEffect fires -> POST /auth/initiate or POST /tenants/{id}/activate
8c. N tenants: show selector -> user clicks -> POST /auth/initiate or activate

9. After resource OAuth callback:
   GET /info -> { status: 'ready', client_id: 'xxx' } -> app renders normally
```

### Route Constants

Defined in `crates/bodhi/src/lib/constants.ts`:

| Constant | Value | Purpose |
|----------|-------|---------|
| `ROUTE_SETUP_TENANTS` | `/ui/setup/tenants` | Tenant registration page |
| `ROUTE_DASHBOARD_CALLBACK` | `/ui/auth/dashboard/callback` | Dashboard OAuth callback |
| `ROUTE_LOGIN` | `/ui/login` | Login / tenant selection |
| `ROUTE_DEFAULT` | `/ui/chat` | Post-auth landing page |
| `ROUTE_SETUP` | `/ui/setup` | Standalone setup wizard |
| `ROUTE_REQUEST_ACCESS` | `/ui/request-access` | Access request page for users with no role |

### API Endpoint Constants

Defined in `crates/bodhi/src/hooks/useAuth.ts`:

| Constant | Value |
|----------|-------|
| `ENDPOINT_AUTH_INITIATE` | `/bodhi/v1/auth/initiate` |
| `ENDPOINT_AUTH_CALLBACK` | `/bodhi/v1/auth/callback` |
| `ENDPOINT_DASHBOARD_AUTH_INITIATE` | `/bodhi/v1/auth/dashboard/initiate` |
| `ENDPOINT_DASHBOARD_AUTH_CALLBACK` | `/bodhi/v1/auth/dashboard/callback` |
| `ENDPOINT_TENANTS` | `/bodhi/v1/tenants` (in `useTenants.ts`) |

## Technical Implementation

### Key Files

| File | Purpose |
|------|---------|
| `crates/bodhi/src/components/AppInitializer.tsx` | Boot routing: `allowedStatus` array support, `tenant_selection` -> `/ui/login`, `setup` + `multi_tenant` -> `/ui/setup/tenants` |
| `crates/bodhi/src/app/ui/login/page.tsx` | Login page: `LoginPage` (top-level), `LoginContent` (standalone), `MultiTenantLoginContent` (4-state multi-tenant flow) |
| `crates/bodhi/src/app/ui/auth/dashboard/callback/page.tsx` | Dashboard OAuth callback: Suspense + useRef guard, extracts code/state, calls dashboard callback API |
| `crates/bodhi/src/app/ui/auth/callback/page.tsx` | Resource OAuth callback: shared by both modes, Suspense + useRef guard |
| `crates/bodhi/src/app/ui/setup/tenants/page.tsx` | Tenant registration: name/description form, auto-OAuth on success |
| `crates/bodhi/src/app/ui/setup/resource-admin/page.tsx` | Resource admin setup: updated to read `client_id` from `/info` for OAuth initiation |
| `crates/bodhi/src/hooks/useAuth.ts` | Auth hooks: `useOAuthInitiate` (now takes `{ client_id }`), `useDashboardOAuthInitiate`, `useDashboardOAuthCallback`, `useOAuthCallback`, `useLogoutHandler` |
| `crates/bodhi/src/hooks/useTenants.ts` | Tenant hooks: `useTenants`, `useCreateTenant`, `useTenantActivate` |
| `crates/bodhi/src/hooks/useInfo.ts` | `useAppInfo` returning `AppInfo` (with `deployment`, `client_id`) |
| `crates/bodhi/src/lib/constants.ts` | Route constants: `ROUTE_SETUP_TENANTS`, `ROUTE_DASHBOARD_CALLBACK` |
| `crates/bodhi/src/components/AuthCard.tsx` | Reusable auth card with title, description, action buttons |
| `crates/bodhi/src/components/LoginMenu.tsx` | Sidebar login widget (standalone only, reads `client_id` from `/info`) |
| `crates/bodhi/src/app/ui/users/page.tsx` | Users management page with invite link UI (multi-tenant: shareable invite URL with copy button) |
| `crates/bodhi/src/test-utils/msw-v2/handlers/info.ts` | MSW handler: `mockAppInfo()` accepts `deployment` and `client_id` params |
| `crates/bodhi/src/test-utils/msw-v2/handlers/user.ts` | MSW handler: `mockUserLoggedIn()` accepts spread params including `has_dashboard_session` |

### Test Files

| File | Coverage |
|------|----------|
| `crates/bodhi/src/components/AppInitializer.test.tsx` | Status routing, role-based access, auth checks. Standalone statuses only -- does NOT test multi-tenant routing |
| `crates/bodhi/src/app/ui/login/page.test.tsx` | Standalone `LoginContent` only: loading, login flow, logout, OAuth redirect, error handling. Does NOT test `MultiTenantLoginContent` |
| `crates/bodhi/src/components/LoginMenu.test.tsx` | Standalone LoginMenu with `client_id` |
| `crates/bodhi/src/hooks/useAuth.test.tsx` | Updated for `client_id` parameter in `useOAuthInitiate`. Dashboard hooks NOT tested |

For full test gap analysis, see [07-testing-infrastructure.md](07-testing-infrastructure.md#known-gaps--techdebt) (Gaps 1-6).

## Decisions

Decisions are referenced by ID. See [08-decisions-index.md](08-decisions-index.md) for the canonical decision table with full descriptions.

| ID | Title | Status |
|----|-------|--------|
| D30 | Auto-redirect for single tenant | Implemented |
| D38 | No mandatory wizard for multi-tenant | Implemented |
| D39 | Login page reused for tenant selection | Implemented |
| D47 | Dashboard callback redirects to /ui/login | Implemented |
| D48 | Registration UI at /ui/setup/tenants | Implemented |
| D67 | Frontend uses deployment field | Implemented |
| D68 | client_id required in OAuth initiate | Implemented |
| D77 | Separate frontend callback routes | Implemented |
| D79 | Tenant created Ready immediately | Implemented |
| D84 | No role in tenant dropdown | Implemented |
| F1 | `/info` status drives tenant flow | Implemented |
| F2 | Single-tenant auto-login in frontend | Implemented |
| F4 | allowedStatus accepts arrays | Implemented |
| F5 | Tenant registration at separate page | Implemented |
| F9 | Single logout clears everything | Implemented |

## Known Gaps & TECHDEBT

1. **No multi-tenant tests for AppInitializer**: `AppInitializer.test.tsx` does not test `tenant_selection` status routing or the `setup` + `deployment=multi_tenant` -> `/ui/setup/tenants` routing. Only standalone statuses are covered.

2. **No tests for MultiTenantLoginContent**: `page.test.tsx` only tests `LoginContent` (standalone). The 4-state multi-tenant component has zero unit test coverage: no dashboard login button test, no auto-login test, no tenant selector test, no fully-authenticated state test. The invite link (`?invite=`) flow, including sessionStorage persistence, invite decision table logic, and role:None redirect to `/ui/request-access`, is also untested.

3. **No tests for DashboardCallbackPage**: No test file for `crates/bodhi/src/app/ui/auth/dashboard/callback/page.tsx`. Suspense wrapper, code/state extraction, error handling, and redirect behavior are untested.

4. **No tests for TenantRegistrationPage**: No test file for `crates/bodhi/src/app/ui/setup/tenants/page.tsx`. Form validation, `useCreateTenant` integration, auto-OAuth on success, and error display are untested.

5. **No tests for tenant hooks**: `useTenants.ts` hooks have no test file. No MSW handler file for tenant endpoints exists.

6. **No tests for dashboard auth hooks**: `useDashboardOAuthInitiate` and `useDashboardOAuthCallback` have no test coverage.

7. **Navigation item visibility not deployment-aware**: The sidebar navigation does not hide LLM-specific items (Models, Downloads, LLM settings) in multi-tenant mode, even though `deployment` is available from `/info`.

8. **LoginMenu is standalone-only**: `LoginMenu.tsx` does not handle multi-tenant mode. If rendered in MT mode it would attempt OAuth without dashboard authentication. Currently not a problem because the sidebar login widget is not shown on the dedicated login page.

9. **Tenant description optional mismatch**: `CreateTenantRequest` in the TypeScript types defines `description` as required, but the registration form's label says "(optional)". The backend validates description as required (min 1 char), so submitting an empty description would fail server-side.

10. **Client-side validation inconsistency**: The tenant registration form validates `name.length < 3` client-side but the backend `CreateTenantRequest` validates `min 1, max 255`. The server-side constraint is more permissive.

11. **E2E/Playwright tests for multi-tenant**: No Playwright E2E tests exist for the multi-tenant frontend flows. See [07-testing-infrastructure.md](07-testing-infrastructure.md#known-gaps--techdebt) (Gap 7).

12. **No tests for invite link UI on users page**: The invite link section in `/ui/users/page.tsx` (read-only URL input, copy button, conditional rendering by deployment mode) has no unit test coverage.

13. **No tests for `?invite=` query parameter handling on login page**: The invite link flow on the login page -- sessionStorage persistence of `login_to_tenant`, invite decision table, role:None guard redirect, and cleanup -- has no unit test coverage.
