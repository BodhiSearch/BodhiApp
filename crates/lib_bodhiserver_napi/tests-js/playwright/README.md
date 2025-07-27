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
# Authentication Server Configuration (Keycloak v26)
INTEG_TEST_MAIN_AUTH_URL=https://main-id.getbodhi.app
INTEG_TEST_AUTH_REALM=bodhi

# Test User Credentials (required)
INTEG_TEST_USERNAME=user@email.com
INTEG_TEST_PASSWORD=pass
```

**⚠️ Important Changes in v26**: 
- The tests now use **dynamic client creation** instead of pre-configured clients
- Client credentials are created on-demand using the OAuth2 Token Exchange v2 flow
- The auth server has been updated from `dev-id.getbodhi.app` to `main-id.getbodhi.app`
- No longer requires `INTEG_TEST_CLIENT_ID` or `INTEG_TEST_CLIENT_SECRET` environment variables

### OAuth2 Token Exchange v2 Flow

The tests implement the new OAuth2 Token Exchange v2 standard with dynamic audience management:

1. **Dev Console Token**: Obtain token using direct access grant for `user@email.com/pass`
2. **App Client Creation**: Create public app client via `/realms/bodhi/bodhi/apps` endpoint
3. **Resource Client Creation**: Create confidential resource client via `/realms/bodhi/bodhi/resources` endpoint
4. **Audience Request**: Resource client requests access via `/realms/bodhi/bodhi/resources/request-access`
5. **User Consent**: App user authorizes with resource-specific scope
6. **Token Exchange**: Standard v2 format without explicit audience parameter

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

The tests cover:

- **Authentication Flow**: Complete OAuth login/logout cycle
- **App Initialization**: Different app states (setup, ready, error)
- **First-Time Setup**: End-to-end setup flow for new users
- **Dynamic Client Management**: On-demand creation of OAuth clients
- **Token Exchange**: OAuth2 Token Exchange v2 validation

### Troubleshooting

**Common Issues:**

1. **Authentication Failures**: Ensure test credentials are valid in the Keycloak realm
2. **Client Creation Errors**: Verify the dev console user has proper permissions
3. **Token Exchange Failures**: Check that the audience access request completed successfully
4. **Network Timeouts**: The auth server may be slow; increase timeout values if needed

**Debug Mode:**
```bash
# Run with verbose logging
npx playwright test --debug --headed

# Run specific test file
npx playwright test first-time-auth-setup.spec.js --debug
``` 