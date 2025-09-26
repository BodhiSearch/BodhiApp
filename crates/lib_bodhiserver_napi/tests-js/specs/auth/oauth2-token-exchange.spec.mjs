import { expect, test } from '@playwright/test';
import { randomPort } from '../../test-helpers.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '../../playwright/auth-server-client.mjs';
import { createServerManager } from '../../playwright/bodhi-app-server.mjs';
import { createStaticServer } from '../../playwright/static-server.mjs';
import { SetupWelcomePage } from '../../pages/SetupWelcomePage.mjs';
import { SetupResourceAdminPage } from '../../pages/SetupResourceAdminPage.mjs';
import { OAuth2TestAppPage } from '../../pages/OAuth2TestAppPage.mjs';
import { OAuth2Fixtures } from '../../fixtures/oauth2Fixtures.mjs';
import { OAuth2ApiHelper } from '../../helpers/OAuth2ApiHelper.mjs';

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
    let port;

    test.beforeEach(async () => {
      port = randomPort();
      const serverConfig = OAuth2Fixtures.getOAuth2ServerConfig(authServerConfig, port);
      serverManager = createServerManager(serverConfig);
      baseUrl = await serverManager.startServer();

      // Setup static server for OAuth test app
      const appPort = randomPort();
      staticServer = createStaticServer(appPort);
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
      // Step 1: Complete initial server setup
      const setupWelcomePage = new SetupWelcomePage(page, baseUrl);
      await setupWelcomePage.navigateToSetup();
      await setupWelcomePage.expectWelcomePage();
      await setupWelcomePage.completeInitialSetup(testData.serverName);

      // Step 2: Complete resource admin setup
      const setupResourceAdminPage = new SetupResourceAdminPage(
        page,
        baseUrl,
        authServerConfig,
        testCredentials
      );
      await setupResourceAdminPage.expectResourceAdminPage();
      await setupResourceAdminPage.performCompleteLogin();

      // Step 3: Get dev console token for client management
      const devConsoleToken = await apiHelper.getDevConsoleToken(
        testCredentials.username,
        testCredentials.password
      );

      // Step 4: Create app client with test app redirect URI
      const redirectUri = `${testAppUrl}/oauth-test-app.html`;
      const appClient = await apiHelper.createAppClient(
        devConsoleToken,
        port,
        testData.clientName,
        testData.clientDescription,
        [redirectUri]
      );

      // Step 5: Request audience access via Bodhi App API
      const requestAccessData = await apiHelper.requestAudienceAccess(appClient.clientId);
      const resourceScope = requestAccessData.scope;

      // Step 6: Navigate to test app and complete OAuth flow
      const oauth2TestAppPage = new OAuth2TestAppPage(page, testAppUrl);
      await oauth2TestAppPage.navigateToTestApp(redirectUri);

      // Configure OAuth form
      const fullScopes = `${testData.scopes} ${resourceScope}`;
      await oauth2TestAppPage.configureOAuthForm(
        authServerConfig.authUrl,
        authServerConfig.authRealm,
        appClient.clientId,
        redirectUri,
        fullScopes
      );

      // Start OAuth flow and handle auth server interaction
      await oauth2TestAppPage.startOAuthFlow();
      await oauth2TestAppPage.waitForAuthServerRedirect(authServerConfig.authUrl);
      await oauth2TestAppPage.handleConsent();
      await oauth2TestAppPage.waitForTokenExchange(testAppUrl);

      // Step 7: Extract and validate access token
      const accessToken = await oauth2TestAppPage.getAccessToken();
      expect(accessToken).toBeTruthy();
      expect(accessToken.length).toBeGreaterThan(100);

      // Step 8: Test API access with OAuth token
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
      const errorConfig = OAuth2Fixtures.getErrorTestConfig(authServerConfig, randomPort());
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
