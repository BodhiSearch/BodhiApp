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
- `http://localhost:55173/oauth-test-app.html` (static OAuth test app)

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
| Static OAuth test app | `55173` |

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

### OAuth Flow Patterns

Tests with an active KC session (from `loginPage.performOAuthLogin()`):
```js
await oauth2TestAppPage.clickLogin();
await oauth2TestAppPage.waitForTokenExchange(testAppUrl);  // KC auto-redirects
```

Tests WITHOUT a prior KC session (e.g., `oauth2-token-exchange`):
```js
await oauth2TestAppPage.clickLogin();
await oauth2TestAppPage.waitForAuthServerRedirect(authServerConfig.authUrl);
await oauth2TestAppPage.handleLogin(username, password);
await oauth2TestAppPage.waitForTokenExchange(testAppUrl);
```
