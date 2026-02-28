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
import { SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';

/**
 * Shared DCR setup helper: login, create MCP server, discover endpoints,
 * dynamically register a client, create OAuth config, and create MCP instance.
 *
 * Returns { serverId, dcrConfigId, mcpInstanceId }.
 */
async function setupDcrMcpInstance(page, sharedServerUrl, authServerConfig, testCredentials) {
  const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
  const mcpsPage = new McpsPage(page, sharedServerUrl);
  const serverData = McpFixtures.createDcrServerData();
  const instanceData = McpFixtures.createDcrInstanceData();

  await loginPage.performOAuthLogin('/ui/chat/');
  await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);

  const serverId = await mcpsPage.getServerUuidByName(serverData.name);

  const discovery = await mcpsPage.discoverMcpEndpointsViaApi(serverData.url);
  const redirectUri = `${sharedServerUrl}/ui/mcps/oauth/callback`;
  const dcrResult = await mcpsPage.dynamicRegisterViaApi({
    registrationEndpoint: discovery.registration_endpoint,
    redirectUri,
  });

  const oauthConfig = await mcpsPage.createOAuthConfigViaApi(serverId, {
    name: 'DCR Config',
    client_id: dcrResult.client_id,
    client_secret: dcrResult.client_secret || undefined,
    authorization_endpoint: discovery.authorization_endpoint,
    token_endpoint: discovery.token_endpoint,
    registration_endpoint: discovery.registration_endpoint,
    registration_type: 'dynamic_registration',
    token_endpoint_auth_method: dcrResult.token_endpoint_auth_method || undefined,
    client_id_issued_at: dcrResult.client_id_issued_at || undefined,
    registration_access_token: dcrResult.registration_access_token || undefined,
  });
  const dcrConfigId = oauthConfig.id;

  await mcpsPage.createMcpInstanceWithOAuth({
    serverName: serverData.name,
    name: instanceData.name,
    slug: instanceData.slug,
    authConfigId: dcrConfigId,
  });
  await mcpsPage.expectMcpsListPage();
  const mcpInstanceId = await mcpsPage.getMcpUuidByName(instanceData.name);
  expect(mcpInstanceId).toBeTruthy();

  return { serverId, dcrConfigId, mcpInstanceId, serverData, instanceData, mcpsPage };
}

