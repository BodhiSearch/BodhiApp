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

/**
 * MCP Authentication Restrictions E2E Tests
 *
 * Tests the OAuth access request flow for MCP servers:
 * 1. WITH MCP access request + WITH scope -> can list and execute MCP tools
 * 2. WITHOUT MCP access request -> MCP list returns empty for OAuth token
 * 3. Approved MCP -> 200, restricted MCP -> 401 for all endpoints
 */

const MCP_URL = McpFixtures.MCP_URL;

test.describe('OAuth Token + MCP Access Request Flow', { tag: ['@oauth', '@mcps'] }, () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test('App WITH MCP scope + OAuth WITH scope can list MCPs and execute tools', async ({
    page,
  }) => {
    let mcpInstanceId;

    await test.step('Phase 1: Session login and create MCP server + instance', async () => {
      const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
      await loginPage.performOAuthLogin();

      const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
      const serverData = McpFixtures.createServerData();
      const instanceData = McpFixtures.createLifecycleData();

      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      await mcpsPage.createMcpInstance(serverData.name, instanceData.name, instanceData.slug);

      // Navigate back to list to get the instance UUID
      await mcpsPage.navigateToMcpsList();
      mcpInstanceId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpInstanceId).toBeTruthy();
    });

    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Phase 2: Navigate and configure OAuth form with MCP request', async () => {
      await app.navigate();

      await app.config.configureOAuthForm({
        bodhiServerUrl: SHARED_SERVER_URL,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'openid profile email',
        requested: JSON.stringify({ mcp_servers: [{ url: MCP_URL }] }),
      });
    });

    await test.step('Phase 3: Submit access request and approve with MCP', async () => {
      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(SHARED_SERVER_URL);

      const reviewPage = new AccessRequestReviewPage(page, SHARED_SERVER_URL);
      await reviewPage.approveWithMcps([{ url: MCP_URL, instanceId: mcpInstanceId }]);

      await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
      await app.accessCallback.waitForLoaded();
      await app.accessCallback.clickLogin();
      await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
    });

    await test.step('Phase 4: Verify MCP access via REST API', async () => {
      await app.rest.navigateTo();

      // GET /mcps with OAuth token should return the approved MCP instance
      await app.rest.sendRequest({
        method: 'GET',
        url: '/bodhi/v1/mcps',
      });

      expect(await app.rest.getResponseStatus()).toBe(200);
      const listData = await app.rest.getResponse();
      expect(listData.mcps).toBeDefined();
      expect(Array.isArray(listData.mcps)).toBe(true);

      const approvedMcp = listData.mcps.find((m) => m.id === mcpInstanceId);
      expect(approvedMcp).toBeTruthy();

      // GET /mcps/{id} should return the MCP details
      await app.rest.sendRequest({
        method: 'GET',
        url: `/bodhi/v1/mcps/${mcpInstanceId}`,
      });

      expect(await app.rest.getResponseStatus()).toBe(200);
      const mcpData = await app.rest.getResponse();
      expect(mcpData.id).toBe(mcpInstanceId);

      // POST /mcps/{id}/tools/refresh to refresh tools
      await app.rest.sendRequest({
        method: 'POST',
        url: `/bodhi/v1/mcps/${mcpInstanceId}/tools/refresh`,
      });
      expect(await app.rest.getResponseStatus()).toBe(200);

      // Execute a tool on the MCP instance
      await app.rest.sendRequest({
        method: 'POST',
        url: `/bodhi/v1/mcps/${mcpInstanceId}/tools/${McpFixtures.EXPECTED_TOOL}/execute`,
        body: JSON.stringify({
          params: McpFixtures.PLAYGROUND_PARAMS,
        }),
      });

      expect(await app.rest.getResponseStatus()).toBe(200);
      const executeData = await app.rest.getResponse();
      expect(executeData).toBeDefined();
    });
  });

  test('App WITHOUT MCP scope + OAuth returns empty MCP list', async ({ page }) => {
    await test.step('Phase 1: Session login and create MCP server + instance', async () => {
      const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
      await loginPage.performOAuthLogin();

      const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
      const serverData = McpFixtures.createServerData();
      const instanceData = McpFixtures.createLifecycleData();

      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      await mcpsPage.createMcpInstance(serverData.name, instanceData.name, instanceData.slug);
    });

    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Phase 2: Configure OAuth without MCP request', async () => {
      await app.navigate();

      await app.config.configureOAuthForm({
        bodhiServerUrl: SHARED_SERVER_URL,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'openid profile email',
        requested: null,
      });
    });

    await test.step('Phase 3: Submit access request, approve, and login', async () => {
      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(SHARED_SERVER_URL);

      const reviewPage = new AccessRequestReviewPage(page, SHARED_SERVER_URL);
      await reviewPage.approve();

      await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
      await app.accessCallback.waitForLoaded();
      await app.accessCallback.clickLogin();

      await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
    });

    await test.step('Phase 4: Verify empty MCP list', async () => {
      await app.rest.navigateTo();

      await app.rest.sendRequest({
        method: 'GET',
        url: '/bodhi/v1/mcps',
      });

      expect(await app.rest.getResponseStatus()).toBe(200);
      const data = await app.rest.getResponse();
      expect(data.mcps).toBeDefined();
      expect(Array.isArray(data.mcps)).toBe(true);
      expect(data.mcps.length).toBe(0);
    });
  });

  test('App with MCP scope can access approved MCP but gets 401 on restricted MCP', async ({
    page,
  }) => {
    let approvedInstanceId;
    let restrictedInstanceId;

    await test.step('Phase 1: Session login and create MCP server + two instances', async () => {
      const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
      await loginPage.performOAuthLogin();

      const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
      const serverData = McpFixtures.createServerData();

      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);

      const ts = String(Date.now()).slice(-8);
      const approvedName = `dw-ok-${ts}`;
      const restrictedName = `dw-no-${ts}`;

      await mcpsPage.createMcpInstanceWithAllTools(serverData.name, approvedName, approvedName);

      await mcpsPage.createMcpInstanceWithAllTools(serverData.name, restrictedName, restrictedName);

      approvedInstanceId = await mcpsPage.getMcpUuidByName(approvedName);
      restrictedInstanceId = await mcpsPage.getMcpUuidByName(restrictedName);
      expect(approvedInstanceId).toBeTruthy();
      expect(restrictedInstanceId).toBeTruthy();
    });

    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Phase 2: Configure OAuth form with MCP request', async () => {
      await app.navigate();

      await app.config.configureOAuthForm({
        bodhiServerUrl: SHARED_SERVER_URL,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'openid profile email',
        requested: JSON.stringify({ mcp_servers: [{ url: MCP_URL }] }),
      });
    });

    await test.step('Phase 3: Submit access request and approve only deepwiki', async () => {
      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(SHARED_SERVER_URL);

      const reviewPage = new AccessRequestReviewPage(page, SHARED_SERVER_URL);
      await reviewPage.approveWithMcps([{ url: MCP_URL, instanceId: approvedInstanceId }]);

      await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
      await app.accessCallback.waitForLoaded();
      await app.accessCallback.clickLogin();
      await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
    });

    await test.step('Phase 4: Verify approved MCP access (200)', async () => {
      await app.rest.navigateTo();

      await app.rest.sendRequest({
        method: 'GET',
        url: `/bodhi/v1/mcps/${approvedInstanceId}`,
      });
      expect(await app.rest.getResponseStatus()).toBe(200);
      const mcpData = await app.rest.getResponse();
      expect(mcpData.id).toBe(approvedInstanceId);

      await app.rest.sendRequest({
        method: 'POST',
        url: `/bodhi/v1/mcps/${approvedInstanceId}/tools/refresh`,
      });
      expect(await app.rest.getResponseStatus()).toBe(200);

      await app.rest.sendRequest({
        method: 'POST',
        url: `/bodhi/v1/mcps/${approvedInstanceId}/tools/${McpFixtures.EXPECTED_TOOL}/execute`,
        body: JSON.stringify({ params: McpFixtures.PLAYGROUND_PARAMS }),
      });
      expect(await app.rest.getResponseStatus()).toBe(200);
      const executeData = await app.rest.getResponse();
      expect(executeData).toBeDefined();
    });

    await test.step('Phase 5: Verify restricted MCP denied (403)', async () => {
      await app.rest.sendRequest({
        method: 'GET',
        url: `/bodhi/v1/mcps/${restrictedInstanceId}`,
      });
      expect(await app.rest.getResponseStatus()).toBe(403);
      const getError = await app.rest.getResponse();
      expect(getError.error.code).toBe('access_request_auth_error-entity_not_approved');

      await app.rest.sendRequest({
        method: 'POST',
        url: `/bodhi/v1/mcps/${restrictedInstanceId}/tools/refresh`,
      });
      expect(await app.rest.getResponseStatus()).toBe(403);
      const refreshError = await app.rest.getResponse();
      expect(refreshError.error.code).toBe('access_request_auth_error-entity_not_approved');

      await app.rest.sendRequest({
        method: 'POST',
        url: `/bodhi/v1/mcps/${restrictedInstanceId}/tools/${McpFixtures.EXPECTED_TOOL}/execute`,
        body: JSON.stringify({ params: McpFixtures.PLAYGROUND_PARAMS }),
      });
      expect(await app.rest.getResponseStatus()).toBe(403);
      const executeError = await app.rest.getResponse();
      expect(executeError.error.code).toBe('access_request_auth_error-entity_not_approved');
    });
  });
});
