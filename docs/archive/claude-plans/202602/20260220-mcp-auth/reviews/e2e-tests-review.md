# E2E Tests Review

## Files Reviewed

### Spec Files
- `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-oauth-auth.spec.mjs` (326 lines) - Pre-registered OAuth flows: UI-driven creation, 3rd-party access request, edit/disconnect, denied access
- `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-oauth-dcr.spec.mjs` (294 lines) - Dynamic Client Registration flows: UI-driven DCR, edit/disconnect, 3rd-party access request
- `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-header-auth.spec.mjs` (268 lines) - Header auth flows: create with header, edit switch auth, 3rd-party access request

### Page Objects
- `crates/lib_bodhiserver_napi/tests-js/pages/McpsPage.mjs` (569 lines) - Primary page object for MCP server/instance management and playground
- `crates/lib_bodhiserver_napi/tests-js/pages/OAuthTestApp.mjs` (37 lines) - Composite page object for external OAuth test app
- `crates/lib_bodhiserver_napi/tests-js/pages/AccessRequestReviewPage.mjs` (105 lines) - Access request review/approve/deny page object
- `crates/lib_bodhiserver_napi/tests-js/pages/BasePage.mjs` (216 lines) - Base page object with shared navigation, toast, and assertion methods
- `crates/lib_bodhiserver_napi/tests-js/pages/sections/ConfigSection.mjs` (84 lines) - OAuth form configuration section
- `crates/lib_bodhiserver_napi/tests-js/pages/sections/OAuthSection.mjs` (73 lines) - OAuth flow wait helpers (redirects, token exchange, login)
- `crates/lib_bodhiserver_napi/tests-js/pages/sections/AccessCallbackSection.mjs` (48 lines) - Access request callback state handling
- `crates/lib_bodhiserver_napi/tests-js/pages/test-app/RESTPage.mjs` (85 lines) - REST client page object for API verification

### Fixtures
- `crates/lib_bodhiserver_napi/tests-js/fixtures/mcpFixtures.mjs` (131 lines) - Test data constants and factory methods for MCP, OAuth, DCR, header auth, and Tavily

### Mock OAuth Server
- `crates/lib_bodhiserver_napi/test-mcp-oauth-server/src/oauth.ts` (352 lines) - OAuth 2.1 authorization server with PKCE S256, DCR, refresh tokens
- `crates/lib_bodhiserver_napi/test-mcp-oauth-server/src/mcp-server.ts` (142 lines) - MCP Streamable HTTP transport with bearer token validation
- `crates/lib_bodhiserver_napi/test-mcp-oauth-server/src/index.ts` (55 lines) - Express app entry point, CORS, signal handling

### Configuration
- `crates/lib_bodhiserver_napi/playwright.config.mjs` (142 lines) - Playwright config with 4 webServer entries (Bodhi, OAuth test app, pre-reg OAuth, DCR OAuth)

---

## User Journey Coverage

| Journey | Tested? | Spec File | Notes |
|---|---|---|---|
| OAuth pre-reg: create server -> config via API -> select in UI -> authorize -> callback -> fill details -> create MCP | Yes | `mcps-oauth-auth.spec.mjs` test 1 | Full UI-driven flow with playground verification |
| OAuth pre-reg: config reuse across MCP instances | Yes | `mcps-oauth-auth.spec.mjs` test 1, step "Create second MCP with same OAuth config" | Same oauthConfigId used for second instance |
| OAuth pre-reg: 3rd-party access request -> approve -> REST tool execution | Yes | `mcps-oauth-auth.spec.mjs` test 2 | 5-phase flow with OAuthTestApp + AccessRequestReviewPage |
| OAuth pre-reg: edit -> disconnect -> update without token | Yes | `mcps-oauth-auth.spec.mjs` test 3 | Verifies connected card, disconnect, save |
| OAuth pre-reg: denied access request -> error callback | Yes | `mcps-oauth-auth.spec.mjs` test 4 | Verifies callback state='error' |
| OAuth DCR: discover -> DCR register -> create config -> authorize -> create MCP -> playground | Yes | `mcps-oauth-dcr.spec.mjs` test 1 | Validates `dyn-` prefix on client_id |
| OAuth DCR: edit -> disconnect -> update | Yes | `mcps-oauth-dcr.spec.mjs` test 2 | Mirror of pre-reg edit flow |
| OAuth DCR: 3rd-party access request -> approve -> REST tool execution | Yes | `mcps-oauth-dcr.spec.mjs` test 3 | Mirror of pre-reg 3rd-party flow |
| Header auth: create -> select from dropdown -> fetch tools -> create -> playground | Yes | `mcps-header-auth.spec.mjs` test 1 | Uses Tavily real API (requires env key) |
| Header auth: edit -> switch to public -> switch back to header -> verify via playground | Yes | `mcps-header-auth.spec.mjs` test 2 | Tests auth config state transitions |
| Header auth: 3rd-party access request -> approve -> REST tool execution | Yes | `mcps-header-auth.spec.mjs` test 3 | Tests with header-auth MCP in 3rd-party flow |
| Tool execution with bearer token | Yes | `mcps-oauth-auth.spec.mjs` test 1 step 6, `mcps-oauth-dcr.spec.mjs` test 1 step 6 | Playground execute verifies `data-test-state='success'` |
| Error: invalid state parameter | No | -- | Not tested (see Finding 1) |
| Error: expired tokens | No | -- | Not tested (see Finding 2) |
| Error: failed DCR | No | -- | Not tested (see Finding 3) |
| Token refresh flow | No | -- | Not tested (see Finding 4) |

