import { AccessRequestReviewPage } from '@/pages/AccessRequestReviewPage.mjs';
import { AppTokensPage } from '@/pages/AppTokensPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { OAuthTestApp } from '@/pages/OAuthTestApp.mjs';
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

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    // Surface missing E2E config loudly rather than silently skipping.
    if (!authServerConfig.authUrl || !testCredentials.username) {
      throw new Error('Missing INTEG_TEST_* env for App Token E2E (auth url / credentials).');
    }
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
      await reviewPage.approveWithGrants({ listModels: true, modelsSpecific: true, listMcps: true });

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
});
