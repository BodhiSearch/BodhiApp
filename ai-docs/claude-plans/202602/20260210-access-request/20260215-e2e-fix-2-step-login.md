# Fix Auto-Approve OAuth Flow: Two-Step Transparent Scope Wiring

## Context

The OAuth test app (`oauth-test-app.html`) auto-redirects to KC immediately after receiving an approved/callback response, hiding what scopes are being sent. KC rejects scopes with `Invalid scopes` and we can't debug why. Additionally, `apiHelper.approveAccessRequest()` and `apiHelper.getAccessRequestStatus()` are hacks that bypass the actual review page UI.

**Goal**: Split the OAuth flow into two visible steps — (1) request access and populate scopes, (2) user clicks Login — making scope wiring transparent and debuggable. Route all flows through the actual HTML test app and review page UI, removing API helper hacks.

## Files to Modify

| File | Action |
|------|--------|
| `crates/lib_bodhiserver_napi/tests-js/test-pages/oauth-test-app.html` | Major changes |
| `crates/lib_bodhiserver_napi/tests-js/pages/OAuth2TestAppPage.mjs` | Rename + add methods |
| `crates/lib_bodhiserver_napi/tests-js/pages/AccessRequestReviewPage.mjs` | **New file** |
| `crates/lib_bodhiserver_napi/tests-js/specs/oauth/oauth2-token-exchange.spec.mjs` | Update flow |
| `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs` | Update all 6 test methods |
| `crates/lib_bodhiserver_napi/tests-js/utils/OAuth2ApiHelper.mjs` | Remove 3 methods |

---

## Phase 0: Foundation — HTML + POM Changes

No tests run yet. Set up all infrastructure.

### 0a. `oauth-test-app.html` — Two-Step Flow

**Add `data-test-state="request-access"` to submit button** (~line 175):
```html
<button type="submit" class="btn" data-test-state="request-access">Request Access &amp; Login</button>
```

**Add scope status display element** (new div, outside `ALL_SECTIONS`):
```html
<div id="scope-status" class="hidden" style="...">
  <div id="scope-status-message"></div>
</div>
```
With `data-test-resource-scope` and `data-test-access-request-scope` attributes for Playwright to read individual scope values.

**Add `showReadyToLogin()` helper**:
- Updates `#scope` input field with resolved scopes
- Sets `data-test-resource-scope` / `data-test-access-request-scope` on `#scope-status`
- Shows status message: `"Scopes resolved: resource_scope: X, access_request_scope: Y"`
- Changes button text → `"Login"`, `data-test-state` → `"login"`
- Shows `config-section` + `scope-status`

**Modify auto-approve branch** in `requestAccessAndLogin()` (~line 332):
Replace `startOAuthFlow(updatedScope)` → call `showReadyToLogin(updatedScope, data.resource_scope, null)`

**Modify `handleAccessRequestCallback()` approved branch** (~line 384):
Replace `startOAuthFlow(updatedScope)` → call `showReadyToLogin(updatedScope, data.resource_scope, data.access_request_scope)`

**Add form submit state routing** (~line 262):
Check `data-test-state` on button — if `"login"` call `loginWithOAuth()`, else call `requestAccessAndLogin()`

**Add `loginWithOAuth()` function**:
Reads `#scope` field value and calls `startOAuthFlow(scope)`

**Update `resetApp()`**: Reset button text/state, hide `scope-status`

### 0b. New POM — `AccessRequestReviewPage.mjs`

Interacts with Bodhi review page at `/ui/apps/access-requests/review?id={id}`.

**Verified data-testids** from `crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx`:
- `review-access-page` (line 322), `review-tool-checkbox-{type}` (line 57)
- `review-instance-select-{type}` (line 77), `review-instance-option-{id}` (line 90)
- `review-approve-button` (line 367), `review-deny-button` (line 357)

**Methods**: `approveWithToolsets([{toolsetType, instanceId}])`, `clickApprove()`, `clickDeny()`