---

## Findings

### Finding 1: No Error Scenario for Invalid OAuth State Parameter
- **Priority**: Nice-to-have
- **File**: `mcps-oauth-auth.spec.mjs`
- **Location**: Not present; relates to the CSRF state validation path
- **Issue**: The test suite covers the happy path for OAuth authorization and the denied access request flow, but does not test what happens when the OAuth callback returns with an invalid or tampered `state` parameter. The mock server (`oauth.ts` line 186-188) faithfully passes through the state parameter without validation on the AS side (as expected -- validation happens on the client side in BodhiApp), but no test verifies the client-side CSRF check.
- **Recommendation**: Add a test or test step that simulates a tampered state parameter in the OAuth callback URL. This could be done by intercepting the redirect via `page.route()` and modifying the `state` query parameter before it hits the callback handler, then asserting that the UI shows an appropriate error.
- **Rationale**: CSRF state validation is a security-critical part of OAuth 2.1. While it is validated in backend unit tests, an E2E test provides defense-in-depth and verifies the UI error handling path.

### Finding 2: No Token Expiry or Refresh E2E Coverage
- **Priority**: Nice-to-have
- **File**: Not present
- **Location**: N/A
- **Issue**: The mock OAuth server issues access tokens with a 3600-second expiry (`oauth.ts` line 299). The refresh token grant is fully implemented (`oauth.ts` lines 317-351) and the BodhiApp backend supports proactive refresh 60 seconds before expiry (per `00-overview.md`), but no E2E test exercises this path. The `@scheduled` tests mentioned in `CLAUDE.md` are excluded from regular runs and focus on Keycloak token refresh, not MCP OAuth token refresh.
- **Recommendation**: This is a known limitation documented in E2E.md ("@scheduled tests are local-only"). To make this testable without long waits, consider adding a configurable short expiry to the mock server (e.g., `?expires_in=5` query param on the token endpoint) so a test could create an MCP, wait briefly, and verify tool execution still succeeds (proving refresh worked). Mark it `@scheduled` if the wait is too long for CI.
- **Rationale**: Token refresh is a critical production path. Without E2E coverage, regressions in the refresh logic would only be caught by manual testing or production incidents.

### Finding 3: No Failed DCR Error Scenario
- **Priority**: Nice-to-have
- **File**: `mcps-oauth-dcr.spec.mjs`
- **Location**: Not present
- **Issue**: All DCR tests use the happy path -- the mock server always returns 201 for `/register`. There is no test for what happens when DCR fails (e.g., server returns 400, or the registration endpoint is unreachable). The mock server has no error simulation capability for the registration endpoint.
- **Recommendation**: Add a test that attempts DCR against a non-existent registration endpoint or adds error simulation to the mock server (e.g., a `/register?fail=true` query param that returns 400). Verify the UI/API returns an appropriate error message to the user.
- **Rationale**: DCR failure is a realistic scenario when connecting to third-party MCP servers. Users need clear feedback when registration fails.

