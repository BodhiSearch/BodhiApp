import { McpFixtures } from '@/fixtures/mcpFixtures.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { McpsPage } from '@/pages/McpsPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

test.describe('MCP Server Management', () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test('MCP Server and Instance CRUD Lifecycle', async ({ page, sharedServerUrl }) => {
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, sharedServerUrl);
    const serverData = McpFixtures.createServerData();
    const instanceData = McpFixtures.createLifecycleData();

    await test.step('Login', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
    });

    await test.step('Create MCP server (admin)', async () => {
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      const row = await mcpsPage.page
        .locator(`[data-test-server-name="${serverData.name}"]`)
        .first();
      await expect(row).toBeVisible();
    });

    await test.step('Create MCP instance', async () => {
      await mcpsPage.createMcpInstance(
        serverData.name,
        instanceData.name,
        instanceData.slug,
        instanceData.description
      );
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

  test('MCP Playground - Tool Execution', async ({ page, sharedServerUrl }) => {
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, sharedServerUrl);
    const serverData = McpFixtures.createEverythingServerData();
    const instanceData = McpFixtures.createPlaygroundData();

    await test.step('Login and create MCP server + instance', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      await mcpsPage.createMcpInstance(
        serverData.name,
        instanceData.name,
        instanceData.slug,
        instanceData.description
      );
    });

    await test.step('Navigate to playground from list', async () => {
      await mcpsPage.expectMcpsListPage();
      const mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpId).toBeTruthy();
      await mcpsPage.clickPlaygroundById(mcpId);
      await mcpsPage.expectPlaygroundPage();
    });

    await test.step('Wait for MCP client to connect and list tools', async () => {
      // The playground uses MCP client SDK to connect via the proxy
      await mcpsPage.expectPlaygroundConnected();
    });

    await test.step('Select tool and execute', async () => {
      await mcpsPage.selectPlaygroundTool('echo');
      await mcpsPage.expectPlaygroundToolSelected('echo');

      const paramField = mcpsPage.page.locator(mcpsPage.selectors.playgroundParam('message'));
      await expect(paramField).toBeVisible();

      await mcpsPage.fillPlaygroundParam('message', 'hello from playground');
      await mcpsPage.clickPlaygroundExecute();
      await mcpsPage.expectPlaygroundResultSuccess();
    });

    await test.step('Verify result tabs', async () => {
      await mcpsPage.clickPlaygroundResultTab('raw');
      const rawContent = await mcpsPage.getPlaygroundResultContent();
      expect(rawContent).toBeTruthy();

      await mcpsPage.clickPlaygroundResultTab('request');
      const requestContent = await mcpsPage.getPlaygroundResultContent();
      expect(requestContent).toContain('echo');

      const copyButton = mcpsPage.page.locator(mcpsPage.selectors.playgroundCopyButton);
      await expect(copyButton).toBeVisible();
    });

    await test.step('Test form/JSON toggle sync', async () => {
      await mcpsPage.switchToJsonMode();
      const jsonContent = await mcpsPage.getPlaygroundJsonContent();
      expect(jsonContent).toContain('hello from playground');

      const newValue = 'updated message';
      const newJson = JSON.stringify({ message: newValue }, null, 2);
      await mcpsPage.fillPlaygroundJson(newJson);

      await mcpsPage.switchToFormMode();
      const paramContainer = mcpsPage.page.locator(mcpsPage.selectors.playgroundParam('message'));
      const input = paramContainer.locator('input, textarea').first();
      await expect(input).toHaveValue(newValue);
    });
  });

  test('MCP Playground - Refresh and Disabled States', async ({ page, sharedServerUrl }) => {
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, sharedServerUrl);
    const serverData = McpFixtures.createEverythingServerData();
    const instanceData = McpFixtures.createPlaygroundData();

    await test.step('Login and create MCP server + instance', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      await mcpsPage.createMcpInstance(
        serverData.name,
        instanceData.name,
        instanceData.slug,
        instanceData.description
      );
    });

    await test.step('Navigate to playground and wait for connection', async () => {
      await mcpsPage.expectMcpsListPage();
      const mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpId).toBeTruthy();
      await mcpsPage.clickPlaygroundById(mcpId);
      await mcpsPage.expectPlaygroundPage();

      // Wait for MCP client to connect
      await mcpsPage.expectPlaygroundConnected();

      const toolSidebar = mcpsPage.page.locator(mcpsPage.selectors.playgroundToolList);
      await expect(toolSidebar).toBeVisible();
    });

    await test.step('Refresh tools', async () => {
      await mcpsPage.clickPlaygroundRefresh();
      const toolSidebar = mcpsPage.page.locator(mcpsPage.selectors.playgroundToolList);
      await expect(toolSidebar).toBeVisible();
    });

    await test.step('Disable MCP instance and verify error on playground load', async () => {
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

      // When MCP is disabled, the proxy returns an error, so the MCP client can't connect
      await mcpsPage.expectPlaygroundConnectionError();
    });
  });
});
