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

  test('MCP Server and Instance CRUD Lifecycle', async ({ page }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
    const serverData = McpFixtures.createServerData();
    const instanceData = McpFixtures.createLifecycleData();

    await test.step('Login', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
    });

    await test.step('Create MCP server (admin)', async () => {
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      const row = await mcpsPage.page.locator(`[data-test-server-name="${serverData.name}"]`).first();
      await expect(row).toBeVisible();
    });

    await test.step('Create MCP instance using server combobox', async () => {
      await mcpsPage.createMcpInstance(
        serverData.name,
        instanceData.name,
        instanceData.slug,
        instanceData.description
      );
      await mcpsPage.expectToolsSection();
    });

    await test.step('Verify MCP instance appears in list', async () => {
      await mcpsPage.clickDone();
      await mcpsPage.page.waitForURL(/\/ui\/mcps(?!\/new)/);
      await mcpsPage.expectMcpsListPage();
      const row = await mcpsPage.getMcpRowByName(instanceData.name);
      await expect(row).toBeVisible();
    });

    await test.step('Delete MCP instance', async () => {
      const mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpId).toBeTruthy();
      await mcpsPage.clickDeleteById(mcpId);
      await mcpsPage.confirmDelete();
    });
  });

  test('MCP Server Tool Discovery', async ({ page }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
    const serverData = McpFixtures.createServerData();
    const instanceData = McpFixtures.createToolDiscoveryData();

    await test.step('Login and create MCP server', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name);
    });

    await test.step('Create MCP instance and fetch tools', async () => {
      await mcpsPage.createMcpInstance(serverData.name, instanceData.name, instanceData.slug);
      await mcpsPage.clickFetchTools();
      await mcpsPage.expectToolsList();
      await mcpsPage.expectToolItem(McpFixtures.EXPECTED_TOOL);
    });
  });

  test('MCP Playground - Tool Execution', async ({ page }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
    const serverData = McpFixtures.createServerData();
    const instanceData = McpFixtures.createPlaygroundData();

    await test.step('Login and create MCP server + instance with tools', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      await mcpsPage.createMcpInstance(
        serverData.name,
        instanceData.name,
        instanceData.slug,
        instanceData.description
      );
      await mcpsPage.clickFetchTools();
      await mcpsPage.expectToolsList();
      await mcpsPage.expectToolItem(McpFixtures.PLAYGROUND_TOOL);
    });

    await test.step('Navigate to playground from list', async () => {
      await mcpsPage.clickDone();
      await mcpsPage.page.waitForURL(/\/ui\/mcps(?!\/new)/);
      await mcpsPage.expectMcpsListPage();
      const mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpId).toBeTruthy();
      await mcpsPage.clickPlaygroundById(mcpId);
      await mcpsPage.expectPlaygroundPage();
    });

    await test.step('Select tool and execute', async () => {
      await mcpsPage.selectPlaygroundTool(McpFixtures.PLAYGROUND_TOOL);
      await mcpsPage.expectPlaygroundToolSelected(McpFixtures.PLAYGROUND_TOOL);
      await mcpsPage.expectNoWhitelistedWarning();

      const paramField = mcpsPage.page.locator(
        mcpsPage.selectors.playgroundParam(McpFixtures.PLAYGROUND_PARAM)
      );
      await expect(paramField).toBeVisible();

      await mcpsPage.fillPlaygroundParam(
        McpFixtures.PLAYGROUND_PARAM,
        McpFixtures.PLAYGROUND_PARAMS.repoName
      );
      await mcpsPage.clickPlaygroundExecute();
      await mcpsPage.expectPlaygroundResultSuccess();
    });

    await test.step('Verify result tabs', async () => {
      await mcpsPage.clickPlaygroundResultTab('raw');
      const rawContent = await mcpsPage.getPlaygroundResultContent();
      expect(rawContent).toBeTruthy();

      await mcpsPage.clickPlaygroundResultTab('request');
      const requestContent = await mcpsPage.getPlaygroundResultContent();
      expect(requestContent).toContain(McpFixtures.PLAYGROUND_TOOL);

      const copyButton = mcpsPage.page.locator(mcpsPage.selectors.playgroundCopyButton);
      await expect(copyButton).toBeVisible();
    });

    await test.step('Test form/JSON toggle sync', async () => {
      await mcpsPage.switchToJsonMode();
      const jsonContent = await mcpsPage.getPlaygroundJsonContent();
      expect(jsonContent).toContain(McpFixtures.PLAYGROUND_PARAMS.repoName);

      const newValue = 'BodhiSearch/BodhiApp';
      const newJson = JSON.stringify({ repoName: newValue }, null, 2);
      await mcpsPage.fillPlaygroundJson(newJson);

      await mcpsPage.switchToFormMode();
      const paramContainer = mcpsPage.page.locator(
        mcpsPage.selectors.playgroundParam(McpFixtures.PLAYGROUND_PARAM)
      );
      const input = paramContainer.locator('input, textarea').first();
      await expect(input).toHaveValue(newValue);
    });
  });

  test('MCP Playground - Non-Whitelisted Tool Error', async ({ page }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
    const serverData = McpFixtures.createServerData();
    const instanceData = McpFixtures.createPlaygroundData();

    await test.step('Login and create MCP server + instance with tools', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      await mcpsPage.createMcpInstance(
        serverData.name,
        instanceData.name,
        instanceData.slug,
        instanceData.description
      );
      await mcpsPage.clickFetchTools();
      await mcpsPage.expectToolsList();
    });

    await test.step('Deselect tool via edit page', async () => {
      await mcpsPage.clickDone();
      await mcpsPage.page.waitForURL(/\/ui\/mcps(?!\/new)/);
      await mcpsPage.expectMcpsListPage();
      const mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpId).toBeTruthy();
      await mcpsPage.clickEditById(mcpId);
      await mcpsPage.expectNewMcpPage();
      await expect(mcpsPage.page.getByText('Edit MCP')).toBeVisible();
      await mcpsPage.expectToolsSection();
      await mcpsPage.toggleTool(McpFixtures.PLAYGROUND_TOOL);
      await mcpsPage.clickUpdate();
      await mcpsPage.expectMcpsListPage();
    });

    await test.step('Navigate to playground and select non-whitelisted tool', async () => {
      const mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpId).toBeTruthy();
      await mcpsPage.clickPlaygroundById(mcpId);
      await mcpsPage.expectPlaygroundPage();

      await mcpsPage.selectPlaygroundTool(McpFixtures.PLAYGROUND_TOOL);
      await mcpsPage.expectPlaygroundToolSelected(McpFixtures.PLAYGROUND_TOOL);
      await mcpsPage.expectNotWhitelistedWarning();
    });

    await test.step('Execute and verify error response', async () => {
      await mcpsPage.fillPlaygroundParam(
        McpFixtures.PLAYGROUND_PARAM,
        McpFixtures.PLAYGROUND_PARAMS.repoName
      );
      await mcpsPage.clickPlaygroundExecute();
      await mcpsPage.expectPlaygroundResultError();
    });
  });

  test('MCP Playground - Refresh and Disabled States', async ({ page }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
    const serverData = McpFixtures.createServerData();
    const instanceData = McpFixtures.createPlaygroundData();

    await test.step('Login and create MCP server + instance with tools', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      await mcpsPage.createMcpInstance(
        serverData.name,
        instanceData.name,
        instanceData.slug,
        instanceData.description
      );
      await mcpsPage.clickFetchTools();
      await mcpsPage.expectToolsList();
    });

    await test.step('Navigate to playground and refresh tools', async () => {
      await mcpsPage.clickDone();
      await mcpsPage.page.waitForURL(/\/ui\/mcps(?!\/new)/);
      await mcpsPage.expectMcpsListPage();
      const mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpId).toBeTruthy();
      await mcpsPage.clickPlaygroundById(mcpId);
      await mcpsPage.expectPlaygroundPage();

      const toolSidebar = mcpsPage.page.locator(mcpsPage.selectors.playgroundToolList);
      await expect(toolSidebar).toBeVisible();

      await mcpsPage.clickPlaygroundRefresh();
      await expect(toolSidebar).toBeVisible();
    });

    await test.step('Disable MCP instance and verify execution error', async () => {
      await mcpsPage.clickPlaygroundBack();
      await mcpsPage.expectMcpsListPage();
      const mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
      await mcpsPage.clickEditById(mcpId);
      await mcpsPage.expectNewMcpPage();
      await expect(mcpsPage.page.getByText('Edit MCP')).toBeVisible();

      const enabledSwitch = mcpsPage.page.locator(mcpsPage.selectors.enabledSwitch);
      await enabledSwitch.click();
      await mcpsPage.clickUpdate();
      await mcpsPage.expectMcpsListPage();

      await mcpsPage.clickPlaygroundById(mcpId);
      await mcpsPage.expectPlaygroundPage();

      await mcpsPage.selectPlaygroundTool(McpFixtures.PLAYGROUND_TOOL);
      await mcpsPage.fillPlaygroundParam(
        McpFixtures.PLAYGROUND_PARAM,
        McpFixtures.PLAYGROUND_PARAMS.repoName
      );
      await mcpsPage.clickPlaygroundExecute();
      await mcpsPage.expectPlaygroundResultError();
    });
  });
});
