# Frontend UI Testing (Playwright)

> **AI Coding Assistant Guide**: This document provides concise Playwright UI testing conventions and patterns for the Bodhi App. Focus on established patterns and avoid conditional logic.

## Required Documentation References

**MUST READ for UI testing:**
- `ai-docs/01-architecture/frontend-testing.md` - Frontend testing philosophy and patterns
- `ai-docs/01-architecture/development-conventions.md` - General testing conventions

**FOR DEEPER CONTEXT:**
- `crates/lib_bodhiserver_napi/tests-js/playwright/` - Existing test examples
- `crates/lib_bodhiserver_napi/playwright.config.js` - Playwright configuration

## Testing Philosophy

### UI Testing Pyramid
- **Few but Valuable** - Comprehensive user journeys over micro tests
- **One Thing Through** - Each test covers one complete user flow
- **Deterministic** - Tests pass or fail clearly without conditional logic
- **Fast Failure** - Fail immediately when conditions aren't met

### Quality Goals
- **Complete User Journeys** - Test full workflows from start to finish
- **Server Integration** - Real server startup and browser automation
- **10-Second Timeout** - All operations use consistent global timeout

## Technology Stack

### Core UI Testing Tools
- **Playwright** - Browser automation with JavaScript/ES modules
- **Chromium** - Single browser target for consistency
- **NAPI Server Bindings** - Real server instances for authentic testing
- **Sequential Execution** - Single worker to avoid port conflicts

### Configuration Location
- **Test Directory**: `crates/lib_bodhiserver_napi/tests-js/playwright/`
- **Config File**: `crates/lib_bodhiserver_napi/playwright.config.js`
- **Helpers**: `crates/lib_bodhiserver_napi/tests-js/playwright/playwright-helpers.js`

## Critical Configuration

### Playwright Timeout Setup
```javascript
// playwright.config.js
export default defineConfig({
  timeout: 10000, // 10s global timeout - DO NOT override in individual tests
  use: {
    navigationTimeout: 10000, // Consistent with global
    actionTimeout: 10000, // Consistent with global
  },
  workers: 1, // Sequential execution for server tests
  fullyParallel: false,
});
```

**NEVER use individual timeouts** in test expectations:
```javascript
// ❌ Don't do this
await expect(element).toBeVisible({ timeout: 15000 });

// ✅ Use global timeout
await expect(element).toBeVisible();
```

## Server Management Patterns

### Standardized Server Setup
```javascript
// Standard pattern for all test files
import { createServerManager, waitForSPAReady, getCurrentPath } from './playwright-helpers.js';

test.describe('Feature Tests', () => {
  let serverManager;
  let baseUrl;

  test.beforeAll(async () => {
    serverManager = createServerManager({ 
      appStatus: 'ready', // or 'setup', 'resource-admin'
      // other config options
    });
    baseUrl = await serverManager.startServer();
  });

  test.afterAll(async () => {
    await serverManager.stopServer();
  });

  // Tests here...
});
```

**NEVER use conditional server management**:
```javascript
// ❌ Don't do this
test.afterAll(async () => {
  if (serverManager) {
    try {
      await serverManager.stopServer();
    } catch (error) {
      console.warn('Failed to stop server:', error);
    }
  }
});

// ✅ Simple and deterministic
test.afterAll(async () => {
  await serverManager.stopServer();
});
```

## Standard Helper Functions

### Required Helper Imports
```javascript
import { 
  createServerManager,
  waitForSPAReady,
  waitForRedirect,
  getCurrentPath 
} from './playwright-helpers.js';
```

### Page Navigation Pattern
```javascript
test('should complete user workflow', async ({ page }) => {
  // Navigate and wait for SPA
  await page.goto(baseUrl);
  await waitForSPAReady(page);

  // Verify current location
  const currentPath = getCurrentPath(page);
  expect(currentPath).toBe('/ui/expected-path/');

  // Interact with elements
  const button = page.locator('button:has-text("Click Me")');
  await expect(button).toBeVisible();
  await button.click();

  // Wait for redirect
  await waitForRedirect(page, '/ui/next-page/');
  
  // Verify final state
  await expect(page.locator('text=Success')).toBeVisible();
});
```

## Test Organization Patterns

### File Structure Standards
```
tests-js/playwright/
├── playwright-helpers.js           # Shared utilities
├── app-initializer-redirects.spec.js   # App routing tests
├── auth-flow-integration.spec.js       # OAuth workflows  
├── first-time-auth-setup.spec.js       # Complete setup flow
└── README.md                           # Test documentation
```

