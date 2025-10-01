import { expect, test } from '@playwright/test';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { randomPort, getCurrentPath } from '@/test-helpers.mjs';
import { SetupWelcomePage } from '@/pages/SetupWelcomePage.mjs';
import { SetupResourceAdminPage } from '@/pages/SetupResourceAdminPage.mjs';
import { SetupDownloadModelsPage } from '@/pages/SetupDownloadModelsPage.mjs';
import { SetupApiModelsPage } from '@/pages/SetupApiModelsPage.mjs';
import { SetupBrowserExtensionPage } from '@/pages/SetupBrowserExtensionPage.mjs';
import { SetupFixtures } from '@/fixtures/setupFixtures.mjs';

test.describe('Browser Extension Setup Integration', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;

  // Page objects
  let welcomePage;
  let resourceAdminPage;
  let downloadModelsPage;
  let apiModelsPage;
  let browserExtensionPage;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test.beforeEach(async ({ page }) => {
    const port = randomPort();
    const serverConfig = SetupFixtures.getServerManagerConfig(authServerConfig, port);
    serverManager = createServerManager(serverConfig);
    baseUrl = await serverManager.startServer();

    welcomePage = new SetupWelcomePage(page, baseUrl);
    resourceAdminPage = new SetupResourceAdminPage(
      page,
      baseUrl,
      authServerConfig,
      testCredentials
    );
    downloadModelsPage = new SetupDownloadModelsPage(page, baseUrl);
    apiModelsPage = new SetupApiModelsPage(page, baseUrl);
    browserExtensionPage = new SetupBrowserExtensionPage(page, baseUrl);
  });

  test.afterEach(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  async function navigateToBrowserExtensionPage(page) {
    // Navigate through the setup flow to reach browser extension page
    await page.goto(baseUrl);
    await page.waitForURL((url) => url.pathname === '/ui/setup/');

    // Complete welcome page
    const setupData = SetupFixtures.scenarios.SKIP_ALL_OPTIONAL_STEPS();
    await welcomePage.completeInitialSetup(setupData.serverName);

    // Complete resource admin page
    await page.waitForURL((url) => url.pathname === '/ui/setup/resource-admin/');
    await resourceAdminPage.performCompleteLogin();

    // Complete download models page (skip)
    await page.waitForURL((url) => url.pathname === '/ui/setup/download-models/');
    await downloadModelsPage.skipDownloads();

    // Complete API models page (skip)
    await page.waitForURL((url) => url.pathname === '/ui/setup/api-models/');
    await apiModelsPage.skipApiSetup();

    // Should now be at browser extension page
    await page.waitForURL((url) => url.pathname === '/ui/setup/browser-extension/');
  }

  test('Browser Extension Setup Flow - Complete Journey', async ({ page }) => {
    // Phase 1: Navigate to browser extension page and verify structure
    await navigateToBrowserExtensionPage(page);

    // Verify we're on the browser extension page with correct step indicator
    await browserExtensionPage.expectBrowserExtensionPage();
    await browserExtensionPage.expectStepIndicator(5);

    // Verify page structure elements are present
    await browserExtensionPage.expectBrowserSelectorPresent();
    await browserExtensionPage.expectHelpSection();

    // Phase 2: Test Chrome browser with no extension installed
    // Running on Chrome without extension - should show supported browser UI
    await browserExtensionPage.expectSupportedBrowserUI();
    await browserExtensionPage.expectExtensionNotFound();

    // Phase 3: Test refresh functionality
    await browserExtensionPage.clickRefresh();

    // Should still be on browser extension page after refresh
    await page.waitForURL((url) => url.pathname === '/ui/setup/browser-extension/');
    await browserExtensionPage.expectBrowserExtensionPage();
    await browserExtensionPage.expectStepIndicator(5);

    // Should still show extension not found after refresh
    await browserExtensionPage.expectExtensionNotFound();

    // Phase 4: Skip extension installation (using unified continue button)
    await browserExtensionPage.clickContinue();

    // Phase 5: Verify navigation to completion
    await page.waitForURL((url) => url.pathname === '/ui/setup/complete/');
    expect(getCurrentPath(page)).toBe('/ui/setup/complete/');

    // Phase 6: Test direct navigation to browser extension page
    await page.goto(`${baseUrl}/ui/setup/browser-extension/`);
    await page.waitForURL((url) => url.pathname === '/ui/setup/browser-extension/');

    // Should work since we're authenticated and can access setup pages
    await browserExtensionPage.expectBrowserExtensionPage();
    await browserExtensionPage.expectStepIndicator(5);

    // Complete the test by continuing through the page
    await browserExtensionPage.clickContinue();
    await browserExtensionPage.expectNavigationToComplete();
  });
});
