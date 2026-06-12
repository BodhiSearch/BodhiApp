# OAuth Test App: React+Vite+Tailwind Replacement — Exploration & Plan

## Objective

BodhiApp's E2E test suite uses `oauth-test-app.html` — a single HTML+JavaScript file — as a stand-in for a 3rd-party OAuth application. It handles access request creation, PKCE-based OAuth2 authorization, token exchange, and API calls. This file has grown organically and is hard to extend.

Your job is to **explore the existing test app, the reference SDK test app, and the BodhiApp APIs**, then produce a **phased plan** to:
- Replace `oauth-test-app.html` with a proper **React + Vite + Tailwind CSS** application
- Design it as a realistic **3rd-party application** (not a raw form), with an **Advanced Options panel** for testing variations
- Cover **full API testing**: authentication, access requests, toolset listing/execution, streaming chat, and generic API calls
- Migrate all E2E tests that use the current test app to work with the new one
- This is a **clean cut** — no backwards compatibility with the HTML file. We accept the cost of migrating E2E tests. Keycloak redirect URIs will be updated as needed.

**Output**: Write your findings and plan to files in `ai-docs/claude-plans/20260215-e2e-cleanup/`.

---

## Exploration Scope

### What to explore deeply

**1. Current oauth-test-app.html — Flow & State Machine**

**File**: `crates/lib_bodhiserver_napi/tests-js/test-pages/oauth-test-app.html`

Understand the complete flow this file implements. It is the primary reference for what the new app must support:

- **Access Request Phase**: `POST /bodhi/v1/apps/request-access` with `{ app_client_id, flow_type, redirect_url, requested: { toolset_types } }`. Two outcomes: auto-approved (scopes returned immediately) or draft (redirect to review page).
- **Access Request Review Phase**: When draft, the user is redirected to BodhiApp's review page (`/ui/apps/request-access/{id}`). After approval, BodhiApp redirects back to the test app with `?id={accessRequestId}`. The test app then fetches the access request status to get resolved scopes (`resource_scope`, `access_request_scope`).
- **OAuth Login Phase**: PKCE challenge generation, redirect to Keycloak auth endpoint with resolved scopes, Keycloak login/consent, redirect back with `?code=&state=`.
- **Token Exchange Phase**: `POST` to Keycloak token endpoint with authorization code + PKCE verifier. Display access token.
- **State persistence**: `sessionStorage.oauthConfig` preserves config across the multi-redirect flow.
- **Error handling**: `?error=invalid_scope` detection when Keycloak rejects unknown scopes.
- **UI state machine**: config-section → access-request-loading → [access-callback-loading] → loading-section → success-section (or error-section).
- **Confidential vs public client** support (client secret toggle, PKCE for public clients only).

**2. OAuth2TestAppPage Page Object — E2E Test Requirements**

**File**: `crates/lib_bodhiserver_napi/tests-js/pages/OAuth2TestAppPage.mjs`

This page object defines how E2E tests interact with the test app. The new React app must support equivalent interactions. Key methods to understand:

