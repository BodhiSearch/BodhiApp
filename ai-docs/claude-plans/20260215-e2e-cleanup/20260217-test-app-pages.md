# Plan: Split Dashboard into Separate Pages with Header Navigation

## Context

The test-oauth-app's `/dashboard` route currently renders 5 sections (Token, User Info, Toolsets, Chat, REST Client) on a single long scrollable page. This makes the app unwieldy and doesn't match how a realistic 3rd-party app would be structured. We're splitting the dashboard into 3 separate pages with header navigation, removing the Toolsets section (its functionality is covered by the REST Client), and merging User Info into the Token page.

## Architecture Decisions

| Decision | Choice |
|----------|--------|
| Pages | 3 pages: `/dashboard` (token+userinfo), `/chat`, `/rest` |
| Route style | Flat top-level routes (not nested under /dashboard) |
| Default post-login landing | `/rest` (OAuth callback redirects here directly) |
| Toolsets section | Removed entirely (REST Client can call those APIs) |
| User Info | Merged into `/dashboard` page, keep "Fetch" button pattern |
| Header (auth pages) | Left: user email + Logout | Center: nav links | Right: brand |
| Header (pre-auth pages) | Keep current minimal style (brand left, label center, action right) |
| Email source | Decoded from JWT payload (no API call) |
| Chat state on navigation | Resets (acceptable for test tool) |
| Nav link style | Text only with active highlight |
| POM pattern | `OAuthTestApp` composing Page objects (not Section objects) |
| Login verification in tests | Check for user email in header (decoded from JWT) |
| Test migration | One test file at a time, run and pass before moving to next |

## Phase 1: React App — Split Pages + Update Routing

### 1a. Create new page components

**Create** `test-oauth-app/src/pages/TokenPage.tsx` (route: `/dashboard`)
- Renders `TokenDisplay` + `UserInfoSection` in cards
- Has `data-testid="page-dashboard"` and `data-test-state="loaded"` when mounted
- Reads token from `AuthContext`

**Create** `test-oauth-app/src/pages/ChatPage.tsx` (route: `/chat`)
- Renders `ChatSection` component (moved from DashboardPage)
- Has `data-testid="page-chat"` and `data-test-state` tracking

**Create** `test-oauth-app/src/pages/RestPage.tsx` (route: `/rest`)
- Renders `RestClientSection` component (moved from DashboardPage)
- Has `data-testid="page-rest"` and `data-test-state="loaded"` when mounted

### 1b. Update routing

**Modify** `test-oauth-app/src/App.tsx`
- Replace `/dashboard` route with 3 new routes: `/dashboard`, `/chat`, `/rest`
- Add redirect: `/dashboard-old` or just remove old route
- Wrap authenticated routes so they redirect to `/` if no token

**Modify** `test-oauth-app/src/pages/OAuthCallbackPage.tsx`
- Change post-token-exchange redirect from `/dashboard` to `/rest`

### 1c. Update header/layout

**Modify** `test-oauth-app/src/components/AppLayout.tsx`
- Detect authenticated state (token exists in AuthContext)
- **Pre-auth pages** (/, /access-callback): Keep current style (brand left, label center, action right)
- **Post-auth pages** (/dashboard, /chat, /rest):
  - Left: user email (decoded from JWT) + Logout button
  - Center: nav links — Dashboard | Chat | REST — using `NavLink` with active styling
  - Right: "OAuth2 Test App" brand
- Nav link data attributes: `data-testid="nav-dashboard"`, `data-testid="nav-chat"`, `data-testid="nav-rest"`
- User email display: `data-testid="header-user-email"`
- Logout button: `data-testid="btn-header-logout"` (keep existing)

### 1d. Remove Toolsets

**Delete** `test-oauth-app/src/components/ToolsetsSection.tsx`
- Remove import from old DashboardPage (which gets deleted)

### 1e. Delete old DashboardPage

**Delete** `test-oauth-app/src/pages/DashboardPage.tsx`
- Replaced by TokenPage, ChatPage, RestPage

### 1f. Build and verify

```bash
cd test-oauth-app && npm run build
```

Manual verify via Chrome MCP:
- Navigate to `http://localhost:55173` → config page (no nav links)
- Complete OAuth flow → lands on `/rest`
- Header shows email + nav links + brand
- Click nav links → navigate between pages
- Logout → returns to `/`

---

## Phase 2: Rewrite Page Objects

### 2a. Create new page-level POMs

**Create** `tests-js/pages/DashboardPage.mjs`
```
DashboardPage
├── navigateTo()         — click nav-dashboard link
├── waitForLoaded()      — wait for page-dashboard[data-test-state="loaded"]
├── getAccessToken()     — read token display
├── fetchUserInfo()      — click button, wait, parse response
├── getUserInfoResponse() — read response
```

**Create** `tests-js/pages/ChatPage.mjs`
```
ChatPage
├── navigateTo()         — click nav-chat link
├── waitForModelsLoaded() — wait for model select loaded
├── selectModel(id)
├── getModels()
├── sendMessage(text)
├── waitForResponse()
├── getLastResponse()
├── getStatus()
```

**Create** `tests-js/pages/RESTPage.mjs`
```
RESTPage
├── navigateTo()          — click nav-rest link
├── waitForLoaded()       — wait for page-rest[data-test-state="loaded"]
├── sendRequest({ method, url, headers, body, useAuth })
├── getResponseStatus()
├── getResponse()
├── getState()
```

### 2b. Create new composite POM

