---
name: Fix OAuth E2E Test
overview: Add thorough component test coverage for MCP OAuth flows, fix two frontend bugs (handleOAuthAuthorize condition + disconnect error handling), add data-test-state attributes, then fix E2E tests to remove all hacky patterns.
todos:
  - id: component-tests-write
    content: Write comprehensive component tests for MCP OAuth flows in page.test.tsx (13 new test cases across 5 describe blocks)
    status: completed
  - id: component-tests-baseline
    content: Run component tests to get baseline - expect ~2 tests to fail due to known bugs
    status: completed
  - id: fix-frontend-bugs
    content: Fix handleOAuthAuthorize condition (no configs = new config), fix disconnect onError to also call store.disconnect()
    status: completed
  - id: add-data-test-state
    content: Add data-test-state to oauth-auto-detect button, oauth-authorize button, oauth-fields-section in page.tsx
    status: completed
  - id: component-tests-pass
    content: Run component tests again - all should pass after fixes
    status: completed
  - id: fix-page-object
    content: "Fix McpsPage.mjs: URL-based assertions, split selectNewOAuthConfig into 3 methods, remove inline timeouts, add data-test-state wait methods"
    status: completed
  - id: rewrite-e2e-tests
    content: "Rewrite all 3 E2E tests: test 1 UI-driven with playground, test 2 UI setup + external app, remove test 3"
    status: completed
  - id: build-and-run
    content: Rebuild UI (make build.ui-rebuild), run E2E tests, fix any issues, format code
    status: completed
isProject: false
---

# Fix MCP OAuth Component Tests, Frontend Bugs, and E2E Tests

## Phase 1: Component Test Coverage (test-first)

### Root Cause Analysis: Two Frontend Bugs

**Bug 1: handleOAuthAuthorize skips config creation when no configs exist**

In `[page.tsx](crates/bodhi/src/app/ui/mcps/new/page.tsx)`, when no OAuth configs exist for a server:

- Form fields are shown directly (no dropdown) at line 1010: `store.isNewOAuthConfig || oauthConfigsList.length === 0 || editId`
- But `isNewOAuthConfig` remains `false` (set to `false` on server change at line 470)
- `handleOAuthAuthorize` (line 536) checks `store.isNewOAuthConfig` which is `false`, skips creation, finds no `selectedOAuthConfigId`, shows toast "No OAuth configuration selected"

**Fix**: Change the condition in `handleOAuthAuthorize` (line 536) from:

```javascript
if (!oauthConfigId && store.isNewOAuthConfig) {
```

to:

```javascript
if (!oauthConfigId && (store.isNewOAuthConfig || oauthConfigsList.length === 0)) {
```

**Bug 2: Disconnect error handler doesn't clear local state**

In `[page.tsx](crates/bodhi/src/app/ui/mcps/new/page.tsx)` lines 285-293, `deleteOAuthTokenMutation` `onError` only shows a toast but does NOT call `store.disconnect()`. User expects local state to be cleared even on API failure.

**Fix**: Add `store.disconnect()` to the `onError` handler at line 291.

### Existing Test Coverage (no duplication)

File: `[page.test.tsx](crates/bodhi/src/app/ui/mcps/new/page.test.tsx)`

Already covered (17 tests):

- Create flow: render, buttons, tools, tool fetch error, create MCP POST
- Edit flow: load existing, update button
- Auth type selector: public default, show/hide header fields, show/hide OAuth fields
- Bearer warning: Authorization header without Bearer, with Bearer, non-Authorization header
- Edit with header auth: loads auth fields, auth type state, placeholder, visibility toggle
- Edit with public auth: auth type state
- OAuth type: shows OAuth fields, hides on switch back, auto-detect endpoints
- Edit with OAuth: loads OAuth type, shows connected card

### New Component Tests to Add

File: `[page.test.tsx](crates/bodhi/src/app/ui/mcps/new/page.test.tsx)`

**MSW setup**: Use request capture pattern for verifying API calls:

```typescript
let capturedBody: any;
server.use(http.post(url, async ({ request }) => {
  capturedBody = await request.json();
  return HttpResponse.json(response, { status: 201 });
}));
```

For `window.location.href` redirect, use existing project pattern from `[review/page.test.tsx](crates/bodhi/src/app/ui/apps/access-requests/review/page.test.tsx)`:

