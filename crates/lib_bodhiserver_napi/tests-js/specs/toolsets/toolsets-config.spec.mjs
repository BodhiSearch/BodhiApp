import { LoginPage } from '@/pages/LoginPage.mjs';
import { ToolsetsPage } from '@/pages/ToolsetsPage.mjs';
import {
  getAuthServerConfig,
  getPreConfiguredResourceClient,
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

const TOOLSET_TYPE = 'builtin-exa-search';

test.describe('Toolsets Configuration', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;
  let toolsetsPage;
  let loginPage;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    const resourceClient = getPreConfiguredResourceClient();
    const port = 51135;

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

  test('displays toolsets list page', async ({ page }) => {
    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    await toolsetsPage.navigateToToolsetsList();
    await toolsetsPage.expectToolsetsListPage();

    // List page should load (may be empty since no toolsets created yet)
  });

  test('navigates to toolset edit page from list', async ({ page }) => {
    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    // Enable the toolset type first
    await toolsetsPage.enableToolsetTypeOnAdmin(TOOLSET_TYPE);

    // Create a toolset via the new page
    await toolsetsPage.navigateToNewToolset();
    await toolsetsPage.expectNewToolsetPage();
    await toolsetsPage.createToolset(TOOLSET_TYPE, 'test-exa', 'test-api-key');

    // Should redirect to list
    await toolsetsPage.expectToolsetsListPage();

    // Click edit button using type selector
    await toolsetsPage.clickEditByScope(TOOLSET_TYPE);

    // Should be on edit page
    await toolsetsPage.expectToolsetEditPage();
  });

  test('displays toolset configuration form', async ({ page }) => {
    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    // Enable the toolset type first
    await toolsetsPage.enableToolsetTypeOnAdmin(TOOLSET_TYPE);

    // Create a toolset
    await toolsetsPage.navigateToNewToolset();
    await toolsetsPage.createToolset(TOOLSET_TYPE, 'test-exa-2', 'test-api-key-2');

    // Navigate to edit page using type selector
    await toolsetsPage.clickEditByScope(TOOLSET_TYPE);
    await toolsetsPage.expectToolsetEditPage();
    await toolsetsPage.expectFormLoaded();
  });

  test('shows admin toggle for resource_admin users', async ({ page }) => {
    // Login first (as admin - already set up in beforeAll)
    await loginPage.performOAuthLogin('/ui/chat/');

    // Navigate to admin page
    await toolsetsPage.navigateToAdmin();
    await toolsetsPage.expectAdminPage();

    // Admin should see the type toggle
    await toolsetsPage.expectTypeToggle(TOOLSET_TYPE);
  });

  test('shows confirmation dialog when toggling app enable', async ({ page }) => {
    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    // Navigate to admin page
    await toolsetsPage.navigateToAdmin();
    await toolsetsPage.expectAdminPage();

    // Toggle the type (regardless of current state)
    await toolsetsPage.toggleTypeEnabled(TOOLSET_TYPE);

    // Should show either enable or disable confirmation dialog
    const enableDialog = page.getByRole('heading', { name: 'Enable Toolset Type' });
    const disableDialog = page.getByRole('heading', { name: 'Disable Toolset Type' });

    // Check which dialog appeared
    await expect(enableDialog.or(disableDialog)).toBeVisible();

    // Confirm the action based on which dialog is visible
    const isEnableDialog = await enableDialog.isVisible();
    const confirmButton = page.getByRole('button', {
      name: isEnableDialog ? 'Enable' : 'Disable',
    });
    await confirmButton.click();

    // Toggle again to verify the opposite dialog appears
    await toolsetsPage.toggleTypeEnabled(TOOLSET_TYPE);
    await expect(enableDialog.or(disableDialog)).toBeVisible();
  });

  test('configures toolset with real API key', async ({ page }) => {
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').not.toBeUndefined();
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').not.toBeNull();

    // Login first
    await loginPage.performOAuthLogin('/ui/chat/');

    // Configure the toolset with the real API key (creates new toolset)
    await toolsetsPage.configureToolsetWithApiKey(TOOLSET_TYPE, exaApiKey, 'test-toolset-new');

    // Should be redirected to list page
    await toolsetsPage.expectToolsetsListPage();

    // Verify toolset row exists using type selector
    const toolsetRow = await toolsetsPage.getToolsetRowByName('test-toolset-new');
    await expect(toolsetRow).toBeVisible();
  });
});
