# Playwright Test Migration Plan: Procedural to Page Object Model

## Overview
This document outlines the phase-wise migration plan for converting procedural Playwright tests from `crates/lib_bodhiserver_napi/tests-js/playwright/` to the Page Object Model pattern in `crates/lib_bodhiserver_napi/tests-js/specs/`.

## Current State Analysis

### Procedural Tests Status
| Test File | Migration Status | Dependencies |
|-----------|-----------------|--------------|
| `app-initializer-redirects.spec.mjs` | ✅ Fully Migrated | Can be removed |
| `auth-flow-integration.spec.mjs` | ✅ Partially Migrated | Basic flows migrated, can be removed |
| `public-host-auth.spec.mjs` | ❌ Not Migrated | auth-server-client, bodhi-app-server |
| `canonical-host-redirect.spec.mjs` | ❌ Not Migrated | auth-server-client, bodhi-app-server |
| `debug-auth-flow.spec.mjs` | ❌ Not Migrated | auth-server-client, bodhi-app-server |
| `oauth2-token-exchange-v2.spec.mjs` | ❌ Not Migrated | auth-server-client, bodhi-app-server, static-server |

### Existing Page Objects Available
- `BasePage.mjs` - Base class with common methods
- `LoginPage.mjs` - OAuth login flows
- `SetupResourceAdminPage.mjs` - Resource admin setup
- `RequestAccessPage.mjs` - Access request flows

### Already Migrated Coverage
- **App Initializer Tests**: Fully covered in `specs/core/app-initializer/app-initializer.spec.mjs`
  - App status-based redirects (setup/ready modes)
  - Basic OAuth authentication flow
  - Protected route redirects to login
- **Setup Flow Tests**: Covered in `specs/setup/setup-flow.spec.mjs`
  - Complete first-time setup flow with OAuth

---

## Phase 1: Public Host Authentication Migration

### Objective
Migrate OAuth flow tests with public host configuration settings.

### Tasks
1. **Create new spec file**: `specs/auth/public-host-auth.spec.mjs`
   - Test OAuth flow with public host environment variables
   - Validate callback URLs use public host settings
   - Test redirect behavior with custom public host configuration

2. **Page Objects Required**
   - Use existing `LoginPage.mjs`
   - No new page objects needed

3. **Test Scenarios to Migrate**
   - OAuth flow completion with public host settings (BODHI_PUBLIC_HOST, BODHI_PUBLIC_SCHEME, BODHI_PUBLIC_PORT)
   - Callback URL validation using public host configuration
   - User authentication state verification after OAuth flow

4. **Validation Steps**
   - Run new spec tests to ensure all scenarios pass
   - Compare with original test output for completeness
   - Verify public host configuration is properly tested

5. **Cleanup**
   - Remove `playwright/public-host-auth.spec.mjs`
   - Update any references in test documentation

### Success Criteria
- All public host authentication scenarios passing in new spec
- Original procedural test file removed
- No regression in test coverage

---

## Phase 2: Canonical Redirect Migration

### Objective
Migrate canonical URL redirect tests for host normalization.

### Tasks
1. **Create new spec file**: `specs/auth/canonical-redirect.spec.mjs`
   - Test redirect behavior when canonical redirect is enabled
   - Test no redirect when canonical redirect is disabled
   - Cover both 127.0.0.1 → localhost scenarios

2. **Page Objects Required**
   - Create `CanonicalRedirectPage.mjs` (extends BasePage)
     - Methods: `expectRedirect()`, `expectNoRedirect()`, `verifyCanonicalUrl()`
   - Reuse existing `BasePage.mjs` utilities

3. **Test Scenarios to Migrate**
   - Canonical redirect enabled: 127.0.0.1 redirects to localhost
   - Canonical redirect disabled: 127.0.0.1 stays as-is
   - Verify URL normalization with different ports
   - Test with different paths (/ui/login, /ui/chat, etc.)

