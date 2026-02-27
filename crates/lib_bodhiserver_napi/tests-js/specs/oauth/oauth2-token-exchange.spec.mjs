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
          requested: null,
        });
      });

      await test.step('Submit access request and approve via review page', async () => {
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);

        const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
        await reviewPage.approve();

        await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
        await app.accessCallback.waitForLoaded();
        await app.accessCallback.clickLogin();
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
          requested: null,
        });
      });

      await test.step('Submit access request and downgrade role on review page', async () => {
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);

        const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
        await reviewPage.approveWithRole('scope_user_user');

        await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
        await app.accessCallback.waitForLoaded();
        await app.accessCallback.clickLogin();
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

  test.describe('Error Handling', () => {
    let serverManager;
    let baseUrl;

    test.beforeEach(async () => {
      const errorConfig = OAuth2Fixtures.getErrorTestConfig(authServerConfig, 41135);
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
