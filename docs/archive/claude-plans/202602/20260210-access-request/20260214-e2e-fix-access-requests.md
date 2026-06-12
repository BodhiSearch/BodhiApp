# Plan: Fix E2E Test Failures — Phased Migration

## Context

The access-request feature revamp **removed the `scope_toolset-*` mechanism** entirely. Toolset authorization is now based on `access_request_id` in the token + approved toolsets in the DB record. The `name→slug` toolset refactor also changed UI `data-test-scope` attribute values and form field names. Together, these break 17 E2E tests across 6 spec files.

**Key architectural change**: Old flow granted toolset access via OAuth scopes (`scope_toolset-builtin-exa-web-search`). New flow uses access-request records: app requests access → user approves with specific toolset instance → middleware checks `access_request_id` claim in token against DB.

All files under `crates/lib_bodhiserver_napi/tests-js/`.

---

## Phase 0: Test Infrastructure / Utility Changes

Update shared page objects and helpers so that per-test fixes in later phases are minimal.

### 0A. ToolsetsPage.mjs — Selector Updates

**File**: `pages/ToolsetsPage.mjs`

**Changes**:

1. **Line 21**: Rename selector to match new UI attribute
   ```js
   // Before:
   toolsetNameInput: '[data-testid="toolset-name-input"]',
   // After:
   toolsetSlugInput: '[data-testid="toolset-slug-input"]',
   ```

2. **Line 221**: Update dropdown selector in `configureToolsetWithApiKey()`. New page uses `data-testid="type-option-${type.toolset_type}"` not `data-test-scope`.
   ```js
   // Before:
   await this.page.click(`[data-test-scope="${scope}"]`);
   // After:
   await this.page.click(`[data-testid="type-option-${scope}"]`);
   ```

3. **Line 225**: Update name input reference in `configureToolsetWithApiKey()`
   ```js
   // Before:
   await this.page.fill(this.selectors.toolsetNameInput, toolsetName);
   // After:
   await this.page.fill(this.selectors.toolsetSlugInput, toolsetName);
   ```

4. **Line 74 & 78**: Update `createToolset()` method similarly
   ```js
   // Line 78 — Before:
   await this.page.fill(this.selectors.toolsetNameInput, name);
   // After:
   await this.page.fill(this.selectors.toolsetSlugInput, name);
   ```

**Note**: Methods like `enableToolsetTypeOnAdmin(scope)`, `expectTypeToggle(scope)`, `typeToggle(scope)` use `data-test-scope` selectors which work correctly with admin page — they just need callers to pass `toolset_type` value (`builtin-exa-search`) instead of `scope_toolset-*` value.

### 0B. OAuth2ApiHelper.mjs — Request-Access API Migration

**File**: `utils/OAuth2ApiHelper.mjs`

**Changes**:

1. **Update `requestAudienceAccess()`** — support new API contract:
   ```js
   // Before:
   async requestAudienceAccess(appClientId) {
     const response = await fetch(`${this.baseUrl}/bodhi/v1/apps/request-access`, {
       method: 'POST',
       headers: { 'Content-Type': 'application/json' },
       body: JSON.stringify({ app_client_id: appClientId }),
     });
     if (response.status !== 200) { throw ... }
     return await response.json();
   }

   // After:
   async requestAudienceAccess(appClientId, flowType = 'popup', requested = null) {
     const body = { app_client_id: appClientId, flow_type: flowType };
     if (requested) {
       body.requested = requested;
     }
     const response = await fetch(`${this.baseUrl}/bodhi/v1/apps/request-access`, {
       method: 'POST',
       headers: { 'Content-Type': 'application/json' },
       body: JSON.stringify(body),
     });
     if (response.status !== 201) {
       throw new Error(`Failed to request access: ${response.status}, ${await response.text()}`);
     }
     return await response.json();
   }
   ```

2. **Add `approveAccessRequest()`** — programmatic approval via session:
   ```js
   async approveAccessRequest(page, arId, approvedToolsetTypes) {
     const response = await page.request.put(
       `${this.baseUrl}/bodhi/v1/access-requests/${arId}/approve`,
       {
         data: {
           approved: { toolset_types: approvedToolsetTypes }
         }
       }
     );
     if (response.status() !== 200) {
       throw new Error(`Failed to approve access request: ${response.status()}, ${await response.text()}`);
     }
     return await response.json();
   }
   ```

3. **Add `getAccessRequestStatus()`** — poll for scopes after approval:
   ```js
   async getAccessRequestStatus(arId, appClientId) {
     const response = await fetch(
       `${this.baseUrl}/bodhi/v1/apps/access-requests/${arId}?app_client_id=${appClientId}`
     );
     if (response.status !== 200) {
       throw new Error(`Failed to get status: ${response.status}, ${await response.text()}`);
     }
     return await response.json();
   }
   ```

