# Plan: Replace oauth-test-app.html with React + Vite + Tailwind App

## Context

BodhiApp's E2E test suite uses `oauth-test-app.html` — a single monolithic HTML+JS file — as a 3rd-party OAuth application stand-in. It handles access request creation, PKCE-based OAuth2 authorization, token exchange, and API calls. The file has grown organically and is hard to extend. We need to replace it with a proper React application that:

- Acts as a realistic 3rd-party app (not a raw form)
- Covers full API testing: auth, access requests, toolsets, streaming chat, generic API calls
- Supports all existing E2E test flows with a new composite page object
- Is independently buildable and testable

## Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Tech stack | React + Vite + Tailwind CSS v4 + TypeScript + shadcn UI | Matches sdk-test-app patterns |
| Location | `crates/lib_bodhiserver_napi/test-oauth-app/` | Separate package in test infrastructure |
| SDK dependency | None — implement OAuth/PKCE flows directly | SDK access-request API is stale |
| Serving | Pre-built (`npm run build`) + `npx serve dist -s -l 55173` | Replaces static server entry in Playwright webServer |
| Port | 55173 (same as current static server) | Minimizes infra changes |
| Config input | Form fill with all fields visible | Like current app, no hidden panels |
| Routing | Separate routes: `/`, `/access-callback`, `/callback`, `/dashboard` | Explicit flow separation |
| Keycloak redirect URI | `http://localhost:55173/callback` | User adds manually to app-client |
| Post-auth navigation | `/callback` → redirect to `/dashboard` | Clear authenticated state |
| Chat backend | Real LLM on shared server | Full end-to-end streaming verification |
| API panel | Freestyle REST client (method, URL, headers, body) | Mini-Postman for exploratory testing |
| Migration | Atomic — build entire app, then migrate all tests at once | No test breakage during development |
| Page object | Composite pattern (TestAppPage with section sub-objects) | Matches sdk-test-app convention |
| React tests | E2E only via Playwright (no unit tests) | It's a test tool, not production |
| Build strategy | Combined webServer command (`build && serve`) | Simple, self-contained |
| Chat E2E | New spec file `oauth-chat-streaming.spec.mjs` | Separate concern, own timeouts |

## Execution Model

### Sub-Agent Orchestration

Each phase is executed by a specialized sub-agent launched by the main agent. The main agent:

1. **Launches a sub-agent** with full context: phase goal, file references, code patterns, and context from previous phases
2. **Waits for completion** — sub-agent executes the phase, runs verification, handles retries
3. **Reviews results** — checks sub-agent output for success/failure/deviations
4. **Updates plan file** — records progress, deviations, useful context for next phase
5. **Makes local commit** — `git add` + `git commit` with descriptive message
6. **Launches next sub-agent** — passes cumulative context from all completed phases

### Sub-Agent Context Template

Each sub-agent receives:
- **Goal**: What this phase must accomplish (specific success criteria)
- **Files to read**: Exact paths of reference files the agent needs
- **Files to create/modify**: What the agent writes
- **Code patterns**: Relevant snippets from existing codebase to follow
- **Verification command**: How to test that the phase succeeded
- **Context from previous phases**: Deviations, file changes, learnings

### Retry Policy for E2E Test Migration (Phase 4)

Each test migration follows this protocol:

1. **Attempt 1**: Migrate test, run with `npx playwright test <spec-file> --reporter=list`
2. **If fails → Debug with Chrome MCP** (see below), fix issues
3. **Attempt 2**: Re-run after fixes
4. **If fails again → Skip**: Mark test as `SKIPPED` in plan, report failure details
5. **After 3 total skipped tests → PAUSE**: Stop migration, request manual intervention from user

### Chrome MCP Debugging Protocol

When a test fails during migration, the sub-agent uses Claude in Chrome MCP to visually debug:

**Setup** (run these in parallel terminals):
```bash
# Terminal 1: Start BodhiApp server
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp
make run.app

# Terminal 2: Build and serve test OAuth app
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/lib_bodhiserver_napi/test-oauth-app
npm run build && npx serve dist -s -l 55173
```

**Debug flow**:
1. Navigate to `http://localhost:55173` (test app) in Chrome
2. Reproduce the failing test steps manually:
   - Fill config form with test values
   - Click "Request Access"
   - Follow the redirect flow
   - Observe what the UI shows at each step
3. Navigate to `http://localhost:1135` (BodhiApp) for review page interactions
4. Compare observed behavior with expected test assertions
5. Take screenshots at each step to identify where the flow diverges
6. Check browser console for errors
7. Fix the React app or page object based on findings

**Key URLs**:
- Test OAuth App: `http://localhost:55173`
- BodhiApp Server: `http://localhost:1135` (dev mode via `make run.app`)
- BodhiApp Auth Callback: `http://localhost:1135/ui/auth/callback`
- Keycloak: Check `.env.test` for `INTEG_TEST_AUTH_URL`

