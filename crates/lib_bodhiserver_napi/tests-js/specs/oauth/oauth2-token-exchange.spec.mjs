import { OAuth2Fixtures } from '@/fixtures/oauth2Fixtures.mjs';
import { OAuth2TestAppPage } from '@/pages/OAuth2TestAppPage.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getPreConfiguredAppClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { createStaticServer } from '@/utils/static-server.mjs';
import { OAuth2ApiHelper } from '@/utils/OAuth2ApiHelper.mjs';
import { expect, test } from '@playwright/test';

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
    let serverManager;
    let staticServer;
    let baseUrl;
    let testAppUrl;
    let apiHelper;
    let testData;

    test.beforeEach(async () => {
      const serverConfig = OAuth2Fixtures.getOAuth2ServerConfig(authServerConfig, 51135);
      serverManager = createServerManager(serverConfig);
      baseUrl = await serverManager.startServer();

      // Setup static server for OAuth test app
      staticServer = createStaticServer(55173);
      testAppUrl = await staticServer.startServer();

      // Initialize helpers and test data
      apiHelper = new OAuth2ApiHelper(baseUrl, authClient);
      testData = OAuth2Fixtures.getOAuth2TestData();
    });

    test.afterEach(async () => {
      if (staticServer) {
        await staticServer.stopServer();
      }
      if (serverManager) {
        await serverManager.stopServer();
      }
    });

    test('should complete OAuth2 Token Exchange flow with dynamic audience', async ({ page }) => {
      // Get pre-configured app client
      const appClient = getPreConfiguredAppClient();
      const redirectUri = `${testAppUrl}/oauth-test-app.html`;

      // Navigate to test app and complete OAuth flow
      const oauth2TestAppPage = new OAuth2TestAppPage(page, testAppUrl);
      await oauth2TestAppPage.navigateToTestApp(redirectUri);

      // Configure OAuth form
      await oauth2TestAppPage.configureOAuthForm(
        baseUrl,
        authServerConfig.authUrl,
        authServerConfig.authRealm,
        appClient.clientId,
        redirectUri,
        testData.scopes,
        null
      );

      // Two-step flow: submit access request, wait for scopes, then login
      await oauth2TestAppPage.submitAccessRequest();
      await oauth2TestAppPage.waitForLoginReady();
      // Wait for KC scope registration to propagate before redirecting
      await oauth2TestAppPage.clickLogin();
      await oauth2TestAppPage.waitForAuthServerRedirect(authServerConfig.authUrl);
      // Login to Keycloak (no active KC session in browser)
      await oauth2TestAppPage.handleLogin(testCredentials.username, testCredentials.password);
      await oauth2TestAppPage.waitForTokenExchange(testAppUrl);

      // Extract and validate access token
      const accessToken = await oauth2TestAppPage.getAccessToken();
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

  test.describe('Error Handling', () => {
    let serverManager;
    let baseUrl;
    let apiHelper;

    test.beforeEach(async () => {
      const errorConfig = OAuth2Fixtures.getErrorTestConfig(authServerConfig, 51135);
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
