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

const OAUTH_PORT = process.env.TEST_MCP_OAUTH_PORT || '55174';
const OAUTH_SERVER_BASE = `http://localhost:${OAUTH_PORT}`;

test.describe('MCP OAuth Authentication', { tag: ['@mcps', '@auth', '@oauth'] }, () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test('UI-driven OAuth flow: new config, authorize, create MCP, verify in playground', async ({
    page,
  }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
    const serverData = McpFixtures.createOAuthServerData();
    const instanceData = McpFixtures.createOAuthInstanceData();
    let mcpInstanceId;

    await test.step('Login and create MCP server pointing to test OAuth server', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
    });

    await test.step('Navigate to new MCP form and configure OAuth with auto-detect', async () => {
      await mcpsPage.navigateToMcpsList();
      await mcpsPage.expectMcpsListPage();
      await mcpsPage.clickNewMcp();
      await mcpsPage.expectNewMcpPage();

      await mcpsPage.selectServerFromCombobox(serverData.name);
      await mcpsPage.selectAuthTypeOAuth();
      await mcpsPage.expectNewOAuthConfigForm();

      await mcpsPage.expectOAuthServerUrlValue(OAUTH_SERVER_BASE);
      await mcpsPage.fillOAuthClientId(McpFixtures.OAUTH_CLIENT_ID);
      await mcpsPage.fillOAuthClientSecret(McpFixtures.OAUTH_CLIENT_SECRET);
      await mcpsPage.clickAutoDetect();
      await mcpsPage.waitForAutoDetectComplete();

      await expect(page.locator(mcpsPage.selectors.oauthAuthorizationEndpoint)).toHaveValue(
        /\/authorize/
      );
      await expect(page.locator(mcpsPage.selectors.oauthTokenEndpoint)).toHaveValue(/\/token/);
    });

    await test.step('Click Authorize and approve on external OAuth server', async () => {
      await mcpsPage.clickAuthorize();
      await page.waitForURL(/\/authorize/);
      await page.click('[data-testid="approve-btn"]');
    });

    await test.step('Callback exchanges token and redirects back with connected state', async () => {
      await page.waitForURL(/\/ui\/mcps\/new/);
      await mcpsPage.expectOAuthConnected();
    });

    await test.step('Fill instance details, discover tools, and create MCP', async () => {
      await mcpsPage.fillName(instanceData.name);
      await mcpsPage.fillSlug(instanceData.slug);
      await mcpsPage.clickFetchTools();
      await mcpsPage.expectToolsList();
      await mcpsPage.expectToolItem(McpFixtures.OAUTH_EXPECTED_TOOL);
      await mcpsPage.clickCreate();
      await mcpsPage.expectMcpsListPage();
      mcpInstanceId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpInstanceId).toBeTruthy();
    });

    await test.step('Execute echo tool in playground and verify success', async () => {
      await mcpsPage.clickPlaygroundById(mcpInstanceId);
      await mcpsPage.expectPlaygroundPage();
      await mcpsPage.selectPlaygroundTool(McpFixtures.OAUTH_EXPECTED_TOOL);
      await mcpsPage.expectPlaygroundToolSelected(McpFixtures.OAUTH_EXPECTED_TOOL);
      await mcpsPage.fillPlaygroundParam('text', 'Hello from OAuth E2E');
      await mcpsPage.clickPlaygroundExecute();
      await mcpsPage.expectPlaygroundResultSuccess();
    });

    await test.step('Create second MCP with existing OAuth config via dropdown', async () => {
      await mcpsPage.clickPlaygroundBack();
      await mcpsPage.expectMcpsListPage();
      await mcpsPage.clickNewMcp();
      await mcpsPage.expectNewMcpPage();

      await mcpsPage.selectServerFromCombobox(serverData.name);
      await mcpsPage.selectAuthTypeOAuth();

      await mcpsPage.selectExistingOAuthConfig();
      await expect(page.locator(mcpsPage.selectors.oauthConfigSummary)).toBeVisible();
      await mcpsPage.clickAuthorizeExisting();

      await page.waitForURL(/\/authorize/);
      await page.click('[data-testid="approve-btn"]');

      await page.waitForURL(/\/ui\/mcps\/new/);
      await mcpsPage.expectOAuthConnected();

      const shortTs = String(Date.now()).slice(-6);
      await mcpsPage.fillName(`${instanceData.name}-existing`);
      await mcpsPage.fillSlug(`oauth-ex-${shortTs}`);
      await mcpsPage.clickFetchTools();
      await mcpsPage.expectToolsList();
      await mcpsPage.clickCreate();
      await mcpsPage.expectMcpsListPage();
    });
  });

  test('OAuth access request: 3rd party app executes tool on OAuth MCP via REST', async ({
    page,
  }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
    const serverData = McpFixtures.createOAuthServerData();
    const instanceData = McpFixtures.createOAuthInstanceData();
    let mcpInstanceId;

    await test.step('Phase 1: Login, create OAuth MCP server and instance via UI', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);

      await mcpsPage.createOAuthMcpInstance({
        serverName: serverData.name,
        mcpName: instanceData.name,
        mcpSlug: instanceData.slug,
        clientId: McpFixtures.OAUTH_CLIENT_ID,
        clientSecret: McpFixtures.OAUTH_CLIENT_SECRET,
      });
      await mcpsPage.expectMcpsListPage();
      mcpInstanceId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpInstanceId).toBeTruthy();
    });

    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Phase 2: Configure external app with OAuth MCP request', async () => {
      await app.navigate();
      await app.config.configureOAuthForm({
        bodhiServerUrl: SHARED_SERVER_URL,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'openid profile email scope_user_user',
        requested: JSON.stringify({ mcp_servers: [{ url: McpFixtures.OAUTH_MCP_URL }] }),
      });
    });

    await test.step('Phase 3: Submit access request and approve with OAuth MCP', async () => {
      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(SHARED_SERVER_URL);

      const reviewPage = new AccessRequestReviewPage(page, SHARED_SERVER_URL);
      await reviewPage.approveWithMcps([
        { url: McpFixtures.OAUTH_MCP_URL, instanceId: mcpInstanceId },
      ]);

      await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
      await app.accessCallback.waitForLoaded();
      await app.accessCallback.clickLogin();
      await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
    });

    await test.step('Phase 4: Verify OAuth MCP access via REST API', async () => {
      await app.rest.navigateTo();

      await app.rest.sendRequest({
        method: 'GET',
        url: `/bodhi/v1/mcps/${mcpInstanceId}`,
      });
      expect(await app.rest.getResponseStatus()).toBe(200);
      const mcpData = await app.rest.getResponse();
      expect(mcpData.id).toBe(mcpInstanceId);
      expect(mcpData.auth_type).toBe('oauth-pre-registered');
    });

    await test.step('Phase 5: Execute echo tool via REST API as 3rd party', async () => {
      await app.rest.sendRequest({
        method: 'POST',
        url: `/bodhi/v1/mcps/${mcpInstanceId}/tools/${McpFixtures.OAUTH_EXPECTED_TOOL}/execute`,
        body: JSON.stringify({
          params: { text: 'Hello from 3rd party' },
        }),
      });
      expect(await app.rest.getResponseStatus()).toBe(200);
      const result = await app.rest.getResponse();
      expect(result.error).toBeUndefined();
      expect(result.result).toBeDefined();
    });
  });

  test('OAuth denied access: 3rd party gets error state when access request denied', async ({
    page,
  }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
    const serverData = McpFixtures.createOAuthServerData();
    const instanceData = McpFixtures.createOAuthInstanceData();
    let mcpInstanceId;

    await test.step('Phase 1: Login, create OAuth MCP server and instance', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      await mcpsPage.createOAuthMcpInstance({
        serverName: serverData.name,
        mcpName: instanceData.name,
        mcpSlug: instanceData.slug,
        clientId: McpFixtures.OAUTH_CLIENT_ID,
        clientSecret: McpFixtures.OAUTH_CLIENT_SECRET,
      });
      await mcpsPage.expectMcpsListPage();
      mcpInstanceId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpInstanceId).toBeTruthy();
    });

    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Phase 2: Configure external app and submit access request', async () => {
      await app.navigate();
      await app.config.configureOAuthForm({
        bodhiServerUrl: SHARED_SERVER_URL,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'openid profile email scope_user_user',
        requested: JSON.stringify({ mcp_servers: [{ url: McpFixtures.OAUTH_MCP_URL }] }),
      });
      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(SHARED_SERVER_URL);
    });

    await test.step('Phase 3: Deny the access request on review page', async () => {
      const reviewPage = new AccessRequestReviewPage(page, SHARED_SERVER_URL);
      await reviewPage.waitForReviewPage();
      await reviewPage.clickDeny();
    });

    await test.step('Phase 4: Verify callback redirects with denied error state', async () => {
      await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
      await app.accessCallback.waitForLoaded();
      const state = await app.accessCallback.getState();
      expect(state).toBe('error');
    });
  });
});