### Progress Tracking

After each phase, update this plan file with:
```markdown
### Phase N: [Title] — STATUS: COMPLETE / IN PROGRESS / BLOCKED

**Status**: Complete
**Deviations from plan**: [any differences from original plan]
**Context for next phase**: [learnings, file paths, gotchas]
**Files created/modified**: [actual file list]
**Commit**: [commit hash and message]
```

After each test migration in Phase 4:
```markdown
- [ ] `test-name` — STATUS: PASS / FAIL (attempt 1) / FAIL (attempt 2) / SKIPPED
  - Error: [brief description if failed]
  - Fix: [what was changed if fixed on retry]
```

---

## Routes & Flow

### Route: `/` (Config Page)
**Purpose**: Landing page with OAuth configuration form

**UI Elements** (all fields visible, no collapsible panels):
- BodhiApp Server URL (`data-testid="input-bodhi-server-url"`)
- Auth Server URL (`data-testid="input-auth-server-url"`)
- Realm (`data-testid="input-realm"`)
- Client ID (`data-testid="input-client-id"`)
- Confidential Client toggle + Client Secret (`data-testid="toggle-confidential"`, `data-testid="input-client-secret"`)
- Redirect URI (`data-testid="input-redirect-uri"`)
- Scope (`data-testid="input-scope"`)
- Requested Toolsets JSON (`data-testid="input-requested-toolsets"`)
- "Request Access" button (`data-testid="btn-request-access"`, `data-test-state="request-access"`)

**Flow**:
1. User fills config → clicks "Request Access"
2. POST `/bodhi/v1/apps/request-access` with `{ app_client_id, flow_type: "redirect", redirect_url, requested: { toolset_types } }`
3. Response `status: "approved"` → store `resource_scope`, show scopes inline, button changes to "Login" (`data-test-state="login"`)
4. Response `status: "draft"` → save config to `sessionStorage.oauthConfig`, redirect browser to `review_url`
5. Click "Login" → generate PKCE, save to sessionStorage, redirect to Keycloak auth endpoint

### Route: `/access-callback?id=...`
**Purpose**: Handle redirect back from BodhiApp access request review page

**UI Elements** (minimal):
- Resolved scope display: `resource_scope` (`data-test-resource-scope`), `access_request_scope` (`data-test-access-request-scope`)
- Editable scope field (`data-testid="input-scope"`) — pre-filled with combined scopes, editable for test manipulation
- "Login" button (`data-testid="btn-login"`, `data-test-state="login"`)
- Loading state while fetching status

**Flow**:
1. Read config from `sessionStorage.oauthConfig`
2. GET `/bodhi/v1/apps/access-requests/{id}?app_client_id={clientId}`
3. Display resolved scopes, populate editable scope field
4. User (or test) can modify scope before clicking Login
5. Click "Login" → generate PKCE, save to sessionStorage, redirect to Keycloak

### Route: `/callback?code=&state=`
**Purpose**: OAuth callback — exchange authorization code for token

**Flow**:
1. Read config from `sessionStorage.oauthConfig`
2. Validate `state` matches stored state
3. POST to Keycloak token endpoint with `code`, `redirect_uri`, and either `code_verifier` (public) or `client_secret` (confidential)
4. Store `access_token` in `sessionStorage.accessToken`
5. Redirect to `/dashboard`

**Error handling**: If `?error=invalid_scope` present, redirect to `/` with error display (`data-testid="error-section"`)

### Route: `/dashboard`
**Purpose**: Authenticated dashboard with API testing sections

**Sections** (all visible on single scrollable page, Card-based layout):

1. **Token Display Section** (`data-testid="section-token"`)
   - Raw access token (`data-testid="access-token"`)
   - Decoded JWT claims (header, payload, expiration)
   - "Logout" button (clears sessionStorage, redirects to `/`)

2. **User Info Section** (`data-testid="section-user-info"`)
   - "Fetch User Info" button
   - GET `/bodhi/v1/user` with Bearer token
   - Display response JSON (auth_status, user_id, username, role)

3. **Toolsets Section** (`data-testid="section-toolsets"`)
   - "List Toolsets" button → GET `/bodhi/v1/toolsets`
   - Display toolset list
   - Execute panel: toolset ID, method, JSON body → POST `/bodhi/v1/toolsets/{id}/execute/{method}`
   - Display execution result

4. **Chat Section** (`data-testid="section-chat"`)
   - Model selector (fetches from GET `/v1/models`)
   - Message input
   - "Send" button → POST `/v1/chat/completions` with `stream: true`
   - Streaming response display (SSE)
   - Message history

5. **REST Client Section** (`data-testid="section-rest-client"`)
   - HTTP method dropdown (GET, POST, PUT, DELETE)
   - URL input
   - Headers editor (key-value pairs)
   - JSON body editor
   - Auto-attach OAuth Bearer token toggle
   - "Send" button
   - Response display (status, headers, body with JSON formatting)

