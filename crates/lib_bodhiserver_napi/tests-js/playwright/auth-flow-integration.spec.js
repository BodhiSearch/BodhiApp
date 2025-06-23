import { expect, test } from '@playwright/test';
import { createServerManager, getCurrentPath, waitForSPAReady } from './playwright-helpers.js';

/**
 * Get test environment variables with defaults
 */
function getTestConfig() {
  return {
    authUrl: process.env.INTEG_TEST_AUTH_URL,
    authRealm: process.env.INTEG_TEST_AUTH_REALM,
    clientId: process.env.INTEG_TEST_CLIENT_ID,
    clientSecret: process.env.INTEG_TEST_CLIENT_SECRET,
    username: process.env.INTEG_TEST_USERNAME,
    password: process.env.INTEG_TEST_PASSWORD,
  };
}

test.describe('OAuth Authentication Flow Integration Tests', () => {
  let serverManager;
  let baseUrl;
  let testConfig;

  test.beforeAll(async () => {
    testConfig = getTestConfig();
    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: testConfig.authUrl,
      authRealm: testConfig.authRealm,
      clientId: testConfig.clientId,
      clientSecret: testConfig.clientSecret,
    });
    baseUrl = await serverManager.startServer();
  });

  test.afterAll(async () => {
    await serverManager.stopServer();
  });

  test('should redirect unauthenticated users and show functional login page', async ({ page }) => {
    const testPaths = ['/', '/ui/chat', '/ui/models', '/ui/settings'];

    // Test redirect behavior for all protected paths
    for (const path of testPaths) {
      await page.goto(`${baseUrl}${path}`);
      await waitForSPAReady(page);

      const pageContent = await page.content();
      const currentPath = getCurrentPath(page);

      expect(pageContent.length).toBeGreaterThan(1000);
      expect(currentPath).toBe('/ui/login/');
    }

    // Verify login page functionality
    await page.goto(`${baseUrl}/ui/login`);
    await waitForSPAReady(page);

    const loginButton = page.locator(
      'button:has-text("Log In"), button:has-text("Login"), button:has-text("Sign In"), button[type="submit"]'
    );
    await expect(loginButton.first()).toBeVisible();
  });

  test('should complete full OAuth authentication flow to protected content', async ({ page }) => {
    await page.goto(`${baseUrl}/ui/login`);
    await waitForSPAReady(page);

    // Click login button to initiate OAuth flow
    const loginButton = page.locator(
      'button:has-text("Log In"), button:has-text("Login"), button:has-text("Sign In"), button[type="submit"]'
    );
    await loginButton.first().click();

    // Should redirect to auth server
    await page.waitForURL((url) => url.origin === 'https://dev-id.getbodhi.app');

    // Fill in auth server credentials
    const usernameField = page.locator('input[name="username"], input[type="email"], #username');
    const passwordField = page.locator('input[name="password"], input[type="password"], #password');
    const submitButton = page.locator(
      'button[type="submit"], input[type="submit"], button:has-text("Sign In")'
    );

    await expect(usernameField).toBeVisible();
    await expect(passwordField).toBeVisible();

    await usernameField.fill(testConfig.username);
    await passwordField.fill(testConfig.password);
    await submitButton.click();

    // Should redirect back to app and land on chat page
    await page.waitForURL((url) => url.pathname === '/ui/chat/');
    const finalPath = getCurrentPath(page);
    expect(finalPath).toBe('/ui/chat/');
  });
});
