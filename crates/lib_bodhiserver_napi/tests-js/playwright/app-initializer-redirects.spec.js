import { test, expect } from '@playwright/test';
import { createServerManager } from './playwright-helpers.js';

/**
 * Wait for SPA to be fully loaded and rendered
 */
async function waitForSPAReady(page) {
  await page.waitForLoadState('networkidle');
  await page.waitForLoadState('domcontentloaded');
}

test.describe('App Initializer Redirect Tests', () => {
  test.describe('App Status: Ready (redirects to /ui/login)', () => {
    let serverManager;
    let baseUrl;

    test.beforeAll(async () => {
      serverManager = createServerManager({ appStatus: 'ready' });
      baseUrl = await serverManager.startServer();
    });

    test.afterAll(async () => {
      if (serverManager) {
        await serverManager.stopServer();
      }
    });

    test('should redirect root path to login page when app status is ready', async ({ page }) => {
      await page.goto(baseUrl);
      await waitForSPAReady(page);

      const pageContent = await page.content();
      const currentPath = new URL(page.url()).pathname;

      expect(pageContent.length).toBeGreaterThan(1000);
      expect(currentPath).toBe('/ui/login/');
    });

    test('should redirect protected paths to login page when app status is ready', async ({
      page,
    }) => {
      const protectedPaths = ['/ui/chat', '/ui/models', '/ui/settings'];

      for (const path of protectedPaths) {
        await page.goto(`${baseUrl}${path}`);
        await waitForSPAReady(page);

        const pageContent = await page.content();
        const currentPath = new URL(page.url()).pathname;

        expect(pageContent.length).toBeGreaterThan(1000);
        expect(currentPath).toBe('/ui/login/');
      }
    });

    test('should serve login page when accessing login directly with ready status', async ({
      page,
    }) => {
      await page.goto(`${baseUrl}/ui/login`);
      await waitForSPAReady(page);

      const pageContent = await page.content();
      const currentPath = new URL(page.url()).pathname;

      expect(pageContent.length).toBeGreaterThan(1000);
      expect(currentPath).toBe('/ui/login/');
    });

    test('should maintain server connectivity when app status is ready', async ({ page }) => {
      await page.goto(baseUrl);
      await waitForSPAReady(page);

      const currentUrl = page.url();
      const pageContent = await page.content();

      expect(currentUrl).toContain(baseUrl.replace('http://', '').replace('https://', ''));
      expect(pageContent.length).toBeGreaterThan(1000);
    });
  });

  test.describe('App Status: Setup (redirects to /ui/setup)', () => {
    let serverManager;
    let baseUrl;

    test.beforeAll(async () => {
      serverManager = createServerManager({ appStatus: 'setup' });
      baseUrl = await serverManager.startServer();
    });

    test.afterAll(async () => {
      if (serverManager) {
        await serverManager.stopServer();
      }
    });

    test('should redirect root path to setup page when app status is setup', async ({ page }) => {
      await page.goto(baseUrl);
      await waitForSPAReady(page);

      const pageContent = await page.content();
      const currentPath = new URL(page.url()).pathname;

      expect(pageContent.length).toBeGreaterThan(1000);
      expect(currentPath).toBe('/ui/setup/');
    });

    test('should redirect protected paths to setup page when app status is setup', async ({
      page,
    }) => {
      const protectedPaths = ['/ui/chat', '/ui/models', '/ui/settings', '/ui/login'];

      for (const path of protectedPaths) {
        await page.goto(`${baseUrl}${path}`);
        await waitForSPAReady(page);

        const pageContent = await page.content();
        const currentPath = new URL(page.url()).pathname;

        expect(pageContent.length).toBeGreaterThan(1000);
        expect(currentPath).toBe('/ui/setup/');
      }
    });

    test('should serve setup page directly when app status is setup', async ({ page }) => {
      await page.goto(`${baseUrl}/ui/setup`);
      await waitForSPAReady(page);

      const pageContent = await page.content();
      const currentPath = new URL(page.url()).pathname;

      expect(pageContent.length).toBeGreaterThan(1000);
      expect(currentPath).toBe('/ui/setup/');
    });
  });

  test.describe('App Status: Resource-Admin (redirects to /ui/setup)', () => {
    let serverManager;
    let baseUrl;

    test.beforeAll(async () => {
      serverManager = createServerManager({ appStatus: 'resource-admin' });
      baseUrl = await serverManager.startServer();
    });

    test.afterAll(async () => {
      if (serverManager) {
        await serverManager.stopServer();
      }
    });

    test('should redirect root path to setup page when app status is resource-admin', async ({
      page,
    }) => {
      await page.goto(baseUrl);
      await waitForSPAReady(page);

      const pageContent = await page.content();
      const currentPath = new URL(page.url()).pathname;

      expect(pageContent.length).toBeGreaterThan(1000);
      expect(currentPath).toBe('/ui/setup/resource-admin/');
    });
  });

  test.describe('App Status: Default/Not Set (redirects to /ui/setup)', () => {
    let serverManager;
    let baseUrl;

    test.beforeAll(async () => {
      serverManager = createServerManager({ appStatus: null });
      baseUrl = await serverManager.startServer();
    });

    test.afterAll(async () => {
      if (serverManager) {
        await serverManager.stopServer();
      }
    });

    test('should redirect root path to setup page when app status is not set', async ({ page }) => {
      await page.goto(baseUrl);
      await waitForSPAReady(page);

      const pageContent = await page.content();
      const currentPath = new URL(page.url()).pathname;

      expect(pageContent.length).toBeGreaterThan(1000);
      expect(currentPath).toBe('/ui/setup/');
    });
  });
});