### Test Grouping Pattern
```javascript
test.describe('Feature Area - Specific Workflow', () => {
  // Server setup once per group
  let serverManager;
  let baseUrl;

  test.beforeAll(async () => {
    serverManager = createServerManager(config);
    baseUrl = await serverManager.startServer();
  });

  test.afterAll(async () => {
    await serverManager.stopServer();
  });

  test('should complete comprehensive workflow with multiple steps', async ({ page }) => {
    // Test complete user journey in single test
    // Step 1: Initial navigation
    // Step 2: User interaction
    // Step 3: Form submission
    // Step 4: Final verification
  });
});
```

## Critical Testing Anti-Patterns

### ❌ NEVER Use These Patterns

**Conditional Logic**:
```javascript
// ❌ Don't do this
if (await element.isVisible()) {
  await element.click();
} else {
  console.log('Element not found');
}

// ✅ Deterministic expectation
await expect(element).toBeVisible();
await element.click();
```

**Try-Catch Blocks**:
```javascript
// ❌ Don't do this
try {
  await page.goto(url);
} catch (error) {
  console.warn('Navigation failed:', error);
  return; // Skip test
}

// ✅ Let it fail clearly
await page.goto(url);
```

**Console Logging**:
```javascript
// ❌ Don't do this
console.log('Navigating to:', url);
console.log('Current path:', currentPath);

// ✅ Use test expectations
expect(getCurrentPath(page)).toBe(expectedPath);
```

**Individual Timeouts**:
```javascript
// ❌ Don't do this  
await expect(element).toBeVisible({ timeout: 30000 });

// ✅ Use global timeout
await expect(element).toBeVisible();
```

**Dotenv Imports**:
```javascript
// ❌ Don't do this
import { config } from 'dotenv';
config({ path: '.env.test' });

// ✅ Environment loaded globally in playwright.config.js
const authUrl = process.env.INTEG_TEST_AUTH_URL;
```

## Parameterized Testing Patterns

### Path Testing Loop
```javascript
test('should redirect all protected paths to login', async ({ page }) => {
  const protectedPaths = ['/', '/ui/chat', '/ui/models', '/ui/settings'];

  for (const path of protectedPaths) {
    await page.goto(`${baseUrl}${path}`);
    await waitForSPAReady(page);
    
    const currentPath = getCurrentPath(page);
    expect(currentPath).toBe('/ui/login/');
  }
});
```

### Server Configuration Testing
```javascript
test.describe.each([
  { appStatus: 'ready', expectedPath: '/ui/login/' },
  { appStatus: 'setup', expectedPath: '/ui/setup/' },
  { appStatus: 'resource-admin', expectedPath: '/ui/setup/resource-admin/' }
])('App Status: $appStatus', ({ appStatus, expectedPath }) => {
  let serverManager;
  let baseUrl;

  test.beforeAll(async () => {
    serverManager = createServerManager({ appStatus });
    baseUrl = await serverManager.startServer();
  });

  test.afterAll(async () => {
    await serverManager.stopServer();
  });

  test(`should redirect to ${expectedPath}`, async ({ page }) => {
    await page.goto(baseUrl);
    await waitForSPAReady(page);
    expect(getCurrentPath(page)).toBe(expectedPath);
  });
});
```

## Element Interaction Patterns

### Form Interactions
```javascript
// Standard form field interaction
const usernameField = page.locator('input[name="username"], input[type="email"]');
const passwordField = page.locator('input[name="password"], input[type="password"]');
const submitButton = page.locator('button[type="submit"], button:has-text("Sign In")');

await expect(usernameField).toBeVisible();
await expect(passwordField).toBeVisible();

await usernameField.fill(credentials.username);
await passwordField.fill(credentials.password);
await submitButton.click();
```

### Multi-Step Workflow
```javascript
test('should complete multi-step setup workflow', async ({ page }) => {
  // Step 1: Initial setup page
  await page.goto(baseUrl);
  await waitForSPAReady(page);
  expect(getCurrentPath(page)).toBe('/ui/setup/');
  
  const setupButton = page.locator('button:has-text("Setup Bodhi App")');
  await expect(setupButton).toBeVisible();
  await setupButton.click();

  // Step 2: Admin setup
  await waitForRedirect(page, '/ui/setup/resource-admin/');
  await expect(page.locator('text=Admin Setup')).toBeVisible();
  
  const loginButton = page.locator('button:has-text("Continue with Login")');
  await expect(loginButton).toBeVisible();
  await loginButton.click();

  // Step 3: External auth (OAuth server)
  await expect(page.locator('text=Sign in to your account')).toBeVisible();
  // ... authentication steps
  
  // Step 4: Return to app and verify final state
  await waitForRedirect(page, '/ui/chat/');
  await expect(page.locator('text=Welcome to Chat')).toBeVisible();
});
```

