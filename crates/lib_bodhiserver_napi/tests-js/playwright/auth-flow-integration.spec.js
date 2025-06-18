import { test, expect } from '@playwright/test';
import { createServerManager } from './playwright-helpers.js';
import { config } from 'dotenv';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

// Load test environment variables
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
config({ path: join(__dirname, '.env.test') });

/**
 * Wait for SPA to be fully loaded and rendered
 */
async function waitForSPAReady(page) {
  await page.waitForLoadState('networkidle');
  await page.waitForLoadState('domcontentloaded');
}

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
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('should redirect unauthenticated users to login page', async ({ page }) => {
    const testPaths = ['/', '/ui/chat', '/ui/models', '/ui/settings'];

    for (const path of testPaths) {
      await page.goto(`${baseUrl}${path}`);
      await waitForSPAReady(page);

      const pageContent = await page.content();
      const currentPath = new URL(page.url()).pathname;

      expect(pageContent.length).toBeGreaterThan(1000);
      expect(currentPath).toBe('/ui/login/');
    }
  });

  test('should display functional login page with authentication configured', async ({ page }) => {
    await page.goto(`${baseUrl}/ui/login`);
    await waitForSPAReady(page);

    const pageContent = await page.content();
    const currentPath = new URL(page.url()).pathname;
    const loginButton = page.locator(
      'button:has-text("Log In"), button:has-text("Login"), button:has-text("Sign In"), button[type="submit"]'
    );

    expect(pageContent.length).toBeGreaterThan(1000);
    expect(currentPath).toBe('/ui/login/');
    await expect(loginButton.first()).toBeVisible();
  });

  test.skip('should complete OAuth authentication flow to protected content', async ({ page }) => {
    // This test is skipped due to known bug with login button
    await page.goto(`${baseUrl}/ui/login`);
    await waitForSPAReady(page);

    const loginButton = page.locator(
      'button:has-text("Log In"), button:has-text("Login"), button:has-text("Sign In"), button[type="submit"]'
    );
    await expect(loginButton.first()).toBeVisible();

    await loginButton.first().click();

    // Should redirect to auth server
    await page.waitForURL((url) => url.includes('dev-id.getbodhi.app'));
    expect(page.url()).toContain('dev-id.getbodhi.app');

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

    // Should redirect back to app callback then to chat
    await page.waitForURL((url) => url.includes('/ui/callback') || url.includes('/ui/chat'));

    const finalPath = new URL(page.url()).pathname;
    expect(finalPath).toBe('/ui/chat/');
  });
});