4. **Remove `toolsetScopeIds` from `createAppClient()`** — no longer needed:
   ```js
   // Before:
   async createAppClient(devConsoleToken, port, clientName, description, redirectUris, toolsetScopeIds = [])

   // After:
   async createAppClient(devConsoleToken, port, clientName, description, redirectUris)
   ```

### 0C. auth-server-client.mjs — Remove Toolset Scope Dependencies

**File**: `utils/auth-server-client.mjs`

**Changes**:

1. **Line 9**: Remove `toolsetScopeExaWebSearchId` from config (or make optional)
2. **Line 17**: Remove from `requiredVars` array
3. **Lines 168-191**: Remove `toolsetScopeIds` parameter from `createAppClient()`; remove `toolset_scope_ids` from request body

### 0D. Rebuild Embedded UI

```bash
make build.ui-rebuild
```

The admin page already uses `data-test-scope={type.toolset_type}` with value `builtin-exa-search`. The new page uses `data-testid="type-option-${type.toolset_type}"`. Both are correct. Rebuild ensures the latest UI is embedded.

---

## Phase 1: toolsets-config.spec.mjs (5 failing tests)

**Root cause**: `data-test-scope` selector value mismatch. Tests pass `scope_toolset-builtin-exa-web-search` but UI renders `builtin-exa-search`.

**File**: `specs/toolsets/toolsets-config.spec.mjs`

**Changes**:

1. **Update constants** (lines 21-22):
   ```js
   // Before:
   const TOOLSET_NAME = 'builtin-exa-web-search';
   const TOOLSET_SCOPE = 'scope_toolset-builtin-exa-web-search';

   // After:
   const TOOLSET_TYPE = 'builtin-exa-search';
   ```

2. **Update all calls** that pass `TOOLSET_SCOPE` or `TOOLSET_NAME` to page object methods:
   - `enableToolsetTypeOnAdmin(TOOLSET_SCOPE)` → `enableToolsetTypeOnAdmin(TOOLSET_TYPE)`
   - `expectTypeToggle(TOOLSET_SCOPE)` → `expectTypeToggle(TOOLSET_TYPE)`
   - `toggleTypeEnabled(TOOLSET_SCOPE)` → `toggleTypeEnabled(TOOLSET_TYPE)`
   - `configureToolsetWithApiKey(TOOLSET_SCOPE, ...)` → `configureToolsetWithApiKey(TOOLSET_TYPE, ...)`
   - `getToolsetUuidByScope(TOOLSET_SCOPE)` → `getToolsetUuidByScope(TOOLSET_TYPE)`
   - `getToolsetRowByScope(TOOLSET_SCOPE)` → `getToolsetRowByScope(TOOLSET_TYPE)`

3. **Update assertions** that check `.scope` field in API responses — if any reference `TOOLSET_SCOPE` for API response validation, update to use `TOOLSET_TYPE` or the new field name.

**Tests fixed**: 5 (all toolsets-config tests)

**Verification**: `npx playwright test tests-js/specs/toolsets/toolsets-config.spec.mjs`

---

## Phase 2: chat-toolsets.spec.mjs (1 failing test)

**Root cause**: Same `data-test-scope` mismatch — `configureToolsetWithApiKey(TOOLSET_SCOPE, ...)`.

**File**: `specs/chat/chat-toolsets.spec.mjs`

**Changes**:

1. **Update constants** (lines 28-29):
   ```js
   const TOOLSET_TYPE = 'builtin-exa-search';
   // Remove TOOLSET_NAME and TOOLSET_SCOPE if unused elsewhere in file
   ```

2. **Update `configureToolsetWithApiKey` call** (line ~81):
   ```js
   toolsetsPage.configureToolsetWithApiKey(TOOLSET_TYPE, exaApiKey)
   ```

**Tests fixed**: 1

**Verification**: `npx playwright test tests-js/specs/chat/chat-toolsets.spec.mjs`

---

## Phase 3: chat-agentic.spec.mjs (1 failing test)

**Root cause**: Same `data-test-scope` mismatch.

**File**: `specs/chat/chat-agentic.spec.mjs`

**Changes**: Same pattern as Phase 2 — update constants and `configureToolsetWithApiKey` call.

**Tests fixed**: 1

**Verification**: `npx playwright test tests-js/specs/chat/chat-agentic.spec.mjs`

---

## Phase 4: oauth2-token-exchange.spec.mjs (1 failing test)

**Root causes**: Missing `flow_type`, `.scope` → `.resource_scope`, `configureOAuthForm` signature change.

**File**: `specs/oauth/oauth2-token-exchange.spec.mjs`

**Changes**:

