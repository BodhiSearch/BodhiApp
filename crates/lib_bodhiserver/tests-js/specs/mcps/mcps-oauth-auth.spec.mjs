import { McpFixtures } from '@/fixtures/mcpFixtures.mjs';
import { AppsAuthPage } from '@/pages/AppsAuthPage.mjs';
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

test.describe('MCP OAuth Authentication', { tag: ['@mcps', '@auth', '@oauth'] }, () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test('UI-driven OAuth flow: select pre-created config, authorize, create MCP, verify in playground', async ({
    page,
    sharedServerUrl,
  }) => {
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, sharedServerUrl);
    const serverData = McpFixtures.createOAuthServerData();
    const instanceData = McpFixtures.createOAuthInstanceData();
    let serverId;
    let oauthConfigId;
    let mcpInstanceId;

    await test.step('Login and create MCP server pointing to test OAuth server', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
    });

    await test.step('Create OAuth config via API', async () => {
      serverId = await mcpsPage.getServerUuidByName(serverData.name);
      expect(serverId).toBeTruthy();

      const oauthConfig = await mcpsPage.createOAuthConfigViaApi(
        serverId,
        McpFixtures.createOAuthConfigData()
      );
      expect(oauthConfig.id).toBeTruthy();
      oauthConfigId = oauthConfig.id;
    });

    await test.step('Navigate to new MCP, select OAuth config from dropdown, and connect', async () => {
      await mcpsPage.navigateToMcpsList();
      await mcpsPage.expectMcpsListPage();
      await mcpsPage.clickNewMcp();
      await mcpsPage.expectNewMcpPage();

      await mcpsPage.selectServerFromCombobox(serverData.name);
      await mcpsPage.selectAuthConfigById(oauthConfigId);
      await mcpsPage.clickOAuthConnect();

      await page.waitForURL(/\/authorize/);
      await page.click('[data-testid="approve-btn"]');
    });

    await test.step('Callback exchanges token and redirects back with connected state', async () => {
      await page.waitForURL(/\/ui\/mcps\/new/);
      await mcpsPage.expectOAuthConnected();
    });

    await test.step('Fill instance details and create MCP', async () => {
      await mcpsPage.fillName(instanceData.name);
      await mcpsPage.fillSlug(instanceData.slug);
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

    await test.step('Create second MCP with same OAuth config (reuse existing)', async () => {
      await mcpsPage.clickPlaygroundBack();
      await mcpsPage.expectMcpsListPage();
      await mcpsPage.clickNewMcp();
      await mcpsPage.expectNewMcpPage();

      await mcpsPage.selectServerFromCombobox(serverData.name);
      await mcpsPage.selectAuthConfigById(oauthConfigId);
      await mcpsPage.clickOAuthConnect();

      await page.waitForURL(/\/authorize/);
      await page.click('[data-testid="approve-btn"]');

      await page.waitForURL(/\/ui\/mcps\/new/);
      await mcpsPage.expectOAuthConnected();

      const shortTs = String(Date.now()).slice(-6);
      await mcpsPage.fillName(`${instanceData.name}-existing`);
      await mcpsPage.fillSlug(`oauth-ex-${shortTs}`);
      await mcpsPage.clickCreate();
      await mcpsPage.expectMcpsListPage();
    });
  });

  test('OAuth access request: 3rd party app accesses OAuth MCP via REST', async ({
    page,
    sharedServerUrl,
  }) => {
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, sharedServerUrl);
    const serverData = McpFixtures.createOAuthServerData();
    const instanceData = McpFixtures.createOAuthInstanceData();
    let serverId;
    let oauthConfigId;
    let mcpInstanceId;

    await test.step('Phase 1: Login, create OAuth MCP server and instance via UI', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);

      serverId = await mcpsPage.getServerUuidByName(serverData.name);
      const oauthConfig = await mcpsPage.createOAuthConfigViaApi(
        serverId,
        McpFixtures.createOAuthConfigData()
      );
      oauthConfigId = oauthConfig.id;

      await mcpsPage.createMcpInstanceWithOAuth({
        serverName: serverData.name,
        name: instanceData.name,
        slug: instanceData.slug,
        authConfigId: oauthConfigId,
      });
      await mcpsPage.expectMcpsListPage();
      mcpInstanceId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpInstanceId).toBeTruthy();
    });

    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Phase 2: Configure external app for the consent flow', async () => {
      await app.navigate();
      await app.config.configureOAuthForm({
        bodhiServerUrl: sharedServerUrl,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'scope_user_user',
      });
    });

    await test.step('Phase 3: Start authorize and approve with the OAuth MCP granted', async () => {
      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);

      // Single-step: approving redirects the browser straight to Keycloak (SSO-silent, since the
      // user is already logged in), which returns to the app's /callback with the code.
      const consentPage = new AppsAuthPage(page, sharedServerUrl);
      await consentPage.approveWithMcps([mcpInstanceId]);

      await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
    });

    await test.step('Phase 4: Verify OAuth MCP access via REST API', async () => {
      await app.rest.navigateTo();

      await app.rest.sendRequest({
        method: 'GET',
        url: `/bodhi/v1/apps/mcps/${mcpInstanceId}`,
      });
      expect(await app.rest.getResponseStatus()).toBe(200);
      const mcpData = await app.rest.getResponse();
      expect(mcpData.id).toBe(mcpInstanceId);
      expect(mcpData.auth_type).toBe('oauth');
    });
  });

  test('Edit OAuth MCP: disconnect and update without reconnecting', async ({
    page,
    sharedServerUrl,
  }) => {
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, sharedServerUrl);
    const serverData = McpFixtures.createOAuthServerData();
    const instanceData = McpFixtures.createOAuthInstanceData();
    let serverId;
    let oauthConfigId;
    let mcpInstanceId;

    await test.step('Login and create OAuth MCP server and instance', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);

      serverId = await mcpsPage.getServerUuidByName(serverData.name);
      const oauthConfig = await mcpsPage.createOAuthConfigViaApi(
        serverId,
        McpFixtures.createOAuthConfigData()
      );
      oauthConfigId = oauthConfig.id;

      await mcpsPage.createMcpInstanceWithOAuth({
        serverName: serverData.name,
        name: instanceData.name,
        slug: instanceData.slug,
        authConfigId: oauthConfigId,
      });
      await mcpsPage.expectMcpsListPage();
      mcpInstanceId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpInstanceId).toBeTruthy();
    });

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
      const row = await mcpsPage.getMcpRowByName(instanceData.name);
      await expect(row).toBeVisible();
    });
  });

  test('OAuth denied access: 3rd party gets error state when access request denied', async ({
    page,
    sharedServerUrl,
  }) => {
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, sharedServerUrl);
    const serverData = McpFixtures.createOAuthServerData();
    const instanceData = McpFixtures.createOAuthInstanceData();
    let serverId;
    let oauthConfigId;
    let mcpInstanceId;

    await test.step('Phase 1: Login, create OAuth MCP server and instance', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);

      serverId = await mcpsPage.getServerUuidByName(serverData.name);
      const oauthConfig = await mcpsPage.createOAuthConfigViaApi(
        serverId,
        McpFixtures.createOAuthConfigData()
      );
      oauthConfigId = oauthConfig.id;

      await mcpsPage.createMcpInstanceWithOAuth({
        serverName: serverData.name,
        name: instanceData.name,
        slug: instanceData.slug,
        authConfigId: oauthConfigId,
      });
      await mcpsPage.expectMcpsListPage();
      mcpInstanceId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpInstanceId).toBeTruthy();
    });

    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Phase 2: Configure external app and start authorize', async () => {
      await app.navigate();
      await app.config.configureOAuthForm({
        bodhiServerUrl: sharedServerUrl,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'scope_user_user',
      });
      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);
    });

    let originalState;
    await test.step('Phase 3: Deny the request on the consent page', async () => {
      const consentPage = new AppsAuthPage(page, sharedServerUrl);
      await consentPage.waitForConsentPage();
      // The authorize params ride on the consent page URL; capture the app's state.
      originalState = new URL(page.url()).searchParams.get('state');
      expect(originalState).toBeTruthy();
      await consentPage.clickDeny();
    });

    await test.step('Phase 4: App callback receives error_source=bodhi with the original state', async () => {
      const { error, errorSource, state } = await app.oauth.expectOAuthError('access_denied');
      expect(error).toBe('access_denied');
      expect(errorSource).toBe('bodhi');
      expect(state).toBe(originalState);
    });
  });

  test('OAuth access request (popup flow): approve in popup, opener receives token', async ({
    page,
    sharedServerUrl,
  }) => {
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, sharedServerUrl);
    const serverData = McpFixtures.createOAuthServerData();
    const instanceData = McpFixtures.createOAuthInstanceData();
    let mcpInstanceId;

    await test.step('Phase 1: Login, create OAuth MCP server and instance', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      const serverId = await mcpsPage.getServerUuidByName(serverData.name);
      const oauthConfig = await mcpsPage.createOAuthConfigViaApi(
        serverId,
        McpFixtures.createOAuthConfigData()
      );
      await mcpsPage.createMcpInstanceWithOAuth({
        serverName: serverData.name,
        name: instanceData.name,
        slug: instanceData.slug,
        authConfigId: oauthConfig.id,
      });
      await mcpsPage.expectMcpsListPage();
      mcpInstanceId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpInstanceId).toBeTruthy();
    });

    const appClient = getPreConfiguredAppClient();
    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Phase 2: Configure external app with popup flow', async () => {
      await app.navigate();
      await app.config.configureOAuthForm({
        bodhiServerUrl: sharedServerUrl,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri: `${SHARED_STATIC_SERVER_URL}/callback`,
        scope: 'scope_user_user',
        flowType: 'popup',
      });
    });

    await test.step('Phase 3: Approve in the popup; opener completes token exchange', async () => {
      const popupPromise = page.waitForEvent('popup');
      await app.config.submitAccessRequest();
      const popup = await popupPromise;
      await popup.waitForLoadState('domcontentloaded');

      // Consent + approve happen inside the popup; it then flows through Keycloak and posts the
      // authorization code back to the opener, which owns the PKCE verifier and exchanges it.
      const consentPage = new AppsAuthPage(popup, sharedServerUrl);
      await consentPage.approveWithMcps([mcpInstanceId]);

      await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
    });

    await test.step('Phase 4: Verify OAuth MCP access via REST API', async () => {
      await app.rest.navigateTo();
      await app.rest.sendRequest({ method: 'GET', url: `/bodhi/v1/apps/mcps/${mcpInstanceId}` });
      expect(await app.rest.getResponseStatus()).toBe(200);
    });
  });
});
