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
import { SHARED_SERVER_URL, SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';

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

    test('should complete OAuth2 Token Exchange flow with dynamic audience', async ({ page }) => {
      const appClient = getPreConfiguredAppClient();
      const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;

      const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

      await test.step('Navigate to test app', async () => {
        await app.navigate();
      });

      await test.step('Configure OAuth form', async () => {
        await app.config.configureOAuthForm({
          bodhiServerUrl: SHARED_SERVER_URL,
          authServerUrl: authServerConfig.authUrl,
          realm: authServerConfig.authRealm,
          clientId: appClient.clientId,
          redirectUri,
          scope: testData.scopes,
          requested: null,
        });
      });

      await test.step('Submit access request and wait for login ready', async () => {
        await app.config.submitAccessRequest();
        await app.config.waitForLoginReady();
      });

      await test.step('Login via Keycloak', async () => {
        await app.config.clickLogin();
        await app.oauth.waitForAuthServerRedirect(authServerConfig.authUrl);
        await app.oauth.handleLogin(testCredentials.username, testCredentials.password);
        await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
      });

      await test.step('Verify logged in and API access with OAuth token', async () => {
        await app.expectLoggedIn();
        await app.rest.navigateTo();

        // Test API access with OAuth token
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
