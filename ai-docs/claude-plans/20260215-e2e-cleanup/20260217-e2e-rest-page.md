# Plan: Migrate E2E Tests to Use REST Page for API Calls

## Context

E2E tests currently make API calls via direct Node.js `fetch` with extracted Bearer tokens — tests extract the OAuth token from the test-oauth-app dashboard, then use it in Node.js `fetch` calls with `Authorization: Bearer` headers. This is fragile and doesn't match real user behavior.

The test-oauth-app now has a **REST page** (`/rest`) — a full-featured REST client UI with method selection, URL input, headers, body, auth toggle, and response display. All elements have `data-testid` and `data-test-state` attributes for reliable Playwright assertions.

**Goal**: Migrate tests that use direct `fetch` with OAuth Bearer tokens to use the REST page UI instead. Also enhance the REST page to accept paths instead of full URLs, automatically resolving against the Bodhi server URL from the OAuth config.

## Scope

**In scope**:
- Enhance REST page component to use stored `bodhiServerUrl` + path-only input
- Migrate `specs/toolsets/toolsets-auth-restrictions.spec.mjs` — 6 direct `fetch` calls with Bearer token
- Migrate `specs/oauth/oauth2-token-exchange.spec.mjs` — 1 `testApiWithToken()` call
- Delete `utils/OAuth2ApiHelper.mjs` (inline the one remaining unauthenticated usage)

**Out of scope** (keep as-is):
- Session-auth `page.evaluate` calls from Bodhi app context
- `token-refresh-integration.spec.mjs` `/dev/secrets` calls (internal state inspection)
- `api-models-forward-all.spec.mjs` session-auth calls

## Part 1: REST Page Component Enhancement

### `test-oauth-app/src/components/RestClientSection.tsx`

**Current behavior**: URL input takes full absolute URL (e.g., `http://localhost:51135/bodhi/v1/user`)

**New behavior**:
- Read `bodhiServerUrl` from `useAuth().config` (stored in sessionStorage during OAuth config)
- Display server URL as a read-only label with `data-testid="rest-server-url"`
- URL input takes path only (e.g., `/bodhi/v1/user`) with `placeholder="/bodhi/v1/user"`
- When sending, construct full URL: `${bodhiServerUrl}${path}`

Changes:
```tsx
// Add import
import { useAuth } from '@/context/AuthContext';

// Inside RestClientSection:
const { config } = useAuth();
const bodhiServerUrl = config?.bodhiServerUrl || '';

// In sendRequest():
const fullUrl = `${bodhiServerUrl}${url}`; // url is now just the path
const res = await fetch(fullUrl, fetchOptions);
```

New UI layout:
```
Server: http://localhost:51135     [data-testid="rest-server-url"]
[GET ▾] [/bodhi/v1/user         ] [data-testid="input-rest-url"]
```

Add `data-testid="rest-server-url"` to the server URL display element.

### `test-oauth-app/src/pages/RestPage.tsx`

No changes needed — it already guards on token and renders `RestClientSection`.

### `tests-js/pages/test-app/RESTPage.mjs` — Page Object Updates

Add selector for server URL display:
```js
selectors = {
  ...existing,
  serverUrl: '[data-testid="rest-server-url"]',
};
```

Add method:
```js
async getServerUrl() {
  return await this.page.locator(this.selectors.serverUrl).textContent();
}
```

The `sendRequest()` API stays the same but `url` param now takes paths:
```js
// Before: app.rest.sendRequest({ url: 'http://localhost:51135/bodhi/v1/toolsets', ... })
// After:  app.rest.sendRequest({ url: '/bodhi/v1/toolsets', ... })
```

## Part 2: Test Migration

### Files to Modify

1. **`tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs`**
2. **`tests-js/specs/oauth/oauth2-token-exchange.spec.mjs`**
3. **`tests-js/utils/OAuth2ApiHelper.mjs`** — delete

### Migration Pattern

