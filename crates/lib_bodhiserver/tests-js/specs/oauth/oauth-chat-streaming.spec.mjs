import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { AccessRequestReviewPage } from '@/pages/AccessRequestReviewPage.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { OAuthTestApp } from '@/pages/OAuthTestApp.mjs';
import {
  getAuthServerConfig,
  getPreConfiguredAppClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { registerApiModelViaUI } from '@/utils/api-model-helpers.mjs';
import { expect, test } from '@/fixtures.mjs';
import { SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';

/**
 * OAuth Chat Streaming E2E Tests
 *
 * Tests 3rd-party app OAuth flow → streaming chat completion.
 * Verifies that an OAuth-authenticated user can use the chat page
 * on the test app to send messages and receive streaming responses.
 */

test.describe('OAuth Chat Streaming', () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test('3rd-party app: OAuth token → streaming chat completion', async ({
    page,
    sharedServerUrl,
  }) => {
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Login to Bodhi server and register API model', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      const modelsPage = new ModelsListPage(page, sharedServerUrl);
      const formPage = new ApiModelFormPage(page, sharedServerUrl);
      const { apiKey } = ApiModelFixtures.getRequiredEnvVars();
      await registerApiModelViaUI(modelsPage, formPage, apiKey);
    });

    await test.step('Complete OAuth flow (draft → review → approve)', async () => {
      await app.navigate();

      await app.config.configureOAuthForm({
        bodhiServerUrl: sharedServerUrl,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'openid profile email',
        requested: JSON.stringify({ version: '1' }),
      });

      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);

      const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
      await reviewPage.approve();

      await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
      await app.accessCallback.waitForLoaded();
      await app.accessCallback.clickLogin();

      await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
    });

    await test.step('Verify logged in and token present', async () => {
      await app.expectLoggedIn();
      await app.dashboard.navigateTo();
      const accessToken = await app.dashboard.getAccessToken();
      expect(accessToken).toBeTruthy();
      expect(accessToken.length).toBeGreaterThan(100);
    });

    await test.step('Navigate to chat and verify models list accessible', async () => {
      await app.chat.navigateTo();
      await app.chat.waitForModelsLoaded();
      const models = await app.chat.getModels();
      expect(models.length).toBeGreaterThan(0);
      await app.chat.selectModel(ApiModelFixtures.OPENAI_MODEL);
    });

    await test.step('Send message and verify streaming response', async () => {
      await app.chat.sendMessage('What is 2+2? Reply with just the number.');
      await app.chat.waitForResponse();

      const status = await app.chat.getStatus();
      expect(status).toBe('idle');

      const response = await app.chat.getLastResponse();
      expect(response).toBeTruthy();
      expect(response.length).toBeGreaterThan(0);
      expect(response).toContain('4');
    });
  });
});