1. **Update `requestAudienceAccess` response field** (line 96):
   ```js
   // Before:
   const resourceScope = requestAccessData.scope;
   // After:
   const resourceScope = requestAccessData.resource_scope;
   ```
   (The `requestAudienceAccess` method itself is fixed in Phase 0B)

2. **Update `configureOAuthForm` call** (line 104) — new 7-param signature:
   ```js
   // Before:
   await oauth2TestAppPage.configureOAuthForm(
     authServerConfig.authUrl,
     authServerConfig.authRealm,
     appClient.clientId,
     redirectUri,
     fullScopes
   );

   // After:
   await oauth2TestAppPage.configureOAuthForm(
     baseUrl,                           // bodhiServerUrl (NEW)
     authServerConfig.authUrl,
     authServerConfig.authRealm,
     appClient.clientId,
     redirectUri,
     fullScopes,
     null                               // requestedToolsets (NEW, null = no tools)
   );
   ```

3. **Remove `toolsetScopeIds` from `createAppClient` call** (line ~86-92) — just drop the parameter.

**Tests fixed**: 1

**Verification**: `npx playwright test tests-js/specs/oauth/oauth2-token-exchange.spec.mjs`

---

## Phase 5: toolsets-auth-restrictions.spec.mjs (7+ failing tests)

**Root causes**: All of the above PLUS the entire `scope_toolset-*` authorization model is removed. Tests must migrate from "toolset scopes in OAuth token" to "access-request with approved toolsets".

**File**: `specs/toolsets/toolsets-auth-restrictions.spec.mjs`

### 5A. Update Constants (lines 31-32)

```js
// Before:
const TOOLSET_NAME = 'builtin-exa-web-search';
const TOOLSET_SCOPE = 'scope_toolset-builtin-exa-web-search';

// After:
const TOOLSET_TYPE = 'builtin-exa-search';
```

Update all `configureToolsetWithApiKey(TOOLSET_SCOPE, ...)` → `configureToolsetWithApiKey(TOOLSET_TYPE, ...)` and `getToolsetUuidByScope(TOOLSET_SCOPE)` → `getToolsetUuidByScope(TOOLSET_TYPE)`.

### 5B. Update `createAppClient` calls — remove `toolsetScopeIds`

All calls to `apiHelper.createAppClient(...)` that pass `[authServerConfig.toolsetScopeExaWebSearchId]` — remove this parameter.

### 5C. Migrate request-access + OAuth flow per test case

**Old flow** (all 6 OAuth test cases):
1. `createAppClient(... toolsetScopeIds)` → register app WITH toolset scope on KC
2. `fetch('/bodhi/v1/apps/request-access', { app_client_id, toolset_scope_ids })` → get `{ scope }` (status 200)
3. Build `fullScopes = "openid profile email scope_user_user ${resourceScope} ${TOOLSET_SCOPE}"`
4. `configureOAuthForm(authUrl, realm, clientId, redirectUri, fullScopes)` → OAuth flow
5. Exchange token → test toolset access

**New flow**:
1. `createAppClient(...)` → register app WITHOUT toolset scopes
2. `apiHelper.requestAudienceAccess(appClientId, 'popup', requestedToolsets)` → get `{ status, id, ... }`
3. If toolsets requested: `apiHelper.approveAccessRequest(page, arId, approvedToolsets)` → approve
4. `apiHelper.getAccessRequestStatus(arId, appClientId)` → get `{ resource_scope, access_request_scope }`
5. Build `fullScopes = "openid profile email scope_user_user ${resource_scope} ${access_request_scope}"`
6. `configureOAuthForm(baseUrl, authUrl, realm, clientId, redirectUri, fullScopes, null)` → OAuth flow
7. Exchange token → test toolset access

### 5D. Migrate each test case

**Case 1** (~line 265): "App WITH toolset scope + OAuth WITH toolset scope → toolset accessible"
→ Becomes: "App WITH approved access-request (with toolsets) → toolset accessible"

```js
// 1. Request access WITH toolset types
const arData = await apiHelper.requestAudienceAccess(
  appClient.clientId, 'popup',
  { toolset_types: [{ toolset_type: TOOLSET_TYPE }] }
);
// arData.status === "draft", arData.id is the AR ID

// 2. Approve with specific instance
await apiHelper.approveAccessRequest(page, arData.id, [
  { toolset_type: TOOLSET_TYPE, status: 'approved', instance_id: toolsetUuid }
]);

// 3. Poll for scopes
const statusData = await apiHelper.getAccessRequestStatus(arData.id, appClient.clientId);
const resourceScope = statusData.resource_scope;
const accessRequestScope = statusData.access_request_scope;

// 4. OAuth with both scopes
const fullScopes = `openid profile email scope_user_user ${resourceScope} ${accessRequestScope}`;
await oauth2TestAppPage.configureOAuthForm(baseUrl, authUrl, realm, clientId, redirectUri, fullScopes, null);
// ... rest of OAuth flow + assert toolset accessible
```