## State Management

- **Cross-redirect state**: `sessionStorage.oauthConfig` — preserves config, PKCE verifier, state token across multi-redirect flow
- **Token storage**: `sessionStorage.accessToken` — cleared on logout
- **In-session state**: React Context (`AuthContext`) provides token and config to all dashboard sections
- **No external state libraries** — hooks + context sufficient for this scope

## Project Structure

```
test-oauth-app/
├── package.json
├── vite.config.ts
├── tsconfig.json
├── index.html
├── serve.json                    # SPA fallback config for npx serve
├── public/
│   └── ping.txt                  # Health check for Playwright webServer
├── src/
│   ├── main.tsx                  # React root + Router setup
│   ├── App.tsx                   # Route definitions
│   ├── context/
│   │   └── AuthContext.tsx        # Token + config context provider
│   ├── lib/
│   │   ├── oauth.ts              # PKCE generation, token exchange
│   │   ├── api.ts                # API client (fetch wrapper with auth)
│   │   └── storage.ts            # sessionStorage helpers
│   ├── pages/
│   │   ├── ConfigPage.tsx         # Route: /
│   │   ├── AccessCallbackPage.tsx # Route: /access-callback
│   │   ├── OAuthCallbackPage.tsx  # Route: /callback
│   │   └── DashboardPage.tsx      # Route: /dashboard
│   ├── components/
│   │   ├── ui/                    # shadcn components (Card, Button, Input, etc.)
│   │   ├── ConfigForm.tsx         # OAuth config form
│   │   ├── ScopeDisplay.tsx       # Resolved scope display
│   │   ├── TokenDisplay.tsx       # JWT token display with decode
│   │   ├── UserInfoSection.tsx    # User info API section
│   │   ├── ToolsetsSection.tsx    # Toolset list + execute
│   │   ├── ChatSection.tsx        # Streaming chat
│   │   └── RestClientSection.tsx  # Freestyle REST client
│   └── hooks/
│       ├── useStreamingChat.ts    # SSE streaming hook
│       └── useApi.ts              # Generic API call hook
└── tailwind.config.js
```

## Phases

### Phase 1: React App Scaffolding + OAuth Core + Config (all fields visible)

**Sub-agent type**: `general-purpose`
**Goal**: Working OAuth flow end-to-end: config form → access request → approval redirect → OAuth login → token display
**Success criteria**: `cd test-oauth-app && npm run build` succeeds. Manual verification via Chrome MCP: navigate to `http://localhost:55173`, complete OAuth flow against local BodhiApp.

**Sub-agent context**:
- Read `oauth-test-app.html` for PKCE implementation, state machine, sessionStorage patterns
- Read SDK test app's `vite.config.ts`, `package.json`, `tailwind.config.js` for project setup patterns
- Follow shadcn UI patterns from SDK test app's `components/ui/` directory
- Use React Router v7 with `createBrowserRouter` for route definitions

**Create**:
- `test-oauth-app/package.json` — React 19, Vite 7, Tailwind v4, React Router v7, shadcn UI deps
- `test-oauth-app/vite.config.ts` — React plugin, path aliases (@, @app), esnext target
- `test-oauth-app/tsconfig.json` — strict TypeScript
- `test-oauth-app/tailwind.config.js` — shadcn theme (HSL CSS vars), content paths
- `test-oauth-app/index.html` — React root mount
- `test-oauth-app/serve.json` — `{ "rewrites": [{ "source": "**", "destination": "/index.html" }] }` for SPA fallback
- `test-oauth-app/public/ping.txt` — health check
- `test-oauth-app/src/main.tsx` — React root with BrowserRouter
- `test-oauth-app/src/App.tsx` — Route definitions (/, /access-callback, /callback, /dashboard)
- `test-oauth-app/src/context/AuthContext.tsx` — token + config context
- `test-oauth-app/src/lib/oauth.ts` — PKCE: `generateCodeVerifier()`, `generateCodeChallenge()`, `generateState()`; token exchange: `exchangeCodeForToken()`
- `test-oauth-app/src/lib/api.ts` — `apiClient.requestAccess()`, `apiClient.getAccessRequestStatus()`
- `test-oauth-app/src/lib/storage.ts` — `saveConfig()`, `loadConfig()`, `saveToken()`, `loadToken()`, `clearAll()`
- `test-oauth-app/src/pages/ConfigPage.tsx` — full config form + Request Access button + auto-approved Login
- `test-oauth-app/src/pages/AccessCallbackPage.tsx` — fetch status, show scopes, editable scope, Login button
- `test-oauth-app/src/pages/OAuthCallbackPage.tsx` — exchange code, redirect to /dashboard
- `test-oauth-app/src/pages/DashboardPage.tsx` — shell with token display + placeholder sections
- `test-oauth-app/src/components/ConfigForm.tsx` — all form fields with data-testid
- `test-oauth-app/src/components/ScopeDisplay.tsx` — resolved scope display
- `test-oauth-app/src/components/TokenDisplay.tsx` — raw token + decoded JWT
- shadcn UI components: Button, Card, Input, Label, Badge

