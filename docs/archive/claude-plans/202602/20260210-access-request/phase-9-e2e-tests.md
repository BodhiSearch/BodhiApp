# Phase 9: E2E Tests with Real Keycloak

## Purpose

Implement end-to-end tests that exercise the complete access request flow with real Keycloak integration, verifying the entire system works together.

## Dependencies

- **Phase 0**: Keycloak SPI deployed and verified
- **Phase 1-8**: All implementation phases complete
- **Test environment**: Keycloak dev instance with SPI configured

## Prerequisites

### Keycloak Dev Environment
- Keycloak instance running with:
  - `scope_access_request` dynamic scope configured at realm level
  - Custom SPI for consent registration deployed (`/bodhi/users/request-access`)
  - Test resource client configured (BodhiApp)
  - Test app client registered (public client)
  - Test user account with credentials

### BodhiApp Test Configuration
- Test Exa API key for toolset testing
- Test user with Exa tool instance configured and enabled
- Database with test fixtures (or clean slate for each test)

## E2E Test Journeys

### Journey 1: Happy Path — Popup Flow (Chrome Extension Pattern)

**Test**: `test_popup_flow_approval_and_toolset_execution`

**Steps**:
1. App calls `POST /bodhi/v1/apps/request-access` with:
   - `flow_type: "popup"`
   - `tools: [{"tool_type": "builtin-exa-search"}]`
2. Assert response contains:
   - `access_request_id` (UUID)
   - `review_url` (valid URL)
   - `scopes` array with `scope_resource-*` and `scope_access_request:<uuid>`
3. Open `review_url` in Playwright browser context
4. Assert: redirected to login page (user not logged in)
5. Fill login form with test user credentials, submit
6. Assert: redirected back to review page
7. Review page displays:
   - App client ID
   - "Exa Web Search" tool type
   - Dropdown with user's Exa instance
8. Select user's Exa instance from dropdown
9. Click "Approve" button
10. Assert: API call to `/apps/access-request/{id}/approve` succeeds
11. Assert: DB row updated (status=approved, user_id set, tools_approved contains instance UUID)
12. Assert: window closes (popup flow) — verify via window.closed or test framework
13. App launches OAuth flow:
    - Redirect to Keycloak authorize endpoint
    - Scopes: `scope_resource-<id>` + `scope_access_request:<uuid>` + `openid`
14. Assert: Keycloak consent screen displays registered description
15. User consents in OAuth flow
16. Assert: OAuth callback receives authorization code
17. App exchanges code for token
18. Assert: Access token contains `access_request_id` claim with correct UUID
19. App calls BodhiApp token exchange with external token
20. Assert: Exchanged token also contains `access_request_id` claim
21. App calls `POST /bodhi/v1/toolsets/{instance-id}/execute/search` with bearer token
22. Assert: Exa search executes successfully (real API call)
23. Assert: Response contains search results

**Acceptance**: Full flow from draft creation to toolset execution succeeds.

### Journey 2: Happy Path — Redirect Flow (3rd Party App)

**Test**: `test_redirect_flow_approval`

**Steps**:
1. App calls `POST /bodhi/v1/apps/request-access` with:
   - `flow_type: "redirect"`
   - `redirect_uri: "http://localhost:9999/callback"`
   - `tools: [{"tool_type": "builtin-exa-search"}]`
2. Receive response with `access_request_id`, `review_url`, `scopes`
3. Open `review_url` in browser
4. Login as test user
5. Review and approve access request
6. Assert: Browser redirected to `http://localhost:9999/callback`
7. Complete OAuth flow (similar to Journey 1 steps 13-18)
8. Verify token has `access_request_id` claim
9. Call toolset API with token
10. Verify successful execution

**Acceptance**: Redirect flow navigates to redirect_uri after approval.

### Journey 3: Denial Flow

**Test**: `test_access_request_denial`

**Steps**:
1. Create draft access request
2. Open review_url, login
3. Click "Deny" button
4. Assert: DB row updated (status=denied, user_id set)
5. Assert: Browser action (close window or redirect) occurs
6. App polls `GET /bodhi/v1/apps/request-access/{id}`
7. Assert: Status is "denied"
8. App attempts OAuth flow anyway (optional — test Keycloak behavior)
9. Assert: Consent screen does NOT show approved tools (or flow fails)

**Acceptance**: Denial is recorded and OAuth flow cannot proceed with approved scopes.

### Journey 4: Expired Draft

**Test**: `test_expired_access_request`

**Steps**:
1. Create draft access request
2. Wait 10 minutes (or manipulate time in test environment)
3. App polls `GET /bodhi/v1/apps/request-access/{id}`
4. Assert: Response has status "expired"
5. Open review_url in browser
6. Assert: Page shows "expired" message
7. Assert: Approve/Deny buttons disabled or hidden

**Acceptance**: Expired drafts cannot be approved and show appropriate UI.

### Journey 5: Wrong User Token Validation

**Test**: `test_access_request_wrong_user`

**Steps**:
1. User A creates and approves access request
2. Assert: DB row has user_id=A, status=approved
3. User B obtains their own session token (separate login)
4. App uses User A's access_request_id but User B's token for OAuth flow
5. Complete OAuth flow with User B's consent
6. Obtain token with `access_request_id` claim (but for User B)
7. Call toolset API with this token
8. Assert: Auth middleware rejects (user_id mismatch)
9. Assert: Error response indicates invalid access request

**Acceptance**: Access requests are bound to the approving user and cannot be used by others.

### Journey 6: Revoked/Tampered Access Request

**Test**: `test_revoked_access_request`

