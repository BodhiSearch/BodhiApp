import { LoginPage } from '@/pages/LoginPage.mjs';
import { ToolsPage } from '@/pages/ToolsPage.mjs';
import { getCurrentPath, randomPort } from '@/test-helpers.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { expect, test } from '@playwright/test';

/**
 * Tools Configuration E2E Tests
 *
 * These tests verify the tools configuration UI for managing AI tools.
 *
 * NOTE: When EXA_API_KEY is provided in the environment, the tests will
 * configure the Exa Web Search tool with a real API key and verify it's enabled.
 */
test.describe('Tools Configuration', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;
  let authClient;
  let resourceClient;
  let toolsPage;
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
    toolsPage = new ToolsPage(page, baseUrl);
    loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
  });

  test('displays tools list page with Exa Web Search tool', async ({ page }) => {
    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    await toolsPage.navigateToToolsList();
    await toolsPage.expectToolsListPage();

    // Verify Exa Web Search is listed
    await toolsPage.expectToolListed('builtin-exa-web-search');
  });

  test('navigates to tool edit page from list', async ({ page }) => {
    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    await toolsPage.navigateToToolsList();
    await toolsPage.expectToolsListPage();

    // Click edit button
    await toolsPage.clickEditTool('builtin-exa-web-search');

    // Should be on edit page (path may have trailing slash)
    await toolsPage.expectToolEditPage();
  });

  test('displays tool configuration form', async ({ page }) => {
    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    await toolsPage.navigateToToolEdit('builtin-exa-web-search');
    await toolsPage.expectToolEditPage();
    await toolsPage.expectFormLoaded();
  });

  test('shows admin toggle for resource_admin users', async ({ page }) => {
    // Login first (as admin - already set up in beforeAll)
    await loginPage.performOAuthLogin('/ui/chat/');

    await toolsPage.navigateToToolEdit('builtin-exa-web-search');
    await toolsPage.expectToolEditPage();

    // Admin should see the app enable toggle
    await toolsPage.expectAdminToggle();
  });

  test('shows confirmation dialog when toggling app enable', async ({ page }) => {
    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    await toolsPage.navigateToToolEdit('builtin-exa-web-search');
    await toolsPage.expectToolEditPage();

    // Click the toggle to disable
    await toolsPage.toggleAppEnabled();

    // Should show confirmation dialog
    await expect(page.locator('text=Disable Tool for Server')).toBeVisible();
  });

  test('configures tool with real API key', async ({ page }) => {
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').not.toBeUndefined();
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').not.toBeNull();

    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    // Configure the tool with the real API key
    await toolsPage.configureToolWithApiKey('builtin-exa-web-search', exaApiKey);

    // Wait for form to be saved
    await toolsPage.waitForFormState('saved');

    // Navigate back to list and verify status
    await toolsPage.navigateToToolsList();
    await toolsPage.expectToolEnabled('builtin-exa-web-search');
  });
});
