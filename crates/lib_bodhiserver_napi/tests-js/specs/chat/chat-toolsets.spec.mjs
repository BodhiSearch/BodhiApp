import { randomPort } from '@/test-helpers.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { expect, test } from '@playwright/test';

import { ChatPage } from '@/pages/ChatPage.mjs';
import { ChatSettingsPage } from '@/pages/ChatSettingsPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ToolsetsPage } from '@/pages/ToolsetsPage.mjs';

/**
 * Chat UI Toolsets Integration E2E Tests
 *
 * These tests verify the toolsets integration in the chat UI:
 * - ToolsetsPopover component for selecting toolsets
 * - Max tool iterations setting in settings sidebar
 * - Tooltip display for disabled toolsets
 * - Badge count display for enabled toolsets
 *
 * NOTE: Tool execution tests require:
 * 1. INTEG_TEST_EXA_API_KEY environment variable for the Exa Web Search toolset
 * 2. A model that supports tool calling (e.g., GPT-4, Claude, etc.)
 */
test.describe('Chat Interface - Toolsets Integration', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;
  let authClient;
  let resourceClient;
  let loginPage;
  let chatPage;
  let chatSettingsPage;
  let toolsetsPage;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    const port = randomPort();
    const serverUrl = `http://localhost:${port}`;

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

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
    chatPage = new ChatPage(page, baseUrl);
    chatSettingsPage = new ChatSettingsPage(page, baseUrl);
    toolsetsPage = new ToolsetsPage(page, baseUrl);
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('toolsets popover trigger is visible in chat input @smoke', async ({ page }) => {
    // Login and navigate to chat
    await loginPage.performOAuthLogin();
    await chatPage.navigateToChat();

    // Verify the toolsets popover trigger exists
    const toolsetsButton = page.locator('[data-testid="toolsets-popover-trigger"]');
    await expect(toolsetsButton).toBeVisible();

    // Verify the wrench icon is visible
    await expect(toolsetsButton.locator('svg')).toBeVisible();
  });

  test('toolsets popover opens and shows toolsets list @integration', async ({ page }) => {
    // Login and navigate to chat
    await loginPage.performOAuthLogin();
    await chatPage.navigateToChat();

    // Click the toolsets button to open the popover
    const toolsetsButton = page.locator('[data-testid="toolsets-popover-trigger"]');
    await toolsetsButton.click();

    // Verify the popover content is visible
    const popoverContent = page.locator('[data-testid="toolsets-popover-content"]');
    await expect(popoverContent).toBeVisible();

    // Verify the "Toolsets" heading is present
    await expect(popoverContent.locator('h4')).toContainText('Toolsets');
  });

  test('toolsets popover shows loading state initially @integration', async ({ page }) => {
    // Login and navigate to chat
    await loginPage.performOAuthLogin();
    await chatPage.navigateToChat();

    // Open the popover immediately
    const toolsetsButton = page.locator('[data-testid="toolsets-popover-trigger"]');
    await toolsetsButton.click();

    // Verify popover is visible
    const popoverContent = page.locator('[data-testid="toolsets-popover-content"]');
    await expect(popoverContent).toBeVisible();

    // Either shows loading or content (depending on timing)
    const hasContent =
      (await popoverContent.locator('[data-testid^="toolset-item-"]').count()) > 0 ||
      (await popoverContent.getByText('Loading...').count()) > 0 ||
      (await popoverContent.getByText('No toolsets available').count()) > 0;
    expect(hasContent).toBe(true);
  });

  test('toolsets popover displays Exa Web Search toolset @integration', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await chatPage.navigateToChat();

    // Open the toolsets popover
    const toolsetsButton = page.locator('[data-testid="toolsets-popover-trigger"]');
    await toolsetsButton.click();

    // Wait for the popover to open
    const popoverContent = page.locator('[data-testid="toolsets-popover-content"]');
    await expect(popoverContent).toBeVisible();

    // Wait for toolsets to load
    await page.waitForSelector('[data-testid^="toolset-item-"]', { timeout: 10000 });

    // Verify Exa Web Search toolset is listed (it's a builtin toolset)
    const exaToolset = popoverContent.locator(
      '[data-testid="toolset-item-builtin-exa-web-search"]'
    );
    await expect(exaToolset).toBeVisible();
  });

  test('unconfigured toolset checkbox is disabled with tooltip @integration', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await chatPage.navigateToChat();

    // Open the toolsets popover
    const toolsetsButton = page.locator('[data-testid="toolsets-popover-trigger"]');
    await toolsetsButton.click();

    // Wait for the popover to open
    const popoverContent = page.locator('[data-testid="toolsets-popover-content"]');
    await expect(popoverContent).toBeVisible();

    // Wait for toolsets to load
    await page.waitForSelector('[data-testid^="toolset-item-"]', { timeout: 10000 });

    // Find the Exa toolset checkbox (should be disabled without API key configured)
    const exaCheckbox = popoverContent.locator(
      '[data-testid="toolset-checkbox-builtin-exa-web-search"]'
    );
    const isDisabled = await exaCheckbox.isDisabled();

    // If the toolset is not configured, checkbox should be disabled
    if (isDisabled) {
      // Hover over the toolset item to show tooltip
      const exaItem = popoverContent.locator(
        '[data-testid="toolset-item-builtin-exa-web-search"]'
      );
      await exaItem.hover();

      // Wait for tooltip to appear and verify it has content
      const tooltip = page.locator('[role="tooltip"]');
      await expect(tooltip).toBeVisible({ timeout: 5000 });

      const tooltipText = await tooltip.textContent();
      expect(tooltipText.length).toBeGreaterThan(0);
      // Should mention configuration or API key
      expect(
        tooltipText.includes('Configure') ||
          tooltipText.includes('API key') ||
          tooltipText.includes('settings') ||
          tooltipText.includes('Disabled')
      ).toBe(true);
    }
  });

  test('max tool iterations setting is visible in settings sidebar @integration', async ({
    page,
  }) => {
    await loginPage.performOAuthLogin();
    await chatPage.navigateToChat();

    // Open settings panel
    await chatPage.openSettingsPanel();

    // Check for max tool iterations input
    const maxIterationsInput = page.locator('[data-testid="max-tool-iterations-input"]');
    await expect(maxIterationsInput).toBeVisible();

    // Verify default value is 5
    await expect(maxIterationsInput).toHaveValue('5');
  });

  test('can modify max tool iterations setting @integration', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await chatPage.navigateToChat();

    // Open settings panel
    await chatPage.openSettingsPanel();

    // Check for max tool iterations input
    const maxIterationsInput = page.locator('[data-testid="max-tool-iterations-input"]');
    await expect(maxIterationsInput).toBeVisible();

    // Test changing the value
    await maxIterationsInput.clear();
    await maxIterationsInput.fill('10');
    await expect(maxIterationsInput).toHaveValue('10');

    // Change back to default
    await maxIterationsInput.clear();
    await maxIterationsInput.fill('5');
    await expect(maxIterationsInput).toHaveValue('5');
  });

  test('toolset selection persists when popover is reopened @integration', async ({ page }) => {
    // This test requires a configured toolset
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    if (!exaApiKey) {
      test.skip();
      return;
    }

    await loginPage.performOAuthLogin();

    // First configure the toolset
    await toolsetsPage.configureToolsetWithApiKey('builtin-exa-web-search', exaApiKey);
    await toolsetsPage.waitForFormState('saved');

    // Navigate to chat
    await chatPage.navigateToChat();

    // Open toolsets popover and enable toolset
    const toolsetsButton = page.locator('[data-testid="toolsets-popover-trigger"]');
    await toolsetsButton.click();

    const popoverContent = page.locator('[data-testid="toolsets-popover-content"]');
    await expect(popoverContent).toBeVisible();

    // Wait for toolsets to load
    await page.waitForSelector('[data-testid^="toolset-item-"]', { timeout: 10000 });

    // Enable the toolset
    const exaCheckbox = popoverContent.locator(
      '[data-testid="toolset-checkbox-builtin-exa-web-search"]'
    );
    await expect(exaCheckbox).toBeEnabled();
    await exaCheckbox.click();

    // Close popover
    await page.keyboard.press('Escape');

    // Verify badge shows count
    const badge = page.locator('[data-testid="toolsets-badge"]');
    await expect(badge).toBeVisible();
    await expect(badge).toContainText('1');

    // Reopen popover
    await toolsetsButton.click();
    await expect(popoverContent).toBeVisible();

    // Verify checkbox is still checked
    await expect(exaCheckbox).toBeChecked();
  });

  test('toolsets badge shows count when toolsets are enabled @integration', async ({ page }) => {
    // This test requires a configured toolset
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    if (!exaApiKey) {
      test.skip();
      return;
    }

    await loginPage.performOAuthLogin();

    // First configure the toolset
    await toolsetsPage.configureToolsetWithApiKey('builtin-exa-web-search', exaApiKey);
    await toolsetsPage.waitForFormState('saved');

    // Navigate to chat
    await chatPage.navigateToChat();

    // Open the toolsets popover
    const toolsetsButton = page.locator('[data-testid="toolsets-popover-trigger"]');
    await toolsetsButton.click();

    // Wait for the popover to open
    const popoverContent = page.locator('[data-testid="toolsets-popover-content"]');
    await expect(popoverContent).toBeVisible();

    // Wait for toolsets to load
    await page.waitForSelector('[data-testid^="toolset-item-"]', { timeout: 10000 });

    // Enable the toolset
    const exaCheckbox = popoverContent.locator(
      '[data-testid="toolset-checkbox-builtin-exa-web-search"]'
    );
    await expect(exaCheckbox).toBeEnabled();
    await exaCheckbox.click();

    // Close the popover
    await page.keyboard.press('Escape');

    // Verify badge appears with count
    const badge = page.locator('[data-testid="toolsets-badge"]');
    await expect(badge).toBeVisible();
    await expect(badge).toContainText('1');
  });
});
