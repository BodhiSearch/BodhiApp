import { test, expect } from '@playwright/test';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { randomPort, getCurrentPath, waitForSPAReady } from '@/test-helpers.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';

test.describe('App Initializer Integration', () => {
  let authServerConfig;
  let testCredentials;
  let port;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    port = randomPort();
  });

  test.describe('App Status Based Redirects', () => {
    test('should redirect all routes to setup when app status is setup', async ({ page }) => {
      const serverManager = createServerManager({
        appStatus: 'setup',
        authUrl: authServerConfig.authUrl,
        authRealm: authServerConfig.authRealm,
        port,
      });

      const baseUrl = await serverManager.startServer();
      const loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);

      try {
        const testPaths = ['/', '/ui/chat', '/ui/models', '/ui/settings', '/ui/login'];

        // Test redirect behavior for all routes when app is in setup mode
        for (const path of testPaths) {
          await page.goto(`${baseUrl}${path}`);
          await waitForSPAReady(page);

          const currentPath = getCurrentPath(page);
          expect(currentPath).toBe('/ui/setup/');
        }
      } finally {
        await serverManager.stopServer();
      }
    });

    test('should redirect unauthenticated users to login when app status is ready', async ({
      page,
    }) => {
      const serverManager = createServerManager({
        appStatus: 'ready',
        authUrl: authServerConfig.authUrl,
        authRealm: authServerConfig.authRealm,
        clientId: 'dummy-client-id',
        clientSecret: 'dummy-client-secret',
        port,
      });

      const baseUrl = await serverManager.startServer();
      const loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);

      try {
        const protectedPaths = ['/', '/ui/chat', '/ui/models', '/ui/settings'];

        // Test redirect behavior for protected paths when not authenticated
        for (const path of protectedPaths) {
          await page.goto(`${baseUrl}${path}`);
          await waitForSPAReady(page);

          const pageContent = await page.content();
          const currentPath = getCurrentPath(page);

          expect(pageContent.length).toBeGreaterThan(1000);
          expect(currentPath).toBe('/ui/login/');
        }

        // Verify login page functionality
        await page.goto(`${baseUrl}/ui/login`);
        await waitForSPAReady(page);
        await loginPage.expectLoginPageVisible();
      } finally {
        await serverManager.stopServer();
      }
    });
  });

  test.describe('Authentication Flow Integration', () => {
    let serverManager;
    let baseUrl;
    let authClient;
    let resourceClient;
    let loginPage;

    test.beforeAll(async () => {
      const serverUrl = `http://localhost:${port}`;

      authClient = createAuthServerTestClient(authServerConfig);
      resourceClient = await authClient.createResourceClient(serverUrl);
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

    test.beforeEach(async ({ page }) => {
      loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
    });

    test.afterAll(async () => {
      if (serverManager) {
        await serverManager.stopServer();
      }
    });

    test('should complete full OAuth authentication flow from app initializer intercept', async ({
      page,
    }) => {
      // Test that protected routes redirect to login
      await page.goto(`${baseUrl}/ui/chat`);
      await waitForSPAReady(page);

      const currentPath = getCurrentPath(page);
      expect(currentPath).toBe('/ui/login/');

      // Perform OAuth login flow
      await loginPage.performOAuthLogin();

      // Should redirect back to the originally requested route after successful login
      const finalUrl = page.url();
      expect(finalUrl).toContain('/ui/chat');
    });

    test('should redirect protected routes to login and complete authentication flow', async ({
      page,
    }) => {
      // Test that trying to access a protected route while unauthenticated redirects to login
      await page.goto(`${baseUrl}/ui/models`);
      await waitForSPAReady(page);

      const loginPath = getCurrentPath(page);
      expect(loginPath).toBe('/ui/login/');

      // Perform OAuth login flow - should succeed and redirect to an authenticated page
      await loginPage.performOAuthLogin(null);

      const finalUrl = page.url();
      expect(finalUrl).toContain(baseUrl);
      expect(finalUrl).toContain('/ui/');

      // Verify we're authenticated by checking the page content is not the login page
      const finalPath = getCurrentPath(page);
      expect(finalPath).not.toBe('/ui/login/');
    });
  });
});
