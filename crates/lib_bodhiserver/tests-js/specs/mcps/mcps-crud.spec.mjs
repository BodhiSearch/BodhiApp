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

    await test.step('Overview shows non-zero tool count', async () => {
      const toolsCount = mcpsPage.page.locator('[data-testid="mcp-playground-capability-count-tools"]');
      await expect(toolsCount).toBeVisible();
      const text = (await toolsCount.textContent())?.trim() ?? '';
      expect(Number(text)).toBeGreaterThan(0);
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
      const rawContent = await mcpsPage.getPlaygroundResultRaw();
      expect(rawContent).toBeTruthy();

      await mcpsPage.clickPlaygroundResultTab('request');
      const requestContent = await mcpsPage.getPlaygroundResultRequest();
      expect(requestContent).toContain('echo');

      const copyButton = mcpsPage.page.locator(mcpsPage.selectors.playgroundCopyButton);
      await expect(copyButton).toBeVisible();
    });

    await test.step('Preview a prompt and render messages', async () => {
      await mcpsPage.page.click(mcpsPage.selectors.playgroundCapability('prompts'));
      await mcpsPage.page.click('[data-testid="mcp-playground-rail-item-simple-prompt"]');
      await expect(mcpsPage.page.locator('[data-testid="mcp-playground-prompt-detail"]')).toBeVisible();
      await mcpsPage.page.click('[data-testid="mcp-playground-prompt-preview-button"]');
      await expect(mcpsPage.page.locator('[data-testid="mcp-playground-result-status"]')).toHaveAttribute(
        'data-test-state',
        'success'
      );
      await expect(mcpsPage.page.locator('[data-testid="mcp-playground-prompt-msg-0"]')).toBeVisible();
    });

    await test.step('Read the first resource and verify success', async () => {
      await mcpsPage.page.click(mcpsPage.selectors.playgroundCapability('resources'));
      const firstResource = mcpsPage.page.locator('[data-testid^="mcp-playground-rail-item-"]').first();
      await expect(firstResource).toBeVisible();
      await firstResource.click();
      await expect(mcpsPage.page.locator('[data-testid="mcp-playground-resource-detail"]')).toBeVisible();
      await mcpsPage.page.click('[data-testid="mcp-playground-resource-read-button"]');
      await expect(mcpsPage.page.locator('[data-testid="mcp-playground-result-status"]')).toHaveAttribute(
        'data-test-state',
        'success'
      );
    });

    await test.step('Resolve a template and verify the resource is read', async () => {
      await mcpsPage.page.click(mcpsPage.selectors.playgroundCapability('templates'));
      const firstTemplate = mcpsPage.page.locator('[data-testid^="mcp-playground-rail-item-"]').first();
      await expect(firstTemplate).toBeVisible();
      await firstTemplate.click();
      await expect(mcpsPage.page.locator('[data-testid="mcp-playground-template-detail"]')).toBeVisible();
      const fields = mcpsPage.page.locator('[data-testid^="mcp-playground-param-"] input, [data-testid^="mcp-playground-param-"] textarea');
      const count = await fields.count();
      for (let i = 0; i < count; i++) {
        await fields.nth(i).fill('1');
      }
      const resolved = mcpsPage.page.locator('[data-testid="mcp-playground-template-resolved"]');
      await expect(resolved).toHaveAttribute('data-filled', 'true');
      await mcpsPage.page.click('[data-testid="mcp-playground-template-read-button"]');
      await expect(mcpsPage.page.locator('[data-testid="mcp-playground-result-status"]')).toHaveAttribute(
        'data-test-state',
        'success'
      );
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

      // V2 lands on the Overview pane; switch to Tools to surface the rail.
      await mcpsPage.page.click(mcpsPage.selectors.playgroundCapability('tools'));
      const toolList = mcpsPage.page.locator(mcpsPage.selectors.playgroundToolList);
      await expect(toolList).toBeVisible();
    });

    await test.step('Refresh tools', async () => {
      await mcpsPage.clickPlaygroundRefresh();
      const toolList = mcpsPage.page.locator(mcpsPage.selectors.playgroundToolList);
      await expect(toolList).toBeVisible();
    });

    await test.step('Disable MCP instance and verify error on playground load', async () => {
      await mcpsPage.clickPlaygroundBack();
      await mcpsPage.expectMcpsListPage();
      const mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
      await mcpsPage.clickEditById(mcpId);
      await mcpsPage.expectNewMcpPage();
      // Scope to the form card (the breadcrumb also reads "Edit MCP Connection").
      await expect(
        mcpsPage.page
          .locator('[data-testid="new-mcp-page"]')
          .getByText('Edit MCP Connection', { exact: true })
      ).toBeVisible();

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
