import { expect, test } from '@playwright/test';
import { getCurrentPath, randomPort, waitForSPAReady } from '../test-helpers.mjs';
import { createAuthServerTestClient, getAuthServerConfig, getTestCredentials } from './auth-server-client.mjs';
import { createServerManager } from './bodhi-app-server.mjs';
import { createStaticServer } from './static-server.mjs';

test.describe('OAuth2 Token Exchange v2 Integration Tests', () => {
  let authServerConfig;
  let testCredentials;
  let authClient;
  let port;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    port = randomPort();
    authClient = createAuthServerTestClient(authServerConfig);
  });

  test('complete OAuth2 Token Exchange v2 flow with dynamic audience', async ({ page, context }) => {
    console.log('=== Starting OAuth2 Token Exchange v2 Flow Test ===');

    // Step 1: Setup server in 'setup' mode and complete resource admin setup
    console.log('Step 1: Setting up Bodhi app server in setup mode...');
    const serverManager = createServerManager({
      appStatus: 'setup',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      port,
    });

    const baseUrl = await serverManager.startServer();
    console.log(`✅ Server started at: ${baseUrl}`);

    try {
      // Navigate to setup page and complete resource admin setup
      await page.goto(baseUrl);
      await waitForSPAReady(page);

      const currentPath = getCurrentPath(page);
      expect(currentPath).toBe('/ui/setup/');
      console.log('✅ Setup page loaded successfully');

      // Fill out the setup form first
      const nameField = page.locator('input[name="name"]');
      const setupSubmitButton = page.locator('button[type="submit"]');

      await expect(nameField).toBeVisible({ timeout: 10000 });
      await nameField.fill('OAuth2 Test Server Instance');
      await setupSubmitButton.click();

      // Wait for redirect to resource admin setup
      await page.waitForURL((url) => url.toString().includes('/ui/setup/resource-admin'), { timeout: 10000 });
      await waitForSPAReady(page);
      console.log('✅ Redirected to resource admin setup');

      // Complete the setup flow by logging in as resource admin
      const loginButton = page.locator('button:has-text("Continue with Login"), button:has-text("Login")');
      await expect(loginButton.first()).toBeVisible({ timeout: 10000 });
      await loginButton.first().click();

      // Handle OAuth login flow
      await page.waitForURL((url) => new URL(url).origin === authServerConfig.authUrl, { timeout: 15000 });
      console.log('✅ Redirected to auth server for setup');

      // Fill credentials for resource admin setup
      const usernameField = page.locator('input[name="username"], input[type="email"], #username');
      const passwordField = page.locator('input[name="password"], input[type="password"], #password');
      const submitButton = page.locator('button[type="submit"], input[type="submit"], button:has-text("Sign In")');

      await expect(usernameField).toBeVisible({ timeout: 10000 });
      await usernameField.fill(testCredentials.username);
      await passwordField.fill(testCredentials.password);
      await submitButton.click();

      // Wait for redirect back to app after setup completion
      await page.waitForURL((url) => new URL(url).origin === new URL(baseUrl).origin, { timeout: 20000 });
      await waitForSPAReady(page);
      console.log('✅ Resource admin setup completed');

      // Step 2: Get dev console token for client management
      console.log('Step 2: Obtaining dev console token...');
      const devConsoleToken = await authClient.getDevConsoleToken(
        testCredentials.username,
        testCredentials.password
      );
      console.log('✅ Dev console token obtained');

      // Step 3: Start static server for OAuth test app
      console.log('Step 3: Starting static server for OAuth test app...');
      const appPort = randomPort();
      const staticServer = createStaticServer(appPort);
      const testAppUrl = await staticServer.startServer();
      const redirectUri = `${testAppUrl}/oauth-test-app.html`;
      console.log(`✅ Test app available at: ${redirectUri}`);

      // Step 4: Create app client (public client) with test app redirect URI
      console.log('Step 4: Creating app client with test app redirect URI...');
      const appClient = await authClient.createAppClient(
        devConsoleToken,
        port,
        'OAuth2 Test App Client',
        'Test app client for OAuth2 Token Exchange v2 testing',
        [redirectUri]  // Use test app as redirect URI
      );
      console.log(`✅ App client created: ${appClient.clientId}`);

      // Step 5: Request audience access via Bodhi App API
      console.log('Step 5: Requesting audience access via Bodhi App API...');
      const requestAccessResponse = await fetch(`${baseUrl}/bodhi/v1/auth/request-access`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          app_client_id: appClient.clientId
        }),
      });

      expect(requestAccessResponse.status).toBe(200);
      const requestAccessData = await requestAccessResponse.json();
      const resourceScope = requestAccessData.scope;
      console.log(`✅ Resource scope obtained via Bodhi App: ${resourceScope}`);

      // Step 6: Navigate to test app and complete OAuth flow
      console.log('Step 6: Completing OAuth flow via test app...');
      await page.goto(redirectUri);
      await page.waitForLoadState('networkidle');

      // Fill in OAuth configuration form
      await page.fill('#auth-server-url', authServerConfig.authUrl);
      await page.fill('#realm', authServerConfig.authRealm);
      await page.fill('#client-id', appClient.clientId);
      await page.fill('#redirect-uri', redirectUri);
      await page.fill('#scope', `openid email profile roles scope_user_user ${resourceScope}`);

      // Start OAuth flow
      await page.click('button[type="submit"]');
      console.log('✅ OAuth flow started');

      // Handle OAuth login or consent on auth server
      await page.waitForURL((url) => new URL(url).origin === authServerConfig.authUrl, { timeout: 15000 });
      console.log('✅ Redirected to auth server for OAuth flow');

      const consentYesButton = page.locator('button:has-text("Yes")');
      expect(consentYesButton).toBeVisible();
      await consentYesButton.click();

      // Wait for redirect back to test app with token
      await page.waitForURL((url) => new URL(url).origin === new URL(testAppUrl).origin, { timeout: 20000 });
      await page.waitForLoadState('networkidle');
      console.log('✅ Redirected back to test app');

      // Wait for token exchange to complete and success section to be visible
      await expect(page.locator('#success-section')).toBeVisible({ timeout: 10000 });
      console.log('✅ OAuth token exchange completed successfully');

      // Extract the access token from the UI
      const accessToken = await page.locator('#access-token').textContent();
      expect(accessToken).toBeTruthy();
      expect(accessToken.length).toBeGreaterThan(100); // JWT tokens are long
      console.log(`✅ Access token captured: ${accessToken.substring(0, 50)}...`);

      // Step 7: Use access token to call Bodhi App API
      console.log('Step 7: Testing API access with OAuth token...');
      const userResponse = await fetch(`${baseUrl}/bodhi/v1/user`, {
        headers: {
          'Authorization': `Bearer ${accessToken}`,
          'Content-Type': 'application/json',
        },
      });

      expect(userResponse.status).toBe(200);
      const userInfo = await userResponse.json();
      console.log(`📋 API Response: ${JSON.stringify(userInfo, null, 2)}`);

      // For now, verify that we got a response and the token was processed
      expect(userInfo).toBeDefined();
      expect(userInfo.logged_in).toBe(true);
      expect(userInfo.email).toBe("user@email.com");
      expect(userInfo.role).toBe("scope_user_user");
      expect(userInfo.token_type).toBe("bearer");
      expect(userInfo.role_source).toBe("scope_user");
      console.log(`✅ OAuth token flow completed - token validation behavior documented`);

      await staticServer.stopServer();
      await serverManager.stopServer();
    } catch (error) {
      console.error('❌ Test failed:', error);
      throw error;
    }
  });

  test('should handle token exchange errors gracefully', async ({ page }) => {
    console.log('=== Testing Token Exchange Error Handling ===');

    // Create server with invalid resource client credentials
    const serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: 'invalid-client-id',
      clientSecret: 'invalid-client-secret',
      port: randomPort(),
    });

    const baseUrl = await serverManager.startServer();

    try {
      // Try to access API without any token - should return logged_in: false
      const userInfoResponse = await fetch(`${baseUrl}/bodhi/v1/user`, {
        headers: {
          'Content-Type': 'application/json',
        },
      });

      // Should get 200 response with logged_in: false for unauthenticated users
      expect(userInfoResponse.status).toBe(200);
      const userInfo = await userInfoResponse.json();
      expect(userInfo.logged_in).toBe(false);
      expect(userInfo.email).toBeNull();
      console.log(`✅ Error handling verified: Unauthenticated user gets logged_in: false`);

    } finally {
      await serverManager.stopServer();
    }
  });
}); 