### Finding 4: OAuthTestApp Does Not Extend BasePage
- **Priority**: Important
- **File**: `crates/lib_bodhiserver_napi/tests-js/pages/OAuthTestApp.mjs`
- **Location**: Lines 8-17
- **Issue**: `OAuthTestApp` is a plain class that does not extend `BasePage`, unlike all other page objects. E2E.md states "All page objects extend `BasePage`" as a convention. While `OAuthTestApp` acts as a composite (composing `ConfigSection`, `OAuthSection`, `AccessCallbackSection`, etc.), it still has a `navigate()` method (line 20-23) that reimplements `BasePage.navigate()` logic (calling `page.goto()` and `waitForLoadState`) without using `BasePage.waitForSPAReady()`.
- **Recommendation**: Either extend `BasePage` and use `super.navigate()` for consistency, or document this as an intentional exception since `OAuthTestApp` is a composite of section objects for an external test app (not a BodhiApp page).
- **Rationale**: Inconsistency in page object patterns makes the codebase harder to maintain and can lead to subtle differences in navigation behavior. However, since the external test app is a simple static React app (not the Bodhi SPA), the `waitForSPAReady()` step may genuinely not be needed, making this more of a documentation gap than a bug.

### Finding 5: Duplicated DCR Setup Code Across Tests
- **Priority**: Important
- **File**: `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-oauth-dcr.spec.mjs`
- **Location**: Lines 42-75 (test 1), lines 128-163 (test 2), lines 196-232 (test 3)
- **Issue**: The DCR discovery + register + create config sequence is repeated nearly identically across all three DCR tests. Each test performs: `discoverMcpEndpointsViaApi()` -> `dynamicRegisterViaApi()` -> `createOAuthConfigViaApi()` with the same parameters (minor differences in whether scopes are passed). This violates DRY and makes maintenance harder -- any API change requires updating three places.
- **Recommendation**: Extract a composite helper method into `McpsPage`, e.g., `createDcrConfigViaApi(serverId, serverUrl)`, that encapsulates the discover -> register -> create config flow and returns `{ dcrConfigId, dcrResult, discovery }`. The first test can call the lower-level APIs individually (to validate each step), while tests 2 and 3 use the composite helper.
- **Rationale**: E2E.md emphasizes "journey boundary" tests, not "unit-style" repetition. The duplicate setup makes tests longer and harder to maintain without adding assertion value.

### Finding 6: API Error Handling Not Checked in page.evaluate() Calls
- **Priority**: Important
- **File**: `crates/lib_bodhiserver_napi/tests-js/pages/McpsPage.mjs`
- **Location**: Lines 174-191 (`createAuthHeaderViaApi`), lines 194-207 (`createOAuthConfigViaApi`), lines 209-222 (`discoverMcpEndpointsViaApi`), lines 224-244 (`dynamicRegisterViaApi`)
- **Issue**: All four API helper methods use `page.evaluate()` with `fetch()` and unconditionally call `resp.json()` without checking `resp.ok` or `resp.status`. If the API returns a 4xx/5xx error, the method silently returns the error JSON body, and the caller proceeds with potentially invalid data (e.g., `oauthConfig.id` would be `undefined`). Some callers do check `expect(oauthConfig.id).toBeTruthy()` (e.g., `mcps-oauth-auth.spec.mjs` line 47), but others do not (e.g., `mcps-oauth-dcr.spec.mjs` test 2 line 141-153 where `dcrConfigId = oauthConfig.id` is assigned without assertion).
- **Recommendation**: Add `if (!resp.ok) throw new Error(...)` in each `page.evaluate` callback to fail fast with a clear error message when an API call fails. This turns cryptic "cannot read property 'id' of undefined" errors into explicit "API call to /mcps/auth-configs returned 400: {body}" errors.
- **Rationale**: Silent API failures produce misleading test errors. When a test fails in CI, the actual root cause (e.g., changed validation rules) is hidden behind a downstream assertion failure, increasing debugging time.