## Environment Configuration

### Test Environment Variables
Environment variables are automatically loaded from `tests-js/playwright/.env.test` by the Playwright configuration. **No need to import dotenv in individual test files.**

```javascript
// Test environment variables are already loaded globally
function getTestConfig() {
  return {
    authUrl: process.env.INTEG_TEST_AUTH_URL,
    username: process.env.INTEG_TEST_USERNAME,
    password: process.env.INTEG_TEST_PASSWORD,
  };
}
```

**NEVER import dotenv in test files**:
```javascript
// ❌ Don't do this - environment is loaded globally
import { config } from 'dotenv';

// ✅ Environment variables are available directly
const authUrl = process.env.INTEG_TEST_AUTH_URL;
```

## Common Test Patterns

### App Status Redirect Testing
```javascript
test('should handle app status redirects correctly', async ({ page }) => {
  // Test with different app statuses in beforeAll setup
  await page.goto(baseUrl);
  await waitForSPAReady(page);
  
  const pageContent = await page.content();
  const currentPath = getCurrentPath(page);
  
  expect(pageContent.length).toBeGreaterThan(1000); // Verify content loaded
  expect(currentPath).toBe(expectedRedirectPath);
});
```

### Authentication Flow Testing
```javascript
test('should complete OAuth authentication to protected content', async ({ page }) => {
  // Start at protected page
  await page.goto(`${baseUrl}/ui/chat`);
  await waitForSPAReady(page);
  
  // Should redirect to login
  expect(getCurrentPath(page)).toBe('/ui/login/');
  
  // Initiate OAuth
  const loginButton = page.locator('button:has-text("Log In")');
  await loginButton.click();
  
  // Handle external auth server
  await page.waitForURL((url) => url.origin === 'https://auth-server.com');
  // ... auth steps
  
  // Verify return to protected content
  await page.waitForURL((url) => url.pathname === '/ui/chat/');
  expect(getCurrentPath(page)).toBe('/ui/chat/');
});
```

## Test Commands

### Running Playwright Tests
```bash
# Run all Playwright tests
npx playwright test --config=playwright.config.js

# Run specific test file
npx playwright test app-initializer-redirects.spec.js

# Run with headed browser (for debugging)
npx playwright test --headed

# Generate test report
npx playwright show-report
```

## Best Practices Summary

### Key Principles for UI Tests
1. **One Complete User Journey** per test
2. **Real Server Integration** with actual browser automation
3. **No Conditional Logic** - tests should be deterministic
4. **Global Timeout** configuration, never individual timeouts
5. **Sequential Execution** to avoid port conflicts
6. **Fast Failure** when expectations aren't met

### Checklist for New UI Tests
- [ ] Uses `createServerManager` for server lifecycle
- [ ] Imports helpers from `./playwright-helpers.js`
- [ ] No try-catch blocks or conditional logic
- [ ] No console.log statements
- [ ] No dotenv imports (loaded globally)
- [ ] Uses global timeout (no individual timeouts)
- [ ] Tests complete user workflow in single test
- [ ] Proper beforeAll/afterAll server management
- [ ] Uses standard element interaction patterns

### File Naming and Organization
- **Feature-based naming**: `feature-workflow.spec.js`
- **Clear test descriptions**: Focus on user journey, not technical implementation
- **Logical grouping**: Related workflows in same describe block
- **Helper functions**: Extract common patterns to playwright-helpers.js

## Related Documentation

- **[Frontend Testing](frontend-testing.md)** - React component testing patterns and philosophy
- **[Development Conventions](development-conventions.md)** - General testing conventions and file organization
- **[Backend Testing](backend-testing.md)** - Server-side testing approaches for integration

## Implementation References

**Key Files Referenced in This Guide**:
- `crates/lib_bodhiserver_napi/playwright.config.js` - Playwright configuration with 10s timeout
- `crates/lib_bodhiserver_napi/tests-js/playwright/playwright-helpers.js` - Standardized helper functions
- `crates/lib_bodhiserver_napi/tests-js/playwright/app-initializer-redirects.spec.js` - App routing test patterns
- `crates/lib_bodhiserver_napi/tests-js/playwright/auth-flow-integration.spec.js` - OAuth workflow testing
- `crates/lib_bodhiserver_napi/tests-js/playwright/first-time-auth-setup.spec.js` - Complete setup flow testing

---

*This guide reflects the established UI testing patterns and conventions. Always check existing test files for the most current patterns before implementing new tests.* 