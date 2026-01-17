import { LoginPage } from '@/pages/LoginPage.mjs';
import { ToolsetsPage } from '@/pages/ToolsetsPage.mjs';
import { randomPort } from '@/test-helpers.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { expect, test } from '@playwright/test';

/**
 * Toolsets Configuration E2E Tests
 *
 * These tests verify the toolsets configuration UI for managing AI toolsets.
 *
 * NOTE: When EXA_API_KEY is provided in the environment, the tests will
 * configure the Exa Web Search toolset with a real API key and verify it's enabled.
 */
test.describe('Toolsets Configuration', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;
  let authClient;
  let resourceClient;
  let toolsetsPage;
  let loginPage;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    const port = randomPort();
    const serverUrl = `http://localhost:${port}`;

    // Set up authentication - same pattern as api-tokens tests
    authClient = createAuthServerTestClient(authServerConfig);
    resourceClient = await authClient.createResourceClient(serverUrl);
    await authClient.makeResourceAdmin(
      resourceClient.clientId,
      resourceClient.clientSecret,
      testCredentials.userId
    );

    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      port,
      host: 'localhost',
    });

    baseUrl = await serverManager.startServer();
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test.beforeEach(async ({ page }) => {
    toolsetsPage = new ToolsetsPage(page, baseUrl);
    loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
  });

  test('displays toolsets list page with Exa Web Search toolset', async ({ page }) => {
    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    await toolsetsPage.navigateToToolsetsList();
    await toolsetsPage.expectToolsetsListPage();

    // Verify Exa Web Search is listed
    await toolsetsPage.expectToolsetListed('builtin-exa-web-search');
  });

  test('navigates to toolset edit page from list', async ({ page }) => {
    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    await toolsetsPage.navigateToToolsetsList();
    await toolsetsPage.expectToolsetsListPage();

    // Click edit button
    await toolsetsPage.clickEditToolset('builtin-exa-web-search');

    // Should be on edit page (path may have trailing slash)
    await toolsetsPage.expectToolsetEditPage();
  });

  test('displays toolset configuration form', async ({ page }) => {
    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    await toolsetsPage.navigateToToolsetEdit('builtin-exa-web-search');
    await toolsetsPage.expectToolsetEditPage();
    await toolsetsPage.expectFormLoaded();
  });

  test('shows admin toggle for resource_admin users', async ({ page }) => {
    // Login first (as admin - already set up in beforeAll)
    await loginPage.performOAuthLogin('/ui/chat/');

    await toolsetsPage.navigateToToolsetEdit('builtin-exa-web-search');
    await toolsetsPage.expectToolsetEditPage();

    // Admin should see the app enable toggle
    await toolsetsPage.expectAdminToggle();
  });

  test('shows confirmation dialog when toggling app enable', async ({ page }) => {
    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    await toolsetsPage.navigateToToolsetEdit('builtin-exa-web-search');
    await toolsetsPage.expectToolsetEditPage();

    // Click the toggle to disable
    await toolsetsPage.toggleAppEnabled();

    // Should show confirmation dialog
    await expect(page.locator('text=Disable Toolset for Server')).toBeVisible();
  });

  test('configures toolset with real API key', async ({ page }) => {
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').not.toBeUndefined();
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').not.toBeNull();

    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    // Configure the toolset with the real API key
    await toolsetsPage.configureToolsetWithApiKey('builtin-exa-web-search', exaApiKey);

    // Wait for form to be saved
    await toolsetsPage.waitForFormState('saved');

    // Navigate back to list and verify status
    await toolsetsPage.navigateToToolsetsList();
    await toolsetsPage.expectToolsetEnabled('builtin-exa-web-search');
  });
});