**Auth note**: Review page requires `authenticated={true}` (line 392). Browser must have Bodhi session — investigate during Phase 4 (first draft-flow test).

### 0c. POM Changes — `OAuth2TestAppPage.mjs`

- **Rename** `startOAuthFlow()` → `submitAccessRequest()` (same impl, clicks submit button)
- **Add** `waitForLoginReady()` — waits for `button[data-test-state="login"]`
- **Add** `clickLogin()` — waits for login state, clicks button
- **Add** `setScopes(value)` — fills `#scope` input
- **Add** `getResourceScope()` — reads `data-test-resource-scope` from `#scope-status`
- **Add** `getAccessRequestScope()` — reads `data-test-access-request-scope` from `#scope-status`

---

## Phase 1: Fix `should complete OAuth2 Token Exchange flow with dynamic audience`

**File**: `oauth2-token-exchange.spec.mjs` (line 61)
**Type**: Auto-approve, no toolsets — simplest test

**Changes**:
- Remove `apiHelper.requestAudienceAccess()` call and `resourceScope` variable
- Replace `startOAuthFlow()` with:
  ```
  submitAccessRequest()    → auto-approve → form updated
  waitForLoginReady()      → button shows "Login"
  clickLogin()             → KC redirect
  ```
- Keep everything after (waitForAuthServerRedirect, handleConsent, waitForTokenExchange, token validation)

**Run & verify**:
```bash
cd crates/lib_bodhiserver_napi && npx playwright test tests-js/specs/oauth/oauth2-token-exchange.spec.mjs -g "should complete OAuth2 Token Exchange flow"
```

---

## Phase 2: Fix `App WITHOUT toolset scope + OAuth WITHOUT toolset scope returns empty list`

**File**: `toolsets-auth-restrictions.spec.mjs` (line 597) — Case 4
**Type**: Auto-approve, no scope modification — simple

**Changes**:
- Remove `apiHelper.requestAudienceAccess()` call
- Replace `startOAuthFlow()` with `submitAccessRequest()` + `waitForLoginReady()` + `clickLogin()`
- Keep `scope_user_user` in base scopes
- Keep verification logic unchanged

**Run & verify**:
```bash
cd crates/lib_bodhiserver_napi && npx playwright test tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs -g "App WITHOUT toolset scope.*OAuth WITHOUT toolset scope"
```

---

## Phase 3: Fix `App WITHOUT toolset scope + OAuth WITH toolset scope returns invalid_scope error`

**File**: `toolsets-auth-restrictions.spec.mjs` (line 539) — Case 3
**Type**: Auto-approve + inject fake scope → KC error

**Changes**:
- Remove `apiHelper.requestAudienceAccess()` call
- After `submitAccessRequest()` + `waitForLoginReady()`:
  - Read current scope from form: `await page.inputValue('#scope')`
  - Append fake scope: `await oauth2TestAppPage.setScopes(current + ' scope_ar_nonexistent')`
  - `clickLogin()`
- Keep `expectOAuthError('invalid_scope')` verification

**Run & verify**:
```bash
cd crates/lib_bodhiserver_napi && npx playwright test tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs -g "App WITHOUT toolset scope.*OAuth WITH toolset scope"
```

---

## Phase 4: Fix `App WITH toolset scope + OAuth WITH toolset scope returns toolset in list and can execute`

**File**: `toolsets-auth-restrictions.spec.mjs` (line 265) — Case 1
**Type**: Draft flow → review page → approve with toolset selection — most complex

**Changes**:
- Remove `apiHelper.requestAudienceAccess()`, `apiHelper.approveAccessRequest()`, `apiHelper.getAccessRequestStatus()`
- Remove manual `fullScopes` construction
- New flow:
  ```
  configureOAuthForm(baseScopes, JSON.stringify([{toolset_type: TOOLSET_TYPE}]))
  submitAccessRequest()                    → draft → redirect to review page
  waitForAccessRequestRedirect(baseUrl)
  reviewPage.approveWithToolsets([{toolsetType: TOOLSET_TYPE, instanceId: toolsetId}])
  waitForAccessRequestCallback(testAppUrl) → HTML callback populates scopes
  waitForLoginReady()
  clickLogin()                             → KC redirect
  waitForAuthServerRedirect() → handleConsent() → waitForTokenExchange()
  ```
