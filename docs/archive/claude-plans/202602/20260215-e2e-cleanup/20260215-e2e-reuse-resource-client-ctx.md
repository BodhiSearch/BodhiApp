# E2E Reuse Resource Client: Discussion Context & Decisions

This document captures the full exploration, Q&A, and decisions from the planning discussion for migrating NAPI E2E tests from dynamic resource client creation to pre-configured credentials from `.env.test`.

---

## Background & Motivation

The OAuth2 auth server migrated from `test-id.getbodhi.app` to `main-id.getbodhi.app`. The new server returns clients without Direct Access Grants enabled, causing a 400 error at token acquisition. This was already fixed for `server_app` (see `20260215-server-app-remove-register-resource.md`). The same fix is now needed for the NAPI E2E Playwright tests.

The `server_app` fix pattern: read pre-configured resource client credentials from `.env.test` instead of dynamically creating one via Keycloak admin API every test run.

---

## Exploration Findings

### Test Infrastructure Architecture

- **25 spec files** total in `crates/lib_bodhiserver_napi/tests-js/specs/`
- **18 test files** use `createResourceClient`, `makeResourceAdmin`, or `createAppClient`
- **Sequential execution**: `workers: 1`, `fullyParallel: false` to avoid port conflicts
- **Per-suite server**: each `test.describe` creates a fresh server with its own temp dir and SQLite DB
- **Env loading**: `playwright.config.mjs` loads `tests-js/.env.test` via dotenv

### Current Resource Client Creation Pattern

Most ready-state tests follow identical setup in `beforeAll`:
```js
const port = randomPort();
const serverUrl = `http://localhost:${port}`;
authClient = createAuthServerTestClient(authServerConfig);
resourceClient = await authClient.createResourceClient(serverUrl);
await authClient.makeResourceAdmin(resourceClient.clientId, resourceClient.clientSecret, testCredentials.userId);
serverManager = createServerManager({
  appStatus: 'ready',
  clientId: resourceClient.clientId,
  clientSecret: resourceClient.clientSecret,
  port, host: 'localhost',
  ...
});
```

### User Personas in Tests

- `user@email.com` (ec18e95b-...) — default test user, resource_admin in most tests
- `admin@email.com` (1f44fdb4-...) — resource_admin in list-users and multi-user tests
- `manager@email.com` (fa6c390f-...) — assigned `resource_manager` role dynamically
- `poweruser@email.com` (a5f9edfd-...) — assigned `resource_power_user` role dynamically

### App Client vs Resource Client

- **Resource client** = Bodhi server's own OAuth client (confidential, has secret, used for token exchange)
- **App client** = External app's OAuth client (public, no secret, used for user-facing OAuth flow)
- The `.env.test` has `INTEG_TEST_RESOURCE_CLIENT_ID` + `_SECRET` + `_SCOPE` and `INTEG_TEST_APP_CLIENT_ID`

### Key Files

- `tests-js/utils/auth-server-client.mjs` — `AuthServerTestClient` class with `createResourceClient()`, `makeResourceAdmin()`, `createAppClient()`, etc.
- `tests-js/test-helpers.mjs` — `createTestServer()`, `createFullTestConfig()`, `randomPort()`, `createTempDir()`
- `tests-js/utils/bodhi-app-server.mjs` — `BodhiAppServer` class for server lifecycle
- `tests-js/utils/static-server.mjs` — Express server for OAuth test app HTML pages
- `tests-js/utils/OAuth2ApiHelper.mjs` — Wrapper for auth client + API testing
- `tests-js/fixtures/publicHostFixtures.mjs` — Takes `resourceClient` as parameter, needs update
- `tests-js/fixtures/oauth2Fixtures.mjs` — Server config for setup-mode tests
- `tests-js/fixtures/setupFixtures.mjs` — Server config for setup flow tests

---

## Key Decisions with Reasoning

### 1. Token Refresh Test: Keep Dynamic Creation

**Decision**: `token-refresh-integration.spec.mjs` keeps its own dynamic resource client creation.

**Reasoning**: This test creates a resource client and then configures it with a 15-second access token lifespan via `configureClientTokenLifespan()` (Keycloak admin API). Cannot modify the shared pre-configured client's lifespan without affecting all other tests. No practical alternative.

### 2. Pre-configured Admin Role: Only user@email.com

**Decision**: Only `user@email.com` is pre-configured as `resource_admin` on the shared resource client.

**Consequence**: Tests that make `admin@email.com` the resource admin (list-users, multi-user) must keep dynamic resource client creation since `admin@email.com` is not admin on the pre-configured client.

### 3. list-users and multi-user: Keep Dynamic Creation

**Decision**: Both `list-users.spec.mjs` and `multi-user-request-approval-flow.spec.mjs` keep dynamic resource client creation.

**Reasoning**:
- They use `admin@email.com` as resource admin (not pre-configured on shared client)
- list-users assigns specific roles (manager, powerUser, user) to multiple test users via Keycloak API
- multi-user tests the access request approval workflow from scratch
- Test isolation requires fresh role state

### 4. toolsets-auth-restrictions: Use Pre-configured App Client

**Decision**: All 3 describe blocks use pre-configured resource client from env. OAuth flow blocks (2 and 3) also use pre-configured app client from env.

**Key insight**: Keycloak API calls for audience/scope registration are **idempotent**. Reusing the same app client across tests is safe — Keycloak won't fail on duplicate registrations.

**Critical consequence**: Reusing the same app client + scopes + user means **Keycloak remembers prior consent**. All `handleConsent()` calls must be removed from these tests. This is a known expected change.

### 5. oauth2-token-exchange: Migrate to Pre-configured Clients

**Decision**: `oauth2-token-exchange.spec.mjs` uses pre-configured resource client and app client.

**Impact**: The test currently starts in SETUP mode and goes through the full setup wizard. After migration:
- Starts in READY mode with pre-configured resource client
- Uses pre-configured app client (skips `createAppClient` via dev console)
- Skips setup wizard steps (welcome page, resource admin login)
- Tests OAuth flow directly: access request → login → token exchange → verify API access
- Remove `handleConsent()` (consent already given for reused app client)

### 6. Fixed Ports

**Decision**:
- Bodhi server: **port 51135** (matches server_app's live_server_utils.rs)
- Static OAuth test app: **port 55173**

**Redirect URIs** (must be pre-configured in Keycloak):
- Resource client: `http://localhost:51135/ui/auth/callback`
- App client: `http://localhost:55173/oauth-test-app.html`