### Finding 7: Header Auth Tests Depend on External Tavily API
- **Priority**: Important
- **File**: `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-header-auth.spec.mjs`
- **Location**: All three tests
- **Issue**: All header auth tests use the real Tavily MCP server (`https://mcp.tavily.com/mcp/`) with a real API key from `process.env.INTEG_TEST_TAVILY_API_KEY`. This creates an external service dependency that can cause flaky test failures due to network issues, rate limiting, API key expiration, or Tavily service outages. This is in contrast to the OAuth tests which use a fully local mock server.
- **Recommendation**: Consider creating a simple Express-based mock that accepts a header auth token and serves MCP tools, similar to the mock OAuth server but without OAuth (just validates `Authorization: Bearer {key}`). This would make header auth tests fully self-contained and deterministic.
- **Rationale**: E2E tests should be as deterministic as possible. External API dependencies are a well-known source of flakiness, especially in CI environments. The Tavily dependency means tests can fail for reasons completely unrelated to BodhiApp code changes.

### Finding 8: Inconsistent Step Naming Convention
- **Priority**: Nice-to-have
- **File**: `mcps-oauth-auth.spec.mjs`, `mcps-header-auth.spec.mjs`
- **Location**: Across all specs
- **Issue**: Tests use two different step naming conventions inconsistently. Some tests use numbered phases (`Phase 1:`, `Phase 2:`, etc.) while others use descriptive names without numbering. For example, in `mcps-oauth-auth.spec.mjs`:
  - Test 1 (line 34-115): descriptive names ("Login and create...", "Create OAuth config via API", "Navigate to new MCP...")
  - Test 2 (line 129-208): numbered phases ("Phase 1: Login...", "Phase 2: Configure...", "Phase 3: Submit...")
  - Test 3 (line 220-258): descriptive names again
  - Test 4 (line 272-324): numbered phases

  The pattern seems to be: simple single-user flows use descriptive names, multi-context flows (involving external app) use numbered phases. This is reasonable but not documented.
- **Recommendation**: Adopt a consistent convention (e.g., always use "Phase N:" for multi-actor flows, descriptive for single-actor) and document it in E2E.md.
- **Rationale**: Consistent naming improves readability in Playwright trace viewer and HTML reports, making failure triage faster.

### Finding 9: Mock OAuth Server Lacks Port Configuration Flexibility for DCR Mode
- **Priority**: Nice-to-have
- **File**: `crates/lib_bodhiserver_napi/test-mcp-oauth-server/src/oauth.ts`
- **Location**: Line 46
- **Issue**: The `PORT` constant used to construct OAuth metadata URLs (`oauth.ts` line 92-96) reads from `TEST_MCP_OAUTH_PORT` at module load time. In DCR mode, the server runs on port 55175 (set via the Playwright config environment override at `playwright.config.mjs` line 136-138), but the `PORT` variable is shared between the OAuth metadata and the Express `app.listen()` in `index.ts` line 5. This works because the environment variable is overridden in the DCR webServer entry, but it means the metadata URLs (issuer, endpoints) are dynamically determined by the port, creating a tight coupling between the Playwright config and the server logic.
- **Recommendation**: This is acceptable for a test-only server, but consider adding a comment in `oauth.ts` explaining that `PORT` determines metadata URLs and must match the actual listen port. If the DCR server ever needs to run on a different port, the env var must be set before import.
- **Rationale**: Documentation prevents future maintenance confusion when someone changes ports.

### Finding 10: No `test.describe.serial` Enforcement
- **Priority**: Nice-to-have
- **File**: All three spec files
- **Location**: `test.describe()` blocks
- **Issue**: While the Playwright config sets `fullyParallel: false` and `workers: 1` globally (`playwright.config.mjs` lines 27, 33), the individual `test.describe()` blocks do not use `test.describe.serial()`. This means if someone changes the global config to enable parallelism, these tests could run concurrently and interfere with each other (they share the same mock OAuth server state). Currently not a problem, but it is a latent risk.
- **Recommendation**: No action needed given the global config. The shared server pattern with `@/fixtures.mjs` auto-reset makes this safe. If parallel execution is ever desired, the mock OAuth server would need per-test isolation (separate ports or state namespacing).
- **Rationale**: Defensive coding for future config changes.

