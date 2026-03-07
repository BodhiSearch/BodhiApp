# E2E Standalone Test Fixes — Kickoff

> **Created**: 2026-03-09
> **Status**: TODO
> **Scope**: Fix pre-existing E2E test failures on standalone project
> **Depends on**: `kickoff-e2e-multi-tenant.md` (Phase 1+2 — **COMPLETED**)

---

## Problem

Several E2E tests using `createServerManager()` (dedicated standalone servers) fail on **both** `standalone` and `multi_tenant` projects. The symptom is identical: `performOAuthLogin()` clicks the "Login" button but the OAuth redirect to Keycloak never happens — the page stays on `/ui/login`.

### Affected tests (all pre-existing failures)

| Spec | Tests | Uses |
|------|-------|------|
| `users/list-users.spec.mjs` | 1 | `createServerManager()` + `performOAuthLogin()` |
| `request-access/multi-user-request-approval-flow.spec.mjs` | 1 | `createServerManager()` + `performOAuthLogin()` |
| `toolsets/toolsets-auth-restrictions.spec.mjs` | 6 | `createServerManager()` + dedicated server |
| `mcps/mcps-auth-restrictions.spec.mjs` | 3 | `createServerManager()` + dedicated server |
| `mcps/mcps-header-auth.spec.mjs` | 3 | `createServerManager()` + dedicated server |
| `mcps/mcps-oauth-auth.spec.mjs` | 4 | `createServerManager()` + dedicated server |
| `mcps/mcps-oauth-dcr.spec.mjs` | 3 | `createServerManager()` + dedicated server |
| `oauth/oauth-chat-streaming.spec.mjs` | 1 | `createServerManager()` + dedicated server |
| `oauth/oauth2-token-exchange.spec.mjs` | 2 | `createServerManager()` + dedicated server |

Note: Tests using the **shared server** (started by `start-shared-server.mjs`) work fine — the issue is specific to `createServerManager()` dedicated servers.

---

## Root Cause Analysis

### The login flow

```
1. Test clicks "Login" button
2. Frontend calls POST /bodhi/v1/auth/initiate { client_id: appInfo.client_id }
3. Backend looks up tenant by client_id
4. Backend builds OAuth authorization URL
5. Frontend redirects browser to Keycloak
6. Test waits for URL to match authServerConfig.authUrl origin
```

### Where it breaks

The flow breaks at **step 2 or 3**. The API call either:
- **Silently fails**: `client_id` from `/bodhi/v1/info` is undefined/wrong, so `handleOAuthInitiate()` doesn't call `initiateOAuth()`:
  ```javascript
  const handleOAuthInitiate = () => {
    if (appInfo?.client_id) {  // ← guards the call
      initiateOAuth({ client_id: appInfo.client_id });
    }
  };
  ```
- **Returns error**: `auth_initiate` handler can't find the tenant by `client_id` → `AuthRouteError::TenantNotFound`
- **Returns wrong location**: If user is already authenticated (stale session), handler returns home URL instead of OAuth URL

### Likely root cause

The `createServerManager()` flow:
1. `createFullTestConfig()` builds `NapiAppOptions` with `clientId`/`clientSecret` via `setClientCredentials()`
2. Server starts via `startServer()` which calls NAPI `startServer(config)`
3. In Rust: `try_build_app_options_internal()` creates a `Tenant` from credentials
4. `ensure_tenant()` inserts it into the DB

**Hypothesis**: Either the tenant isn't being persisted to the database before the test navigates, or the `/info` endpoint returns `client_id: null` for the dedicated server's configuration, causing the frontend guard `if (appInfo?.client_id)` to skip the API call entirely.

---

## Investigation Steps

### Step 1: Check `/info` response on dedicated server

Start a dedicated server and check what `/bodhi/v1/info` returns:

```bash
cd crates/lib_bodhiserver_napi
# Add console.log in the test to print the info response before login attempt
```

Or use Claude in Chrome:
1. Start a test that uses `createServerManager()` with a breakpoint/pause
2. Navigate to `http://localhost:<port>/bodhi/v1/info`
3. Check if `client_id` is present in the response

### Step 2: Check API call in browser

Use the Playwright test's failed screenshot/video or add `page.on('response')` logging:

```javascript
page.on('response', response => {
  if (response.url().includes('/auth/initiate')) {
    console.log('Auth initiate response:', response.status(), response.url());
  }
});
```

### Step 3: Check server logs

The dedicated servers log to stdout. Look for `TenantNotFound` or other errors when the test clicks "Login".

### Step 4: Verify tenant persistence

Check if `ensure_tenant()` is being called and if the tenant is actually in the DB when the test navigates:

```javascript
// In test, after server starts:
const response = await fetch(`${baseUrl}/dev/secrets`);
console.log('Server state:', await response.json());
```

---

## Likely Fix Approaches

### If `/info` doesn't return `client_id`

The `/info` endpoint may need authentication context to return `client_id` (it was moved behind `optional_auth_middleware` in M5). For unauthenticated users, it may not include `client_id`. The standalone `LoginContent` component depends on `appInfo?.client_id` being set.

**Fix**: Check if the `/info` endpoint returns `client_id` for standalone mode when unauthenticated. If not, the login page needs to handle this case differently (e.g., use a hardcoded endpoint or fetch client_id from a different source).

### If `auth_initiate` returns `TenantNotFound`

The tenant may not be inserted by the time the test navigates.

**Fix**: Ensure `startServer()` awaits `ensure_tenant()` completion before returning.

### If stale Keycloak session causes issues

Previous test runs may leave Keycloak cookies that interfere.

**Fix**: Use fresh browser contexts (`browser.newContext()`) for each test — most tests already do this.

---

## Files to Investigate

### Test infrastructure
- `tests-js/utils/bodhi-app-server.mjs` — `createServerManager()`, `BodhiAppServer` class
- `tests-js/test-helpers.mjs` — `createFullTestConfig()`, `createTestServer()`

### Frontend
- `crates/bodhi/src/app/ui/login/page.tsx` — `LoginContent` component, `handleOAuthInitiate`
- `crates/bodhi/src/hooks/useAuth.ts` — `useOAuthInitiate` hook
- `crates/bodhi/src/hooks/useInfo.ts` — `useAppInfo` hook, `/info` endpoint

### Backend
- `crates/routes_app/src/auth/routes_auth.rs` — `auth_initiate` handler
- `crates/routes_app/src/apps/routes_apps.rs` — `/info` endpoint
- `crates/lib_bodhiserver_napi/src/server.rs` — server startup, `ensure_tenant()`
- `crates/lib_bodhiserver_napi/src/config.rs` — `try_build_app_options_internal()`

### Page objects
- `tests-js/pages/LoginPage.mjs` — `performOAuthLogin()`
- `tests-js/pages/BasePage.mjs` — `waitForSPAReady()`

---

## Execution Plan

```
Step 1: Reproduce with a single failing test (list-users.spec.mjs)
   ↓
Step 2: Add logging to identify where the flow breaks
   ↓
Step 3: Fix the root cause
   ↓
Step 4: Verify fix on all affected tests
   ↓
Step 5: Verify no regressions on passing tests
```

Start with `list-users.spec.mjs` as the simplest reproduction case — it creates a dedicated server and immediately tries to login.
