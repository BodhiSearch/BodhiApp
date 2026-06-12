# Plan: Migrate NAPI E2E Tests to Pre-configured Resource/App Client

**Context file**: [`20260215-e2e-reuse-resource-client-ctx.md`](20260215-e2e-reuse-resource-client-ctx.md) — captures full Q&A, exploration findings, and decision rationale.

## Context

The NAPI E2E tests (Playwright) currently create a fresh OAuth resource client via the Keycloak admin API for every test suite, then make the test user a resource admin. This mirrors what was already fixed in `server_app` (see `20260215-server-app-remove-register-resource.md`). The new auth server (`main-id.getbodhi.app`) returns clients without Direct Access Grants enabled, causing failures. Pre-configured clients with Direct Access Grants are now available in `.env.test`.

**Goal**: Replace dynamic resource client creation + `makeResourceAdmin` with pre-configured credentials from `.env.test`. Fix ports. Use pre-configured app client where possible. Remove consent screen handling for reused app client/user combinations.

---

## Implementation Approach

### Phase 0: Utility Setup

**File: `tests-js/utils/auth-server-client.mjs`**

Add two helper functions:

```js
export function getPreConfiguredResourceClient() {
  const client = {
    clientId: process.env.INTEG_TEST_RESOURCE_CLIENT_ID,
    clientSecret: process.env.INTEG_TEST_RESOURCE_CLIENT_SECRET,
    scope: process.env.INTEG_TEST_RESOURCE_CLIENT_SCOPE,
  };
  for (const [key, value] of Object.entries(client)) {
    if (!value) throw new Error(`Required env var missing: ${key}`);
  }
  return client;
}

export function getPreConfiguredAppClient() {
  const clientId = process.env.INTEG_TEST_APP_CLIENT_ID;
  if (!clientId) throw new Error('INTEG_TEST_APP_CLIENT_ID not set');
  return { clientId };
}
```

**File: `tests-js/.env.test.example`**

Update to include resource client and app client vars:
```
INTEG_TEST_RESOURCE_CLIENT_ID=<secret>
INTEG_TEST_RESOURCE_CLIENT_SECRET=<secret>
INTEG_TEST_RESOURCE_CLIENT_SCOPE=<secret>
INTEG_TEST_APP_CLIENT_ID=<secret>
INTEG_TEST_EXA_API_KEY=<secret>
INTEG_TEST_SCOPE_TOOLSET_EXA_WEB_SEARCH_ID=<secret>
```

---

### Phase 1: Migrate Simple Ready-State Tests (no OAuth test app)

These 12 test files follow the identical pattern: `createResourceClient` → `makeResourceAdmin` → `createServerManager({ready})`. They only use session-based login through the Bodhi server (no static OAuth test app).

**Migration pattern** — replace this:
```js
const port = randomPort();
const serverUrl = `http://localhost:${port}`;
authClient = createAuthServerTestClient(authServerConfig);
resourceClient = await authClient.createResourceClient(serverUrl);
await authClient.makeResourceAdmin(resourceClient.clientId, resourceClient.clientSecret, testCredentials.userId);
serverManager = createServerManager({ ..., clientId: resourceClient.clientId, clientSecret: resourceClient.clientSecret, port });
```

With this:
```js
const resourceClient = getPreConfiguredResourceClient();
const port = 51135;
serverManager = createServerManager({ ..., clientId: resourceClient.clientId, clientSecret: resourceClient.clientSecret, port });
```

Remove: `createAuthServerTestClient` import, `authClient` variable, `randomPort` import (if unused).

**Files (run one at a time, fix if needed):**

1. `specs/tokens/api-tokens.spec.mjs`
2. `specs/toolsets/toolsets-config.spec.mjs`
3. `specs/chat/chat.spec.mjs`
4. `specs/chat/chat-toolsets.spec.mjs`
5. `specs/chat/chat-agentic.spec.mjs`
6. `specs/api-models/api-models.spec.mjs`
7. `specs/api-models/api-models-prefix.spec.mjs`
8. `specs/api-models/api-models-no-key.spec.mjs`
9. `specs/api-models/api-models-forward-all.spec.mjs`
10. `specs/models/model-alias.spec.mjs`
11. `specs/models/model-metadata.spec.mjs`
12. `specs/settings/public-host-auth.spec.mjs` — also update `publicHostFixtures.mjs` to not require `resourceClient` parameter

**Debugging**: If a test fails, use Claude in Chrome to navigate to `http://localhost:51135`, reproduce the test steps manually, and fix.

---

### Phase 2: Migrate OAuth Flow Tests

#### 2a. `specs/oauth/oauth2-token-exchange.spec.mjs`

**Current state**: Starts in SETUP mode, goes through setup wizard (welcome page → resource admin login), creates app client via dev console, does OAuth flow, tests token exchange.

