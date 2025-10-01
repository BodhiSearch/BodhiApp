import { test, expect } from '@playwright/test';
import { getAuthServerConfig } from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { randomPort } from '@/test-helpers.mjs';
import { CanonicalRedirectPage } from '@/pages/CanonicalRedirectPage.mjs';
import { CanonicalRedirectFixtures } from '@/fixtures/canonicalRedirectFixtures.mjs';

test.describe('Canonical Redirect - ENABLED', () => {
  let authServerConfig;
  let port;
  let serverManager;
  let baseUrl;
  let canonicalRedirectPage;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    port = randomPort();

    const serverConfig = CanonicalRedirectFixtures.getServerManagerConfig(
      authServerConfig,
      port,
      true // canonicalRedirectEnabled = true
    );

    serverManager = createServerManager(serverConfig);
    baseUrl = await serverManager.startServer();
  });

  test.beforeEach(async ({ page }) => {
    canonicalRedirectPage = new CanonicalRedirectPage(page, port);
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('should redirect 127.0.0.1 to canonical localhost host', async () => {
    // Navigate to 127.0.0.1
    await canonicalRedirectPage.navigateWithHost('127.0.0.1', '/ui/login');

    // Wait for redirect to localhost
    await canonicalRedirectPage.waitForRedirectTo('localhost');

    // Assert the final URL is the canonical localhost URL
    const currentUrl = canonicalRedirectPage.getCurrentUrl();
    expect(currentUrl).toBe(`http://localhost:${port}/ui/login/`);
  });
});

test.describe('Canonical Redirect - DISABLED', () => {
  let authServerConfig;
  let port;
  let serverManager;
  let baseUrl;
  let canonicalRedirectPage;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    port = randomPort();

    const serverConfig = CanonicalRedirectFixtures.getServerManagerConfig(
      authServerConfig,
      port,
      false // canonicalRedirectEnabled = false
    );

    serverManager = createServerManager(serverConfig);
    baseUrl = await serverManager.startServer();
  });

  test.beforeEach(async ({ page }) => {
    canonicalRedirectPage = new CanonicalRedirectPage(page, port);
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('should not redirect 127.0.0.1 to canonical localhost host', async () => {
    // Navigate to 127.0.0.1
    await canonicalRedirectPage.navigateWithHost('127.0.0.1', '/ui/login');

    // Wait for page to load without redirect
    await canonicalRedirectPage.waitForSPAReady();

    // Assert the URL remains as 127.0.0.1 (no redirect occurred)
    const currentUrl = canonicalRedirectPage.getCurrentUrl();
    expect(currentUrl).toBe(`http://127.0.0.1:${port}/ui/login/`);
  });
});