**Before** (direct fetch with Bearer token):
```js
await app.dashboard.navigateTo();
const accessToken = await app.dashboard.getAccessToken();
expect(accessToken).toBeTruthy();

const response = await fetch(`${SHARED_SERVER_URL}/bodhi/v1/toolsets`, {
  headers: {
    Authorization: `Bearer ${accessToken}`,
    'Content-Type': 'application/json',
  },
});
expect(response.status).toBe(200);
const data = await response.json();
```

**After** (REST page with path-only):
```js
await app.rest.navigateTo();

await app.rest.sendRequest({
  method: 'GET',
  url: '/bodhi/v1/toolsets',
  headers: 'Content-Type: application/json',
  useAuth: true,
});
expect(await app.rest.getResponseStatus()).toBe(200);
const data = await app.rest.getResponse();
```

**POST with body**:
```js
await app.rest.sendRequest({
  method: 'POST',
  url: `/bodhi/v1/toolsets/${id}/execute/search`,
  headers: 'Content-Type: application/json',
  body: JSON.stringify({ params: { query: 'test', num_results: 3 } }),
  useAuth: true,
});
```

### Detailed Changes

#### `toolsets-auth-restrictions.spec.mjs`

**Session Auth test** (line 59): Keep as-is (uses `page.evaluate` from Bodhi app context)

**Test Case 1** (`WITH toolset scope + WITH scope`) — Phase 4 (lines 171-227):
- Remove `await app.dashboard.navigateTo()` + `getAccessToken()` + `expect(accessToken)` block
- Add `await app.rest.navigateTo()`
- Replace 2 fetch calls with `app.rest.sendRequest()` + `getResponseStatus()` + `getResponse()`
- Extract `exaToolset.id` from first response for second call

**Test Case 2** (`WITH toolset scope + WITHOUT scope`) — lines 308-349:
- Same pattern: replace 2 fetch calls

**Test Case 4** (`WITHOUT toolset scope + WITHOUT scope`) — lines 450-470:
- Replace 1 fetch call

**Test Case 5** (`GET and PUT returns 401`) — lines 538-563:
- Replace 2 fetch calls (GET expects 401, PUT with body expects 401)

#### `oauth2-token-exchange.spec.mjs`

**Complete OAuth2 Flow test** (line 69-85):
- Remove `apiHelper.testApiWithToken(accessToken)`
- Navigate to REST page: `await app.rest.navigateTo()`
- Use `app.rest.sendRequest({ url: '/bodhi/v1/user', useAuth: true })`
- Assert status 200 and response body fields

**Error Handling test** (lines 107-114):
- Inline `apiHelper.testUnauthenticatedApi()` as direct `fetch`:
  ```js
  const response = await fetch(`${baseUrl}/bodhi/v1/user`, {
    headers: { 'Content-Type': 'application/json' },
  });
  ```
- This test has no `{ page }` fixture and uses a standalone server — cannot use REST page

#### `OAuth2ApiHelper.mjs` — Delete file

Both methods replaced:
- `testApiWithToken()` → REST page calls
- `testUnauthenticatedApi()` → inlined as direct `fetch` in the one remaining usage

## Verification

1. Run the specific test files:
   ```bash
   cd crates/lib_bodhiserver_napi/tests-js
   npx playwright test specs/toolsets/toolsets-auth-restrictions.spec.mjs
   npx playwright test specs/oauth/oauth2-token-exchange.spec.mjs
   ```

2. Verify no remaining usages of `OAuth2ApiHelper`:
   ```bash
   grep -r "OAuth2ApiHelper" tests-js/
   ```

3. Verify no remaining direct `fetch` with Bearer token in migrated files:
   ```bash
   grep -n "Authorization.*Bearer" tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs
   grep -n "Authorization.*Bearer" tests-js/specs/oauth/oauth2-token-exchange.spec.mjs
   ```

4. Manual verification via Chrome: Start servers, navigate to REST page, verify path-only input resolves correctly against stored server URL
