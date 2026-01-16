import { SetupFixtures } from '@/fixtures/setupFixtures.mjs';
import { SetupApiModelsPage } from '@/pages/SetupApiModelsPage.mjs';
import { SetupBrowserExtensionPage } from '@/pages/SetupBrowserExtensionPage.mjs';
import { SetupDownloadModelsPage } from '@/pages/SetupDownloadModelsPage.mjs';
import { SetupResourceAdminPage } from '@/pages/SetupResourceAdminPage.mjs';
import { SetupToolsPage } from '@/pages/SetupToolsPage.mjs';
import { SetupWelcomePage } from '@/pages/SetupWelcomePage.mjs';
import { getCurrentPath, randomPort } from '@/test-helpers.mjs';
import {
  getAuthServerConfig,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { expect, test } from '@playwright/test';

/**
 * Tools Setup E2E Tests
 *
 * These tests verify the tools setup page in the onboarding flow.
 * The happy path test requires EXA_API_KEY environment variable.
 */
test.describe('Tools Setup Integration', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;

  // Page objects
  let welcomePage;
  let resourceAdminPage;
  let downloadModelsPage;
  let apiModelsPage;
  let toolsPage;
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
    toolsPage = new SetupToolsPage(page, baseUrl);
    browserExtensionPage = new SetupBrowserExtensionPage(page, baseUrl);
  });

  test.afterEach(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  async function navigateToToolsPage(page) {
    // Navigate through the setup flow to reach tools page
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

    // Should now be at tools page
    await page.waitForURL((url) => url.pathname === '/ui/setup/tools/');
  }

  test('Tools Setup Page - displays correctly and can skip', async ({ page }) => {
    await navigateToToolsPage(page);

    // Verify we're on the tools page with correct step indicator
    await toolsPage.expectToolsPage();
    await toolsPage.expectStepIndicator(5);

    // Verify form structure
    await toolsPage.expectInitialFormState();

    // The migration seed enables the tool by default, so after backend state is fetched,
    // the toggle should be ON and form should be enabled
    await toolsPage.expectAppToggleOn();
    await toolsPage.expectNoAppDisabledMessage();
    await toolsPage.expectFormEnabled();

    // Skip tools setup
    await toolsPage.skipToolsSetup();

    // Should navigate to browser extension page
    await page.waitForURL((url) => url.pathname === '/ui/setup/browser-extension/');
    expect(getCurrentPath(page)).toBe('/ui/setup/browser-extension/');
  });

  test('Tools Setup - configures Exa Web Search with API key', async ({ page }) => {
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').not.toBeUndefined();

    await navigateToToolsPage(page);

    // Verify we're on the tools page
    await toolsPage.expectToolsPage();
    await toolsPage.expectStepIndicator(5);

    // The migration seed enables the tool by default
    await toolsPage.expectAppToggleOn();
    await toolsPage.expectFormEnabled();

    // Fill in API key (this auto-enables the tool toggle)
    await toolsPage.fillApiKey(exaApiKey);

    // Submit the form
    await toolsPage.submitForm();

    // Wait for success toast
    await toolsPage.waitForToast('Tool configuration saved');

    // Should navigate to browser extension page after success
    await page.waitForURL((url) => url.pathname === '/ui/setup/browser-extension/');
    expect(getCurrentPath(page)).toBe('/ui/setup/browser-extension/');
  });
});