```typescript
const setupWindowLocation = () => {
  originalLocationDescriptor = Object.getOwnPropertyDescriptor(window, 'location');
  const loc = window.location;
  Object.defineProperty(window, 'location', {
    value: { ...loc }, writable: true, configurable: true,
  });
};
```

#### describe: "OAuth authorize - no existing configs"

MSW: `mockListOAuthConfigs({ oauth_configs: [] })`, `mockCreateOAuthConfig()`, `mockOAuthLogin()`

1. **FAILING (Bug 1)**: "clicking Authorize with filled OAuth form creates new config then redirects"
  - Select server, select OAuth, fill client_id/secret/endpoints
  - Click Authorize
  - Expected: `POST /mcp-servers/{id}/oauth-configs` called, then `POST .../login` called, then `window.location.href` set to authorization URL
  - Actual (before fix): toast "No OAuth configuration selected"
2. "clicking Authorize with missing required fields shows validation toast"
  - Select server, select OAuth, leave client_id empty
  - Click Authorize
  - Expected: toast "Please fill in all required OAuth fields"
3. "create OAuth config API error shows error toast, does not redirect"
  - MSW override: `mockCreateOAuthConfig` returns 500 error
  - Expected: error toast shown, `window.location.href` NOT changed

#### describe: "OAuth authorize - with existing configs"

MSW: `mockListOAuthConfigs({ oauth_configs: [mockOAuthConfig] })`

1. "shows config dropdown when existing configs exist for server"
  - Select server, select OAuth
  - Expected: `data-testid="oauth-config-dropdown"` visible, `data-testid="oauth-config-select"` visible
2. "selecting existing config shows summary with Authorize button"
  - Select existing config from dropdown
  - Expected: `data-testid="oauth-config-summary"` visible, `data-testid="oauth-authorize-existing"` visible
3. "Authorize on existing config calls /login directly, no create-config"
  - Select existing config, click Authorize
  - Expected: `POST .../login` called (NOT `POST .../oauth-configs`), `window.location.href` set
4. "selecting New Configuration from dropdown shows OAuth form fields"
  - Click dropdown, select "New Configuration"
  - Expected: `data-testid="oauth-config-option-new"` clicked, form fields (client_id, secret, endpoints) visible
5. "filling new config form from dropdown and clicking Authorize creates config then login"
  - Select New from dropdown, fill form, click Authorize
  - Expected: `POST /mcp-servers/{id}/oauth-configs` called first, then `/login`

#### describe: "OAuth session restore after callback"

Setup: Pre-populate `sessionStorage` with `mcp_oauth_form_state` containing name, slug, mcp_server_id, auth_type, oauth_config_id, oauth_token_id, server_url, server_name. MSW: `mockListMcpServers`, `mockListOAuthConfigs`

1. "restores form with Connected status and populated fields after OAuth callback"
  - Render page (no `?id=` param, so create mode)
  - Expected: name/slug populated, auth type select shows `oauth-pre-registered` via `data-test-state`, connected card visible with Connected badge, Disconnect button visible

#### describe: "OAuth disconnect flow"

Setup: Pre-populate store with `completeOAuthFlow(tokenId)` and sessionStorage restore

1. "Disconnect calls DELETE oauth-token and on success shows config dropdown"
  - MSW: `mockDeleteOAuthToken()` returns 204, `mockListOAuthConfigs({ oauth_configs: [mockOAuthConfig] })`
    - Click Disconnect
    - Expected: connected card gone, config dropdown visible (since config exists after disconnect)
2. **FAILING (Bug 2)**: "Disconnect on API failure still clears local connected state"
  - MSW: `DELETE /mcps/oauth-tokens/:tokenId` returns 500 error
    - Click Disconnect
    - Expected (after fix): connected card gone (local state cleared), error toast shown
    - Actual (before fix): connected card remains

#### describe: "OAuth data-test-state attributes"

1. "auto-detect button has data-test-state reflecting mutation state"
  - Click auto-detect
    - Expected: `data-test-state="loading"` during request, `data-test-state="success"` after
2. "authorize button has data-test-state reflecting mutation state"
  - Fill form, click authorize
    - Expected: `data-test-state="loading"` during config creation/login

### Phase 1 Execution Order

