import { test } from '@playwright/test';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { randomPort } from '@/test-helpers.mjs';
import { SetupWelcomePage } from '@/pages/SetupWelcomePage.mjs';
import { SetupResourceAdminPage } from '@/pages/SetupResourceAdminPage.mjs';
import { SetupDownloadModelsPage } from '@/pages/SetupDownloadModelsPage.mjs';
import { SetupApiModelsPage } from '@/pages/SetupApiModelsPage.mjs';
import { SetupFixtures } from '@/fixtures/setupFixtures.mjs';
import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';

test.describe('API Models Setup Integration', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;

  // Page objects
  let welcomePage;
  let resourceAdminPage;
  let downloadModelsPage;
  let apiModelsPage;

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
  });

  test.afterEach(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  async function navigateToApiModelsPage(page) {
    // Navigate through the setup flow to reach API models page
    await welcomePage.navigateAndWaitForPage('/ui/setup/');
    await welcomePage.expectToBeOnPage('/ui/setup/');

    // Complete welcome page
    const setupData = SetupFixtures.scenarios.SKIP_ALL_OPTIONAL_STEPS();
    await welcomePage.completeInitialSetup(setupData.serverName);

    // Complete resource admin page
    await resourceAdminPage.expectNavigationToResourceAdmin();
    await resourceAdminPage.performCompleteLogin();

    // Complete download models page (skip)
    await downloadModelsPage.expectNavigationToDownloadModels();
    await downloadModelsPage.skipDownloads();

    // Should now be at API models page
    await apiModelsPage.expectNavigationToApiModels();
  }

  test('API Models Setup Flow - Complete Journey', async ({ page }) => {
    // Phase 1: Navigate and verify initial state
    await navigateToApiModelsPage(page);

    // Verify we're on the API models page with correct step indicator
    await apiModelsPage.expectApiModelsPage();
    await apiModelsPage.expectStepIndicator(4);

    // Verify initial form state (empty fields, disabled buttons)
    await apiModelsPage.expectInitialFormState();
    await apiModelsPage.expectHelpSection();

    // Phase 2: Test form interactions
    // Select API format (OpenAI)
    await apiModelsPage.selectApiFormat('openai');

    // Verify base URL auto-populates
    await apiModelsPage.form.expectBaseUrlValue('https://api.openai.com/v1');

    // Fill API key
    await apiModelsPage.fillApiKey('sk-test-api-key-12345');

    // Verify fetch models button becomes enabled (only requires API key + base URL)
    await apiModelsPage.form.expectFetchModelsButtonEnabled();

    // Test connection button should still be disabled (requires models to be selected)
    await apiModelsPage.form.expectTestConnectionButtonDisabled();

    // Phase 3: Test validation and error states
    // Clear API key - with optional API key, Fetch Models should remain enabled
    await apiModelsPage.form.clearApiKey();
    await apiModelsPage.form.expectTestConnectionButtonDisabled();
    await apiModelsPage.form.expectFetchModelsButtonEnabled();

    // Fill API key again for submission test
    await apiModelsPage.fillApiKey('sk-test-api-key-12345');

    // Attempt form submission with no models selected (should fail validation)
    await apiModelsPage.submitForm();

    // Should show error and stay on page (no navigation)
    await apiModelsPage.expectCurrentPath('/ui/setup/api-models/');

    // Phase 4: Skip functionality
    await apiModelsPage.skipApiSetup();

    // Verify navigation to browser extension page
    await apiModelsPage.expectNavigationToBrowserExtension();

    // Phase 5: Direct navigation test
    await apiModelsPage.navigateAndWaitForPage('/ui/setup/api-models/');

    // Should work since we're authenticated and can access setup pages
    await apiModelsPage.expectApiModelsPage();

    // Skip setup again to complete test
    await apiModelsPage.skipApiSetup();
    await apiModelsPage.expectNavigationToBrowserExtension();
  });

  test('API Models Setup - Happy Path Model Creation', async ({ page }) => {
    // Get API key from environment
    const { apiKey } = ApiModelFixtures.getRequiredEnvVars();

    // Navigate to API models page
    await navigateToApiModelsPage(page);

    // Select API format (OpenAI)
    await apiModelsPage.selectApiFormat('openai');

    // Fill API key from environment
    await apiModelsPage.fillApiKey(apiKey);

    // Fetch models from real OpenAI API and select gpt-3.5-turbo
    await apiModelsPage.fetchAndSelectModels(['gpt-3.5-turbo']);

    // Test connection with real API
    await apiModelsPage.testConnectionWithRetry();

    // Submit the form and verify navigation to browser extension page
    await apiModelsPage.createModelAndNavigateToBrowserExtension();
  });
});
