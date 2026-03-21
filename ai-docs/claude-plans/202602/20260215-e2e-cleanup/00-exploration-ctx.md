# E2E Cleanup: Discussion Context & User Preferences

This document captures the full context from the discussion that led to `00-exploration-prompt.md`. It includes user preferences, reasoning behind decisions, constraints discovered during exploration, and nuances that may not be fully captured in the prompt itself.

---

## Background & Motivation

The OAuth test app (`oauth-test-app.html`) evolved organically and remained plain HTML+JavaScript even as the flow it orchestrates became complex (access request creation, admin review, PKCE OAuth, token exchange). The user wants to modernize it.

The E2E test suite was "convenient" - tests were written at the E2E level because it was easy, not because they needed to be there. The user explicitly framed this as a **test pyramid problem**: too many tests at the top (E2E), not enough at lower levels (routes_app, server_app).

The user's mental model of the test levels:
- **routes_app**: "unit level like test for api" - has the actual router, but tests only a single API call using `router.oneshot`. Setup services, call the API, assert on response.
- **server_app**: "a bit integration test like" - starts the server, has multi-API interactions.
- **E2E (lib_bodhiserver_napi)**: Full browser-based tests with Playwright against real Keycloak.

The user specifically noted tests like "creating tokens and invoking toolset endpoints etc. those can be handled setting up services properly then api endpoint to get 403" - meaning authorization tests that return 401/403 don't need E2E, they just need the right service setup + a single API call.

---

## Key Decisions with Reasoning

### 1. OAuth scope-based filtering: "Move to server_app via cache bypass"

The user's insight: server_app has a constraint - no browser/Playwright. But the token validation pipeline has a cache layer. If we can pre-populate the cache with a synthetic token exchange result, we can simulate 3rd-party OAuth tokens without needing Keycloak's browser-based login flow.

**User's exact words**: "we can simulate session, insert into cache and simulate 3rd party token exchange result, but we have not tried it yet, we can note these constraint and ask to have 3rd party test token inserted into cache service so it does not go to auth server for token exchange and we can simulate 3rd party app api calls"

**User also asked**: "have the prompt to have dedicated look at auth_middleware.rs and token_service.rs to use cache server behaviour to have 3rd party oauth tokens pass through and test api endpoints"

This is exploratory - it hasn't been validated yet. The agent should document feasibility, not assume it works.

### 2. Token exchange tests: "Preference for exploring moving to server_app"

The user prefers exploring the cache bypass approach for token exchange tests too, rather than keeping them in E2E. This is aspirational - if the cache bypass works, many OAuth-dependent tests could move down.

### 3. Multi-user flows: "Keep both in E2E"

Both `multi-user-request-approval-flow.spec.mjs` and `list-users.spec.mjs` stay in E2E. The user values testing "user sees they are logged out" and "user re-authenticates with new role" - these are browser-observable behaviors that can't be tested with HTTP clients alone.

### 4. OAuth test app: "Create a Vite+React+Tailwind app with simple router, serve via npx serve"

**User's exact words**: "create a vite+react+tailwindcss based app with simple router and serve via npx serve to support redirects to path"

Key details:
- Separate package within `crates/lib_bodhiserver_napi/` (NOT shared with E2E test code)
- Both flows supported: access request + OAuth login, AND direct OAuth login
- React Router for route-based flow selection
- Modular routes: `/access-request` and `/oauth-login`
- Serve via `npx serve` to support client-side routing (SPA fallback)

### 5. App initializer tests: "Trim duplicates"

Remove `app-initializer.spec.mjs` from E2E because `AppInitializer.test.tsx` (Vitest) already covers the redirect logic. The user chose "Trim duplicates" and also agreed to move canonical redirect to server_app.

### 6. Toolset UI tests: "Stay in E2E"

Tests like "displays toolsets list page", "navigates to edit page", "shows admin toggle for resource_admin" stay in E2E. Even though they test UI rendering, they test it against a real server with real data - not something component tests with MSW can fully replicate.

### 7. Chat tests: "Out of scope"

Chat E2E tests are well-structured and test genuine browser UI interactions. Don't touch them in this effort.

### 8. Network IP tests: "Keep in E2E"

Tests real network behavior (0.0.0.0 binding, cross-IP compatibility) that can't be unit tested. Stay in E2E.

### 9. Phase ordering: "Migrate first"

The user chose to do test migrations first, then build the React app. Reasoning: reduce E2E test count first, then modernize the remaining ones with the new app.

---

## Constraints Discovered During Exploration

### E2E Test Infrastructure

- **25 spec files** total, ~50+ test cases
- **16 out of 26 tests** use identical resource client creation pattern
- Resource client creation: `createResourceClient(serverUrl)` + `makeResourceAdmin()` - always the same
- Each test creates its own server on a random port with fresh SQLite DB
- Tests use `test.beforeAll()` for setup, not per-test setup
- The `?live_test=true` parameter on Keycloak allows resource creation without admin token

### routes_app Test Infrastructure

- All tests use `router.oneshot(request)` - strictly single request/response
- Test utilities: `build_test_router()`, `create_authenticated_session()`, `session_request()`, `unauth_request()`
- Assertion helpers: `assert_auth_rejected()`, `assert_forbidden()`, `assert_auth_passed()`
- Uses `#[rstest]` with `#[case]` and `#[values]` for parameterized testing
- Mix of mock services (MockToolService) and real services (SQLite DB)
- Existing 401/403 coverage: comprehensive across all route modules

### server_app Test Infrastructure

- Full server with real Keycloak (credentials from env vars)
- Tests run serially (`#[serial_test::serial(live)]`) due to shared llama.cpp binary
- Uses `reqwest` HTTP client for API calls
- `setup_minimal_app_service()` creates resource client, assigns admin, starts server
- Multi-turn request patterns (create + verify + modify + verify)

### Token Validation Pipeline (preliminary)

- Bearer tokens validated in `auth_middleware.rs`
- `TokenService` checks cache first, then calls auth server for token exchange on cache miss
- Cache key is likely a token digest
- Pre-populating cache could bypass Keycloak - feasibility TBD

---

## Resource Client Reuse Analysis

From exploration:
- 16 tests use identical setup: `createResourceClient(serverUrl)` + `makeResourceAdmin(clientId, clientSecret, userId)` + `appStatus: 'ready'`
- 2 tests need custom user roles (list-users, multi-user-request-approval)
- 1 test creates a public app client (oauth2-token-exchange)
- 3 tests don't create any client (setup flow tests)

**Potential blocker**: redirect_uris are registered per client as `http://localhost:{port}/ui/auth/callback`. Each test uses a different port. A shared client would need either:
- A wildcard redirect URI
- Registration with multiple redirect URIs
- Or the Keycloak endpoint to support updating redirect URIs

This needs investigation before committing to the approach.

---

## User's Vision (Reading Between the Lines)

1. **Test pyramid discipline**: The user wants tests at the right level. E2E for journeys, unit for contracts.
2. **Reliability over speed**: E2E tests are flaky by nature. Moving tests lower reduces flakiness.
3. **Early failure detection**: Authorization bugs caught at routes_app level fail fast in `cargo test`, not after spinning up Keycloak + browser.
4. **Maintainability**: The HTML test app is hard to maintain. React + proper tooling is the long-term answer.
5. **Incremental migration**: Each phase commits independently. No big-bang refactor.
6. **Cache bypass is speculative**: The user knows this is unproven. Document feasibility first, implement later.
7. **The prompt should be directional, not prescriptive**: The user explicitly said "do not make your prompt prescriptive, asking it to do things exactly" - the agent should explore and propose, not follow a rigid script.
