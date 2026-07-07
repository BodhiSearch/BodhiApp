import { AccessRequestReviewPage } from '@/pages/AccessRequestReviewPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { OAuth2Fixtures } from '@/fixtures/oauth2Fixtures.mjs';
import { OAuthTestApp } from '@/pages/OAuthTestApp.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getPreConfiguredAppClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { expect, test } from '@/fixtures.mjs';
import { SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';

test.describe('OAuth2 Token Exchange Integration Tests', { tag: '@oauth' }, () => {
  let authServerConfig;
  let testCredentials;
  let authClient;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    authClient = createAuthServerTestClient(authServerConfig);
  });

  test.describe('Complete OAuth2 Flow', () => {
    let testData;

    test.beforeEach(async () => {
      testData = OAuth2Fixtures.getOAuth2TestData();
    });

    test('should complete OAuth2 Token Exchange flow with dynamic audience', async ({
      page,
      sharedServerUrl,
    }) => {
      const appClient = getPreConfiguredAppClient();
      const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;

      const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

      await test.step('Login to Bodhi server', async () => {
        const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
        await loginPage.performOAuthLogin();
      });

      await test.step('Navigate to test app', async () => {
        await app.navigate();
      });

      await test.step('Configure OAuth form', async () => {
        await app.config.configureOAuthForm({
          bodhiServerUrl: sharedServerUrl,
          authServerUrl: authServerConfig.authUrl,
          realm: authServerConfig.authRealm,
          clientId: appClient.clientId,
          redirectUri,
          scope: testData.scopes,
          requested: JSON.stringify({ version: '1' }),
        });
      });

      await test.step('Submit access request and approve via review page', async () => {
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);

        const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
        await reviewPage.approve();

        // KC session already exists from performOAuthLogin, so Keycloak auto-redirects
        await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
      });

      await test.step('Verify logged in and API access with OAuth token', async () => {
        await app.expectLoggedIn();
        await app.rest.navigateTo();

        await app.rest.sendRequest({
          method: 'GET',
          url: '/bodhi/v1/user',
        });

        expect(await app.rest.getResponseStatus()).toBe(200);
        const userInfo = await app.rest.getResponse();

        expect(userInfo).toBeDefined();
        expect(userInfo.auth_status).toBe('logged_in');
        expect(userInfo.username).toBe('user@email.com');
        expect(userInfo.role).toBe('scope_user_user');
      });
    });
  });

  test.describe('Role Downgrade Flow', () => {
    let testData;

    test.beforeEach(async () => {
      testData = OAuth2Fixtures.getOAuth2TestData();
    });

    test('should downgrade role from power_user to user on review approval', async ({
      page,
      sharedServerUrl,
    }) => {
      const appClient = getPreConfiguredAppClient();
      const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;

      const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

      await test.step('Login to Bodhi server', async () => {
        const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
        await loginPage.performOAuthLogin();
      });

      await test.step('Navigate to test app', async () => {
        await app.navigate();
      });

      await test.step('Configure OAuth form with power_user role', async () => {
        await app.config.configureOAuthForm({
          bodhiServerUrl: sharedServerUrl,
          authServerUrl: authServerConfig.authUrl,
          realm: authServerConfig.authRealm,
          clientId: appClient.clientId,
          redirectUri,
          scope: testData.scopes,
          requestedRole: 'scope_user_power_user',
          requested: JSON.stringify({ version: '1' }),
        });
      });

      await test.step('Submit access request and downgrade role on review page', async () => {
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);

        const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
        await reviewPage.approveWithRole('scope_user_user');

        // KC session already exists from performOAuthLogin, so Keycloak auto-redirects
        await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
      });

      await test.step('Verify logged in and token role is downgraded to user', async () => {
        await app.expectLoggedIn();
        await app.rest.navigateTo();

        await app.rest.sendRequest({
          method: 'GET',
          url: '/bodhi/v1/user',
        });

        expect(await app.rest.getResponseStatus()).toBe(200);
        const userInfo = await app.rest.getResponse();

        expect(userInfo).toBeDefined();
        expect(userInfo.auth_status).toBe('logged_in');
        expect(userInfo.username).toBe('user@email.com');
        expect(userInfo.role).toBe('scope_user_user');
      });
    });
  });

  test.describe('Exchange / Upgrade Flow', () => {
    let testData;

    test.beforeEach(async () => {
      testData = OAuth2Fixtures.getOAuth2TestData();
    });

    test('exchange pre-populates the review from the source grant and elevates the token', async ({
      page,
      sharedServerUrl,
    }) => {
      const appClient = getPreConfiguredAppClient();
      const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
      const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);
      // The app asks for model + MCP access + both listings; the owner grants concretely.
      const requested = JSON.stringify({
        version: '1',
        models_list: true,
        models_access: true,
        mcps_list: true,
        mcps_access: true,
      });

      await test.step('Login to Bodhi server', async () => {
        const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
        await loginPage.performOAuthLogin();
      });

      await test.step('Grant an initial token (source grant): all models + all MCPs, role user', async () => {
        await app.navigate();
        await app.config.configureOAuthForm({
          bodhiServerUrl: sharedServerUrl,
          authServerUrl: authServerConfig.authUrl,
          realm: authServerConfig.authRealm,
          clientId: appClient.clientId,
          redirectUri,
          scope: testData.scopes,
          requested,
        });
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);
        const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
        await reviewPage.approveWithGrants({
          listModels: true,
          allModels: true,
          listMcps: true,
          allMcps: true,
        });
        await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
      });

      await test.step('Source token reflects the granted access (role user, all models/MCPs)', async () => {
        await app.rest.navigateTo();
        await app.rest.sendRequest({ method: 'GET', url: '/bodhi/v1/user' });
        expect(await app.rest.getResponseStatus()).toBe(200);
        const info = await app.rest.getResponse();
        expect(info.role).toBe('scope_user_user');
        expect(info.access.models.type).toBe('all');
        expect(info.access.mcps.type).toBe('all');
      });

      await test.step('Submit an exchange request (elevate to power_user) with the current token', async () => {
        await app.rest.sendRequest({
          method: 'POST',
          url: '/bodhi/v1/apps/request-access',
          useAuth: true,
          body: {
            exchange: true,
            app_client_id: appClient.clientId,
            requested_role: 'scope_user_power_user',
            requested: JSON.parse(requested),
          },
        });
        expect(await app.rest.getResponseStatus()).toBe(201);
      });

      const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
      await test.step('Review is pre-populated from the source grant', async () => {
        await app.rest.clickReviewLink();
        await reviewPage.waitForReviewPage();
        // Listings held by the source grant load pre-checked.
        expect(await reviewPage.isListModelsChecked()).toBe(true);
        expect(await reviewPage.isListMcpsChecked()).toBe(true);
      });

      await test.step('Approve the upgrade — role defaults to the elevated power_user', async () => {
        // Grants are already pre-populated (all models/MCPs, listings on); role defaults to
        // the requested power_user. Approve commits the remaining set.
        await reviewPage.clickApprove();
        await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
      });

      await test.step('New token reflects the elevated grant (role power_user)', async () => {
        await app.rest.navigateTo();
        await app.rest.sendRequest({ method: 'GET', url: '/bodhi/v1/user' });
        expect(await app.rest.getResponseStatus()).toBe(200);
        const info = await app.rest.getResponse();
        expect(info.auth_status).toBe('logged_in');
        expect(info.role).toBe('scope_user_power_user');
        expect(info.access.models.type).toBe('all');
        expect(info.access.mcps.type).toBe('all');
      });
    });
  });

  test.describe('Error Handling', () => {
    let serverManager;
    let baseUrl;

    test.beforeEach(async () => {
      const errorConfig = OAuth2Fixtures.getErrorTestConfig(authServerConfig, 31135);
      serverManager = createServerManager(errorConfig);
      baseUrl = await serverManager.startServer();
    });

    test.afterEach(async () => {
      if (serverManager) {
        await serverManager.stopServer();
      }
    });

    test('should handle token exchange errors gracefully', async () => {
      // Try to access API without any token - should return logged_out status
      const response = await fetch(`${baseUrl}/bodhi/v1/user`, {
        headers: { 'Content-Type': 'application/json' },
      });

      // Should get 200 response with auth_status: 'logged_out' for unauthenticated users
      expect(response.status).toBe(200);
      const data = await response.json();
      expect(data.auth_status).toBe('logged_out');
    });
  });
});
