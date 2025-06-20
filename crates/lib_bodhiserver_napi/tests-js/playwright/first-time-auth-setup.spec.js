import { test, expect } from '@playwright/test';
import {
  createServerManager,
  waitForSPAReady,
  waitForRedirect,
  getCurrentPath,
} from './playwright-helpers.js';

/**
 * Get test environment variables with defaults
 */
function getTestConfig() {
  return {
    authUrl: process.env.INTEG_TEST_AUTH_URL,
    authRealm: process.env.INTEG_TEST_AUTH_REALM,
    username: process.env.INTEG_TEST_USERNAME,
    password: process.env.INTEG_TEST_PASSWORD,
  };
}

test.describe('First-Time Authentication Setup Flow', () => {
  let testConfig;
  let serverManager;
  let baseUrl;

  test.beforeAll(async () => {
    testConfig = getTestConfig();
    expect(testConfig.username).toBeDefined();
    expect(testConfig.password).toBeDefined();
    expect(testConfig.authUrl).toBeDefined();
    expect(testConfig.authRealm).toBeDefined();

    serverManager = createServerManager({
      appStatus: 'setup',
      authUrl: testConfig.authUrl,
      authRealm: testConfig.authRealm,
      logLevel: 'debug',
    });
    baseUrl = await serverManager.startServer();
  });

  test.afterAll(async () => {
    await serverManager.stopServer();
  });

  test('should complete full setup flow from initial setup to chat page', async ({ page }) => {
    // Navigate to app and verify redirect to setup page
    await page.goto(baseUrl);
    await waitForSPAReady(page);

    let currentPath = getCurrentPath(page);
    expect(currentPath).toBe('/ui/setup/');

    // Verify setup page content (Step 1 of 4)
    await expect(page.locator('text=Welcome to Bodhi App')).toBeVisible();
    await expect(page.locator('text=Step 1 of 4')).toBeVisible();

    // Click "Setup Bodhi App" button to go to Admin Setup
    const setupButton = page.locator('button:has-text("Setup Bodhi App")');
    await expect(setupButton).toBeVisible();
    await setupButton.click();

    // Wait for redirect to admin setup page (Step 2 of 4)
    await waitForRedirect(page, '/ui/setup/resource-admin/');

    // Verify admin setup page content
    await expect(page.locator('text=Admin Setup')).toBeVisible();
    await expect(page.locator('text=Step 2 of 4')).toBeVisible();
    await expect(page.locator('text=Continue with Login')).toBeVisible();

    // Click "Continue with Login" to go to authentication server
    const loginButton = page.locator('button:has-text("Continue with Login")');
    await expect(loginButton).toBeVisible();
    await loginButton.click();

    // Wait for redirect to auth server and verify login page
    await expect(page.locator('text=Sign in to your account')).toBeVisible();

    // Fill in authentication credentials and submit
    const emailField = page.locator(
      'input[name="username"], input[type="email"], textbox[name="Email"]'
    );
    const passwordField = page.locator(
      'input[name="password"], input[type="password"], textbox[name="Password"]'
    );

    await expect(emailField).toBeVisible();
    await expect(passwordField).toBeVisible();

    await emailField.fill(testConfig.username);
    await passwordField.fill(testConfig.password);

    // Click Sign In button
    const signInButton = page.locator('input:has-text("Sign In")');
    await expect(signInButton).toBeVisible();
    await signInButton.click();

    // Wait for redirect to Download Models page (Step 3 of 4)
    await waitForRedirect(page, '/ui/setup/download-models/');
    await expect(page.locator('text=Recommended Models')).toBeVisible();
    await expect(page.locator('text=Step 3 of 4')).toBeVisible();

    // Skip downloading models and click Continue
    const continueButton = page.locator('button:has-text("Continue")');
    await expect(continueButton).toBeVisible();
    await continueButton.click();

    // Wait for Setup Complete page (Step 4 of 4)
    await waitForRedirect(page, '/ui/setup/complete/');
    await expect(page.locator('text=Setup Complete')).toBeVisible();

    // Click "Start Using Bodhi App" to complete setup
    const startButton = page.locator('button:has-text("Start Using Bodhi App")');
    await expect(startButton).toBeVisible();
    await startButton.click();

    // Verify final redirect to chat page
    await waitForRedirect(page, '/ui/chat/');
    const finalUrl = getCurrentPath(page);
    expect(finalUrl).toBe('/ui/chat/');

    // Verify chat page elements are present
    await expect(page.locator('text=Welcome to Chat')).toBeVisible();
  });
});
