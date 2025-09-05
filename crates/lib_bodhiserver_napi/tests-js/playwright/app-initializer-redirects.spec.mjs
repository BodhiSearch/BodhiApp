import { expect, test } from '@playwright/test';
import { getCurrentPath, randomPort, waitForSPAReady } from '../test-helpers.mjs';
import { getAuthServerConfig } from './auth-server-client.mjs';
import { createServerManager } from './bodhi-app-server.mjs';

test.describe('App Initializer Redirect Tests', () => {
  let authServerConfig;
  let port;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    port = randomPort();
  });

  test('should redirect to setup page when app status is setup', async ({ page }) => {
    const serverManager = createServerManager({
      appStatus: 'setup',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      port,
    });

    const baseUrl = await serverManager.startServer();

    try {
      await page.goto(baseUrl);
      await waitForSPAReady(page);

      const currentPath = getCurrentPath(page);
      expect(currentPath).toBe('/ui/setup/');
    } finally {
      await serverManager.stopServer();
    }
  });

  test('should redirect to login page when app status is ready and user not authenticated', async ({
    page,
  }) => {
    // For this test, we don't need actual dynamic clients since we're just testing redirect behavior
    // Use dummy client credentials that won't be used for actual authentication
    const serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: 'dummy-client-id',
      clientSecret: 'dummy-client-secret',
      port,
    });

    const baseUrl = await serverManager.startServer();

    try {
      await page.goto(baseUrl);
      await waitForSPAReady(page);

      const currentPath = getCurrentPath(page);
      expect(currentPath).toBe('/ui/login/');
    } finally {
      await serverManager.stopServer();
    }
  });
});
