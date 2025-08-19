import { expect, test } from '@playwright/test';
import { getCurrentPath, randomPort, waitForSPAReady } from '../test-helpers.mjs';
import { createAuthServerTestClient, getAuthServerConfig, getTestCredentials } from './auth-server-client.mjs';
import { createServerManager } from './bodhi-app-server.mjs';

test.describe('Canonical URL Redirect Tests', () => {
  let serverManager;
  let baseUrl;
  let authServerConfig;
  let testCredentials;
  let authClient;
  // let resourceClient;
  let port;
  let serverUrl;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    port = 8080;
    serverUrl = `http://localhost:${port}`;

    authClient = createAuthServerTestClient(authServerConfig);
    // resourceClient = await authClient.createResourceClient(serverUrl);
    // await authClient.makeResourceAdmin(resourceClient.clientId, resourceClient.clientSecret, testCredentials.username);

    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: "dummy-client-id",
      clientSecret: "dummy-client-secret",
      port: '8080',
      host: '0.0.0.0',
      envVars: {
        'BODHI_PUBLIC_HOST': 'localhost',
        'BODHI_PUBLIC_SCHEME': 'http',
        'BODHI_PUBLIC_PORT': '8080',
      }
    });
    baseUrl = await serverManager.startServer();
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('should redirect 127.0.0.1 to canonical localhost host', async ({ page }) => {
    await page.goto('http://127.0.0.1:8080/ui/login');
    await page.waitForURL((url) => url.origin === 'http://localhost:8080' && url.pathname === '/ui/login/');
    const currentUrl = page.url();
    expect(currentUrl).toBe('http://localhost:8080/ui/login/');
  });
});