import { expect, test } from '@playwright/test';
import { createAuthServerTestClient, getAuthServerConfig, getTestCredentials } from '../../playwright/auth-server-client.mjs';
import { createServerManager } from '../../playwright/bodhi-app-server.mjs';
import { randomPort, getCurrentPath } from '../../test-helpers.mjs';
import { SetupWelcomePage } from '../../pages/SetupWelcomePage.mjs';
import { SetupResourceAdminPage } from '../../pages/SetupResourceAdminPage.mjs';
import { SetupDownloadModelsPage } from '../../pages/SetupDownloadModelsPage.mjs';
import { SetupCompletePage } from '../../pages/SetupCompletePage.mjs';
import { SetupFixtures } from '../../fixtures/setupFixtures.mjs';

test.describe('First-Time Setup Flow Integration', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;
  
  // Page objects
  let welcomePage;
  let resourceAdminPage;
  let downloadModelsPage;
  let completePage;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    const port = randomPort();
    const serverConfig = SetupFixtures.getServerManagerConfig(authServerConfig, port);
    serverManager = createServerManager(serverConfig);
    baseUrl = await serverManager.startServer();
  });

  test.beforeEach(async ({ page }) => {
    welcomePage = new SetupWelcomePage(page, baseUrl);
    resourceAdminPage = new SetupResourceAdminPage(page, baseUrl, authServerConfig, testCredentials);
    downloadModelsPage = new SetupDownloadModelsPage(page, baseUrl);
    completePage = new SetupCompletePage(page, baseUrl);
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('comprehensive setup flow with multiple validations', async ({ page }) => {
    const setupData = SetupFixtures.scenarios.QUICK_SETUP();
    
    // Step 1: Navigate and verify initial setup page
    await page.goto(baseUrl);
    await page.waitForURL((url) => url.pathname === '/ui/setup/');
    expect(getCurrentPath(page)).toBe('/ui/setup/');
    
    // Use page objects for structured interactions
    await welcomePage.expectWelcomePage();
    await welcomePage.expectBenefitsDisplayed();
    await welcomePage.expectStepIndicator(1);
    await welcomePage.completeInitialSetup(setupData.serverName);
    
    // Step 2: Resource admin page using page object
    await page.waitForURL((url) => url.pathname === '/ui/setup/resource-admin/');
    await resourceAdminPage.expectResourceAdminPage();
    await resourceAdminPage.expectStepIndicator(2);
    await resourceAdminPage.performCompleteLogin();
    
    // Step 3: Models download page using page object
    await page.waitForURL((url) => url.pathname === '/ui/setup/download-models/');
    await downloadModelsPage.expectDownloadModelsPage();
    await downloadModelsPage.expectStepIndicator(3);
    await downloadModelsPage.expectRecommendedModelsDisplayed();
    await downloadModelsPage.skipDownloads();
    
    // Step 4: Setup completion using page object
    await page.waitForURL((url) => url.pathname === '/ui/setup/complete/');
    await completePage.expectSetupCompletePage();
    await completePage.clickStartUsingApp();
    
    // Verify final redirect to chat page
    expect(getCurrentPath(page)).toBe('/ui/chat/');
  });
});