**Create** `tests-js/pages/OAuthTestApp.mjs`
```
OAuthTestApp
├── config: ConfigSection       — form fill, submit (keep as-is)
├── accessCallback: AccessCallbackSection  — scope display (keep as-is)
├── oauth: OAuthSection         — KC login/consent (update waitForTokenExchange)
├── dashboard: DashboardPage    — token + user info page
├── chat: ChatPage              — streaming chat page
├── rest: RESTPage              — REST client page (default landing)
├── navigate(url)               — goto test app root
├── expectLoggedIn()            — verify header shows email + logout
├── getHeaderEmail()            — read email from header
├── logout()                    — click logout in header
```

### 2c. Update OAuthSection

**Modify** `tests-js/pages/sections/OAuthSection.mjs`
- `waitForTokenExchange(testAppUrl)`: Change URL wait from `/dashboard` to `/rest`
- Update `data-test-state` wait to match new RestPage's loaded state

### 2d. Keep existing sections unchanged

These remain as-is since they handle pre-auth flows:
- `ConfigSection.mjs`
- `AccessCallbackSection.mjs`

---

## Phase 3: Migrate Tests (one at a time)

### 3a. `oauth2-token-exchange.spec.mjs`

Changes:
- Import `OAuthTestApp` instead of `TestAppPage`
- `new TestAppPage(page, url)` → `new OAuthTestApp(page, url)`
- After OAuth flow: `waitForTokenExchange` now lands on `/rest`
- Login verification: `app.expectLoggedIn()` (checks header email/logout)
- Token extraction: `app.dashboard.navigateTo()` → `app.dashboard.getAccessToken()`
- API testing: keep using `apiHelper.testApiWithToken(accessToken)` (raw fetch, no POM)

**Run**: `npx playwright test tests-js/specs/oauth/oauth2-token-exchange.spec.mjs --reporter=list`

### 3b. `oauth-chat-streaming.spec.mjs`

Changes:
- Import `OAuthTestApp` instead of `TestAppPage`
- After OAuth flow: lands on `/rest`, verify logged in via header
- Token check: `app.dashboard.navigateTo()` → `app.dashboard.getAccessToken()`
- Chat: `app.chat.navigateTo()` → use chat methods as before
- Model selection, messaging, response verification same API

**Run**: `npx playwright test tests-js/specs/oauth/oauth-chat-streaming.spec.mjs --reporter=list`

### 3c. `toolsets-auth-restrictions.spec.mjs`

Changes:
- Import `OAuthTestApp` instead of `TestAppPage`
- All `testAppPage.dashboard.getAccessToken()` → navigate to dashboard first
- Remove `ToolsetsSection` POM usage (tests already use raw `fetch()` for toolset API calls)
- Same pattern: navigate → get token → use raw fetch for API assertions

**Run**: `npx playwright test tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs --reporter=list`

---

## Phase 4: Cleanup

- Delete old `tests-js/pages/TestAppPage.mjs`
- Delete old section POMs that are now page POMs:
  - `tests-js/pages/sections/DashboardSection.mjs` (replaced by `pages/DashboardPage.mjs`)
  - `tests-js/pages/sections/ChatSection.mjs` (replaced by `pages/ChatPage.mjs`)
  - `tests-js/pages/sections/RestClientSection.mjs` (replaced by `pages/RESTPage.mjs`)
  - `tests-js/pages/sections/UserInfoSection.mjs` (merged into DashboardPage)
  - `tests-js/pages/sections/ToolsetsSection.mjs` (removed)
- Verify no remaining imports reference deleted files
- Run full test suite: `npx playwright test --reporter=list`

---

## Files Summary

### React App (create/modify/delete)

| Action | File |
|--------|------|
| Create | `test-oauth-app/src/pages/TokenPage.tsx` |
| Create | `test-oauth-app/src/pages/ChatPage.tsx` |
| Create | `test-oauth-app/src/pages/RestPage.tsx` |
| Modify | `test-oauth-app/src/App.tsx` |
| Modify | `test-oauth-app/src/pages/OAuthCallbackPage.tsx` |
| Modify | `test-oauth-app/src/components/AppLayout.tsx` |
| Delete | `test-oauth-app/src/pages/DashboardPage.tsx` |
| Delete | `test-oauth-app/src/components/ToolsetsSection.tsx` |

### Page Objects (create/modify/delete)

| Action | File |
|--------|------|
| Create | `tests-js/pages/OAuthTestApp.mjs` |
| Create | `tests-js/pages/DashboardPage.mjs` |
| Create | `tests-js/pages/ChatPage.mjs` |
| Create | `tests-js/pages/RESTPage.mjs` |
| Modify | `tests-js/pages/sections/OAuthSection.mjs` |
| Delete | `tests-js/pages/TestAppPage.mjs` |
| Delete | `tests-js/pages/sections/DashboardSection.mjs` |
| Delete | `tests-js/pages/sections/ChatSection.mjs` |
| Delete | `tests-js/pages/sections/RestClientSection.mjs` |
| Delete | `tests-js/pages/sections/UserInfoSection.mjs` |
| Delete | `tests-js/pages/sections/ToolsetsSection.mjs` |

### Test Specs (modify)

| File | Key Changes |
|------|-------------|
| `specs/oauth/oauth2-token-exchange.spec.mjs` | Import OAuthTestApp, navigate to /dashboard for token, verify login via header |
| `specs/oauth/oauth-chat-streaming.spec.mjs` | Import OAuthTestApp, navigate between pages for token/chat |
| `specs/toolsets/toolsets-auth-restrictions.spec.mjs` | Import OAuthTestApp, navigate to /dashboard for token |

## Verification

### Per-phase
1. **Phase 1**: `cd test-oauth-app && npm run build` + manual Chrome MCP walkthrough
2. **Phase 2-3**: Run each test file individually after migration
3. **Phase 4**: `cd crates/lib_bodhiserver_napi && npx playwright test --reporter=list` — all tests pass
