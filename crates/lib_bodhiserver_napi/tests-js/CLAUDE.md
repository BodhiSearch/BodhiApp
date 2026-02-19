# CLAUDE.md - E2E Tests (Playwright)

_For detailed test writing patterns, anti-patterns, and conventions, see [E2E.md](E2E.md)._

## Purpose

Playwright E2E tests that validate **user journeys** through the Bodhi App. NAPI bindings embed the Rust server directly into Node.js so tests run against a real server without Docker or external processes. Tests are written in JavaScript/MJS.

These are user journey tests, NOT unit tests. Each test covers a complete flow: login, perform actions, verify results. See [E2E.md](E2E.md) for the test philosophy and how to write new specs.

## Core Testing Rules

**Strict black-box testing**: E2E tests MUST interact only through UI components. Never use `page.evaluate()` + `fetch()` to call APIs directly as a substitute for UI interaction. Test assertions must observe results through the UI, not by calling internal APIs. The only exception is `page.evaluate()` fetch calls in McpsPage's setup helper methods (`createAuthHeaderViaApi`, `createOAuthConfigViaApi`, `discoverMcpEndpointsViaApi`, `dynamicRegisterViaApi`) which cover backend operations that have no UI equivalent -- these are setup-only and must include `resp.ok` checks. All assertion steps must still use UI interactions.

**No inline timeouts**: Do not use `page.waitForTimeout()`, `setTimeout()`, or fixed waits in tests. Instead, wait for UI element state changes using `data-testid`, `data-test-state`, or other `data-test*` attributes via Playwright's built-in waiting (e.g., `expect(locator).toBeVisible()`, `page.waitForURL()`). Inline timeouts hide actual bugs and make tests flaky. The global `PLAYWRIGHT_TIMEOUT` env var handles slow CI infrastructure.

## Running Tests

All commands run from the `crates/lib_bodhiserver_napi/` package root:

| Command                             | Purpose                        |
| ----------------------------------- | ------------------------------ |
| `npm run test:playwright`           | Headless run (CI default)      |
| `npm run test:playwright:headed`    | Visible browser                |
| `npm run test:playwright:ui`        | Playwright UI mode             |
| `npm run test:playwright:scheduled` | Local-only token refresh tests |

Tests run sequentially (`workers: 1`, `fullyParallel: false`) to avoid port conflicts. The `@scheduled` tag marks long-running token refresh debugging tests that are excluded by default via `grepInvert: /@scheduled/` in `playwright.config.mjs`.

## Environment and Credentials

Tests load OAuth credentials from `.env.test` (see `.env.test.example` for required variables).

### Resource Client

The pre-configured resource client (`INTEG_TEST_RESOURCE_CLIENT_ID`, `INTEG_TEST_RESOURCE_CLIENT_SECRET`) must have:

- **Direct Access Grants** enabled in Keycloak (used by `getResourceUserToken()` with `grant_type: 'password'`)
- `user@email.com` assigned as **resource admin** on the client
- `manager@email.com` assigned as **resource manager** on the client

This allows tests to start the Bodhi server in `ready` mode without dynamic client creation.

### App Client

The pre-configured app client (`INTEG_TEST_APP_CLIENT_ID`) is a public OAuth client in Keycloak with redirect URIs pre-configured for:

- `http://localhost:51135/ui/auth/callback` (Bodhi server)
- `http://localhost:55173/callback` (React OAuth test app)

Since the app client is reused across tests with the same user, Keycloak remembers prior consent -- `handleConsent()` is NOT needed. Tests with an active KC session (from prior `performOAuthLogin()`) also skip `waitForAuthServerRedirect()` since Keycloak auto-redirects instantly.

### Realm Admin

`INTEG_TEST_REALM_ADMIN` and `INTEG_TEST_REALM_ADMIN_PASS` are used by `AuthServerTestClient` for dedicated-server tests that need dynamic resource client creation and role assignment.

### Test Users

| User                | Env Vars                                                                       | Role on Resource Client |
| ------------------- | ------------------------------------------------------------------------------ | ----------------------- |
| `user@email.com`    | `INTEG_TEST_USERNAME`, `INTEG_TEST_USERNAME_ID`, `INTEG_TEST_PASSWORD`         | Resource admin          |
| `manager@email.com` | `INTEG_TEST_USER_MANAGER`, `INTEG_TEST_USER_MANAGER_ID`, `INTEG_TEST_PASSWORD` | Resource manager        |

Multi-user tests (e.g., token isolation, user management) require both users set up on the resource client.

### Fixed Ports

| Service                    | Port    |
| -------------------------- | ------- |
| Bodhi server               | `51135` |
| React OAuth test app       | `55173` |
| Test MCP OAuth server      | `55174` |
| Test MCP OAuth server (DCR)| `55175` |

## Architecture Overview

### Directory Layout

| Directory   | Contents                                                                                                 |
| ----------- | -------------------------------------------------------------------------------------------------------- |
| `specs/`    | 25 test files across 11 domain folders                                                                   |
| `pages/`    | 28 page objects extending `BasePage`                                                                     |
| `fixtures/` | 8 test data modules (classes with static methods)                                                        |
| `utils/`    | `auth-server-client.mjs`, `bodhi-app-server.mjs`, `browser-with-extension.mjs`, `mock-openai-server.mjs` |
| `scripts/`  | `start-shared-server.mjs` (shared server startup)                                                        |
| `data/`     | Test GGUF model refs for model-metadata tests                                                            |