**Migration**:
- Change from SETUP mode → READY mode with pre-configured resource client
- Use pre-configured app client from env (skip `createAppClient` via dev console)
- Remove setup wizard steps (welcome page, resource admin login)
- Fix Bodhi server port to 51135, static server port to 55173
- **Remove `handleConsent()`** — Keycloak remembers prior consent for reused app client + user
- Test still validates: OAuth flow → token exchange → API access with exchanged token

**Error Handling block**: Keep `appStatus: 'ready'` with invalid credentials (testing error case). Fix port to 51135.

#### 2b. `specs/toolsets/toolsets-auth-restrictions.spec.mjs`

This file has 3 `test.describe` blocks:

**Block 1 — "Session Auth - Toolset Endpoints"** (lines 37-120):
- Simple ready-state test, session auth only
- Migrate: resource client from env, fixed port 51135
- Remove: `authClient`, `createResourceClient`, `makeResourceAdmin`

**Block 2 — "OAuth Token + Toolset Scope Combinations"** (lines 122-579):
- Uses `beforeEach` to create resource client + server per test
- Migrate: resource client from env, fixed port 51135, fixed static server port 55173
- Use pre-configured app client (`INTEG_TEST_APP_CLIENT_ID`) instead of dynamic creation
- **Remove `handleConsent()` calls** — reusing same app client + user means Keycloak remembers consent
- Remove: `createResourceClient`, `makeResourceAdmin`, `createAppClient`, `getDevConsoleToken`
- Keycloak access request + scope registration calls are idempotent

**Block 3 — "OAuth Token - Toolset CRUD Endpoints"** (lines 581-771):
- Same migration as Block 2
- Use pre-configured app client from env
- Remove `handleConsent()` calls

**Static server**: Fix port to 55173. The redirect URI `http://localhost:55173/oauth-test-app.html` is already configured on the pre-configured app client in Keycloak.

**Debugging**: Start static server with `npm run test:static-server`, navigate to it, input values with server address `http://localhost:51135`, submit form, and fix any variations.

---

### Phase 3: Fixture Updates

**File: `fixtures/publicHostFixtures.mjs`**
- `getServerManagerConfig` currently takes `resourceClient` as parameter
- Change to read from env internally or accept pre-configured client format

**File: `fixtures/oauth2Fixtures.mjs`**
- Update `getOAuth2ServerConfig` for ready mode with pre-configured clients (used by oauth2-token-exchange)

**File: `fixtures/setupFixtures.mjs`**
- No changes (setup-mode tests)

---

## Tests Kept As-Is (No Migration)

| Test File | Reason |
|---|---|
| `specs/auth/token-refresh-integration.spec.mjs` | Modifies client token lifespan via Keycloak admin API |
| `specs/users/list-users.spec.mjs` | Uses `admin@email.com` as resource admin (not pre-configured), complex role assignments |
| `specs/request-access/multi-user-request-approval-flow.spec.mjs` | Uses `admin@email.com` as resource admin, tests approval workflow |
| `specs/setup/setup-flow.spec.mjs` | Setup mode (no resource client) |
| `specs/setup/setup-api-models.spec.mjs` | Setup mode |
| `specs/setup/setup-browser-extension.spec.mjs` | Setup mode |
| `specs/setup/setup-browser-extension-with-extension-installed.spec.mjs` | Setup mode |
| `specs/setup/setup-toolsets.spec.mjs` | Setup mode |
| `specs/settings/network-ip-setup-flow.spec.mjs` | Setup mode |

---

## Key Configuration

| Setting | Value |
|---|---|
| Bodhi server port | `51135` |
| Bodhi server host | `localhost` |
| Bodhi redirect URI | `http://localhost:51135/ui/auth/callback` |
| Static OAuth test app port | `55173` |
| App client redirect URI | `http://localhost:55173/oauth-test-app.html` |
| Pre-configured resource admin | `user@email.com` (ec18e95b-...) |

---

## Known Issues

1. **No consent screen** — Reusing the same app client, scopes, and user means Keycloak remembers prior consent. All `handleConsent()` calls must be removed from migrated OAuth flow tests.
2. **Keycloak state accumulation** — Access request scope registrations accumulate in Keycloak across test runs. Calls are idempotent, so this is safe but means Keycloak state grows.
3. **Fixed port risk** — Tests share port 51135. Depends on proper server stop before next test starts. Sequential execution (workers: 1) mitigates this.

---

## Verification

For each test file migration:
1. Run the specific test: `npx playwright test specs/<path-to-test> --headed`
2. If it fails:
   - For non-OAuth tests: Use Claude in Chrome → navigate to `http://localhost:51135` → reproduce test steps → identify issue
   - For OAuth test app tests: Run `npm run test:static-server` → navigate to static server → input values with server `http://localhost:51135` → submit form → fix variations
3. After all migrations: `npx playwright test` (full suite, excluding @scheduled)

---

## Files Modified Summary

