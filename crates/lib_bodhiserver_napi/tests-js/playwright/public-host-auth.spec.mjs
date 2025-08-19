import { expect, test } from '@playwright/test';
import { getCurrentPath, randomPort, waitForSPAReady } from '../test-helpers.mjs';
import { createAuthServerTestClient, getAuthServerConfig, getTestCredentials } from './auth-server-client.mjs';
import { createServerManager } from './bodhi-app-server.mjs';

test.describe('Public Host Configuration Authentication Tests', () => {
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
    await authClient.makeResourceAdmin(resourceClient.clientId, resourceClient.clientSecret, testCredentials.username);
    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      port: '8080',
      host: '0.0.0.0',
      envVars: {
        'BODHI_PUBLIC_HOST': 'localhost',
        'BODHI_PUBLIC_SCHEME': 'http',
        'BODHI_PUBLIC_PORT': port.toString(),
      }
    });
    baseUrl = await serverManager.startServer();
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('should complete OAuth flow with public host settings for callback URLs', async ({ page }) => {
    await page.goto(`${baseUrl}/ui/login`);
    await waitForSPAReady(page);

    // Click login button to initiate OAuth flow
    const loginButton = page.locator(
      'button:has-text("Login")'
    );
    await expect(loginButton).toBeVisible();
    await loginButton.click();
    await page.waitForURL((url) => url.origin === authServerConfig.authUrl);
    const usernameField = page.locator('input[name="username"]');
    const passwordField = page.locator('input[name="password"]');

    await expect(usernameField).toBeVisible();
    await expect(passwordField).toBeVisible();
    await usernameField.fill(testCredentials.username);
    await passwordField.fill(testCredentials.password);

    const submitButton = page.locator('button[type="submit"]');
    await expect(submitButton).toBeVisible();
    await submitButton.click();

    await page.waitForURL((url) => url.pathname === '/ui/chat/');
    const finalPath = getCurrentPath(page);
    expect(finalPath).toBe('/ui/chat/');
    expect(page.locator(
      'button:has-text("Login")'
    )).not.toBeVisible();
  });
});