### Finding 11: Second MCP Instance Creation Re-Authorizes Instead of Reusing Token
- **Priority**: Important
- **File**: `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-oauth-auth.spec.mjs`
- **Location**: Lines 92-115, step "Create second MCP with same OAuth config (reuse existing)"
- **Issue**: The test for config reuse (line 92-115) goes through the full OAuth authorization flow again for the second MCP instance -- navigating to `/authorize` and clicking approve. This tests that the same auth config *can be selected again*, but it does not verify that an existing OAuth token is reused without re-authorization. The flow is: select config -> click connect -> navigate to authorize page -> approve -> callback -> create. If the feature intent is that a user who already has a valid token for a config should skip re-authorization, this test does not verify that behavior. If re-authorization is always required per-instance, then the test is correct but the step name "reuse existing" is misleading.
- **Recommendation**: Clarify the expected behavior. If token reuse is supported (existing token detected, connect button shows "Connected" without needing to re-authorize), update the test to verify that. If re-authorization is always required, rename the step to "Create second MCP with same OAuth config (re-authorize required)".
- **Rationale**: The step name "reuse existing" implies the token should be reused, but the test code shows a full re-authorization flow. This ambiguity makes it unclear whether the test is validating the correct behavior.

### Finding 12: Playground Result Content Not Validated
- **Priority**: Nice-to-have
- **File**: `mcps-oauth-auth.spec.mjs`, `mcps-oauth-dcr.spec.mjs`
- **Location**: Lines 82-90 (oauth-auth test 1), Lines 108-116 (oauth-dcr test 1)
- **Issue**: The playground execution steps verify `expectPlaygroundResultSuccess()` (which checks `data-test-state='success'`), but do not inspect the actual result content. The echo tool should return `"echo: Hello from OAuth E2E"`, and verifying this would confirm the bearer token was correctly forwarded to the MCP server and the tool executed with the right parameters. The `McpsPage` has a `getPlaygroundResultContent()` method (line 560-562) that is never used in any OAuth test.
- **Recommendation**: Add `const result = await mcpsPage.getPlaygroundResultContent(); expect(result).toContain('echo: Hello from OAuth E2E');` to at least one playground execution step to verify end-to-end data flow.
- **Rationale**: Checking only the status badge could miss issues where the tool returns success but with wrong data (e.g., token forwarded incorrectly, tool receives empty params).

### Finding 13: XSS Risk in Mock OAuth Server Authorization Page
- **Priority**: Important
- **File**: `crates/lib_bodhiserver_napi/test-mcp-oauth-server/src/oauth.ts`
- **Location**: Lines 142-159
- **Issue**: The authorization page HTML renders `client_id` and `scope` query parameters directly into the HTML without escaping: `<p>Client <strong>${client_id}</strong> is requesting access to: <strong>${scope ?? 'mcp:tools'}</strong></p>`. Similarly, form hidden fields inject `redirect_uri`, `state`, etc. as unescaped values. While this is a test-only server, injecting user-controlled values into HTML creates an XSS vector. If a test ever passes a crafted `client_id` containing `<script>` tags or event handlers, it could interfere with Playwright's page interactions.
- **Recommendation**: Add basic HTML escaping for interpolated values (e.g., `encodeURIComponent` or a simple `escapeHtml()` helper). For a test server this is low risk, but it prevents potential test interference from unexpected characters in test data.
- **Rationale**: Even in test infrastructure, good security practices prevent subtle bugs and set a positive example.

---

## Summary

The E2E test suite provides strong coverage of the three primary MCP auth journeys (OAuth pre-registered, OAuth DCR, header auth) including both first-party UI flows and third-party access request flows. The test structure follows E2E.md conventions well: `test.step()` is used consistently, tests tell coherent user stories, and the page object model provides good encapsulation.

**Strengths:**
- Comprehensive happy-path coverage across all three auth modes (10 tests total)
- The mock OAuth server is well-implemented with proper PKCE S256, refresh tokens, and DCR support
- Proper use of `test.step()` throughout, making Playwright traces readable
- Config reuse scenario explicitly tested
- Access request deny path covered (not just approve)
- Clean separation between API setup (via `page.evaluate`) and UI interaction

**Key gaps to address:**
- API helper methods in McpsPage need error checking (Finding 6) -- this will cause confusing failures in CI
- External Tavily dependency creates flakiness risk for header auth tests (Finding 7)
- DCR setup code is duplicated across three tests (Finding 5)
- No error scenario tests (invalid state, failed DCR, expired tokens)
- Playground result content not validated beyond status badge (Finding 12)