4. **Environment Variables**
   - BODHI_CANONICAL_REDIRECT: true/false
   - BODHI_PUBLIC_HOST, BODHI_PUBLIC_SCHEME, BODHI_PUBLIC_PORT

5. **Validation Steps**
   - Test both enabled and disabled states
   - Verify redirect preserves path and query parameters
   - Ensure no infinite redirect loops

6. **Cleanup**
   - Remove `playwright/canonical-host-redirect.spec.mjs`
   - Update test documentation

### Success Criteria
- Canonical redirect behavior fully tested in both states
- Clean page object abstraction for redirect testing
- Original test file removed

---

## Phase 3: OAuth2 Token Exchange V2 Migration

### Objective
Migrate complex OAuth2 token exchange flow with dynamic audience and app client management.

### Tasks
1. **Create new spec file**: `specs/auth/oauth2-token-exchange.spec.mjs`
   - Complete OAuth2 token exchange v2 flow
   - Dynamic audience request via Bodhi App API
   - Token validation and API access testing
   - Error handling scenarios

2. **Page Objects Required**
   - Create `OAuth2SetupPage.mjs`
     - Methods: `setupResourceAdmin()`, `completeInitialSetup()`
   - Create `OAuth2TestAppPage.mjs`
     - Methods: `configureOAuth()`, `startFlow()`, `extractToken()`, `verifyTokenExchange()`
   - Create `TokenExchangePage.mjs`
     - Methods: `requestAudience()`, `validateToken()`, `testApiAccess()`
   - Reuse existing `LoginPage.mjs`, `SetupResourceAdminPage.mjs`

3. **Test Scenarios to Migrate**
   - Complete OAuth2 Token Exchange v2 flow with dynamic audience
     - Server setup in 'setup' mode
     - Resource admin configuration
     - App client creation with redirect URIs
     - Audience access request via API
     - OAuth flow through test app
     - Token extraction and validation
     - API access with OAuth token
   - Token exchange error handling
     - Invalid client credentials
     - Unauthenticated user responses

4. **Complex Flow Steps**
   ```
   1. Setup server in 'setup' mode
   2. Complete resource admin setup
   3. Obtain dev console token
   4. Start static server for OAuth test app
   5. Create app client with test app redirect URI
   6. Request audience access via Bodhi App API
   7. Navigate to test app and complete OAuth flow
   8. Extract access token from UI
   9. Test API access with OAuth token
   10. Validate error handling scenarios
   ```

5. **Static Server Integration**
   - Reuse `static-server.mjs` for OAuth test app hosting
   - Serve `oauth-test-app.html` for token exchange testing

6. **Validation Steps**
   - Full end-to-end OAuth2 flow validation
   - Token audience and scope verification
   - API access with exchanged tokens
   - Error response validation

7. **Cleanup**
   - Remove `playwright/oauth2-token-exchange-v2.spec.mjs`
   - Archive static-server if no longer needed elsewhere

### Success Criteria
- Complex OAuth2 flow fully migrated with page objects
- Clean abstraction of multi-step authentication flow
- Error scenarios properly tested
- Original test file removed

---

## Phase 4: Final Migration and Cleanup

### Objective
Migrate remaining test scenarios and remove all procedural tests.

### Tasks
1. **Migrate Debug Authentication Flow**
   - Target: Add to existing `specs/core/app-initializer/app-initializer.spec.mjs`
   - Create debug helper methods in `LoginPage.mjs`
   - Add console logging scenarios for debugging
   - Test user info API responses
   - Navigation flow validation with debug output

2. **Remove Already Migrated Tests**
   - Delete `playwright/app-initializer-redirects.spec.mjs` (fully migrated)
   - Delete `playwright/auth-flow-integration.spec.mjs` (fully migrated)
   - Delete `playwright/debug-auth-flow.spec.mjs` (after adding to app-initializer)

