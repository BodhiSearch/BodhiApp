import { expect, test } from '@playwright/test';
import {
  getCurrentPath,
  getLocalNetworkIP,
  randomPort,
  waitForRedirect,
  waitForSPAReady
} from '../test-helpers.mjs';
import { createAuthServerTestClient, getAuthServerConfig, getTestCredentials } from './auth-server-client.mjs';
import {
  createServerManager,
} from './bodhi-app-server.mjs';

test.describe('Network IP Authentication Setup Flow', () => {
  let authServerConfig;
  let testCredentials;
  let authClient;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    authClient = createAuthServerTestClient(authServerConfig);
  });

  test('should complete setup flow and handle login when accessed via local network IP', async ({ page }) => {
    // Get actual local network IP
    const localIP = getLocalNetworkIP();

    // Fail test with clear message if no network IP available
    if (!localIP) {
      throw new Error('No local network IP available for testing. This test requires a network interface with a non-loopback IPv4 address.');
    }

    const port = randomPort();

    // Start server on all interfaces (0.0.0.0) so it accepts connections from network IP
    const serverManager = createServerManager({
      appStatus: 'setup',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      host: '0.0.0.0', // Bind to all interfaces
      port,
      logLevel: 'debug',
    });

    try {
      await serverManager.startServer();

      // Navigate directly to the local network IP - browser sets Host header naturally
      const networkUrl = `http://${localIP}:${port}`;
      await page.goto(networkUrl);
      await waitForSPAReady(page);

      const currentPath = getCurrentPath(page);
      expect(currentPath).toBe('/ui/setup/');

      // Verify setup page content (Step 1 of 4)
      await expect(page.locator('text=Welcome to Bodhi App')).toBeVisible();
      await expect(page.locator('text=Step 1 of 4')).toBeVisible();

      // Fill in the server setup form
      const serverNameField = page.locator('input[name="name"]');
      await expect(serverNameField).toBeVisible();
      await serverNameField.fill('My Network IP Bodhi Server');

      // Click "Setup Bodhi Server" button to submit the form
      // Browser naturally sends Host: ${localIP}:${port}
      // Our setup handler will detect this and register it as a redirect URI
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
      const emailField = page.locator('input[name="username"]');
      const passwordField = page.locator('input[name="password"]');
      await expect(emailField).toBeVisible();
      await expect(passwordField).toBeVisible();
      await emailField.fill(testCredentials.username);
      await passwordField.fill(testCredentials.password);
      const submitButton = page.locator('button[type="submit"]');
      await expect(submitButton).toBeVisible();
      await submitButton.click();

      // Wait for redirect to Download Models page (Step 3 of 4)
      // The callback should work with the network IP that was registered
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

      // Now test cross-compatibility: setup was done via network IP, try login via localhost
      // Clear all cookies and session storage to simulate a fresh user
      await page.context().clearCookies();
      await page.evaluate(() => {
        localStorage.clear();
        sessionStorage.clear();
      });

      // Navigate to localhost URL instead of network IP - should redirect to login
      const localhostUrl = `http://localhost:${port}`;
      await page.goto(localhostUrl);
      await waitForSPAReady(page);

      // Should redirect to login since not authenticated
      await waitForRedirect(page, '/ui/login/');
      const loginButtonAgain = page.locator('button:has-text("Login")');
      await expect(loginButtonAgain).toBeVisible();
      await loginButtonAgain.click();

      // Should redirect to auth server for authentication
      await expect(page.locator('text=Sign in to your account')).toBeVisible();
      const emailFieldAgain = page.locator('input[name="username"]');
      const passwordFieldAgain = page.locator('input[name="password"]');
      await expect(emailFieldAgain).toBeVisible();
      await expect(passwordFieldAgain).toBeVisible();
      await emailFieldAgain.fill(testCredentials.username);
      await passwordFieldAgain.fill(testCredentials.password);
      const submitButtonAgain = page.locator('button[type="submit"]');
      await expect(submitButtonAgain).toBeVisible();
      await submitButtonAgain.click();

      // Should redirect back to chat page after successful authentication
      await waitForRedirect(page, '/ui/chat/');
      const loggedInUrl = getCurrentPath(page);
      expect(loggedInUrl).toBe('/ui/chat/');

    } finally {
      await serverManager.stopServer();
    }
  });

  test('should complete setup flow via localhost and handle login via network IP', async ({ page }) => {
    // Get actual local network IP for cross-compatibility test
    const localIP = getLocalNetworkIP();

    // Fail test with clear message if no network IP available
    if (!localIP) {
      throw new Error('No local network IP available for testing. This test requires a network interface with a non-loopback IPv4 address.');
    }

    const port = randomPort();

    // Server bound to 0.0.0.0 but accessed via localhost
    const serverManager = createServerManager({
      appStatus: 'setup',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      host: '0.0.0.0',  // Bind to all interfaces
      port,
      logLevel: 'debug',
    });

    try {
      await serverManager.startServer();

      // Navigate to localhost - this is how users typically access locally
      const localhostUrl = `http://localhost:${port}`;
      await page.goto(localhostUrl);
      await waitForSPAReady(page);

      // Browser naturally sends Host: localhost:${port}
      // Setup should register all loopback hosts + any server IP detected

      const currentPath = getCurrentPath(page);
      expect(currentPath).toBe('/ui/setup/');

      // Complete the setup flow
      const serverNameField = page.locator('input[name="name"]');
      await expect(serverNameField).toBeVisible();
      await serverNameField.fill('My Localhost Bodhi Server');

      const setupButton = page.locator('button:has-text("Setup Bodhi Server")');
      await expect(setupButton).toBeVisible();
      await setupButton.click();

      await waitForRedirect(page, '/ui/setup/resource-admin/');

      const loginButton = page.locator('button:has-text("Continue with Login")');
      await expect(loginButton).toBeVisible();
      await loginButton.click();

      await expect(page.locator('text=Sign in to your account')).toBeVisible();

      const emailField = page.locator('input[name="username"]');
      const passwordField = page.locator('input[name="password"]');
      await emailField.fill(testCredentials.username);
      await passwordField.fill(testCredentials.password);
      const submitButton = page.locator('button[type="submit"]');
      await submitButton.click();

      await waitForRedirect(page, '/ui/setup/download-models/');

      const continueButton = page.locator('button:has-text("Continue")');
      await continueButton.click();

      await waitForRedirect(page, '/ui/setup/complete/');

      const startButton = page.locator('button:has-text("Start Using Bodhi App")');
      await startButton.click();

      await waitForRedirect(page, '/ui/chat/');
      const finalUrl = getCurrentPath(page);
      expect(finalUrl).toBe('/ui/chat/');

      // Now test cross-compatibility: setup was done via localhost, try login via network IP
      // Clear all cookies and session storage to simulate a fresh user
      await page.context().clearCookies();
      await page.evaluate(() => {
        localStorage.clear();
        sessionStorage.clear();
      });

      // Navigate to network IP URL instead of localhost - should redirect to login
      const networkUrl = `http://${localIP}:${port}`;
      await page.goto(networkUrl);
      await waitForSPAReady(page);

      // Should redirect to login since not authenticated
      await waitForRedirect(page, '/ui/login/');
      const loginButtonAgain = page.locator('button:has-text("Login")');
      await expect(loginButtonAgain).toBeVisible();
      await loginButtonAgain.click();

      // Should redirect to auth server for authentication
      await expect(page.locator('text=Sign in to your account')).toBeVisible();
      const emailFieldAgain = page.locator('input[name="username"]');
      const passwordFieldAgain = page.locator('input[name="password"]');
      await expect(emailFieldAgain).toBeVisible();
      await expect(passwordFieldAgain).toBeVisible();
      await emailFieldAgain.fill(testCredentials.username);
      await passwordFieldAgain.fill(testCredentials.password);
      const submitButtonAgain = page.locator('button[type="submit"]');
      await expect(submitButtonAgain).toBeVisible();
      await submitButtonAgain.click();

      // Should redirect back to chat page after successful authentication
      await waitForRedirect(page, '/ui/chat/');
      const loggedInUrl = getCurrentPath(page);
      expect(loggedInUrl).toBe('/ui/chat/');

    } finally {
      await serverManager.stopServer();
    }
  });

});