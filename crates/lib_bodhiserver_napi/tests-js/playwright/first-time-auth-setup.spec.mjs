import { expect, test } from '@playwright/test';
import {
  getCurrentPath,
  randomPort,
  waitForRedirect,
  waitForSPAReady
} from '../test-helpers.mjs';
import { createAuthServerTestClient, getAuthServerConfig, getTestCredentials } from './auth-server-client.mjs';
import {
  createServerManager,
} from './bodhi-app-server.mjs';

test.describe('First-Time Authentication Setup Flow', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;
  let authClient;
  let dynamicClients;
  let port;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    port = randomPort();

    // Create auth server test client and setup dynamic clients
    authClient = createAuthServerTestClient(authServerConfig);
    dynamicClients = await authClient.setupDynamicClients(testCredentials.username, testCredentials.password, port);

    serverManager = createServerManager({
      appStatus: 'setup',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      port,
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

    const currentPath = getCurrentPath(page);
    expect(currentPath).toBe('/ui/setup/');

    // Verify setup page content (Step 1 of 4)
    await expect(page.locator('text=Welcome to Bodhi App')).toBeVisible();
    await expect(page.locator('text=Step 1 of 4')).toBeVisible();

    // Fill in the server setup form
    const serverNameField = page.locator('input[name="name"]');
    await expect(serverNameField).toBeVisible();
    await serverNameField.fill('My Test Bodhi Server');

    // Click "Setup Bodhi Server" button to submit the form
    const setupButton = page.locator('button:has-text("Setup Bodhi Server")');
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
    const submitButton = page.locator(
      'button[type="submit"], input[type="submit"], button:has-text("Sign In")'
    );

    await expect(emailField).toBeVisible();
    await expect(passwordField).toBeVisible();

    await emailField.fill(testCredentials.username);
    await passwordField.fill(testCredentials.password);
    await submitButton.click();

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