1. Write all 13 component tests
2. Run: `cd crates/bodhi && npm run test` -- get baseline (tests 1 and 11 fail)
3. Fix Bug 1: change `handleOAuthAuthorize` condition at line 536
4. Fix Bug 2: add `store.disconnect()` to `deleteOAuthTokenMutation.onError`
5. Add `data-test-state` attributes to auto-detect and authorize buttons
6. Run: `cd crates/bodhi && npm run test` -- all pass
7. Run: `cd crates/bodhi && npm run format`

---

## Phase 2: E2E Test Fixes

### Page Object Changes

File: `[McpsPage.mjs](crates/lib_bodhiserver_napi/tests-js/pages/McpsPage.mjs)`

**Fix `expectMcpsListPage()` (line 192)**: Replace container + inline timeout with URL assertion:

```javascript
async expectMcpsListPage() {
  await this.page.waitForURL(/\/ui\/mcps(?:\/)?$/);
  await this.waitForSPAReady();
}
```

**Fix `expectNewMcpPage()` (line 229)**: Replace container with URL assertion:

```javascript
async expectNewMcpPage() {
  await this.page.waitForURL(/\/ui\/mcps\/new/);
  await this.waitForSPAReady();
}
```

**Split `selectNewOAuthConfig()` into three methods**:

- `expectNewOAuthConfigForm()` -- asserts dropdown NOT visible (no existing configs), fails if visible
- `selectExistingOAuthConfig(configId?)` -- asserts dropdown IS visible, selects config, fails if not visible
- `selectNewFromDropdown()` -- asserts dropdown IS visible, selects "New Configuration", fails if not visible

**Remove inline timeouts**: `expectToolsList()` (line 435), `expectServersListPage()` (line 138)

**Add data-test-state wait methods**:

- `waitForAutoDetectComplete()` -- waits for `[data-testid="oauth-auto-detect"][data-test-state="success"]`
- `waitForAutoDetectTerminal()` -- waits for success OR error terminal state

**Add composite method**: `createOAuthMcpInstance()` for full OAuth UI flow reuse between tests

### E2E Test Changes

File: `[mcps-oauth-auth.spec.mjs](crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-oauth-auth.spec.mjs)`

**Remove `apiCall` helper entirely** (lines 21-43)

**Rewrite Test 1: "UI-driven OAuth flow: new config, authorize, create, playground"**

- Login, create MCP server via `createMcpServer()`
- Navigate to new MCP form, select server, select OAuth
- `expectNewOAuthConfigForm()` (no dropdown since no existing configs)
- Fill OAuth credentials (client_id, client_secret from McpFixtures)
- Click auto-detect, wait for `data-test-state="success"` (no inline timeout)
- Verify endpoints populated via `toHaveValue()` (no inline timeout)
- Click Authorize, approve on test OAuth server (`data-testid="approve-btn"`), callback redirects back
- Wait for URL `/ui/mcps/new` (default Playwright timeout)
- Verify connected state via connected badge
- Fill name/slug, Fetch Tools, verify tools, Create MCP
- Navigate to playground via `clickPlaygroundById()`, select echo tool, fill params, execute, verify success result via `data-test-state="success"`

**Rewrite Test 2: "3rd party app executes tool on OAuth MCP via REST"**

- Phase 1: Create OAuth MCP via UI (reuse `createOAuthMcpInstance()` composite method)
- Get MCP instance ID from list via `getMcpUuidByName()`
- Phases 2-5: Keep existing OAuthTestApp page object interactions (external app OAuth config, access request submit + approve, /rest endpoint API verification)

**Remove Test 3**: "OAuth discovery returns correct metadata" -- already covered by test 1 auto-detect step

**Add existing config path to Test 1 (or new test)**: After creating the first OAuth MCP, navigate back to create a second MCP for the same server. The OAuth config dropdown should now appear (config was created in first flow). Select the existing config, Authorize, complete flow -- verifying the "existing config" E2E path.

### Phase 2 Execution Order

1. `make build.ui-rebuild` (rebuild NAPI with UI changes from Phase 1)
2. Update McpsPage.mjs page object
3. Rewrite E2E test file
4. Run E2E: `cd crates/lib_bodhiserver_napi && npm run test:playwright`
5. If fetch-tools fails (M2.5a backend bug -- `fetch_tools_for_server` doesn't resolve OAuth tokens), note and proceed -- that is next scope
6. Format all code

