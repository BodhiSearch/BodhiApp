import { OAuthTestApp } from '@/pages/OAuthTestApp.mjs';
import {
  getAuthServerConfig,
  getPreConfiguredAppClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';
import { SHARED_SERVER_URL, SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';

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

  test('3rd-party app: OAuth token → streaming chat completion', async ({ page }) => {
    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Complete OAuth flow (auto-approved, no toolsets)', async () => {
      await app.navigate();

      await app.config.configureOAuthForm({
        bodhiServerUrl: SHARED_SERVER_URL,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'openid profile email scope_user_user',
        requested: null,
      });

      await app.config.submitAccessRequest();
      await app.config.waitForLoginReady();
      await app.config.clickLogin();

      await app.oauth.waitForAuthServerRedirect(authServerConfig.authUrl);
      await app.oauth.handleLogin(testCredentials.username, testCredentials.password);
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