**Steps**:
1. Complete approval flow (Journey 1), obtain valid token
2. Delete access request from DB (simulating revocation)
3. Call toolset API with token (still has `access_request_id` claim)
4. Assert: Auth middleware rejects (access request not found in DB)
5. Assert: Error response indicates invalid or revoked access request

**Acceptance**: Even with valid token, deleted/revoked access requests are rejected.

### Journey 7: Keycloak SPI Idempotency

**Test**: `test_kc_consent_idempotency`

**Steps**:
1. Create and approve access request (first registration)
2. Assert: KC SPI returns 201 Created
3. Simulate retry: call approve endpoint again with same access_request_id
4. Assert: KC SPI returns 200 OK (idempotent)
5. Assert: Same scopes returned
6. Complete OAuth flow with returned scopes
7. Verify token is valid and toolset API call succeeds

**Acceptance**: Idempotent retries work correctly.

### Journey 8: Keycloak UUID Collision (409 Conflict)

**Test**: `test_kc_uuid_collision`

**Steps**:
1. Create access request with UUID `uuid-1` for resource-A, app-X, user-1
2. Approve and register with KC
3. Attempt to create new access request with same UUID `uuid-1` but for resource-B, app-Y, user-2 (different context)
4. Approve and attempt KC registration
5. Assert: KC SPI returns 409 Conflict
6. Assert: BodhiApp surfaces error to user
7. Retry with new UUID
8. Assert: Registration succeeds with 201 Created

**Acceptance**: UUID collisions are detected and handled gracefully.

## Test Implementation

### Test Files

**File**: `crates/bodhi/tests/e2e/access_request_flow.spec.ts` (Playwright)

Use Playwright for browser automation:
- Navigate to review URLs
- Fill login forms
- Interact with review page
- Verify redirects and window closures

**File**: `crates/routes_app/tests/integration/access_request_e2e.rs` (Rust integration test)

Use Rust integration tests for API calls:
- Call BodhiApp API endpoints
- Verify database state
- Mock or call real Keycloak (depending on test environment)

### Keycloak Interaction

**Option A: Real Keycloak** (preferred for true e2e):
- Tests require running Keycloak dev instance
- Use test credentials and clients configured in KC
- Full OAuth flow including consent screen

**Option B: Mocked Keycloak** (for CI/CD if KC not available):
- Use mockito or similar to mock KC endpoints
- Mock consent registration SPI
- Mock OAuth token endpoints
- Limits coverage but enables automated testing

### Test Environment Setup

**Script**: `scripts/setup-e2e-test-env.sh` (new, if needed)

Automate test environment setup:
- Start Keycloak (Docker?)
- Configure realm, clients, users
- Deploy SPI
- Start BodhiApp test instance
- Set up test data (Exa API key, tool instances)

**Cleanup**: Ensure tests clean up after themselves (delete test users, access requests).

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `crates/bodhi/tests/e2e/access_request_flow.spec.ts` | Create | Playwright e2e tests |
| `crates/routes_app/tests/integration/access_request_e2e.rs` | Create | Rust integration tests (optional) |
| `scripts/setup-e2e-test-env.sh` | Create | Test environment setup (if needed) |

## Research Questions

1. **Existing e2e tests**: How are current e2e tests structured? (Check existing Playwright tests)
2. **Keycloak test setup**: Is there existing test KC configuration? (Check test fixtures)
3. **OAuth flow testing**: How do we test OAuth in Playwright? (Check existing OAuth tests)
4. **Time manipulation**: How do we fast-forward time for expiry tests? (Check test utilities)
5. **Test isolation**: How do we ensure tests don't interfere with each other? (Check existing patterns)
6. **CI/CD integration**: Can KC be spun up in CI, or do we mock? (Check CI config)

## Acceptance Criteria

### Journey Coverage
- [ ] Journey 1 (popup flow) passes end-to-end
- [ ] Journey 2 (redirect flow) passes
- [ ] Journey 3 (denial) passes
- [ ] Journey 4 (expired) passes
- [ ] Journey 5 (wrong user) passes
- [ ] Journey 6 (revoked) passes
- [ ] Journey 7 (idempotency) passes
- [ ] Journey 8 (UUID collision) passes

### Test Quality
- [ ] Tests are deterministic (no flaky failures)
- [ ] Tests clean up after themselves
- [ ] Clear test names and documentation
- [ ] Tests run in reasonable time (<5min total)
- [ ] Tests can run in CI/CD (or documented as manual only)

### Documentation
- [ ] Test setup instructions documented
- [ ] Prerequisites clearly listed
- [ ] How to run tests documented
- [ ] Known limitations documented

## Notes for Sub-Agent

- **Playwright skill**: Use for data-testid patterns and page object structure
- **Real KC preferred**: If possible, use real Keycloak for true e2e validation
- **Time manipulation**: May need test utilities for expiry testing
- **Parallel execution**: Consider whether tests can run in parallel
- **Test data**: Use unique identifiers to avoid conflicts between tests
- **Cleanup**: Always clean up test data to avoid pollution
- **Debugging**: Add screenshots/videos on failure for debugging

## Verification

```bash
# Setup test environment (if automated)
./scripts/setup-e2e-test-env.sh

# Run e2e tests
cd crates/bodhi && npm run test:e2e

# Or with Playwright
npx playwright test access_request_flow
```

## After Implementation

Document any manual setup steps required for running these tests in `README.md` or `TESTING.md`.

## Final Verification

After Phase 9 passes, the access request revamp is **complete**. Verify:
- [ ] All phases implemented
- [ ] All tests passing (unit, integration, component, e2e)
- [ ] Documentation updated
- [ ] OpenAPI specs regenerated
- [ ] TypeScript client updated
- [ ] No breaking changes in other flows
- [ ] Performance acceptable (no significant slowdowns)

🎉 **Access Request Revamp Complete!**