3. **Update Test Infrastructure**
   - Move reusable helpers from `playwright/` to shared location if needed
   - Update `auth-server-client.mjs` imports in specs
   - Update `bodhi-app-server.mjs` imports in specs

4. **Documentation Updates**
   - Update test README with new structure
   - Document page object patterns used
   - Add examples of how to create new tests with page objects

5. **Validation**
   - Run full test suite to ensure no regressions
   - Verify all scenarios from procedural tests are covered
   - Check test execution time hasn't significantly increased

### Success Criteria
- No procedural test files remain in `playwright/` directory
- All test scenarios migrated to page object model
- Test suite passes without regressions
- Documentation updated

---

## Migration Guidelines

### Page Object Best Practices
1. **Single Responsibility**: Each page object represents one page/component
2. **Encapsulation**: Hide implementation details, expose semantic methods
3. **Reusability**: Create methods that can be reused across tests
4. **Maintainability**: Centralize selectors and wait conditions

### Method Naming Conventions
- Actions: `performAction()`, `clickButton()`, `fillForm()`
- Validations: `expectCondition()`, `verifyState()`, `assertValue()`
- Navigation: `navigateTo()`, `goToPage()`
- Getters: `getElementText()`, `getCurrentUrl()`

### Test Organization
```
specs/
├── auth/                     # Authentication related tests
│   ├── public-host-auth.spec.mjs
│   ├── canonical-redirect.spec.mjs
│   └── oauth2-token-exchange.spec.mjs
├── core/                     # Core functionality
│   └── app-initializer/
│       └── app-initializer.spec.mjs (enhanced)
└── setup/                    # Setup flows (existing)
```

### Common Patterns to Follow
1. **Setup/Teardown**: Use beforeAll/afterAll for server management
2. **Page Object Initialization**: Create in beforeEach for fresh instances
3. **Async/Await**: Consistent use of async operations
4. **Error Handling**: Proper try/finally blocks for cleanup
5. **Timeouts**: Use page.waitForURL, waitForSPAReady consistently

---

## Risk Mitigation

### Potential Risks
1. **Test Coverage Gap**: Some edge cases might be missed during migration
   - Mitigation: Run both old and new tests in parallel initially

2. **Performance Impact**: Page objects might add overhead
   - Mitigation: Profile test execution times, optimize if needed

3. **Flaky Tests**: New abstractions might introduce timing issues
   - Mitigation: Add proper wait conditions, avoid hard-coded timeouts

4. **Complex Scenarios**: OAuth2 flow has many moving parts
   - Mitigation: Break down into smaller, testable components

### Rollback Plan
- Keep procedural tests in a backup branch until migration is validated
- Can revert individual phases if issues are found
- Maintain test execution logs for comparison

---

## Timeline Estimate

| Phase | Estimated Duration | Dependencies |
|-------|-------------------|--------------|
| Phase 1: Public Host Auth | 2-3 hours | LoginPage exists |
| Phase 2: Canonical Redirect | 3-4 hours | Need new page object |
| Phase 3: OAuth2 Token Exchange | 6-8 hours | Complex flow, multiple page objects |
| Phase 4: Final Migration | 2-3 hours | Previous phases complete |
| **Total** | **13-18 hours** | |

---

## Verification Checklist

### Per Phase
- [ ] All test scenarios identified and migrated
- [ ] Page objects created and documented
- [ ] Tests passing consistently
- [ ] Original test file removed
- [ ] No regression in coverage

### Final Validation
- [ ] All procedural tests removed
- [ ] Full test suite passing
- [ ] Documentation updated
- [ ] CI/CD pipeline validated
- [ ] Test execution time acceptable
- [ ] No flaky tests introduced

---

## Notes
- Static server and auth server client utilities should be preserved as they're used across multiple specs
- Consider creating a shared fixtures directory for common test data
- Browser extension tests are already migrated and can serve as reference
- The migration should maintain or improve test execution speed