**data-testid attributes** (critical for E2E):
- Form inputs: `input-bodhi-server-url`, `input-auth-server-url`, `input-realm`, `input-client-id`, `input-redirect-uri`, `input-scope`, `input-requested-toolsets`, `toggle-confidential`, `input-client-secret`
- Buttons: `btn-request-access` (with `data-test-state="request-access"|"login"`), `btn-login`
- Scopes: `data-test-resource-scope`, `data-test-access-request-scope` attributes on scope display elements
- Status: `access-request-loading`, `access-callback-loading`, `error-section`, `success-section`
- Token: `access-token`

**Verify manually**: `cd test-oauth-app && npm run dev` → fill config → complete OAuth flow with running BodhiApp

**Commit**: Phase 1 complete — OAuth flow works end-to-end

---

### Phase 2: Dashboard API Sections

**Sub-agent type**: `general-purpose`
**Goal**: User info, toolsets, and REST client sections on /dashboard
**Success criteria**: `npm run build` succeeds. Manual verification via Chrome MCP: complete OAuth flow → dashboard → each section renders and responds to clicks.

**Sub-agent context**:
- Read Phase 1 commit for actual file paths and patterns established
- Read SDK test app's `ApiTestSection.tsx` for REST client patterns
- Reference API endpoint table in this plan for request/response formats
- Follow the `useApi.ts` hook pattern established in Phase 1

**Create**:
- `test-oauth-app/src/components/UserInfoSection.tsx`
  - "Fetch" button → GET `/bodhi/v1/user` with Bearer token
  - Display response (auth_status, user_id, username, role)
  - `data-testid="section-user-info"`, `data-testid="btn-fetch-user"`, `data-testid="user-info-response"`

- `test-oauth-app/src/components/ToolsetsSection.tsx`
  - "List Toolsets" → GET `/bodhi/v1/toolsets`
  - Display list with toolset IDs, types, names
  - Execute form: toolset ID selector, method input, JSON body textarea
  - "Execute" → POST `/bodhi/v1/toolsets/{id}/execute/{method}`
  - Display result JSON
  - `data-testid="section-toolsets"`, `data-testid="btn-list-toolsets"`, `data-testid="toolsets-list"`, `data-testid="btn-execute-toolset"`, `data-testid="toolset-result"`

- `test-oauth-app/src/components/RestClientSection.tsx`
  - Method dropdown (GET/POST/PUT/DELETE)
  - URL input, headers key-value editor, JSON body textarea
  - "Auto-attach token" checkbox (default: on)
  - "Send" → execute fetch
  - Response: status code, headers, formatted JSON body
  - `data-testid="section-rest-client"`, `data-testid="input-rest-url"`, `data-testid="select-rest-method"`, `data-testid="btn-rest-send"`, `data-testid="rest-response"`

- `test-oauth-app/src/hooks/useApi.ts` — generic fetch wrapper with token injection

**Modify**:
- `DashboardPage.tsx` — add UserInfoSection, ToolsetsSection, RestClientSection

**Verify manually**: Complete OAuth flow → dashboard shows all sections → test each section against running BodhiApp

**Commit**: Phase 2 complete — all API testing sections functional

---

### Phase 3: Streaming Chat Section

**Sub-agent type**: `general-purpose`
**Goal**: SSE streaming chat via OAuth token against real LLM
**Success criteria**: `npm run build` succeeds. Manual verification via Chrome MCP: OAuth flow → dashboard → select model → send message → streaming response visible.

**Sub-agent context**:
- Read SDK test app's `useAgenticChat.ts` for SSE streaming patterns (simplified — no tool calling)
- Read SDK test app's `ChatSection.tsx` for UI patterns
- The SSE format: `data: {"choices":[{"delta":{"content":"..."}}]}` lines, terminated by `data: [DONE]`
- Use native `fetch()` with `ReadableStream` — no EventSource library needed

**Create**:
- `test-oauth-app/src/hooks/useStreamingChat.ts`
  - Manages message array, streaming state, error state
  - POST `/v1/chat/completions` with `stream: true` and Bearer token
  - Parse SSE: iterate `text/event-stream` response
  - Accumulate `delta.content` chunks, handle `[DONE]`
  - No tool calling (keep simple — toolset execution is a separate section)
  - `data-testid="section-chat"`, `data-testid="chat-model-select"`, `data-testid="chat-input"`, `data-testid="btn-chat-send"`, `data-testid="chat-messages"`, `data-testid="chat-status"`

