# Remove E2E Test KC SPI Hacks — Direct KC `/users/request-access` Calls

## Context

E2e tests in `oauth2-token-exchange.spec.mjs` and `toolsets-auth-restrictions.spec.mjs` directly call KC's `/bodhi/users/request-access` SPI endpoint via `authClient.registerUserAccess()`. This exposes internal Keycloak wiring in what should be black-box tests.

**Two KC SPI endpoints involved:**
- `/bodhi/resources/apps/request-access` (service account token) — registers `scope_resource-*` as optional scope on the app client
- `/bodhi/users/request-access` (user token) — registers user consent, returns `scope_resource-*` + `scope_access_request:*`

**Auto-approve flow (no toolsets):** Only needs `/resources/apps/request-access`. KC handles user consent via its own consent screen when the 3rd party app initiates OAuth login. No Bodhi review screen needed.

**With-toolsets flow:** Creates draft → user reviews on Bodhi review screen → `approve_request` backend handler calls `/users/request-access` with user token.

## Root Cause

**For `oauth2-token-exchange.spec.mjs` (auto-approve):**
- `create_draft()` auto-approves → sets `status: "approved"`
- `approve_request()` rejects with `AlreadyProcessed` because `status != "draft"`
- No API path to call KC `/users/request-access` after auto-approve
- E2e test works around this by calling KC directly
- But this call should NOT be needed — `/resources/apps/request-access` wires the scope, KC consent screen handles user-level consent

**For `toolsets-auth-restrictions.spec.mjs` (with toolsets):**
- Test calls `approveAccessRequest()` which goes through backend `approve_request` → already calls KC `/users/request-access`
- The separate `registerUserAccess()` after it is redundant — copy-pasted from auto-approve hack

## Plan

### Step 1: Remove hack from `oauth2-token-exchange.spec.mjs`
**File**: `crates/lib_bodhiserver_napi/tests-js/specs/oauth/oauth2-token-exchange.spec.mjs` (lines 98-108)

Remove Step 5b entirely (the `devSecrets` fetch and `registerUserAccess` call). The auto-approve already returns `resource_scope` which the test uses for the OAuth flow.

### Step 2: Remove redundant `registerUserAccess` from `toolsets-auth-restrictions.spec.mjs`
**File**: `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs`

Remove all 6 occurrences of the `registerUserAccess` hack (lines ~314-322, ~473, ~590, ~657, ~814, ~880). The `approveAccessRequest()` call that precedes each one already triggers the backend KC wiring.

### Step 3: Run affected e2e tests
```bash
cd crates/lib_bodhiserver_napi && npm run test -- specs/oauth/oauth2-token-exchange.spec.mjs
cd crates/lib_bodhiserver_napi && npm run test -- specs/toolsets/toolsets-auth-restrictions.spec.mjs
```

If auto-approve test fails (Step 1), investigate whether `/resources/apps/request-access` is insufficient and a backend fix is needed in `create_draft()`.

### Step 4 (if needed): Clean up `registerUserAccess` utility
If all tests pass without the hack, consider removing `registerUserAccess` from `auth-server-client.mjs` if it has no other callers.

## Verification
1. Run `oauth2-token-exchange.spec.mjs` e2e test — should pass without KC hack
2. Run `toolsets-auth-restrictions.spec.mjs` e2e test — should pass without redundant KC calls
3. Search for remaining `registerUserAccess` usages to ensure full cleanup