**Case 2** (~line 407): "App WITH toolset scope + OAuth WITHOUT toolset scope → toolset NOT accessible"
→ Becomes: "App WITH auto-approved access-request (no toolsets) → toolset NOT accessible"

```js
// Request access WITHOUT toolset types → auto-approved
const arData = await apiHelper.requestAudienceAccess(appClient.clientId, 'popup');
// arData.status === "approved", arData.resource_scope present, NO access_request_scope

const resourceScope = arData.resource_scope;
const fullScopes = `openid profile email scope_user_user ${resourceScope}`;
// OAuth flow → toolset endpoints return 403
```

**Case 3** (~line 492): "App WITHOUT toolset scope + OAuth WITH toolset scope → invalid_scope"
→ **Remove this test.** The concept of "app-registered toolset scope" vs "OAuth-requested toolset scope" no longer exists. Neither Keycloak nor BodhiApp checks app client configs for toolset authorization anymore.

**Case 4** (~line 549): "App WITHOUT toolset scope + OAuth WITHOUT toolset scope → toolset NOT accessible"
→ Same as Case 2 (auto-approved, no toolsets, toolset access denied)

**Cases 5-6** (~lines 701, 761): "GET/PUT /toolsets/{id} with OAuth token → session-only, returns 401"
→ These test that toolset CRUD is session-only. The OAuth scope changes don't fundamentally affect this — just update the request-access call and `configureOAuthForm` signature. But still remove `scope_toolset-*` from scopes and use access-request flow instead.

### 5E. Remove `scope_toolset-*` from response assertions

Lines that check `t.scope === TOOLSET_SCOPE` in API responses (e.g., lines 193, 363) — these fields may no longer exist in the response. Update or remove assertions based on new `ToolsetDefinition` type (which has `toolset_type` but no `scope`).

### 5F. Update `configureOAuthForm` calls

All 6 calls need the new 7-parameter signature:
```js
await oauth2TestAppPage.configureOAuthForm(
  baseUrl, authServerConfig.authUrl, authServerConfig.authRealm,
  appClient.clientId, redirectUri, fullScopes, null
);
```

---

## Phase 6: network-ip-setup-flow.spec.mjs (2 failing tests)

**Root cause**: After OAuth login via different origin, redirects to `/ui/setup/download-models/` instead of `/ui/chat/`. Possibly related to app status detection in auth_callback_handler.

**Approach**: Investigate after Phases 0-5 are complete. Check:
- `crates/routes_app/src/routes_auth/login.rs` — `auth_callback_handler` redirect logic
- Whether the access-request changes affected the login callback flow
- Whether app status is correctly detected as "ready" during network-IP login

**Tests affected**: 2

---

## Phase 7: Final Verification

```bash
# Run all tests
cd crates/lib_bodhiserver_napi && npx playwright test

# Or run each spec individually
npx playwright test tests-js/specs/toolsets/toolsets-config.spec.mjs
npx playwright test tests-js/specs/chat/chat-toolsets.spec.mjs
npx playwright test tests-js/specs/chat/chat-agentic.spec.mjs
npx playwright test tests-js/specs/oauth/oauth2-token-exchange.spec.mjs
npx playwright test tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs
npx playwright test tests-js/specs/settings/network-ip-setup-flow.spec.mjs
```

---

## Files Modified Summary

| Phase | File | Key Changes |
|-------|------|-------------|
| 0A | `pages/ToolsetsPage.mjs` | `toolset-name-input` → `toolset-slug-input`, dropdown selector update |
| 0B | `utils/OAuth2ApiHelper.mjs` | `requestAudienceAccess` API migration, add `approveAccessRequest`, `getAccessRequestStatus` |
| 0C | `utils/auth-server-client.mjs` | Remove `toolsetScopeExaWebSearchId` requirement, remove `toolsetScopeIds` param |
| 0D | UI rebuild | `make build.ui-rebuild` |
| 1 | `specs/toolsets/toolsets-config.spec.mjs` | `TOOLSET_SCOPE` → `TOOLSET_TYPE` constant, all selector calls |
| 2 | `specs/chat/chat-toolsets.spec.mjs` | Same constant + selector update |
| 3 | `specs/chat/chat-agentic.spec.mjs` | Same constant + selector update |
| 4 | `specs/oauth/oauth2-token-exchange.spec.mjs` | `.scope` → `.resource_scope`, `configureOAuthForm` signature, remove `toolsetScopeIds` |
| 5 | `specs/toolsets/toolsets-auth-restrictions.spec.mjs` | Full access-request flow migration, all 6 OAuth test cases |
| 6 | `specs/settings/network-ip-setup-flow.spec.mjs` | Investigate redirect issue |