| File | Change Type |
|---|---|
| `tests-js/utils/auth-server-client.mjs` | Add `getPreConfiguredResourceClient()`, `getPreConfiguredAppClient()` |
| `tests-js/.env.test.example` | Add resource/app client vars |
| `tests-js/fixtures/publicHostFixtures.mjs` | Remove `resourceClient` parameter dependency |
| `tests-js/fixtures/oauth2Fixtures.mjs` | Update for ready mode with pre-configured clients |
| `tests-js/specs/tokens/api-tokens.spec.mjs` | Migrate to pre-configured resource client |
| `tests-js/specs/toolsets/toolsets-config.spec.mjs` | Migrate |
| `tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs` | Migrate all 3 blocks (resource + app client from env, remove consent) |
| `tests-js/specs/oauth/oauth2-token-exchange.spec.mjs` | Migrate: setup→ready mode, pre-configured resource + app client, remove consent |
| `tests-js/specs/chat/chat.spec.mjs` | Migrate |
| `tests-js/specs/chat/chat-toolsets.spec.mjs` | Migrate |
| `tests-js/specs/chat/chat-agentic.spec.mjs` | Migrate |
| `tests-js/specs/api-models/api-models.spec.mjs` | Migrate |
| `tests-js/specs/api-models/api-models-prefix.spec.mjs` | Migrate |
| `tests-js/specs/api-models/api-models-no-key.spec.mjs` | Migrate |
| `tests-js/specs/api-models/api-models-forward-all.spec.mjs` | Migrate |
| `tests-js/specs/models/model-alias.spec.mjs` | Migrate |
| `tests-js/specs/models/model-metadata.spec.mjs` | Migrate |
| `tests-js/specs/settings/public-host-auth.spec.mjs` | Migrate |

---

## Implementation Progress

**Status: COMPLETE** — All phases implemented and verified. Full test suite: 48 passed, 1 skipped.

| Phase | Status | Notes |
|---|---|---|
| Phase 0: Utility Setup | Done | `getPreConfiguredResourceClient()`, `getPreConfiguredAppClient()` added |
| Phase 1: 12 Simple Tests | Done | All 12 files migrated, 26 tests pass |
| Phase 2a: oauth2-token-exchange | Done | 2 tests pass |
| Phase 2b: toolsets-auth-restrictions | Done | 6 tests pass (was 7 before merge) |
| Phase 3: Fixture Updates | Done | `oauth2Fixtures.mjs`, `publicHostFixtures.mjs` updated |

---

## Deviations and Discoveries

### 1. `oauth2-token-exchange`: `handleLogin()` required (not just consent removal)

**Plan said**: Remove `handleConsent()` — Keycloak remembers prior consent for reused app client + user.

**What happened**: The sub-agent removed both `handleConsent()` AND `handleLogin()`. But this test has NO prior Keycloak session in the browser (unlike toolsets tests which do `loginPage.performOAuthLogin()` first). When Keycloak redirected the browser, it showed the login screen, not the consent screen.

**Fix**: Added `handleLogin(testCredentials.username, testCredentials.password)` after `waitForAuthServerRedirect()`. The plan's assumption about consent was correct, but the test also needed explicit login since there's no active KC session.

### 2. `toolsets-auth-restrictions`: `waitForAuthServerRedirect()` fails with pre-configured client + active KC session

**Plan said**: Remove `handleConsent()` calls.

**What happened**: After switching to pre-configured app client, the `waitForAuthServerRedirect(authServerConfig.authUrl)` call timed out. With a pre-configured app client + active KC session (from prior `performOAuthLogin()`) + no consent needed (Keycloak remembers consent), Keycloak auto-redirects so fast that the browser never "stays" at the auth server URL long enough for Playwright to detect it.

**Fix**: Removed `waitForAuthServerRedirect()` from all OAuth tests in this file (Cases 1, 2, 4, and merged CRUD test). Go directly from `clickLogin()` to `waitForTokenExchange(testAppUrl)`.

**Key insight**: `waitForAuthServerRedirect` is only needed when the browser actually pauses at Keycloak (e.g., login screen or consent screen). When KC auto-redirects, this wait becomes a timeout.

### 3. Block 3 GET/PUT tests merged into single test

**Plan said**: Migrate Block 3 with pre-configured app client.

**What happened**: User requested merging tests that share the same OAuth flow scope and requested toolsets. The Block 3 GET and PUT tests had identical OAuth flows (both used `scope_user_user` with no toolsets). Merged into a single test `'GET and PUT /toolsets/{id} with OAuth token returns 401 (session-only)'` that performs one OAuth flow and tests both endpoints. Reduced from 7 to 6 tests in this file.

### 4. `INTEG_TEST_SCOPE_TOOLSET_EXA_WEB_SEARCH_ID` not added to `.env.test.example`

**Plan said**: Add `INTEG_TEST_SCOPE_TOOLSET_EXA_WEB_SEARCH_ID=<secret>` to `.env.test.example`.

**What happened**: This env var was not needed by any of the migrated tests. The toolsets tests use `INTEG_TEST_EXA_API_KEY` (which was added) but the scope ID is discovered at runtime via the toolsets page UI. Omitted from `.env.test.example`.