### McpsPage and MCP Auth Config E2E

**McpsPage** (`pages/McpsPage.mjs`): Refactored for MCP Auth Config Redesign. Removed inline auth form methods (header key/value, OAuth client fields). Added dropdown methods for selecting existing auth configs. Auth configs are created via API helper methods in test setup before navigating to the MCP form. All API endpoints use the unified `/bodhi/v1/mcps/` prefix.

**API helper methods for test setup**: Some test setup steps (creating auth configs, discovering OAuth endpoints, DCR registration) have no UI equivalent -- they represent backend operations that the UI calls internally. McpsPage provides `page.evaluate()` + `fetch()` wrappers for these: `createAuthHeaderViaApi(serverId, {...})`, `createOAuthConfigViaApi(serverId, config)`, `discoverMcpEndpointsViaApi(mcpServerUrl)`, `dynamicRegisterViaApi({...})`. These are **setup-only helpers** -- the actual test assertions must still go through the UI. All auth config creation uses unified `POST /bodhi/v1/mcps/auth-configs` with discriminated `type` field. See `mcpFixtures.mjs` and MCP specs for usage.

**Important**: The `resp.ok` check is mandatory in every `page.evaluate()` fetch call. Always include `if (!resp.ok) throw new Error(\`HTTP \${resp.status}: \${await resp.text()}\`)` before `resp.json()` so failures surface immediately with a clear error message rather than silently returning unexpected data.

### Key Mechanisms

- **Path alias**: `@/` maps to `tests-js/` (e.g., `import { test } from '@/fixtures.mjs'`)
- **Shared server**: Auto-started by Playwright `webServer` config on port 51135 via `scripts/start-shared-server.mjs`
- **Auto DB reset**: `@/fixtures.mjs` extends Playwright's `test` with an `autoResetDb` fixture that calls `POST /dev/db-reset` before each test
- **Page Object Model**: All page objects extend `BasePage` from `@/pages/BasePage.mjs`

## Server Patterns

- **Shared server** (default): For app state=ready with `user@email.com` as admin. Import `test` and `expect` from `@/fixtures.mjs` to get auto DB reset.
- **Dedicated server**: For non-ready app state (setup flow), multi-user scenarios, or custom config (e.g., overridden HF_HOME). Import from `@playwright/test` directly and manage server lifecycle with `createServerManager()` in `beforeAll`/`afterAll`.

See [E2E.md](E2E.md) "Server Configuration Decision Tree" for the full decision criteria.

## Known Quirks

### Model Selection Before API Token

Select the model FIRST, then set the API token in chat settings. Model selection triggers a React re-render that clears the token input.

```js
// Correct:
await chatSettings.selectModelQwen();
await chatSettings.setApiToken(true, token);

// Wrong (token cleared by re-render):
await chatSettings.setApiToken(true, token);
await chatSettings.selectModelQwen();
```

### OAuth Flow Variants

Tests WITH an active KC session (from `loginPage.performOAuthLogin()`):

```js
await testAppPage.config.clickLogin();
await testAppPage.oauth.waitForTokenExchange(testAppUrl); // KC auto-redirects
```

Tests WITHOUT a prior KC session (e.g., `oauth2-token-exchange`):

```js
await testAppPage.config.clickLogin();
await testAppPage.oauth.waitForAuthServerRedirect(authServerConfig.authUrl);
await testAppPage.oauth.handleLogin(username, password);
await testAppPage.oauth.waitForTokenExchange(testAppUrl);
```

For draft/review flows (access-callback page):

```js
await testAppPage.oauth.waitForAccessRequestCallback(testAppUrl);
await testAppPage.accessCallback.waitForLoaded();
await testAppPage.accessCallback.clickLogin();
```

### Other Quirks

- **SPA navigation**: Always call `waitForSPAReady()` after navigation. Use `page.waitForURL()` for route changes, not `page.goto()` + immediate assertion.
- **Page object lifecycle**: Create page objects in `beforeEach`, NOT `beforeAll`. Playwright's `page` is per-test.
- **KC session persistence**: Consent auto-skipped on reused app client. `waitForAuthServerRedirect()` skipped when KC session exists from earlier login.
- **Toast handling**: Tech debt -- 5+ methods in BasePage with fallbacks. Use `waitForToast` for critical assertions only; `waitForToastOptional` for non-critical confirmations.
- **Flakiness sources**: Toast appearance timing, KC redirect timing, LLM model loading variability. Never add inline timeouts -- rely on the config-level `PLAYWRIGHT_TIMEOUT`.

## E2E vs server_app Testing Boundary

E2E tests validate:

- External auth service (Keycloak) behavior (error responses, consent, scope validation)
- UI wiring -- that the UI is plugged in properly for user journeys
- Browser-dependent flows (background tab token refresh, multi-user contexts)

Tests that only validate our code's behavior given auth state should be in server_app using `ExternalTokenSimulator`. The server_app OAuth test infrastructure (Phase 1) is in place but the toolset auth and user info test migration (Phases 3-4) was reverted -- stubbed tokens hide token exchange complexity that needs real Keycloak behavior for 3rd-party app OAuth flows.

## CI Considerations

- `HEADLESS=true` set in `test:playwright` npm script
- Environment variables from GitHub repo vars/secrets (`.env.test`)
- `PLAYWRIGHT_TIMEOUT` env var for slower CI infrastructure (default: 120000ms)
- `@scheduled` tests excluded in CI -- local-only for token refresh debugging
