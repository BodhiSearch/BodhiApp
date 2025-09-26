import { test, expect } from '@playwright/test';
import {
  getAuthServerConfig,
  getTestCredentials,
} from '../../playwright/auth-server-client.mjs';
import { createServerManager } from '../../playwright/bodhi-app-server.mjs';
import { randomPort, getCurrentPath } from '../../test-helpers.mjs';
import { SetupWelcomePage } from '../../pages/SetupWelcomePage.mjs';
import { SetupResourceAdminPage } from '../../pages/SetupResourceAdminPage.mjs';
import { SetupDownloadModelsPage } from '../../pages/SetupDownloadModelsPage.mjs';
import { SetupApiModelsPage } from '../../pages/SetupApiModelsPage.mjs';
import { SetupBrowserExtensionPage } from '../../pages/SetupBrowserExtensionPage.mjs';
import { SetupFixtures } from '../../fixtures/setupFixtures.mjs';
import { BrowserWithExtension } from '../../utils/browser-with-extension.mjs';

test.describe('Browser Extension Detection with Chrome Extension', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;
  let browserWithExt;
  let extensionPage;

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

  test.beforeEach(async () => {
    browserWithExt = new BrowserWithExtension({
      headless: process.env.CI ? true : false,
      timeout: 30000
    });

    await browserWithExt.launch();
    extensionPage = await browserWithExt.createPage();

    const port = randomPort();
    const serverConfig = SetupFixtures.getServerManagerConfig(authServerConfig, port);
    serverManager = createServerManager(serverConfig);
    baseUrl = await serverManager.startServer();

    welcomePage = new SetupWelcomePage(extensionPage, baseUrl);
    resourceAdminPage = new SetupResourceAdminPage(
      extensionPage,
      baseUrl,
      authServerConfig,
      testCredentials
    );
    downloadModelsPage = new SetupDownloadModelsPage(extensionPage, baseUrl);
    apiModelsPage = new SetupApiModelsPage(extensionPage, baseUrl);
    browserExtensionPage = new SetupBrowserExtensionPage(extensionPage, baseUrl);
  });

  test.afterEach(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
    if (browserWithExt) {
      await browserWithExt.close();
    }
  });

  async function navigateToBrowserExtensionPage() {
    await extensionPage.goto(baseUrl);
    await extensionPage.waitForURL((url) => url.pathname === '/ui/setup/');

    const setupData = SetupFixtures.scenarios.SKIP_ALL_OPTIONAL_STEPS();
    await welcomePage.completeInitialSetup(setupData.serverName);

    await extensionPage.waitForURL((url) => url.pathname === '/ui/setup/resource-admin/');
    await resourceAdminPage.performCompleteLogin();

    await extensionPage.waitForURL((url) => url.pathname === '/ui/setup/download-models/');
    await downloadModelsPage.skipDownloads();

    await extensionPage.waitForURL((url) => url.pathname === '/ui/setup/api-models/');
    await apiModelsPage.skipApiSetup();

    await extensionPage.waitForURL((url) => url.pathname === '/ui/setup/browser-extension/');
  }

  test('Browser Extension Setup Flow - With Chrome Extension Installed and Complete Journey', async () => {
    await navigateToBrowserExtensionPage();

    await browserExtensionPage.expectBrowserExtensionPage();
    await browserExtensionPage.expectStepIndicator(5);

    await browserExtensionPage.expectBrowserSelectorPresent();
    await browserExtensionPage.expectHelpSection();

    await browserExtensionPage.expectExtensionFound();

    await expect(extensionPage.locator('[data-testid="extension-found"]')).toBeVisible();
    await expect(extensionPage.locator('[data-testid="extension-id-display"]')).toBeVisible();
    await expect(extensionPage.locator('[data-testid="next-button"]')).toBeVisible();
    await expect(extensionPage.locator('[data-testid="skip-button"]')).not.toBeVisible();

    await browserExtensionPage.clickNext();

    await extensionPage.waitForURL((url) => url.pathname === '/ui/setup/complete/');
    expect(getCurrentPath(extensionPage)).toBe('/ui/setup/complete/');

    await extensionPage.goto(`${baseUrl}/ui/setup/browser-extension/`);
    await extensionPage.waitForURL((url) => url.pathname === '/ui/setup/browser-extension/');

    await browserExtensionPage.expectBrowserExtensionPage();
    await browserExtensionPage.expectStepIndicator(5);

    await expect(extensionPage.locator('[data-testid="extension-found"]')).toBeVisible();
  });
});