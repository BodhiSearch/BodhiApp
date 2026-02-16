# CLAUDE.md - NAPI E2E Tests (Playwright)

## Pre-configured Test Credentials

The E2E tests use pre-configured OAuth credentials from `.env.test` (see `.env.test.example`).

### Resource Client

The pre-configured resource client (`INTEG_TEST_RESOURCE_CLIENT_ID`, `INTEG_TEST_RESOURCE_CLIENT_SECRET`) must have:
- **Direct Access Grants** enabled in Keycloak
- `user@email.com` assigned as **resource admin** on the client
- `manager@email.com` assigned as **resource manager** on the client

This allows tests to start the Bodhi server in `ready` mode without needing to create a resource client dynamically via the Keycloak admin API.

### App Client

The pre-configured app client (`INTEG_TEST_APP_CLIENT_ID`) is a public OAuth client in Keycloak with redirect URIs configured for:
- `http://localhost:51135/ui/auth/callback` (Bodhi server)
- `http://localhost:55173/callback` (React OAuth test app)

The redirect URI for the test app is also registered dynamically in `test.beforeAll` via `authClient.addRedirectUri()`.

Since the app client is reused across tests with the same user, Keycloak remembers prior consent — so `handleConsent()` is NOT needed. Tests that have an active KC session (from prior `performOAuthLogin()`) also skip `waitForAuthServerRedirect()` since Keycloak auto-redirects instantly.

### Test Users

| User | Env Vars | Role on Resource Client |
|---|---|---|
| `user@email.com` | `INTEG_TEST_USERNAME`, `INTEG_TEST_USERNAME_ID`, `INTEG_TEST_PASSWORD` | Resource admin |
| `manager@email.com` | `INTEG_TEST_USER_MANAGER`, `INTEG_TEST_USER_MANAGER_ID`, `INTEG_TEST_PASSWORD` | Resource manager |

Multi-user tests (e.g., token isolation, user management) require both users to be set up on the resource client with their respective roles.

## Fixed Ports

| Service | Port |
|---|---|
| Bodhi server | `51135` |
| React OAuth test app | `55173` |

Tests run sequentially (`workers: 1`) to avoid port conflicts.

## Common Patterns

### Chat Settings: Model Selection Before API Token

When setting both a model and an API token in chat settings, always select the model FIRST, then set the API token. Model selection triggers a React re-render that can clear the token input field.

```js
// Correct order:
await chatSettings.selectModelQwen();
await chatSettings.setApiToken(true, token);

// Wrong order (token gets cleared by model selection re-render):
await chatSettings.setApiToken(true, token);
await chatSettings.selectModelQwen();
```

### OAuth Flow Patterns (TestAppPage composite)

Tests with an active KC session (from `loginPage.performOAuthLogin()`):
```js
await testAppPage.config.clickLogin();
await testAppPage.oauth.waitForTokenExchange(testAppUrl);  // KC auto-redirects
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

### E2E vs server_app Testing Boundary

E2E tests validate:
- External auth service (Keycloak) behavior (error responses, consent, scope validation)
- UI wiring — that the UI is plugged in properly for user journeys
- Browser-dependent flows (background tab token refresh, multi-user contexts)

Tests that only validate our code's behavior given auth state should be migrated to
server_app using ExternalTokenSimulator. The server_app OAuth test infrastructure
(Phase 1 of the migration plan) is in place but the toolset auth and user info test
migration (Phases 3-4) was reverted — stubbed tokens hide token exchange complexity
that needs real Keycloak behavior for 3rd-party app OAuth flows.