- `test-oauth-app/src/components/ChatSection.tsx`
  - Model selector (fetches `/v1/models`, dropdown)
  - Message input + Send button
  - Message history with user/assistant bubbles
  - Streaming indicator
  - Status badge: idle | streaming | error

**Modify**:
- `DashboardPage.tsx` — add ChatSection

**Verify manually**: OAuth login → dashboard → select model → send message → see streaming response

**Commit**: Phase 3 complete — streaming chat functional

---

### Phase 4: Composite Page Object + E2E Test Migration

**Sub-agent type**: `general-purpose` (one sub-agent per test file migration)
**Goal**: New page object, migrate all E2E tests, add chat streaming spec
**Success criteria**: Each migrated spec passes: `cd crates/lib_bodhiserver_napi && npx playwright test <spec-file> --reporter=list`

**Execution flow** (main agent orchestrates):

1. **Sub-agent 4a**: Create all page objects (no test changes yet)
   - Create all section files under `tests-js/pages/sections/`
   - Create composite `TestAppPage.mjs`
   - Verify: page objects import without errors

2. **Sub-agent 4b**: Update `playwright.config.mjs` webServer to serve React app
   - Replace static server entry with `cd test-oauth-app && npm run build && npx serve dist -s -l 55173`
   - Verify: `npx playwright test --list` succeeds (servers start)

3. **Sub-agent 4c**: Migrate `oauth2-token-exchange.spec.mjs`
   - Read current spec + current page object for exact method calls
   - Rewrite using new TestAppPage
   - Run: `npx playwright test tests-js/specs/oauth/oauth2-token-exchange.spec.mjs --reporter=list`
   - On failure: debug with Chrome MCP (see protocol above), retry once
   - **Update plan** with status, commit if passing

4. **Sub-agent 4d**: Migrate `toolsets-auth-restrictions.spec.mjs`
   - This is the most complex spec (6 tests, multi-phase flows, scope manipulation)
   - Read current spec for all 6 test cases
   - Rewrite using new TestAppPage + test.step
   - Run: `npx playwright test tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs --reporter=list`
   - On failure: debug with Chrome MCP, retry once
   - **Update plan** with status, commit if passing

5. **Sub-agent 4e**: Create new `oauth-chat-streaming.spec.mjs`
   - New spec, no migration needed
   - Run: `npx playwright test tests-js/specs/oauth/oauth-chat-streaming.spec.mjs --reporter=list`
   - On failure: debug with Chrome MCP (likely model warm-up timeout), retry once

6. **Sub-agent 4f**: Create new `oauth-api-testing.spec.mjs` (optional)
   - Tests dashboard sections end-to-end
   - Skip if 3+ previous test failures

**Retry tracking**:
```markdown
- [ ] oauth2-token-exchange.spec.mjs — STATUS: PENDING
- [ ] toolsets-auth-restrictions.spec.mjs — STATUS: PENDING
- [ ] oauth-chat-streaming.spec.mjs — STATUS: PENDING
- [ ] oauth-api-testing.spec.mjs — STATUS: PENDING
Total failures: 0/3 (pause at 3)
```

**Create — Page Objects** (Sub-agent 4a):
- `tests-js/pages/TestAppPage.mjs` — composite root
  ```
  TestAppPage
  ├── config: ConfigSection      — form fill, submit, wait for states
  ├── accessCallback: AccessCallbackSection — scope display, scope editing, login
  ├── oauth: OAuthSection        — Keycloak login/consent handling
  ├── dashboard: DashboardSection — token display, logout
  ├── userInfo: UserInfoSection   — fetch user, read response
  ├── toolsets: ToolsetsSection   — list, execute, read results
  ├── chat: ChatSection           — send message, wait for streaming
  └── restClient: RestClientSection — send request, read response
  ```

- `tests-js/pages/sections/ConfigSection.mjs`
  - `configureOAuthForm({ bodhiServerUrl, authServerUrl, realm, clientId, redirectUri, scope, requestedToolsets, isConfidential, clientSecret })`
  - `submitAccessRequest()` — click btn-request-access
  - `waitForLoginReady()` — wait for `data-test-state="login"`
  - `clickLogin()` — click Login button
  - `setScopes(value)` — update scope input
  - `getResourceScope()` — read `data-test-resource-scope`
  - `getAccessRequestScope()` — read `data-test-access-request-scope`

- `tests-js/pages/sections/AccessCallbackSection.mjs`
  - `waitForLoaded()` — wait for scopes to appear
  - `getResourceScope()`, `getAccessRequestScope()`
  - `setScopes(value)` — edit scope field
  - `clickLogin()` — click Login button

- `tests-js/pages/sections/OAuthSection.mjs`
  - `waitForAccessRequestRedirect(bodhiServerUrl)` — wait for redirect to review page
  - `waitForAccessRequestCallback(testAppUrl)` — wait for `/access-callback?id=`
  - `waitForAuthServerRedirect(authServerUrl)` — wait for Keycloak redirect
  - `handleLogin(username, password)` — fill KC login form
  - `handleConsent()` — click Yes on consent
  - `waitForTokenExchange(testAppUrl)` — wait for `/dashboard` navigation
  - `expectOAuthError(expectedError)` — wait for error on config page