- Keep toolset list + execute verification logic unchanged

**Auth investigation**: If review page redirects to login, handle the auth chain here.

**Run & verify**:
```bash
cd crates/lib_bodhiserver_napi && npx playwright test tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs -g "App WITH toolset scope.*OAuth WITH toolset scope"
```

---

## Phase 5: Fix `App WITH toolset scope + OAuth WITHOUT toolset scope returns empty list`

**File**: `toolsets-auth-restrictions.spec.mjs` (line 411) — Case 2
**Type**: Draft flow + scope modification (remove access_request_scope)

**Changes**:
- Same as Phase 4 up to `waitForLoginReady()`
- Then modify scopes:
  ```
  const arScope = await oauth2TestAppPage.getAccessRequestScope();
  const current = await page.inputValue('#scope');
  const modified = current.replace(arScope, '').replace(/\s+/g, ' ').trim();
  await oauth2TestAppPage.setScopes(modified);
  clickLogin()
  ```
- Keep toolset empty-list verification

**Run & verify**:
```bash
cd crates/lib_bodhiserver_napi && npx playwright test tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs -g "App WITH toolset scope.*OAuth WITHOUT toolset scope"
```

---

## Phase 6: Fix `GET /toolsets/{id} with OAuth token returns 401 (session-only)`

**File**: `toolsets-auth-restrictions.spec.mjs` (line 747) — CRUD test 1
**Type**: Auto-approve, simple

**Changes**:
- Remove `apiHelper.requestAudienceAccess()` call
- Replace `startOAuthFlow()` with `submitAccessRequest()` + `waitForLoginReady()` + `clickLogin()`

**Run & verify**:
```bash
cd crates/lib_bodhiserver_napi && npx playwright test tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs -g "GET /toolsets.*OAuth token"
```

---

## Phase 7: Fix `PUT /toolsets/{id} with OAuth token returns 401 (session-only)`

**File**: `toolsets-auth-restrictions.spec.mjs` (line 803) — CRUD test 2
**Type**: Auto-approve, simple

**Changes**: Same pattern as Phase 6.

**Run & verify**:
```bash
cd crates/lib_bodhiserver_napi && npx playwright test tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs -g "PUT /toolsets.*OAuth token"
```

---

## Phase 8: Cleanup + Full Test Run

### 8a. Remove unused methods from `OAuth2ApiHelper.mjs`
- `requestAudienceAccess()` (lines 21-37)
- `approveAccessRequest()` (lines 39-57)
- `getAccessRequestStatus()` (lines 59-67)

Verify no other files reference these methods before deletion.

### 8b. Run all tests
```bash
# Full oauth2 test suite
cd crates/lib_bodhiserver_napi && npx playwright test tests-js/specs/oauth/oauth2-token-exchange.spec.mjs

# Full toolsets test suite
cd crates/lib_bodhiserver_napi && npx playwright test tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs
```

---

## Key Design Decisions (from user)

- `data-testid` stays same on button, `data-test-state` changes (`request-access` → `login`)
- `scope_user_user` is NOT removed from any test scopes
- Status message shows scope values for debugging
- `setScopes()` POM method for negative test cases to modify auto-populated scopes
- HTML handles all scope population (auto-approve + draft callback via status API)
- All flows go through HTML test app + review page UI (no API helper hacks)
- Single PR for everything
- No UI rebuild needed — test app HTML is static, review page unchanged

## Potential Concerns

1. **Review page auth**: Requires `authenticated={true}`. Browser needs Bodhi session. Investigate in Phase 4 if beforeAll login carries over.
2. **Shadcn Select**: Radix renders options in portal. Use `page.locator()` not scoped selectors.
3. **sessionStorage**: Persists per origin across navigations. Test app origin sessionStorage survives redirect to review page and back.
