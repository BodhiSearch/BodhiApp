import { test, expect } from '@playwright/test';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '@/playwright/auth-server-client.mjs';
import { createServerManager } from '@/playwright/bodhi-app-server.mjs';
import { randomPort, getCurrentPath } from '@/test-helpers.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { PublicHostFixtures } from '@/fixtures/publicHostFixtures.mjs';

test.describe('Public Host Configuration Authentication Tests', () => {
  let authServerConfig;
  let testCredentials;
  let port;
  let serverManager;
  let baseUrl;
  let authClient;
  let resourceClient;
  let loginPage;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    port = randomPort();
    const serverUrl = `http://localhost:${port}`;

    authClient = createAuthServerTestClient(authServerConfig);
    resourceClient = await authClient.createResourceClient(serverUrl);
    await authClient.makeResourceAdmin(
      resourceClient.clientId,
      resourceClient.clientSecret,
      testCredentials.userId
    );

    const serverConfig = PublicHostFixtures.getServerManagerConfig(
      authServerConfig,
      resourceClient,
      port
    );

    serverManager = createServerManager(serverConfig);
    baseUrl = await serverManager.startServer();
  });

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('should complete OAuth flow with public host settings for callback URLs', async ({
    page,
  }) => {
    // Navigate to login page
    await loginPage.navigateToLogin();

    // Expect login page to be visible
    await loginPage.expectLoginPageVisible();

    // Perform OAuth login flow using the page object
    await loginPage.performOAuthLogin('/ui/chat/');

    // Verify successful login - should be on chat page
    const finalPath = getCurrentPath(page);
    expect(finalPath).toBe('/ui/chat/');

    // Verify login button is no longer visible (user is authenticated)
    const loginButton = page.locator('button:has-text("Login")');
    await expect(loginButton).not.toBeVisible();
  });
});