- `tests-js/pages/sections/DashboardSection.mjs`
  - `waitForLoaded()` — wait for dashboard to render
  - `getAccessToken()` — read token display
  - `logout()`

- `tests-js/pages/sections/UserInfoSection.mjs`
  - `fetchUserInfo()` → click fetch button, return response

- `tests-js/pages/sections/ToolsetsSection.mjs`
  - `listToolsets()` → click list button, return array
  - `executeToolset(id, method, body)` → fill form, click execute, return result

- `tests-js/pages/sections/ChatSection.mjs`
  - `selectModel(modelId)`
  - `sendMessage(text)` → type + click send
  - `waitForResponse()` → wait for streaming complete
  - `getLastResponse()` → read assistant message

- `tests-js/pages/sections/RestClientSection.mjs`
  - `sendRequest({ method, url, headers, body })` → fill form, click send
  - `getResponse()` → read response display

**Migrate — E2E Specs**:

1. **`oauth2-token-exchange.spec.mjs`** — update page object import + methods:
   - `OAuth2TestAppPage` → `TestAppPage`
   - `navigateToTestApp()` → `testAppPage.navigate(testAppUrl)`
   - `configureOAuthForm(...)` → `testAppPage.config.configureOAuthForm(...)`
   - `submitAccessRequest()` → `testAppPage.config.submitAccessRequest()`
   - `waitForLoginReady()` → `testAppPage.config.waitForLoginReady()`
   - `clickLogin()` → `testAppPage.config.clickLogin()`
   - `handleLogin()` → `testAppPage.oauth.handleLogin()`
   - `waitForTokenExchange()` → `testAppPage.oauth.waitForTokenExchange()`
   - `getAccessToken()` → `testAppPage.dashboard.getAccessToken()`
   - **Redirect URI**: change from `oauth-test-app.html` to `/callback` in form config

2. **`toolsets-auth-restrictions.spec.mjs`** — update page object + flow:
   - All method renames as above
   - Access callback flow:
     - `waitForAccessRequestCallback()` → `testAppPage.oauth.waitForAccessRequestCallback()`
     - On `/access-callback` page: `testAppPage.accessCallback.getResourceScope()`, `.getAccessRequestScope()`
     - Scope manipulation: `testAppPage.accessCallback.setScopes(value)`
     - Login from callback: `testAppPage.accessCallback.clickLogin()`
   - Auto-approved flow (Cases 3, 4):
     - Scopes on config page: `testAppPage.config.getResourceScope()`, `.setScopes(value)`
     - Login from config: `testAppPage.config.clickLogin()`

3. **New: `oauth-chat-streaming.spec.mjs`**:
   - Complete OAuth flow (auto-approved, no toolsets)
   - Navigate to dashboard
   - Select model, send message, verify streaming response
   - Test timeout: extended for model warm-up

**Method mapping matrix** (current → new):

| Current OAuth2TestAppPage | New TestAppPage |
|---------------------------|-----------------|
| `navigateToTestApp(url)` | `navigate(url)` |
| `configureOAuthForm(...)` | `config.configureOAuthForm(...)` |
| `submitAccessRequest()` | `config.submitAccessRequest()` |
| `waitForLoginReady()` | `config.waitForLoginReady()` |
| `clickLogin()` | `config.clickLogin()` or `accessCallback.clickLogin()` |
| `setScopes(value)` | `config.setScopes(value)` or `accessCallback.setScopes(value)` |
| `getResourceScope()` | `config.getResourceScope()` or `accessCallback.getResourceScope()` |
| `getAccessRequestScope()` | `config.getAccessRequestScope()` or `accessCallback.getAccessRequestScope()` |
| `waitForAccessRequestRedirect(url)` | `oauth.waitForAccessRequestRedirect(url)` |
| `waitForAccessRequestCallback(url)` | `oauth.waitForAccessRequestCallback(url)` |
| `waitForAuthServerRedirect(url)` | `oauth.waitForAuthServerRedirect(url)` |
| `handleLogin(u, p)` | `oauth.handleLogin(u, p)` |
| `handleConsent()` | `oauth.handleConsent()` |
| `waitForTokenExchange(url)` | `oauth.waitForTokenExchange(url)` |
| `getAccessToken()` | `dashboard.getAccessToken()` |
| `expectOAuthError(err)` | `oauth.expectOAuthError(err)` |

**E2E Test Outlines** (using `test.step`, multi-assertion per test):

#### Migrated: `oauth2-token-exchange.spec.mjs`

