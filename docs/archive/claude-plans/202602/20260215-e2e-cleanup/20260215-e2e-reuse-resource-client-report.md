# Report: Migrate NAPI E2E Tests to Pre-configured Resource/App Client

**Date**: 2026-02-16
**Plan**: [`20260215-e2e-reuse-resource-client.md`](20260215-e2e-reuse-resource-client.md)
**Context**: [`20260215-e2e-reuse-resource-client-ctx.md`](20260215-e2e-reuse-resource-client-ctx.md)
**Status**: Complete

## Summary

Migrated 15 NAPI E2E test files + 2 fixture files from dynamic OAuth resource/app client creation (via Keycloak admin API) to pre-configured credentials from `.env.test`. This eliminates per-test-suite Keycloak admin calls, fixes Direct Access Grants issues with the new auth server, and uses fixed ports for deterministic test execution.

**Net diff**: 18 files changed, 122 insertions, 386 deletions (net -264 lines)

## Test Results

Full suite run after all changes:
- **48 passed**
- **1 skipped** (api-tokens test requiring `INTEG_TEST_OPENAI_API_KEY`)
- **0 failed**
- **Duration**: ~8 minutes

## Changes by Phase

### Phase 0: Utility Setup (2 files)

| File | Change |
|---|---|
| `tests-js/utils/auth-server-client.mjs` | Added `getPreConfiguredResourceClient()` and `getPreConfiguredAppClient()` helper functions (+30 lines) |
| `tests-js/.env.test.example` | Added `INTEG_TEST_RESOURCE_CLIENT_ID`, `INTEG_TEST_RESOURCE_CLIENT_SECRET`, `INTEG_TEST_RESOURCE_CLIENT_SCOPE`, `INTEG_TEST_APP_CLIENT_ID`, `INTEG_TEST_EXA_API_KEY` |

### Phase 1: Simple Ready-State Tests (12 files)

All follow the same pattern: replaced `randomPort()` + `createAuthServerTestClient()` + `createResourceClient()` + `makeResourceAdmin()` with `getPreConfiguredResourceClient()` + fixed port `51135`.

| File | Tests | Lines Removed |
|---|---|---|
| `specs/tokens/api-tokens.spec.mjs` | 3 passed, 1 skipped | -10 |
| `specs/toolsets/toolsets-config.spec.mjs` | 4 passed | -11 |
| `specs/chat/chat.spec.mjs` | 1 passed | -10 |
| `specs/chat/chat-toolsets.spec.mjs` | 1 passed | -10 |
| `specs/chat/chat-agentic.spec.mjs` | 1 passed | -10 |
| `specs/api-models/api-models.spec.mjs` | 3 passed | -10 |
| `specs/api-models/api-models-prefix.spec.mjs` | 4 passed | -10 |
| `specs/api-models/api-models-no-key.spec.mjs` | 2 passed | -8 |
| `specs/api-models/api-models-forward-all.spec.mjs` | 2 passed | -8 |
| `specs/models/model-alias.spec.mjs` | 1 passed | -8 |
| `specs/models/model-metadata.spec.mjs` | 1 passed | -8 |
| `specs/settings/public-host-auth.spec.mjs` | 3 passed | -12 |

### Phase 2: OAuth Flow Tests (2 files)

**`specs/oauth/oauth2-token-exchange.spec.mjs`** (-43 lines)
- Changed from SETUP mode to READY mode with pre-configured resource client
- Replaced dynamic app client creation (setup wizard + dev console token + createAppClient) with `getPreConfiguredAppClient()`
- Removed `handleConsent()`, added `handleLogin()` (no prior KC session in this test)
- Fixed ports: Bodhi server 51135, static server 55173
- 2 tests pass

**`specs/toolsets/toolsets-auth-restrictions.spec.mjs`** (-206 lines, most significant change)
- Block 1 (Session Auth): Replaced dynamic resource client with `getPreConfiguredResourceClient()`
- Block 2 (OAuth Scope Combinations): Replaced dynamic app client with `getPreConfiguredAppClient()`, removed `OAuth2ApiHelper`/`getDevConsoleToken`/`createAppClient`, removed `handleConsent()` AND `waitForAuthServerRedirect()`
- Block 3 (CRUD Endpoints): Same as Block 2 + merged GET and PUT tests into single test
- 6 tests pass (reduced from 7 by merging GET/PUT CRUD test)

