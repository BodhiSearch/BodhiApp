import { AppsAuthPage } from '@/pages/AppsAuthPage.mjs';
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
    test('should complete OAuth2 authorize flow via the consent page', async ({
      page,
      sharedServerUrl,
    }) => {
      const appClient = getPreConfiguredAppClient();
      const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;

      const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);
      const consentPage = new AppsAuthPage(page, sharedServerUrl);

      await test.step('Login to Bodhi server', async () => {
        const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
        await loginPage.performOAuthLogin();
      });

      await test.step('Navigate to test app', async () => {
        await app.navigate();
      });

      await test.step('Configure OAuth form with default scope', async () => {
        await app.config.configureOAuthForm({
          bodhiServerUrl: sharedServerUrl,
          authServerUrl: authServerConfig.authUrl,
          realm: authServerConfig.authRealm,
          clientId: appClient.clientId,
          redirectUri,
          scope: 'scope_user_user',
        });
      });

      await test.step('Start authorize navigation to the consent page', async () => {
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);
        await consentPage.waitForConsentPage();
      });

      await test.step('Default scope renders both resource sections', async () => {
        await expect(page.locator(consentPage.selectors.modelsSection)).toBeVisible();
        await expect(page.locator(consentPage.selectors.mcpsSection)).toBeVisible();
      });

      await test.step('Approve; Keycloak authorizes and the app exchanges the code', async () => {
        await consentPage.clickApprove();
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
    test('should downgrade role from power_user to user on consent approval', async ({
      page,
      sharedServerUrl,
    }) => {
      const appClient = getPreConfiguredAppClient();
      const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;

      const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);
      const consentPage = new AppsAuthPage(page, sharedServerUrl);

      await test.step('Login to Bodhi server', async () => {
        const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
        await loginPage.performOAuthLogin();
      });

      await test.step('Navigate to test app', async () => {
        await app.navigate();
      });

      await test.step('Configure OAuth form: power_user role, MCP access suppressed', async () => {
        await app.config.configureOAuthForm({
          bodhiServerUrl: sharedServerUrl,
          authServerUrl: authServerConfig.authUrl,
          realm: authServerConfig.authRealm,
          clientId: appClient.clientId,
          redirectUri,
          scope: 'scope_apps:mcps:false',
          requestedRole: 'scope_user_power_user',
        });
      });

      await test.step('Start authorize navigation to the consent page', async () => {
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);
        await consentPage.waitForConsentPage();
      });

      await test.step('scope_apps:mcps:false suppresses the MCP section', async () => {
        await expect(page.locator(consentPage.selectors.modelsSection)).toBeVisible();
        await expect(page.locator(consentPage.selectors.mcpsSection)).toHaveCount(0);
      });

      await test.step('Downgrade role to user and approve', async () => {
        await consentPage.approveWithRole('scope_user_user');

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

  test.describe('Reauthorize / Upgrade Flow', () => {
    test('reauthorize pre-fills the consent from the source grant and elevates the token', async ({
      page,
      sharedServerUrl,
    }) => {
      const appClient = getPreConfiguredAppClient();
      const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
      const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);
      const consentPage = new AppsAuthPage(page, sharedServerUrl);

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
          scope: 'scope_user_user',
        });
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);
        await consentPage.approveWithGrants({
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

      await test.step('Reauthorize with an elevated scope from the REST page', async () => {
        await app.rest.reauthorize('scope_user_power_user');
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);
        await consentPage.waitForConsentPage();
      });

      await test.step('Consent is pre-filled from the source grant (explicit reauth)', async () => {
        await expect(page.locator(consentPage.selectors.reauthBanner)).toBeVisible();
        // Listings held by the source grant load pre-checked.
        expect(await consentPage.isListModelsChecked()).toBe(true);
        expect(await consentPage.isListMcpsChecked()).toBe(true);
      });

      await test.step('Select power_user role and approve the upgrade', async () => {
        // Grants are pre-populated (all models/MCPs, listings on); select the elevated
        // role explicitly and approve to commit the upgraded grant.
        await consentPage.selectApprovedRole('scope_user_power_user');
        await consentPage.clickApprove();
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

  test.describe('Consent Surface', () => {
    test('role-only scope: summary shown, approved token has no inference access', async ({
      page,
      sharedServerUrl,
    }) => {
      const appClient = getPreConfiguredAppClient();
      const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
      const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);
      const consentPage = new AppsAuthPage(page, sharedServerUrl);

      await test.step('Login to Bodhi server', async () => {
        const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
        await loginPage.performOAuthLogin();
      });

      await test.step('Configure a role-only scope and start authorize', async () => {
        await app.navigate();
        await app.config.configureOAuthForm({
          bodhiServerUrl: sharedServerUrl,
          authServerUrl: authServerConfig.authUrl,
          realm: authServerConfig.authRealm,
          clientId: appClient.clientId,
          redirectUri,
          scope: 'scope_user_user scope_apps:llms:false scope_apps:mcps:false',
        });
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);
        await consentPage.waitForConsentPage();
      });

      await test.step('Consent shows the role-only summary and no resource sections', async () => {
        await expect(page.locator(consentPage.selectors.roleOnlySummary)).toBeVisible();
        await expect(page.locator(consentPage.selectors.modelsSection)).toHaveCount(0);
        await expect(page.locator(consentPage.selectors.mcpsSection)).toHaveCount(0);
      });

      await test.step('Approve; the app receives a token', async () => {
        await consentPage.clickApprove();
        await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
      });

      await test.step('Role-gated API works but inference is forbidden (403)', async () => {
        await app.rest.sendRequest({ method: 'GET', url: '/bodhi/v1/user' });
        expect(await app.rest.getResponseStatus()).toBe(200);
        const info = await app.rest.getResponse();
        expect(info.auth_status).toBe('logged_in');
        expect(info.role).toBe('scope_user_user');

        await app.rest.sendRequest({
          method: 'POST',
          url: '/v1/chat/completions',
          body: { model: 'gpt-4', messages: [{ role: 'user', content: 'hi' }] },
        });
        expect(await app.rest.getResponseStatus()).toBe(403);
      });
    });

    test('unregistered redirect_uri: in-app consent error, no redirect back to the app', async ({
      page,
      sharedServerUrl,
    }) => {
      const appClient = getPreConfiguredAppClient();
      const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);
      const consentPage = new AppsAuthPage(page, sharedServerUrl);

      await test.step('Login to Bodhi server', async () => {
        const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
        await loginPage.performOAuthLogin();
      });

      await test.step('Start authorize with an unregistered redirect_uri', async () => {
        await app.navigate();
        await app.config.configureOAuthForm({
          bodhiServerUrl: sharedServerUrl,
          authServerUrl: authServerConfig.authUrl,
          realm: authServerConfig.authRealm,
          clientId: appClient.clientId,
          redirectUri: `${SHARED_STATIC_SERVER_URL}/unregistered-callback`,
          scope: 'scope_user_user',
        });
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);
      });

      await test.step('Consent shows an in-app error and stays on the Bodhi origin', async () => {
        await consentPage.waitForError();
        expect(new URL(page.url()).origin).toBe(new URL(sharedServerUrl).origin);
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
