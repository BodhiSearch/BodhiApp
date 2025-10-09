import { test, expect } from '@playwright/test';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
  getRealmAdminCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { randomPort } from '@/test-helpers.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';

/**
 * Token Refresh Integration Tests
 *
 * This test suite validates automatic access token refresh when tokens expire,
 * specifically testing the scenario where users switch to another tab/window
 * and the token expires while the app is in the background.
 *
 * Test Scenario:
 * 1. User logs in and navigates to models page
 * 2. User opens a new tab (e.g., google.com)
 * 3. Token expires while app is in background tab (30s wait)
 * 4. User closes new tab and returns to app
 * 5. User waits on app tab without navigation (5s)
 * 6. User navigates to another page (settings)
 * 7. Token should refresh automatically without logging user out
 * 8. User can continue navigating (back to models)
 *
 * Due to the long execution time (configures short-lived tokens and waits for expiration),
 * these tests are excluded from regular test runs and should be run on a schedule.
 *
 * To run these tests:
 *   npm run test:playwright:scheduled
 *
 * Or directly:
 *   npx playwright test --grep @scheduled
 */
test.describe('Token Refresh Integration', { tag: '@scheduled' }, () => {
  let authClient, testCredentials, authServerConfig, realmAdminCredentials;
  let serverManager, baseUrl;
  let resourceClient, adminToken;
  let port;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    realmAdminCredentials = getRealmAdminCredentials();
    port = randomPort();

    const serverUrl = `http://localhost:${port}`;

    authClient = createAuthServerTestClient(authServerConfig);

    resourceClient = await authClient.createResourceClient(serverUrl);

    // Get admin token using realm admin credentials with admin-cli client
    adminToken = await authClient.getRealmAdminToken(
      realmAdminCredentials.username,
      realmAdminCredentials.password
    );

    // Configure client with 15-second access token lifespan
    await authClient.configureClientTokenLifespan(adminToken, resourceClient.clientId, 15);

    await authClient.makeResourceAdmin(
      resourceClient.clientId,
      resourceClient.clientSecret,
      testCredentials.userId
    );

    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      port,
      host: 'localhost',
    });

    baseUrl = await serverManager.startServer();
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('should preserve session when token expires in background tab', async ({
    page,
    context,
  }) => {
    test.setTimeout(180000); // 3 minutes timeout for long-running test
    const loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);

    await page.goto(`${baseUrl}/ui/chat`);

    await loginPage.performOAuthLogin();

    // Navigate to models page
    await page.goto(`${baseUrl}/ui/models`);
    await page.waitForLoadState('domcontentloaded');

    console.log('User logged in and on models page');

    // Get initial session tokens
    const initialAccessToken = await page.evaluate(async () => {
      const response = await fetch('/dev/secrets');
      const data = await response.json();
      return data.session.access_token;
    });

    console.log('Initial access token (first 20 chars):', initialAccessToken.substring(0, 20));

    // Open a new tab with google.com
    const googleTab = await context.newPage();
    await googleTab.goto('https://www.google.com');
    console.log('Opened new tab with google.com');

    // Wait for token to expire (15s lifespan + 15s buffer for clock skew between machines)
    console.log('Waiting 30 seconds for token to expire in background...');
    await googleTab.waitForTimeout(30000);

    // Close the google tab to return focus to app tab
    await googleTab.close();
    console.log('Closed google tab, returning to app tab');

    // Wait additional 5 seconds while on app tab (no navigation yet)
    console.log('Waiting 5 seconds on app tab without navigation...');
    await page.waitForTimeout(5000);

    // Verify user is still on models page and not logged out
    const currentUrl = page.url();
    expect(currentUrl).toContain('/ui/models');
    console.log('User still on models page, not logged out');

    // Navigate via URL to settings page - should trigger token refresh without logout
    console.log('Navigating to settings page...');
    await page.goto(`${baseUrl}/ui/settings`);
    await page.waitForLoadState('domcontentloaded');

    // Verify we're on settings page (not redirected to login)
    const settingsUrl = page.url();
    expect(settingsUrl).toContain('/ui/settings');
    console.log('Successfully navigated to settings page');

    // Get refreshed session tokens
    const refreshedAccessToken = await page.evaluate(async () => {
      const response = await fetch('/dev/secrets');
      const data = await response.json();
      return data.session.access_token;
    });

    console.log('Refreshed access token (first 20 chars):', refreshedAccessToken.substring(0, 20));

    // Verify token was refreshed (should be different from initial)
    expect(refreshedAccessToken).not.toBe(initialAccessToken);

    // Navigate to models page via URL
    console.log('Navigating back to models page via URL...');
    await page.goto(`${baseUrl}/ui/models`);
    await page.waitForLoadState('domcontentloaded');

    // Verify we're on models page (not redirected to login)
    const modelsUrl = page.url();
    expect(modelsUrl).toContain('/ui/models');
    console.log('Successfully navigated to models page');

    // Verify session is still preserved (user not logged out)
    const finalAccessToken = await page.evaluate(async () => {
      const response = await fetch('/dev/secrets');
      const data = await response.json();
      return data.session.access_token;
    });

    // Token should exist (user still logged in)
    expect(finalAccessToken).toBeTruthy();
    expect(finalAccessToken).toContain('eyJhbGciOiJSUzI1NiIs'); // Valid JWT header

    console.log('Final access token (first 20 chars):', finalAccessToken.substring(0, 20));
    console.log(
      'Session preserved successfully - user not logged out after token expiry in background'
    );
  });
});
