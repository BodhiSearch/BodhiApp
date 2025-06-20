# Playwright Integration Tests

This directory contains Playwright integration tests for the Bodhi App, including end-to-end authentication flows.

## Test Files

- `auth-flow-integration.spec.js` - OAuth authentication flow tests (login/logout)
- `app-initializer-redirects.spec.js` - App status-based redirect behavior tests  
- `first-time-auth-setup.spec.js` - **NEW** Complete first-time authentication setup flow
- `playwright-helpers.js` - Shared utilities for test setup

## First-Time Authentication Setup Test

The `first-time-auth-setup.spec.js` test simulates the complete flow for a new user setting up the Bodhi App with authentication for the first time:

### Test Flow

1. **Setup Registration**: App registers with auth backend (Keycloak)
2. **Resource Admin Page**: User clicks "Continue with Login" 
3. **Keycloak Login**: User enters credentials on external auth server
4. **Auth Callback**: App processes authentication callback
5. **Download Models**: User is redirected to model download page

### Environment Setup

Create a `.env.test` file in this directory with the following variables:

```bash
# Authentication Server Configuration
INTEG_TEST_AUTH_URL=https://dev-id.getbodhi.app
INTEG_TEST_AUTH_REALM=bodhi

# Test User Credentials (required)
INTEG_TEST_USERNAME=your-test-username@example.com  
INTEG_TEST_PASSWORD=your-test-password

# Optional: Client Configuration
INTEG_TEST_CLIENT_ID=
INTEG_TEST_CLIENT_SECRET=
```

**⚠️ Important**: The test requires valid Keycloak credentials. Ensure your test user exists in the specified realm.

### Running the Tests

```bash
# Install dependencies (if not already done)
npm install

# Run all Playwright tests
npx playwright test

# Run only the first-time auth setup test
npx playwright test first-time-auth-setup.spec.js

# Run tests with UI mode for debugging
npx playwright test --ui

# Run tests with debug mode
npx playwright test --debug
```

### Test Scenarios

1. **Happy Path**: Complete successful authentication flow
2. **Error Handling**: Invalid credentials and error recovery
3. **Progress Tracking**: Verify setup progress indicators throughout

### Technical Details

- **Server Configuration**: Tests start with `appStatus: 'setup'` to simulate first-time setup
- **Timeout Handling**: Extended timeouts for external auth server redirects
- **Cross-Origin Flow**: Handles redirects between local app and external Keycloak
- **SPA Ready Checks**: Waits for full page load and DOM content before interactions

### Troubleshooting

**Common Issues:**

1. **Environment Variables**: Ensure `.env.test` file exists with valid credentials
2. **Network Timeouts**: External auth server may be slow - timeouts are set to 15-20 seconds
3. **Keycloak Changes**: If auth server HTML structure changes, update selectors
4. **Server Startup**: Tests manage their own server lifecycle - no manual server start needed

**Debug Tips:**

- Use `--debug` flag to step through tests
- Check browser developer tools during test execution
- Review generated screenshots/videos in `test-results/` directory
- Verify auth server is accessible at configured URL

### Dependencies

- `@playwright/test` - Test framework
- `dotenv` - Environment variable loading
- Local NAPI bindings for server management

The tests use the existing `createServerManager` utility to start isolated server instances with specific configurations for each test scenario. 