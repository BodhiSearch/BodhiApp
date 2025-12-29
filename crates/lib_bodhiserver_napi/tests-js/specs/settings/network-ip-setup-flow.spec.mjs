import { SetupFixtures } from '@/fixtures/setupFixtures.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { SetupApiModelsPage } from '@/pages/SetupApiModelsPage.mjs';
import { SetupBrowserExtensionPage } from '@/pages/SetupBrowserExtensionPage.mjs';
import { SetupCompletePage } from '@/pages/SetupCompletePage.mjs';
import { SetupDownloadModelsPage } from '@/pages/SetupDownloadModelsPage.mjs';
import { SetupResourceAdminPage } from '@/pages/SetupResourceAdminPage.mjs';
import { SetupWelcomePage } from '@/pages/SetupWelcomePage.mjs';
import { getCurrentPath, getLocalNetworkIP, randomPort } from '@/test-helpers.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { expect, test } from '@playwright/test';

test.describe('Network IP Authentication Setup Flow', () => {
  let authServerConfig;
  let testCredentials;
  let authClient;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    authClient = createAuthServerTestClient(authServerConfig);
  });

  test('should complete setup flow and handle login when accessed via local network IP', async ({
    page,
  }) => {
    const localIP = getLocalNetworkIP();

    if (!localIP) {
      throw new Error(
        'No local network IP available for testing. This test requires a network interface with a non-loopback IPv4 address.'
      );
    }

    const port = randomPort();
    const setupData = SetupFixtures.scenarios.NETWORK_IP_SETUP();

    // Start server on all interfaces (0.0.0.0) so it accepts connections from network IP
    const serverConfig = SetupFixtures.getNetworkIPServerConfig(authServerConfig, port);
    const serverManager = createServerManager(serverConfig);

    try {
      await serverManager.startServer();
      const networkUrl = `http://${localIP}:${port}`;

      // Initialize page objects
      const welcomePage = new SetupWelcomePage(page, networkUrl);
      const resourceAdminPage = new SetupResourceAdminPage(
        page,
        networkUrl,
        authServerConfig,
        testCredentials
      );
      const downloadModelsPage = new SetupDownloadModelsPage(page, networkUrl);
      const apiModelsPage = new SetupApiModelsPage(page, networkUrl);
      const browserExtensionPage = new SetupBrowserExtensionPage(page, networkUrl);
      const completePage = new SetupCompletePage(page, networkUrl);
      const chatPage = new ChatPage(page);

      // Step 1: Navigate and complete initial setup via network IP
      await page.goto(networkUrl);
      await page.waitForURL((url) => url.pathname === '/ui/setup/');
      expect(getCurrentPath(page)).toBe('/ui/setup/');

      await welcomePage.expectWelcomePage();
      await welcomePage.expectBenefitsDisplayed();
      await welcomePage.completeInitialSetup(setupData.serverName);

      // Step 2: Resource admin page
      await resourceAdminPage.expectResourceAdminPage();
      await resourceAdminPage.performCompleteLogin();

      // Step 3: Models download page
      await downloadModelsPage.expectDownloadModelsPage();
      await downloadModelsPage.skipDownloads();

      // Step 4: API Models page
      await apiModelsPage.expectApiModelsPage();
      await apiModelsPage.skipApiSetup();

      // Step 5: Browser Extension page
      await browserExtensionPage.expectBrowserExtensionPage();
      await browserExtensionPage.clickContinue();

      // Step 6: Setup completion
      await completePage.expectSetupCompletePage();
      await completePage.clickStartUsingApp();

      // Verify final redirect to chat page
      await chatPage.expectChatPage();
      await chatPage.verifyChatEmpty();

      // Test cross-compatibility: setup was done via network IP, try login via localhost
      // Create a fresh browser context for isolated testing
      const freshContext = await page.context().browser().newContext();
      const freshPage = await freshContext.newPage();

      // Navigate to localhost URL with fresh context - should redirect to login
      const localhostUrl = `http://localhost:${port}`;
      const loginPage = new LoginPage(freshPage, localhostUrl, authServerConfig, testCredentials);

      await freshPage.goto(localhostUrl);
      await freshPage.waitForURL((url) => url.pathname === '/ui/login/');
      await loginPage.expectLoginPage();
      await loginPage.performOAuthLogin('/ui/chat/');

      // Verify successful cross-compatibility login
      expect(getCurrentPath(freshPage)).toBe('/ui/chat/');

      // Clean up fresh context
      await freshContext.close();
    } finally {
      await serverManager.stopServer();
    }
  });

  test('should complete setup flow via localhost and handle login via network IP', async ({
    page,
  }) => {
    const localIP = getLocalNetworkIP();

    if (!localIP) {
      throw new Error(
        'No local network IP available for testing. This test requires a network interface with a non-loopback IPv4 address.'
      );
    }

    const port = randomPort();
    const setupData = SetupFixtures.scenarios.NETWORK_IP_SETUP();

    // Server bound to 0.0.0.0 but accessed via localhost
    const serverConfig = SetupFixtures.getNetworkIPServerConfig(authServerConfig, port);
    const serverManager = createServerManager(serverConfig);

    try {
      await serverManager.startServer();
      const localhostUrl = `http://localhost:${port}`;

      // Initialize page objects for localhost access
      const welcomePage = new SetupWelcomePage(page, localhostUrl);
      const resourceAdminPage = new SetupResourceAdminPage(
        page,
        localhostUrl,
        authServerConfig,
        testCredentials
      );
      const downloadModelsPage = new SetupDownloadModelsPage(page, localhostUrl);
      const apiModelsPage = new SetupApiModelsPage(page, localhostUrl);
      const browserExtensionPage = new SetupBrowserExtensionPage(page, localhostUrl);
      const completePage = new SetupCompletePage(page, localhostUrl);

      // Navigate to localhost and complete setup flow
      await page.goto(localhostUrl);
      await page.waitForURL((url) => url.pathname === '/ui/setup/');
      expect(getCurrentPath(page)).toBe('/ui/setup/');

      // Complete the setup flow using page objects
      await welcomePage.expectWelcomePage();
      await welcomePage.completeInitialSetup(setupData.serverName);

      await page.waitForURL((url) => url.pathname === '/ui/setup/resource-admin/');
      await resourceAdminPage.performCompleteLogin();

      await page.waitForURL((url) => url.pathname === '/ui/setup/download-models/');
      await downloadModelsPage.skipDownloads();

      await apiModelsPage.expectApiModelsPage();
      await apiModelsPage.skipApiSetup();

      await browserExtensionPage.expectBrowserExtensionPage();
      await browserExtensionPage.clickContinue();

      await completePage.expectSetupCompletePage();
      await completePage.clickStartUsingApp();

      await page.waitForURL((url) => url.pathname === '/ui/chat/');
      expect(getCurrentPath(page)).toBe('/ui/chat/');

      // Test cross-compatibility: setup was done via localhost, try login via network IP
      // Create a fresh browser context for isolated testing
      const freshContext = await page.context().browser().newContext();
      const freshPage = await freshContext.newPage();

      // Navigate to network IP URL with fresh context - should redirect to login
      const networkUrl = `http://${localIP}:${port}`;
      const loginPage = new LoginPage(freshPage, networkUrl, authServerConfig, testCredentials);

      await freshPage.goto(networkUrl);
      await freshPage.waitForURL((url) => url.pathname === '/ui/login/');
      await loginPage.expectLoginPage();
      await loginPage.performOAuthLogin('/ui/chat/');

      // Verify successful cross-compatibility login
      expect(getCurrentPath(freshPage)).toBe('/ui/chat/');

      // Clean up fresh context
      await freshContext.close();
    } finally {
      await serverManager.stopServer();
    }
  });
});
