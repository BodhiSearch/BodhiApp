import { OAuth2Fixtures } from '@/fixtures/oauth2Fixtures.mjs';
import { OAuthTestApp } from '@/pages/OAuthTestApp.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getPreConfiguredAppClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { OAuth2ApiHelper } from '@/utils/OAuth2ApiHelper.mjs';
import { expect, test } from '@/fixtures.mjs';
import { SHARED_SERVER_URL, SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';

test.describe('OAuth2 Token Exchange Integration Tests', () => {
  let authServerConfig;
  let testCredentials;
  let authClient;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    authClient = createAuthServerTestClient(authServerConfig);
  });

  test.describe('Complete OAuth2 Flow', () => {
    let apiHelper;
    let testData;

    test.beforeEach(async () => {
      // Use shared servers started by Playwright webServer
      apiHelper = new OAuth2ApiHelper(SHARED_SERVER_URL, authClient);
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
          requestedToolsets: null,
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

      await test.step('Verify logged in and extract access token', async () => {
        await app.expectLoggedIn();
        await app.dashboard.navigateTo();
        const accessToken = await app.dashboard.getAccessToken();
        expect(accessToken).toBeTruthy();
        expect(accessToken.length).toBeGreaterThan(100);

        // Test API access with OAuth token
        const userResponse = await apiHelper.testApiWithToken(accessToken);
        expect(userResponse.status).toBe(200);

        const userInfo = userResponse.data;
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
    let apiHelper;

    test.beforeEach(async () => {
      const errorConfig = OAuth2Fixtures.getErrorTestConfig(authServerConfig, 41135);
      serverManager = createServerManager(errorConfig);
      baseUrl = await serverManager.startServer();
      apiHelper = new OAuth2ApiHelper(baseUrl, authClient);
    });

    test.afterEach(async () => {
      if (serverManager) {
        await serverManager.stopServer();
      }
    });

    test('should handle token exchange errors gracefully', async () => {
      // Try to access API without any token - should return logged_out status
      const userInfoResponse = await apiHelper.testUnauthenticatedApi();

      // Should get 200 response with auth_status: 'logged_out' for unauthenticated users
      expect(userInfoResponse.status).toBe(200);
      expect(userInfoResponse.data.auth_status).toBe('logged_out');
    });
  });
});