**Risk**: Fixed port depends on proper server cleanup between tests. Mitigated by sequential execution (workers: 1).

---

## Implementation Approach Preferences

The user specified a particular implementation order:

### Phase 0: Utility Setup
Set up all helper functions (`getPreConfiguredResourceClient()`, `getPreConfiguredAppClient()`) and update `.env.test.example` first.

### Incremental Test Migration
Run **one test at a time** and fix if it fails. Do NOT batch all migrations and hope they pass.

### Priority Order
1. **First**: Tests that do NOT use oauth-test-app (only Bodhi App session login). Easier to debug.
2. **Then**: Tests that use oauth-test-app (static server + OAuth flow). More complex debugging.

### Debugging Strategy for Non-OAuth Tests
If a test fails after migration:
1. Use Claude in Chrome
2. Navigate to `http://localhost:51135` where Bodhi App is running
3. Reproduce the test steps manually
4. Identify and fix the issue

### Debugging Strategy for OAuth Test App Tests
If a test fails after migration:
1. Start the static server: `npm run test:static-server`
2. Navigate to it in browser
3. Input values as specified in the test
4. Server address: `http://localhost:51135`
5. Submit form
6. Fix test for any variations

### Known Issue: No Consent Screen
Reusing the same app client, scopes, and user means Keycloak remembers prior consent. The consent screen (`handleConsent()`) will not appear. Tests must be updated to remove these steps.

---

## Test Classification

### Migrate to Pre-configured Resource Client (14 files)

| File | Additional Notes |
|---|---|
| `specs/tokens/api-tokens.spec.mjs` | Simple pattern |
| `specs/toolsets/toolsets-config.spec.mjs` | Simple pattern |
| `specs/toolsets/toolsets-auth-restrictions.spec.mjs` | 3 blocks, also use pre-configured app client |
| `specs/chat/chat.spec.mjs` | Simple pattern |
| `specs/chat/chat-toolsets.spec.mjs` | Simple pattern |
| `specs/chat/chat-agentic.spec.mjs` | Simple pattern |
| `specs/api-models/api-models.spec.mjs` | Simple pattern |
| `specs/api-models/api-models-prefix.spec.mjs` | Simple pattern |
| `specs/api-models/api-models-no-key.spec.mjs` | Simple pattern |
| `specs/api-models/api-models-forward-all.spec.mjs` | Simple pattern |
| `specs/models/model-alias.spec.mjs` | Simple pattern |
| `specs/models/model-metadata.spec.mjs` | Simple pattern, custom hfHomePath |
| `specs/settings/public-host-auth.spec.mjs` | Also update publicHostFixtures.mjs |
| `specs/oauth/oauth2-token-exchange.spec.mjs` | Changes from setup→ready mode, uses pre-configured app client |

### Keep Dynamic Creation (3 files)

| File | Reason |
|---|---|
| `specs/auth/token-refresh-integration.spec.mjs` | Modifies client token lifespan via Keycloak admin API |
| `specs/users/list-users.spec.mjs` | Uses admin@email.com as admin (not pre-configured), complex role assignments |
| `specs/request-access/multi-user-request-approval-flow.spec.mjs` | Uses admin@email.com as admin, tests approval workflow |

### No Changes (setup-mode tests, 7 files)

| File | Reason |
|---|---|
| `specs/setup/setup-flow.spec.mjs` | Setup mode, no resource client |
| `specs/setup/setup-api-models.spec.mjs` | Setup mode |
| `specs/setup/setup-browser-extension.spec.mjs` | Setup mode |
| `specs/setup/setup-browser-extension-with-extension-installed.spec.mjs` | Setup mode |
| `specs/setup/setup-toolsets.spec.mjs` | Setup mode |
| `specs/settings/network-ip-setup-flow.spec.mjs` | Setup mode |

---

## Constraints & Risks

1. **Keycloak state accumulation**: Access request scope registrations accumulate across test runs. Safe because calls are idempotent, but Keycloak state grows over time.

2. **Fixed port conflict risk**: All migrated tests share port 51135. If a server doesn't fully release the port before the next test starts, the test will fail. Mitigated by sequential execution and `await server.stop()`.

3. **Consent screen removal**: Must remove ALL `handleConsent()` calls in migrated OAuth tests. Missing one will cause the test to hang waiting for a consent page that never appears.

4. **oauth2-token-exchange transformation**: This test changes from testing "setup flow → OAuth flow" to testing just "OAuth flow in ready state". The setup flow coverage is still provided by `specs/setup/setup-flow.spec.mjs`.

5. **getAuthServerConfig still requires devConsoleClientSecret**: The function validates this env var is set. Tests that no longer need it will still call the function (for authUrl/authRealm). The env var is set in .env.test, so no issue, but it's unused for migrated tests.