- `configureOAuthForm(...)` — What parameters are set and how
- `submitAccessRequest()` / `waitForAccessRequestRedirect()` / `waitForAccessRequestCallback()` — The multi-redirect dance
- `waitForLoginReady()` — How the test knows the app is ready for OAuth login
- `setScopes(value)` — Mid-flow scope manipulation (critical for testing scope variations)
- `getAccessRequestScope()` / `getResourceScope()` — How tests extract resolved scopes
- `clickLogin()` / `waitForTokenExchange()` / `getAccessToken()` — OAuth completion
- `expectOAuthError()` — Error flow testing
- `handleLogin()` / `handleConsent()` — Keycloak interaction (via Keycloak's own UI, not the test app)

Map each method to what the new React app needs to expose for Playwright automation.

**3. E2E Spec Files Using the Test App**

Two spec files use the OAuth test app. Read them to understand every flow variation:

**File**: `crates/lib_bodhiserver_napi/tests-js/specs/oauth/oauth2-token-exchange.spec.mjs`
- Simple OAuth2 flow (auto-approved access request, no toolsets)
- Token exchange + GET /user verification
- Error handling (unauthenticated → logged_out)

**File**: `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs`
- Complex multi-phase: session login → configure toolsets → OAuth flow → API verification
- **Case 1**: Access request WITH toolsets → approve on review page → OAuth with access_request_scope → list+execute toolsets
- **Case 2**: Same setup but remove access_request_scope before login → execute denied
- **Case 3**: Auto-approved (no toolsets) + inject non-existent scope → Keycloak invalid_scope error
- **Case 4**: Auto-approved + standard scopes → empty toolsets list
- **Session-only tests**: OAuth token → GET/PUT /toolsets/{id} → 401
- **Session auth test**: Session cookie → GET /toolsets → returns toolset_types

Note how tests share `beforeEach` setup (server + static server) and how some tests combine session auth (for setup) with OAuth auth (for verification).

**4. Reference: SDK Test App — UI Patterns & Configurability**

**Location**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhi-browser/sdk-test-app`

This is a **reference for UI patterns only**. The new test app will NOT import bodhi-js-sdk. The SDK's access-request flow is stale (old API version). Use this app as inspiration for:

- **Section-based layout**: How SetupSection, ApiTestSection, ChatSection, and Ext2ExtTestSection organize different testing concerns on one page
- **Component structure**: How React components, hooks, and shadcn UI compose a test interface
- **Configuration via UI**: How the setup section exposes configurable parameters
- **Streaming UI**: How ChatSection implements SSE streaming display
- **Tech stack patterns**: Vite config, Tailwind setup, TypeScript usage, project structure

Key files to explore:
- `sdk-test-app/web/src/App.tsx` — Root component, section layout
- `sdk-test-app/web/src/app/components/` — All UI sections
- `sdk-test-app/web/src/app/hooks/` — Data fetching hooks (useChatModels, useChatToolsets, useAgenticChat)
- `sdk-test-app/web/vite.config.ts` — Build configuration
- `sdk-test-app/web/package.json` — Dependencies and tech stack
- `sdk-test-app/e2e/tests/pages/` — Page Object Model patterns for React test apps

**Do NOT** use the SDK's OAuth/access-request implementation as a guide for the new test app — it uses an outdated request-access API. Instead, implement the flows based on what `oauth-test-app.html` does (which reflects the current BodhiApp API).

**5. BodhiApp APIs to Support**

The new test app should cover full API testing as a 3rd-party app. Explore these endpoints:

- **Access request**: `POST /bodhi/v1/apps/request-access` — Understand the request/response format, draft vs approved flows
- **User info**: `GET /bodhi/v1/user` — What fields are returned for OAuth vs session auth
- **Toolset listing**: `GET /bodhi/v1/toolsets` — How OAuth scope filtering works (toolsets vs toolset_types in response)
- **Toolset execution**: `POST /bodhi/v1/toolsets/{id}/execute/{method}` — How access_request_id in token enables execution
- **Chat completions**: `POST /v1/chat/completions` — OpenAI-compatible streaming endpoint
- **Models listing**: `GET /v1/models` — Available models

Explore the route definitions in `crates/routes_app/src/routes.rs` to understand which endpoints accept OAuth tokens (`user_or_token_apis`) vs session-only (`user_session_apis`).

**6. Static Server & Test Infrastructure**

**File**: `crates/lib_bodhiserver_napi/tests-js/utils/static-server.mjs`

Understand the current Express.js static server that serves the test app on port 55173. The new React app will be served via `npm run dev` (Vite dev server) or `npx serve dist/` — both support client-side routing (React Router), unlike a plain static file server which only serves the root. The serving mechanism must support multi-path navigation (e.g., `/callback`, `/api-test`, etc.) with proper SPA fallback.

**7. Keycloak Redirect URI Configuration**

**File**: `crates/lib_bodhiserver_napi/tests-js/CLAUDE.md`

Understand the pre-configured app client redirect URIs:
- `http://localhost:51135/ui/auth/callback` (Bodhi server)
- `http://localhost:55173/oauth-test-app.html` (current static OAuth test app)

The new React app will use a different callback path (e.g., `http://localhost:{port}/callback`). Keycloak redirect URIs will need updating. Document what changes are needed.

---

## Decisions Already Made

These decisions have been made through discussion. Follow them in your plan:

| Decision | Choice |
|----------|--------|
| Tech stack | React + Vite + Tailwind CSS + TypeScript |
| UI library | shadcn UI (follow sdk-test-app pattern) |
| Location | `crates/lib_bodhiserver_napi/test-oauth-app/` (separate package) |
| SDK dependency | **None** — implement OAuth/access-request flows independently. SDK is reference for UI patterns only. |
| API coverage | Full: auth, access requests, toolsets, streaming chat, generic API calls |
| Test variation support | **Advanced Options panel** — collapsible section with editable fields for scopes, toolsets, client ID, auth server URL, etc. |
| App identity | Behaves like a real 3rd-party app by default (not a raw form). "Login with Bodhi" landing → authenticated dashboard with API sections. |
| Serving mechanism | `npm run dev` (Vite) or `npx serve dist/` — must support client-side routing for callback paths |
| Backwards compatibility | **None** — clean cut. All E2E tests will be migrated to new page object + new app. |
| Keycloak changes | Will update redirect URIs for the new callback path. Not a blocker. |
| E2E test migration | In scope. New page object must support all flows the current `OAuth2TestAppPage` supports. |

---

## Constraints

- The new test app is in the BodhiApp repo, NOT in the bodhi-browser repo.
- No dependency on bodhi-js-sdk packages. Implement OAuth/access-request flows directly.
- E2E tests use Playwright. The new React app must expose `data-testid` attributes for reliable test automation (follow project convention — see `crates/bodhi/src/` for patterns).
- The test app serves alongside BodhiApp server during E2E tests. BodhiApp runs on port 51135. The test app needs its own port.
- All E2E tests run sequentially (`workers: 1`) due to shared server port.
- The access request review page is BodhiApp's own UI (`/ui/apps/request-access/{id}`). The test app redirects TO it and receives a redirect BACK from it. This multi-redirect dance must work with Playwright's page navigation.
- Chat-related E2E tests (`chat.spec`, `chat-agentic`, `chat-toolsets`) are **out of scope** — they test the main BodhiApp UI, not the 3rd-party test app.

---

## What Your Output Should Look Like

### File 1: Exploration Findings

A thorough analysis of:
- Every E2E test flow that uses the current test app, with the specific sequence of actions and assertions
- What the new React app must support for each flow (mapped from page object methods)
- UI section breakdown — what sections the app needs and what each does
- OAuth/access-request implementation requirements (based on current HTML, NOT the SDK)
- BodhiApp API endpoint inventory for 3rd-party app usage (which endpoints, what auth, what responses)
- Streaming chat implementation requirements (SSE handling in React)
- Keycloak configuration changes needed
- Serving mechanism details for E2E integration

### File 2: Phased Implementation Plan

A detailed phase-by-phase plan with:
- **Phase 1**: React app scaffolding + OAuth/access-request core flow (no E2E migration yet)
- **Phase 2**: API testing sections (toolsets, user info, generic API panel)
- **Phase 3**: Streaming chat section
- **Phase 4**: Advanced Options panel + all test variation support
- **Phase 5**: New page object + E2E test migration
- **Phase 6**: Keycloak config updates + test infra changes (static server → Vite serve)
- **Phase 7**: Cleanup — remove old HTML file, update documentation

Each phase should specify: what files are created/modified, what to test, what to commit.

### File 3 (optional): E2E Test Migration Matrix

Map each test case in `oauth2-token-exchange.spec.mjs` and `toolsets-auth-restrictions.spec.mjs` to:
- What page object methods it calls
- What the new page object needs to support
- Any changes to the test logic itself

---

## Important Notes

- Be thorough in exploration. Read actual code, not just file names.
- The current `oauth-test-app.html` is the primary reference for OAuth/access-request flows — it reflects the current BodhiApp API. The SDK's implementation is outdated.
- The sdk-test-app is only a reference for UI patterns (sections, components, hooks, configuration). Do not use its OAuth implementation.
- Pay attention to the multi-redirect flow: test app → BodhiApp review page → test app → Keycloak → test app. State must survive across these redirects.
- The Advanced Options panel is for E2E test automation — tests interact with it via Playwright to change scopes, toolset requests, etc. mid-flow.
- The app should look like a real 3rd-party app, not a developer tool. Think "Login with Bodhi" button, authenticated dashboard, clean sections — with advanced options tucked away.
- Reference specific file paths in your findings.
- Each phase must be independently committable and leave all tests passing (E2E tests may be temporarily broken during migration phases — document which phases have this constraint).
