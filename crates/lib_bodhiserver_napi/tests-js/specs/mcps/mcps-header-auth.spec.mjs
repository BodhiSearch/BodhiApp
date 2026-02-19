import { McpFixtures } from '@/fixtures/mcpFixtures.mjs';
import { AccessRequestReviewPage } from '@/pages/AccessRequestReviewPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { McpsPage } from '@/pages/McpsPage.mjs';
import { OAuthTestApp } from '@/pages/OAuthTestApp.mjs';
import {
  getAuthServerConfig,
  getPreConfiguredAppClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';
import { SHARED_SERVER_URL, SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';

test.describe('MCP Header Authentication', { tag: ['@mcps', '@auth'] }, () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test('Create MCP with header auth, fetch tools, and execute via playground', async ({ page }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
    const serverData = McpFixtures.createTavilyServerData();
    const instanceData = McpFixtures.createTavilyInstanceData();

    await test.step('Login', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
    });

    await test.step('Create Tavily MCP server', async () => {
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      const row = mcpsPage.page.locator(`[data-test-server-name="${serverData.name}"]`).first();
      await expect(row).toBeVisible();
    });

    await test.step('Create MCP instance with header auth', async () => {
      await mcpsPage.createMcpInstanceWithAuth(
        serverData.name,
        instanceData.name,
        instanceData.slug,
        'Authorization',
        `Bearer ${McpFixtures.TAVILY_API_KEY}`,
        instanceData.description
      );
      await mcpsPage.expectMcpsListPage();
      const row = await mcpsPage.getMcpRowByName(instanceData.name);
      await expect(row).toBeVisible();
    });

    await test.step('Navigate to playground and execute tavily_search', async () => {
      const mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpId).toBeTruthy();
      await mcpsPage.clickPlaygroundById(mcpId);
      await mcpsPage.expectPlaygroundPage();

      await mcpsPage.selectPlaygroundTool(McpFixtures.TAVILY_EXPECTED_TOOL);
      await mcpsPage.expectPlaygroundToolSelected(McpFixtures.TAVILY_EXPECTED_TOOL);

      await mcpsPage.fillPlaygroundParam('query', McpFixtures.TAVILY_SEARCH_PARAMS.query);
      await mcpsPage.clickPlaygroundExecute();
      await mcpsPage.expectPlaygroundResultSuccess();
    });
  });

  test('Edit MCP: switch header auth to public and back', async ({ page }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
    const serverData = McpFixtures.createTavilyServerData();
    const instanceData = McpFixtures.createTavilyInstanceData();

    await test.step('Login and create MCP server + instance with auth', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name);
      await mcpsPage.createMcpInstanceWithAuth(
        serverData.name,
        instanceData.name,
        instanceData.slug,
        'Authorization',
        `Bearer ${McpFixtures.TAVILY_API_KEY}`
      );
    });

    await test.step('Edit MCP: switch to public auth', async () => {
      await mcpsPage.expectMcpsListPage();
      const mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpId).toBeTruthy();
      await mcpsPage.clickEditById(mcpId);
      await mcpsPage.expectNewMcpPage();

      await mcpsPage.expectAuthTypeState('header');
      await mcpsPage.expectAuthHeaderFields();
      await mcpsPage.expectAuthHeaderKeyValue('Authorization');

      await mcpsPage.selectAuthType('public');
      await mcpsPage.expectNoAuthHeaderFields();
      await mcpsPage.clickUpdate();
      await mcpsPage.expectMcpsListPage();
    });

    await test.step('Edit MCP: switch back to header auth', async () => {
      const mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
      await mcpsPage.clickEditById(mcpId);
      await mcpsPage.expectNewMcpPage();

      await mcpsPage.expectAuthTypeState('public');
      await mcpsPage.expectNoAuthHeaderFields();

      await mcpsPage.selectAuthType('header');
      await mcpsPage.expectAuthHeaderFields();
      await mcpsPage.fillAuthHeaderKey('Authorization');
      await mcpsPage.fillAuthHeaderValue(`Bearer ${McpFixtures.TAVILY_API_KEY}`);
      await mcpsPage.clickUpdate();
      await mcpsPage.expectMcpsListPage();
    });

    await test.step('Verify auth works via playground execution', async () => {
      const mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
      await mcpsPage.clickPlaygroundById(mcpId);
      await mcpsPage.expectPlaygroundPage();

      await mcpsPage.selectPlaygroundTool(McpFixtures.TAVILY_EXPECTED_TOOL);
      await mcpsPage.fillPlaygroundParam('query', 'test');
      await mcpsPage.clickPlaygroundExecute();
      await mcpsPage.expectPlaygroundResultSuccess();
    });
  });

  test('OAuth access request with header-auth MCP and tool execution via REST', async ({
    page,
  }) => {
    let mcpInstanceId;
    const serverData = McpFixtures.createTavilyServerData();
    const instanceData = McpFixtures.createTavilyInstanceData();

    await test.step('Phase 1: Session login, create Tavily MCP server + header-auth instance', async () => {
      const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
      await loginPage.performOAuthLogin();

      const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      await mcpsPage.createMcpInstanceWithAuth(
        serverData.name,
        instanceData.name,
        instanceData.slug,
        'Authorization',
        `Bearer ${McpFixtures.TAVILY_API_KEY}`,
        instanceData.description
      );

      await mcpsPage.navigateToMcpsList();
      mcpInstanceId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpInstanceId).toBeTruthy();
    });

    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Phase 2: Configure OAuth form with Tavily MCP request', async () => {
      await app.navigate();

      await app.config.configureOAuthForm({
        bodhiServerUrl: SHARED_SERVER_URL,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'openid profile email scope_user_user',
        requested: JSON.stringify({ mcp_servers: [{ url: McpFixtures.TAVILY_URL }] }),
      });
    });

    await test.step('Phase 3: Submit access request and approve with Tavily MCP', async () => {
      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(SHARED_SERVER_URL);

      const reviewPage = new AccessRequestReviewPage(page, SHARED_SERVER_URL);
      await reviewPage.approveWithMcps([
        { url: McpFixtures.TAVILY_URL, instanceId: mcpInstanceId },
      ]);

      await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
      await app.accessCallback.waitForLoaded();
      await app.accessCallback.clickLogin();
      await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
    });

    await test.step('Phase 4: Verify header-auth MCP access via REST API', async () => {
      await app.rest.navigateTo();

      await app.rest.sendRequest({
        method: 'GET',
        url: '/bodhi/v1/mcps',
      });
      expect(await app.rest.getResponseStatus()).toBe(200);
      const listData = await app.rest.getResponse();
      expect(listData.mcps).toBeDefined();
      const approvedMcp = listData.mcps.find((m) => m.id === mcpInstanceId);
      expect(approvedMcp).toBeTruthy();
      expect(approvedMcp.auth_type).toBe('header');
      expect(approvedMcp.auth_uuid).toBeTruthy();

      await app.rest.sendRequest({
        method: 'GET',
        url: `/bodhi/v1/mcps/${mcpInstanceId}`,
      });
      expect(await app.rest.getResponseStatus()).toBe(200);
      const mcpData = await app.rest.getResponse();
      expect(mcpData.id).toBe(mcpInstanceId);
      expect(mcpData.auth_type).toBe('header');
    });

    await test.step('Phase 5: Execute tavily_search via REST API', async () => {
      await app.rest.sendRequest({
        method: 'POST',
        url: `/bodhi/v1/mcps/${mcpInstanceId}/tools/refresh`,
      });
      expect(await app.rest.getResponseStatus()).toBe(200);

      await app.rest.sendRequest({
        method: 'POST',
        url: `/bodhi/v1/mcps/${mcpInstanceId}/tools/${McpFixtures.TAVILY_EXPECTED_TOOL}/execute`,
        body: JSON.stringify({
          params: McpFixtures.TAVILY_SEARCH_PARAMS,
        }),
      });
      expect(await app.rest.getResponseStatus()).toBe(200);
      const executeData = await app.rest.getResponse();
      expect(executeData.error).toBeUndefined();
      expect(executeData.result).toBeDefined();
    });
  });
});
