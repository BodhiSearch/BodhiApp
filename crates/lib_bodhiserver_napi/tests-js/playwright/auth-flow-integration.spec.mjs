import { expect, test } from '@playwright/test';
import { getCurrentPath, randomPort, waitForSPAReady } from '../test-helpers.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from './auth-server-client.mjs';
import { createServerManager } from './bodhi-app-server.mjs';

test.describe('OAuth Authentication Flow Integration Tests', () => {
  let serverManager;
  let baseUrl;
  let authServerConfig;
  let testCredentials;
  let authClient;
  let resourceClient;
  let port;
  let serverUrl;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    port = randomPort();
    serverUrl = `http://localhost:${port}`;

    authClient = createAuthServerTestClient(authServerConfig);
    resourceClient = await authClient.createResourceClient(serverUrl);
    await authClient.makeResourceAdmin(
      resourceClient.clientId,
      resourceClient.clientSecret,
      testCredentials.username
    );
    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      port,
      host: 'localhost',
      envVars: {},
    });
    baseUrl = await serverManager.startServer();
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
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

    // Should redirect to auth server (updated URL)
    await page.waitForURL((url) => url.origin === authServerConfig.authUrl);

    // Fill in auth server credentials
    const usernameField = page.locator('input[name="username"], input[type="email"], #username');
    const passwordField = page.locator('input[name="password"], input[type="password"], #password');
    const submitButton = page.locator(
      'button[type="submit"], input[type="submit"], button:has-text("Sign In")'
    );

    await expect(usernameField).toBeVisible();
    await expect(passwordField).toBeVisible();

    await usernameField.fill(testCredentials.username);
    await passwordField.fill(testCredentials.password);
    await submitButton.click();

    // Should redirect back to app and land on chat page
    await page.waitForURL((url) => url.origin === baseUrl && url.pathname === '/ui/chat/');
    await waitForSPAReady(page);
    const finalPath = getCurrentPath(page);
    expect(finalPath).toBe('/ui/chat/');
  });
});