```javascript
test('complete OAuth2 flow: access request → token exchange → API verification', async ({ page }) => {
  await test.step('Configure OAuth and submit access request', async () => {
    // Navigate to test app, fill config form, submit access request
    // Wait for auto-approve (no toolsets), Login button ready
  });

  await test.step('Complete OAuth login and token exchange', async () => {
    // Click Login, handle Keycloak login (if no active session)
    // Wait for redirect to /dashboard
  });

  await test.step('Verify token received and user info accessible', async () => {
    // Assert access token present and valid (>100 chars)
    // Use REST client section: GET /bodhi/v1/user with Bearer token
    // Assert auth_status: "logged_in", correct username, role type "exchanged_token"
  });
});

test('unauthenticated API access returns logged_out status', async ({ page }) => {
  // Use REST client section without token to GET /bodhi/v1/user
  // Assert auth_status: "logged_out"
});
```

#### Migrated: `toolsets-auth-restrictions.spec.mjs`

```javascript
// Session Auth describe block
test('session auth: configure toolset → list returns toolset_types', async ({ page }) => {
  await test.step('Login and configure Exa toolset', async () => { /* ... */ });
  await test.step('Verify GET /toolsets returns toolset_types', async () => { /* ... */ });
});

// OAuth Token + Toolset Scope Combinations describe block
test('Case 1+2: WITH toolset scope approved → verify list + execute with/without scope', async ({ page }) => {
  // Combines Cases 1 and 2 into one test (same setup, different scope configs)

  await test.step('Phase 1: Session login and configure Exa toolset', async () => {
    // Login, configure toolset, get toolset UUID
  });

  await test.step('Phase 2: Submit access request with toolsets → draft → approve', async () => {
    // Navigate to test app, fill config, submit
    // Redirect to review page, approve with toolset instance
    // Wait for /access-callback, verify scopes displayed
  });

  await test.step('Phase 3: OAuth login WITH access_request_scope', async () => {
    // Click Login on /access-callback (scopes include access_request_scope)
    // Wait for token exchange, navigate to /dashboard
  });

  await test.step('Phase 4: Verify toolset list returns approved toolset', async () => {
    // GET /bodhi/v1/toolsets → assert toolset in list
    // Assert toolset_types contains exa config
  });

  await test.step('Phase 5: Verify toolset execution succeeds with scope', async () => {
    // POST /bodhi/v1/toolsets/{id}/execute/search → assert 200
    // Assert result contains search results
  });
});

test('Case 2: WITH toolset scope + remove access_request_scope → execute denied', async ({ page }) => {
  await test.step('Phase 1: Session login, configure toolset, get UUID', async () => { /* ... */ });

  await test.step('Phase 2: Access request with toolsets → approve', async () => { /* ... */ });

  await test.step('Phase 3: Remove access_request_scope, then OAuth login', async () => {
    // On /access-callback: read scopes, modify scope field (remove access_request_scope)
    // Click Login, wait for token exchange
  });

  await test.step('Phase 4: Verify list succeeds but execute fails', async () => {
    // GET /toolsets → assert toolset visible in list (user owns it)
    // POST /toolsets/{id}/execute → assert NOT 200 (denied without scope)
  });
});

test('Case 3: auto-approved + inject fake scope → invalid_scope error', async ({ page }) => {
  await test.step('Session login and configure OAuth', async () => { /* ... */ });
  await test.step('Submit auto-approved request, inject non-existent scope', async () => {
    // On config page: inject scope_ar_nonexistent into scope field
  });
  await test.step('Verify Keycloak returns invalid_scope error', async () => { /* ... */ });
});

test('Case 4: auto-approved + standard scopes → empty toolsets list', async ({ page }) => {
  await test.step('Complete OAuth flow without toolset scope', async () => { /* ... */ });
  await test.step('Verify toolsets list empty, toolset_types present', async () => { /* ... */ });
});

test('OAuth token blocked from session-only CRUD endpoints', async ({ page }) => {
  await test.step('Complete OAuth flow (auto-approved)', async () => { /* ... */ });
  await test.step('Verify GET /toolsets/{id} returns 401', async () => { /* ... */ });
  await test.step('Verify PUT /toolsets/{id} returns 401', async () => { /* ... */ });
});
```

#### New: `oauth-chat-streaming.spec.mjs`

```javascript
test('3rd-party app: OAuth token → streaming chat completion', async ({ page }) => {
  await test.step('Complete OAuth flow (auto-approved, no toolsets)', async () => {
    // Navigate to test app, fill config, submit access request
    // Auto-approved → click Login → Keycloak → /dashboard
  });

  await test.step('Verify models list accessible', async () => {
    // Dashboard chat section: model selector populated
    // Assert at least one model available
  });

  await test.step('Send message and verify streaming response', async () => {
    // Select model, type message, click send
    // Wait for streaming to complete (extended timeout for model warm-up)
    // Assert assistant response non-empty
    // Assert streaming status transitions: idle → streaming → idle
  });
});
```

