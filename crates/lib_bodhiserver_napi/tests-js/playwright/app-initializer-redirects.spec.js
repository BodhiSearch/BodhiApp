import { expect, test } from '@playwright/test';
import { createServerManager, getCurrentPath, waitForSPAReady } from './playwright-helpers.js';

test.describe('App Initializer Redirect Tests', () => {
  test.describe('App Status Ready - Authentication Flow', () => {
    let serverManager;
    let baseUrl;

    test.beforeAll(async () => {
      serverManager = createServerManager({ appStatus: 'ready' });
      baseUrl = await serverManager.startServer();
    });

    test.afterAll(async () => {
      await serverManager.stopServer();
    });

    test('should redirect all paths to login page when app status is ready', async ({ page }) => {
      const testPaths = ['/', '/ui/chat', '/ui/models', '/ui/settings', '/ui/login'];

      for (const path of testPaths) {
        await page.goto(`${baseUrl}${path}`);
        await waitForSPAReady(page);

        await page.waitForURL(`${baseUrl}/ui/login/`);
        const currentPath = getCurrentPath(page);
        expect(currentPath).toBe('/ui/login/');
      }
    });
  });

  test.describe('App Status Setup - Initial Setup Flow', () => {
    let serverManager;
    let baseUrl;

    test.beforeAll(async () => {
      serverManager = createServerManager({ appStatus: 'setup' });
      baseUrl = await serverManager.startServer();
    });

    test.afterAll(async () => {
      await serverManager.stopServer();
    });

    test('should redirect all paths to setup page when app status is setup', async ({ page }) => {
      const testPaths = ['/', '/ui/chat', '/ui/models', '/ui/settings', '/ui/login'];

      for (const path of testPaths) {
        await page.goto(`${baseUrl}${path}`);
        await waitForSPAReady(page);

        const pageContent = await page.content();
        const currentPath = getCurrentPath(page);

        expect(pageContent.length).toBeGreaterThan(1000);
        expect(currentPath).toBe('/ui/setup/');
      }
    });
  });

  test.describe('App Status Resource-Admin - Admin Setup Flow', () => {
    let serverManager;
    let baseUrl;

    test.beforeAll(async () => {
      serverManager = createServerManager({ appStatus: 'resource-admin' });
      baseUrl = await serverManager.startServer();
    });

    test.afterAll(async () => {
      await serverManager.stopServer();
    });

    test('should redirect to resource-admin page when app status is resource-admin', async ({
      page,
    }) => {
      await page.goto(baseUrl);
      await waitForSPAReady(page);

      const pageContent = await page.content();
      const currentPath = getCurrentPath(page);

      expect(pageContent.length).toBeGreaterThan(1000);
      expect(currentPath).toBe('/ui/setup/resource-admin/');
    });
  });
});
