import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { McpFixtures } from '@/fixtures/mcpFixtures.mjs';
import { AccessRequestReviewPage } from '@/pages/AccessRequestReviewPage.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { AppTokensPage } from '@/pages/AppTokensPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { McpsPage } from '@/pages/McpsPage.mjs';
import { ModelsListPageV2 } from '@/pages/ModelsListPageV2.mjs';
import { OAuthTestApp } from '@/pages/OAuthTestApp.mjs';
import { registerApiModelViaUI } from '@/utils/api-model-helpers.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getPreConfiguredAppClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';
import { SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';

// End-to-end App Token grant lifecycle:
//   external app requests access → owner approves with a scoped grant (no models) →
//   the exchanged token reflects + enforces the grant (403) → owner revokes it.
test.describe('App Tokens - grants, enforcement & revoke', { tag: '@oauth' }, () => {
  let authServerConfig;
  let testCredentials;
  let testApiKey;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    // Surface missing E2E config loudly rather than silently skipping.
    if (!authServerConfig.authUrl || !testCredentials.username) {
      throw new Error('Missing INTEG_TEST_* env for App Token E2E (auth url / credentials).');
    }
    // Throws (no skip) when the OpenAI key is missing — the positive-grant test infers.
    testApiKey = ApiModelFixtures.getRequiredEnvVars().apiKey;
    createAuthServerTestClient(authServerConfig);
  });

  test('approve a no-model grant, enforce it on the exchanged token, then revoke', async ({
    page,
    sharedServerUrl,
  }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });

    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Owner logs in to Bodhi', async () => {
      const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
      await loginPage.performOAuthLogin();
    });

    await test.step('External app requests access with model + MCP grant controls', async () => {
      await app.navigate();
      await app.config.configureOAuthForm({
        bodhiServerUrl: sharedServerUrl,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'openid email profile roles',
        // Ask for the model + MCP grant controls (no by-url MCPs → no instance selection needed).
        requested: JSON.stringify({
          version: '1',
          models_list: true,
          models_access: true,
          mcps_list: true,
          mcps_access: true,
        }),
      });
      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);
    });

    await test.step('Owner approves: list models on, model access = Specific (none)', async () => {
      const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
      // Specific with nothing selected ⇒ a deterministic "no model access" grant.
      await reviewPage.approveWithGrants({ listModels: true, modelIds: [], listMcps: true });

      await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
      await app.accessCallback.waitForLoaded();
      await app.accessCallback.clickLogin();
      await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
    });

    await test.step('Exchanged token reflects the grant on /bodhi/v1/user', async () => {
      await app.expectLoggedIn();
      await app.rest.navigateTo();
      await app.rest.sendRequest({ method: 'GET', url: '/bodhi/v1/user' });

      expect(await app.rest.getResponseStatus()).toBe(200);
      const info = await app.rest.getResponse();
      expect(info.auth_status).toBe('logged_in');
      // Phase-4 reflection: models_list on, but no specific models granted.
      expect(info.access).toBeDefined();
      expect(info.access.models.type).toBe('specific');
      expect(info.access.models.list).toBe(true);
      expect(info.access.models.ids).toEqual([]);
    });

    await test.step('Grant is enforced: inference on any model is forbidden (403)', async () => {
      await app.rest.sendRequest({
        method: 'POST',
        url: '/v1/chat/completions',
        body: { model: 'gpt-4', messages: [{ role: 'user', content: 'hi' }] },
      });
      // No model granted ⇒ AccessPolicy rejects before routing.
      expect(await app.rest.getResponseStatus()).toBe(403);
    });

    // Drive the owner's management UI on a second page (shares the session
    // cookie) so the test-app page stays authenticated for the token-death check.
    const ownerPage = await page.context().newPage();
    const appTokensPage = new AppTokensPage(ownerPage, sharedServerUrl);

    let rowId;
    await test.step('Owner sees the app under App Tokens with a "No models" summary', async () => {
      await appTokensPage.navigateToAppTokens();
      rowId = await appTokensPage.findRowIdByClientId(appClient.clientId);
      expect(rowId).not.toBeNull();
      await expect(ownerPage.locator(appTokensPage.row(rowId))).toContainText('No models');
    });

    await test.step('Owner opens the rail and revokes access', async () => {
      await appTokensPage.openRail(rowId);
      await appTokensPage.revokeAccess();
      // Revoked grants drop out of the list (only Approved are shown).
      await expect(ownerPage.locator(appTokensPage.row(rowId))).toHaveCount(0);
      await ownerPage.close();
    });

    await test.step('Revoked immediately: the previously-working token is now rejected', async () => {
      // Revoke evicts the cached token-exchange result, so the very next call with
      // the same token is rejected (no 5-minute TTL wait). The test-app page is
      // still on its authenticated dashboard. /bodhi/v1/user is optional-auth, so
      // a rejected token falls back to logged_out.
      await app.rest.navigateTo();
      await app.rest.sendRequest({ method: 'GET', url: '/bodhi/v1/user' });
      expect(await app.rest.getResponseStatus()).toBe(200);
      const info = await app.rest.getResponse();
      expect(info.auth_status).toBe('logged_out');
    });
  });

  // Positive counterpart to the deny test: the owner grants ONE specific model and
  // ONE specific MCP (owner-extra grant), then the exchanged token infers the granted
  // model, reaches the granted MCP, and is denied the non-granted model + MCP.
  test('approve specific model + MCP grants; enforce inference, MCP connect, and listings', async ({
    page,
    sharedServerUrl,
  }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });

    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const modelsPage = new ModelsListPageV2(page, sharedServerUrl);
    const apiModelFormPage = new ApiModelFormPage(page, sharedServerUrl);
    const mcpsPage = new McpsPage(page, sharedServerUrl);

    const stamp = `${Date.now()}`;
    // Grant the OpenAI model (inferenceable in E2E) and the first MCP instance; the
    // second MCP is the restricted control.
    const grantedModelId = ApiModelFixtures.OPENAI_MODEL;
    let grantedMcpId;
    let restrictedMcpId;

    await test.step('Owner logs in, registers a model + two MCP instances', async () => {
      await loginPage.performOAuthLogin();
      await registerApiModelViaUI(modelsPage, apiModelFormPage, testApiKey);

      const serverData = McpFixtures.createEverythingServerData();
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      await mcpsPage.createMcpInstance(serverData.name, `Ev-A-${stamp}`, `ev-a-${stamp}`);
      await mcpsPage.createMcpInstance(serverData.name, `Ev-B-${stamp}`, `ev-b-${stamp}`);
      await mcpsPage.expectMcpsListPage();
      grantedMcpId = await mcpsPage.getMcpUuidByName(`Ev-A-${stamp}`);
      restrictedMcpId = await mcpsPage.getMcpUuidByName(`Ev-B-${stamp}`);
      expect(grantedMcpId).toBeTruthy();
      expect(restrictedMcpId).toBeTruthy();
    });

    await test.step('External app requests model + MCP grant controls', async () => {
      await app.navigate();
      await app.config.configureOAuthForm({
        bodhiServerUrl: sharedServerUrl,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'openid email profile roles',
        requested: JSON.stringify({
          version: '1',
          models_list: true,
          models_access: true,
          mcps_list: true,
          mcps_access: true,
        }),
      });
      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);
    });

    await test.step('Owner approves one specific model + one specific MCP (owner-extra grant)', async () => {
      const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
      // list-all OFF so non-granted resources are hidden (404), not just connect-denied.
      await reviewPage.approveWithGrants({
        modelIds: [grantedModelId],
        mcpIds: [grantedMcpId],
      });

      await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
      await app.accessCallback.waitForLoaded();
      await app.accessCallback.clickLogin();
      await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
    });

    await test.step('Exchanged token reflects the specific model grant on /bodhi/v1/user', async () => {
      await app.expectLoggedIn();
      await app.rest.navigateTo();
      await app.rest.sendRequest({ method: 'GET', url: '/bodhi/v1/user' });
      expect(await app.rest.getResponseStatus()).toBe(200);
      const info = await app.rest.getResponse();
      expect(info.auth_status).toBe('logged_in');
      expect(info.access.models.type).toBe('specific');
      expect(info.access.models.ids).toContain(grantedModelId);
    });

    await test.step('Inference: granted model allowed (200), non-granted model denied (403)', async () => {
      await app.rest.sendRequest({
        method: 'POST',
        url: '/v1/chat/completions',
        body: { model: grantedModelId, messages: [{ role: 'user', content: 'hi' }] },
      });
      expect(await app.rest.getResponseStatus()).toBe(200);

      await app.rest.sendRequest({
        method: 'POST',
        url: '/v1/chat/completions',
        body: { model: 'gpt-4', messages: [{ role: 'user', content: 'hi' }] },
      });
      expect(await app.rest.getResponseStatus()).toBe(403);
    });

    await test.step('MCP: granted instance reachable (200), restricted hidden (404)', async () => {
      await app.rest.sendRequest({ method: 'GET', url: `/bodhi/v1/apps/mcps/${grantedMcpId}` });
      expect(await app.rest.getResponseStatus()).toBe(200);

      await app.rest.sendRequest({ method: 'GET', url: `/bodhi/v1/apps/mcps/${restrictedMcpId}` });
      expect(await app.rest.getResponseStatus()).toBe(404);
    });

    await test.step('MCP listing reflects the grant: granted present, restricted hidden', async () => {
      await app.rest.sendRequest({ method: 'GET', url: '/bodhi/v1/apps/mcps' });
      expect(await app.rest.getResponseStatus()).toBe(200);
      const list = await app.rest.getResponse();
      const ids = (list.mcps ?? []).map((m) => m.id);
      expect(ids).toContain(grantedMcpId);
      expect(ids).not.toContain(restrictedMcpId);
    });
  });
});