test.describe(
  'MCP OAuth Dynamic Client Registration',
  { tag: ['@mcps', '@auth', '@oauth', '@dcr'] },
  () => {
    let authServerConfig;
    let testCredentials;

    test.beforeAll(async () => {
      authServerConfig = getAuthServerConfig();
      testCredentials = getTestCredentials();
    });

    test('UI-driven DCR flow: discover via API, register, select config, authorize, create MCP, verify in playground', async ({
      page,
      sharedServerUrl,
    }) => {
      const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
      const mcpsPage = new McpsPage(page, sharedServerUrl);
      const serverData = McpFixtures.createDcrServerData();
      const instanceData = McpFixtures.createDcrInstanceData();
      let serverId;
      let dcrConfigId;
      let mcpInstanceId;

      await test.step('Login and create MCP server pointing to DCR test server', async () => {
        await loginPage.performOAuthLogin('/ui/chat/');
        await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      });

      await test.step('Discover endpoints and dynamically register client via API', async () => {
        serverId = await mcpsPage.getServerUuidByName(serverData.name);
        expect(serverId).toBeTruthy();

        const discovery = await mcpsPage.discoverMcpEndpointsViaApi(serverData.url);
        expect(discovery.registration_endpoint).toBeTruthy();
        expect(discovery.authorization_endpoint).toBeTruthy();
        expect(discovery.token_endpoint).toBeTruthy();

        const redirectUri = `${sharedServerUrl}/ui/mcps/oauth/callback`;
        const dcrResult = await mcpsPage.dynamicRegisterViaApi({
          registrationEndpoint: discovery.registration_endpoint,
          redirectUri,
          scopes: discovery.scopes_supported?.join(' ') || undefined,
        });
        expect(dcrResult.client_id).toBeTruthy();
        expect(dcrResult.client_id).toMatch(/^dyn-/);

        const oauthConfig = await mcpsPage.createOAuthConfigViaApi(serverId, {
          name: 'DCR Config',
          client_id: dcrResult.client_id,
          client_secret: dcrResult.client_secret || undefined,
          authorization_endpoint: discovery.authorization_endpoint,
          token_endpoint: discovery.token_endpoint,
          registration_endpoint: discovery.registration_endpoint,
          registration_type: 'dynamic_registration',
          scopes: discovery.scopes_supported?.join(' ') || undefined,
          token_endpoint_auth_method: dcrResult.token_endpoint_auth_method || undefined,
          client_id_issued_at: dcrResult.client_id_issued_at || undefined,
          registration_access_token: dcrResult.registration_access_token || undefined,
        });
        expect(oauthConfig.id).toBeTruthy();
        dcrConfigId = oauthConfig.id;
      });

      await test.step('Select DCR config from dropdown and authorize', async () => {
        await mcpsPage.navigateToMcpsList();
        await mcpsPage.expectMcpsListPage();
        await mcpsPage.clickNewMcp();
        await mcpsPage.expectNewMcpPage();

        await mcpsPage.selectServerFromCombobox(serverData.name);
        await mcpsPage.selectAuthConfigById(dcrConfigId);
        await mcpsPage.clickOAuthConnect();

        await page.waitForURL(/\/authorize/);
        await page.click('[data-testid="approve-btn"]');
      });

      await test.step('Callback exchanges token and returns with connected state', async () => {
        await page.waitForURL(/\/ui\/mcps\/new/);
        await mcpsPage.expectOAuthConnected();
      });

      await test.step('Fill instance details, discover tools, and create MCP', async () => {
        await mcpsPage.fillName(instanceData.name);
        await mcpsPage.fillSlug(instanceData.slug);
        await mcpsPage.clickFetchTools();
        await mcpsPage.expectToolsList();
        await mcpsPage.expectToolItem(McpFixtures.OAUTH_DCR_EXPECTED_TOOL);
        await mcpsPage.clickCreate();
        await mcpsPage.expectMcpsListPage();
        mcpInstanceId = await mcpsPage.getMcpUuidByName(instanceData.name);
        expect(mcpInstanceId).toBeTruthy();
      });

      await test.step('Execute echo tool in playground and verify success', async () => {
        await mcpsPage.clickPlaygroundById(mcpInstanceId);
        await mcpsPage.expectPlaygroundPage();
        await mcpsPage.selectPlaygroundTool(McpFixtures.OAUTH_DCR_EXPECTED_TOOL);
        await mcpsPage.expectPlaygroundToolSelected(McpFixtures.OAUTH_DCR_EXPECTED_TOOL);
        await mcpsPage.fillPlaygroundParam('text', 'Hello from DCR E2E');
        await mcpsPage.clickPlaygroundExecute();
        await mcpsPage.expectPlaygroundResultSuccess();
      });
    });

    test('Edit DCR MCP: disconnect and update without reconnecting', async ({
      page,
      sharedServerUrl,
    }) => {
      let mcpInstanceId;
      let instanceName;

      await test.step('Login and create DCR MCP server and instance', async () => {
        const setup = await setupDcrMcpInstance(
          page,
          sharedServerUrl,
          authServerConfig,
          testCredentials
        );
        mcpInstanceId = setup.mcpInstanceId;
        instanceName = setup.instanceData.name;
      });

      const mcpsPage = new McpsPage(page, sharedServerUrl);

      await test.step('Navigate to edit page and verify connected card', async () => {
        await mcpsPage.clickEditById(mcpInstanceId);
        await mcpsPage.expectNewMcpPage();
        await mcpsPage.expectOAuthConnected();
      });

      await test.step('Disconnect - connected card disappears, dropdown available', async () => {
        await mcpsPage.clickDisconnect();
        await mcpsPage.expectOAuthDisconnected();
      });

      await test.step('Click Update to save without OAuth token', async () => {
        await mcpsPage.clickUpdate();
        await mcpsPage.expectMcpsListPage();
        const row = await mcpsPage.getMcpRowByName(instanceName);
        await expect(row).toBeVisible();
      });
    });

    test('DCR access request: 3rd party app executes tool on DCR MCP via REST', async ({
      page,
      sharedServerUrl,
    }) => {
      let mcpInstanceId;

      await test.step('Phase 1: Login, create DCR MCP server and instance via UI', async () => {
        const setup = await setupDcrMcpInstance(
          page,
          sharedServerUrl,
          authServerConfig,
          testCredentials
        );
        mcpInstanceId = setup.mcpInstanceId;
      });

      const appClient = getPreConfiguredAppClient();
      const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
      const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

      await test.step('Phase 2: Configure external app with DCR MCP request', async () => {
        await app.navigate();
        await app.config.configureOAuthForm({
          bodhiServerUrl: sharedServerUrl,
          authServerUrl: authServerConfig.authUrl,
          realm: authServerConfig.authRealm,
          clientId: appClient.clientId,
          redirectUri,
          scope: 'openid profile email',
          requested: JSON.stringify({ mcp_servers: [{ url: McpFixtures.OAUTH_DCR_MCP_URL }] }),
        });
      });

      await test.step('Phase 3: Submit access request and approve with DCR MCP', async () => {
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);

        const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
        await reviewPage.approveWithMcps([
          { url: McpFixtures.OAUTH_DCR_MCP_URL, instanceId: mcpInstanceId },
        ]);

        await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
        await app.accessCallback.waitForLoaded();
        await app.accessCallback.clickLogin();
        await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
      });

      await test.step('Phase 4: Verify DCR MCP access via REST API', async () => {
        await app.rest.navigateTo();

        await app.rest.sendRequest({
          method: 'GET',
          url: `/bodhi/v1/mcps/${mcpInstanceId}`,
        });
        expect(await app.rest.getResponseStatus()).toBe(200);
        const mcpData = await app.rest.getResponse();
        expect(mcpData.id).toBe(mcpInstanceId);
        expect(mcpData.auth_type).toBe('oauth');
      });

      await test.step('Phase 5: Execute echo tool via REST API as 3rd party', async () => {
        await app.rest.sendRequest({
          method: 'POST',
          url: `/bodhi/v1/mcps/${mcpInstanceId}/tools/${McpFixtures.OAUTH_DCR_EXPECTED_TOOL}/execute`,
          body: JSON.stringify({
            params: { text: 'Hello from 3rd party DCR' },
          }),
        });
        expect(await app.rest.getResponseStatus()).toBe(200);
        const result = await app.rest.getResponse();
        expect(result.error).toBeUndefined();
        expect(result.result).toBeDefined();
      });
    });
  }
);