### Phase 3: Fixture Updates (2 files)

| File | Change |
|---|---|
| `fixtures/oauth2Fixtures.mjs` | `getOAuth2ServerConfig` now defaults to `appStatus: 'ready'`, reads resource client internally via `getPreConfiguredResourceClient()` |
| `fixtures/publicHostFixtures.mjs` | `getServerManagerConfig` removed `resourceClient` parameter, reads from env internally |

## Deviations from Plan

### 1. `handleLogin()` needed in oauth2-token-exchange (not just consent removal)

The plan said to remove `handleConsent()`. The test also needed `handleLogin()` added because this test has no prior Keycloak session — unlike toolsets tests which do `performOAuthLogin()` first. Keycloak shows the login screen, not just consent.

### 2. `waitForAuthServerRedirect()` removal in toolsets-auth-restrictions

**Not anticipated in plan**. With pre-configured app client + active KC session + remembered consent, Keycloak auto-redirects instantly. The browser never stays at the auth server URL, causing `waitForAuthServerRedirect()` to timeout. Removed from all OAuth tests in Blocks 2 and 3.

This only affects tests where: (a) user already has an active KC session, AND (b) consent is pre-remembered. The `oauth2-token-exchange` test keeps `waitForAuthServerRedirect` because it has no prior KC session.

### 3. Merged Block 3 GET/PUT CRUD tests

User requested merging tests with identical OAuth flow scope and requested toolsets. The GET and PUT tests in Block 3 had identical flows, so they were merged into `'GET and PUT /toolsets/{id} with OAuth token returns 401 (session-only)'`.

### 4. `INTEG_TEST_SCOPE_TOOLSET_EXA_WEB_SEARCH_ID` omitted from `.env.test.example`

Plan included it but no migrated test uses it. The scope ID is discovered at runtime via the toolsets page UI.

## Removed Patterns

The following patterns were removed from all migrated test files:

```js
// REMOVED: Dynamic resource client creation
import { randomPort } from '@/test-helpers.mjs';
import { createAuthServerTestClient } from '@/utils/auth-server-client.mjs';
const port = randomPort();
const serverUrl = `http://localhost:${port}`;
authClient = createAuthServerTestClient(authServerConfig);
resourceClient = await authClient.createResourceClient(serverUrl);
await authClient.makeResourceAdmin(resourceClient.clientId, resourceClient.clientSecret, testCredentials.userId);

// REMOVED: Dynamic app client creation (OAuth tests)
import { OAuth2ApiHelper } from '@/utils/OAuth2ApiHelper.mjs';
const apiHelper = new OAuth2ApiHelper(baseUrl, authClient);
const devConsoleToken = await apiHelper.getDevConsoleToken(username, password);
const appClient = await apiHelper.createAppClient(devConsoleToken, port, name, desc, [redirectUri]);

// REMOVED: Consent handling (OAuth tests with pre-configured app client)
await oauth2TestAppPage.handleConsent();

// REMOVED: Auth server redirect wait (when KC auto-redirects)
await oauth2TestAppPage.waitForAuthServerRedirect(authServerConfig.authUrl);
```

## New Patterns

```js
// NEW: Pre-configured resource client from env
import { getPreConfiguredResourceClient } from '@/utils/auth-server-client.mjs';
const resourceClient = getPreConfiguredResourceClient();
const port = 51135;

// NEW: Pre-configured app client from env (OAuth tests)
import { getPreConfiguredAppClient } from '@/utils/auth-server-client.mjs';
const appClient = getPreConfiguredAppClient();

// NEW: Direct clickLogin → waitForTokenExchange (when KC auto-redirects)
await oauth2TestAppPage.clickLogin();
await oauth2TestAppPage.waitForTokenExchange(testAppUrl);
```

## Tests Unchanged (as planned)

| Test File | Reason |
|---|---|
| `specs/auth/token-refresh-integration.spec.mjs` | Modifies client token lifespan via KC admin API |
| `specs/users/list-users.spec.mjs` | Uses `admin@email.com` as resource admin, complex role assignments |
| `specs/request-access/multi-user-request-approval-flow.spec.mjs` | Uses `admin@email.com` as resource admin |
| `specs/setup/*.spec.mjs` (5 files) | Setup mode tests — no resource client needed |
| `specs/settings/network-ip-setup-flow.spec.mjs` | Setup mode |
