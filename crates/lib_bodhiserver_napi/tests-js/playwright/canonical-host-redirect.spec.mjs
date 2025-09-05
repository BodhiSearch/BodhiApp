import { expect, test } from '@playwright/test';
import { randomPort, waitForSPAReady } from '../test-helpers.mjs';
import { getAuthServerConfig } from './auth-server-client.mjs';
import { createServerManager } from './bodhi-app-server.mjs';

test.describe('Canonical URL Redirect Tests - enabled', () => {
  let serverManager;
  let baseUrl;
  let authServerConfig;
  let port;
  let serverUrl;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    port = randomPort();
    serverUrl = `http://localhost:${port}`;

    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: 'dummy-client-id',
      clientSecret: 'dummy-client-secret',
      port: port.toString(),
      host: '0.0.0.0',
      envVars: {
        BODHI_PUBLIC_HOST: 'localhost',
        BODHI_PUBLIC_SCHEME: 'http',
        BODHI_PUBLIC_PORT: port.toString(),
        BODHI_CANONICAL_REDIRECT: 'true',
      },
    });
    baseUrl = await serverManager.startServer();
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('should redirect 127.0.0.1 to canonical localhost host', async ({ page }) => {
    await page.goto(`http://127.0.0.1:${port}/ui/login`);
    await page.waitForURL((url) => url.origin === `http://localhost:${port}`, { timeout: 1000 });
    const currentUrl = page.url();
    expect(currentUrl).toBe(`http://localhost:${port}/ui/login/`);
  });
});

test.describe('Canonical URL Redirect Tests - disabled', () => {
  let serverManager;
  let baseUrl;
  let authServerConfig;
  let port;
  let serverUrl;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    port = randomPort();
    serverUrl = `http://localhost:${port}`;

    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: 'dummy-client-id',
      clientSecret: 'dummy-client-secret',
      port: port.toString(),
      host: '0.0.0.0',
      envVars: {
        BODHI_PUBLIC_HOST: 'localhost',
        BODHI_PUBLIC_SCHEME: 'http',
        BODHI_PUBLIC_PORT: port.toString(),
        BODHI_CANONICAL_REDIRECT: 'false',
      },
    });
    baseUrl = await serverManager.startServer();
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('should not redirect 127.0.0.1 to canonical localhost host', async ({ page }) => {
    await page.goto(`http://127.0.0.1:${port}/ui/login`);
    await waitForSPAReady(page);
    const currentUrl = page.url();
    expect(currentUrl).toBe(`http://127.0.0.1:${port}/ui/login/`);
  });
});
