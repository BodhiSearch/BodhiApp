import { McpFixtures } from '@/fixtures/mcpFixtures.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { McpsPage } from '@/pages/McpsPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';
import { SHARED_SERVER_URL } from '@/test-helpers.mjs';

test.describe('MCP Server Management', () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test('MCP Server CRUD Lifecycle', async ({ page }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
    const testData = McpFixtures.createLifecycleData();

    await test.step('Login and verify empty MCP list', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.navigateToMcpsList();
      await mcpsPage.expectMcpsListPage();
      await mcpsPage.expectEmptyState();
    });

    await test.step('Navigate to new MCP page', async () => {
      await mcpsPage.clickNewMcp();
      await mcpsPage.expectNewMcpPage();
    });

    await test.step('Create MCP server with admin enable flow', async () => {
      await mcpsPage.createMcpWithAdminEnable(
        testData.url, testData.name, testData.slug, testData.description
      );
      await mcpsPage.expectToolsSection();
    });

    await test.step('Verify MCP appears in list', async () => {
      await mcpsPage.clickDone();
      await mcpsPage.page.waitForURL(/\/ui\/mcps(?!\/new)/);
      await mcpsPage.expectMcpsListPage();
      const row = await mcpsPage.getMcpRowByName(testData.name);
      await expect(row).toBeVisible();
    });

    await test.step('Delete MCP server with confirmation', async () => {
      const mcpId = await mcpsPage.getMcpUuidByName(testData.name);
      expect(mcpId).toBeTruthy();
      await mcpsPage.clickDeleteById(mcpId);
      await expect(mcpsPage.page.getByText('Delete MCP Server')).toBeVisible();
      await mcpsPage.confirmDelete();
      await mcpsPage.expectEmptyState();
    });
  });

  test('MCP Server Tool Discovery', async ({ page }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
    const testData = McpFixtures.createToolDiscoveryData();

    await test.step('Login and create MCP server', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpWithAdminEnable(testData.url, testData.name, testData.slug);
    });

    await test.step('Fetch and verify tools from remote server', async () => {
      await mcpsPage.clickFetchTools();
      await mcpsPage.expectToolsList();
      await mcpsPage.expectToolItem(McpFixtures.EXPECTED_TOOL);
    });
  });
});