#### New: `oauth-api-testing.spec.mjs` (optional, validates dashboard sections)

```javascript
test('3rd-party app: full API journey → user info + toolsets + REST client', async ({ page }) => {
  await test.step('Complete OAuth flow with toolset scope', async () => {
    // Full access request + approval + OAuth flow
  });

  await test.step('Verify user info section', async () => {
    // Click fetch → assert user_id, username, role type
  });

  await test.step('Verify toolsets section: list + execute', async () => {
    // Click list → assert toolset in results
    // Select toolset, fill params, execute → assert success
  });

  await test.step('Verify REST client: custom API call', async () => {
    // Configure GET /v1/models with auto-token
    // Send → assert 200, models array in response
  });
});
```

**Commit**: Phase 4 complete — all E2E tests migrated + new specs with test.step

---

### Phase 5: Infrastructure Changes + Cleanup

**Sub-agent type**: `general-purpose`
**Goal**: Remove old infrastructure, verify full test suite passes
**Success criteria**: `cd crates/lib_bodhiserver_napi && npx playwright test --reporter=list` — all tests pass (49+ tests)

**Sub-agent context**:
- Phase 4 already updated Playwright config to serve React app
- This phase only removes dead files and the old page object
- Must verify NO other test files import from removed files (grep before deleting)

**Modify**:
- `playwright.config.mjs` — replace static server webServer entry:
  ```javascript
  {
    command: 'cd test-oauth-app && npm run build && npx serve dist -s -l 55173',
    url: 'http://localhost:55173/ping.txt',
    reuseExistingServer: false,
    timeout: 30000,
  }
  ```

**Delete**:
- `tests-js/test-pages/oauth-test-app.html`
- `tests-js/test-pages/xhr-checks.html` (unused — grep confirmed no references)
- `tests-js/test-pages/ping.txt` (moved to test-oauth-app/public/)
- `tests-js/pages/OAuth2TestAppPage.mjs` (replaced by TestAppPage)
- `tests-js/scripts/serve-test-pages.mjs` (no longer needed — React app self-serves)
- `tests-js/test-pages/` directory (empty after cleanup)

**Verify**: `cd crates/lib_bodhiserver_napi && npx playwright test --reporter=list` — all tests pass

**Commit**: Phase 5 complete — old infrastructure removed, clean cut

---

## Critical Files Reference

### Files to Read (existing, for implementation reference)
- `tests-js/test-pages/oauth-test-app.html` — OAuth flow implementation, PKCE, state machine
- `tests-js/pages/OAuth2TestAppPage.mjs` — current page object, all selectors/methods
- `tests-js/specs/oauth/oauth2-token-exchange.spec.mjs` — OAuth test flows
- `tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs` — toolset auth test matrix
- `tests-js/scripts/start-shared-server.mjs` — server startup pattern
- `playwright.config.mjs` — webServer configuration
- SDK test app (reference for UI patterns):
  - `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhi-browser/sdk-test-app/web/src/App.tsx`
  - `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhi-browser/sdk-test-app/web/vite.config.ts`
  - `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhi-browser/sdk-test-app/web/package.json`
  - `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhi-browser/sdk-test-app/e2e/tests/pages/`

### API Endpoints Used by Test App
| Endpoint | Auth | Purpose |
|----------|------|---------|
| `POST /bodhi/v1/apps/request-access` | None | Create access request |
| `GET /bodhi/v1/apps/access-requests/{id}?app_client_id=` | None | Check access request status |
| `GET /bodhi/v1/user` | Optional | Get user info |
| `GET /bodhi/v1/toolsets` | OAuth/Session | List toolsets |
| `POST /bodhi/v1/toolsets/{id}/execute/{method}` | OAuth/Session | Execute toolset |
| `POST /v1/chat/completions` | OAuth/Session/Token | Streaming chat |
| `GET /v1/models` | OAuth/Session/Token | List models |
| Keycloak token endpoint | N/A | Exchange auth code for token |

### Keycloak Configuration
- App client redirect URI: `http://localhost:55173/callback` (user adds manually)
- Existing URI `http://localhost:55173/oauth-test-app.html` can be removed after migration

## Verification

### Per-Phase Verification
1. **Phase 1**: `cd test-oauth-app && npm run dev` → manual OAuth flow against `http://localhost:51135`
2. **Phase 2**: Manual — each API section against running BodhiApp
3. **Phase 3**: Manual — streaming chat with real LLM
4. **Phase 4**: `npx playwright test tests-js/specs/oauth/ tests-js/specs/toolsets/ --reporter=list` — all migrated tests pass
5. **Phase 5**: `npx playwright test --reporter=list` — full suite passes (49+ tests)

### End-to-End Verification
```bash
cd crates/lib_bodhiserver_napi
npx playwright test --reporter=list
# Expected: All tests pass, including new oauth-chat-streaming spec
```
