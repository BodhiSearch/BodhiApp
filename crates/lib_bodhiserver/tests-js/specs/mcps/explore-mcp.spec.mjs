import { LoginPage } from '@/pages/LoginPage.mjs';
import { McpExplorePage } from '@/pages/McpExplorePage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

// Black-box E2E for Explore · MCP Servers (screen-v2). The external MCP catalog is STUBBED via
// page.route for determinism. ONE test grows across phases via test.step. Phase 1: list + search +
// pagination.

test.describe('Explore · MCP Servers', () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  let loginPage;
  let mcpPage;

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    mcpPage = new McpExplorePage(page, sharedServerUrl);
  });

  test('browses the MCP-server catalog @integration', async () => {
    await test.step('Login, stub the catalog, and open Explore · MCP Servers', async () => {
      await loginPage.performOAuthLogin();
      await mcpPage.stubCatalog();
      await mcpPage.navigateToExplore();
      await mcpPage.waitForListSettled();
      await expect(mcpPage.page.locator(mcpPage.selectors.list)).toBeVisible();
    });

    await test.step('List renders the first page of server rows', async () => {
      expect(await mcpPage.getRowCount()).toBe(50);
      await expect(mcpPage.page.locator(mcpPage.selectors.row('srv-0'))).toContainText('Server 0');
      // The auth_type placeholder shows on every row.
      await expect(mcpPage.page.locator(mcpPage.selectors.row('srv-0'))).toContainText('http');
    });

    await test.step('Numbered pager navigates to page 2 and back', async () => {
      expect(await mcpPage.hasPagination()).toBe(true);
      await mcpPage.gotoPage(2);
      // 60 servers, 50/page → page 2 has the remaining 10.
      await expect(mcpPage.page.locator(mcpPage.selectors.anyRow)).toHaveCount(10);
      await mcpPage.gotoPage(1);
      await expect(mcpPage.page.locator(mcpPage.selectors.anyRow)).toHaveCount(50);
    });

    await test.step('Search narrows the list server-side and writes ?q=', async () => {
      await mcpPage.searchFor('Server 7');
      await expect(mcpPage.page.locator(mcpPage.selectors.anyRow)).toHaveCount(1);
      expect(mcpPage.searchParams().get('q')).toBe('Server 7');
      await mcpPage.clearSearch();
      await expect(mcpPage.page.locator(mcpPage.selectors.anyRow)).toHaveCount(50);
    });

    await test.step('Opening a server shows the rail with connection + metadata, and writes ?select', async () => {
      await mcpPage.openServer('srv-0');
      const conn = mcpPage.page.locator(mcpPage.selectors.railConnection);
      await expect(conn).toContainText('streamable-http');
      await expect(conn).toContainText('mcp.example.com');
      await expect(mcpPage.page.locator(mcpPage.selectors.railMetadata)).toContainText('mcpservers.org');
      expect(mcpPage.urlParam('select')).toBe('srv-0');
    });

    await test.step('Reload restores the rail from ?select; closing strips it', async () => {
      await mcpPage.page.reload();
      await mcpPage.waitForSPAReady();
      await expect(mcpPage.page.locator(mcpPage.selectors.railConnection)).toBeVisible();
      expect(mcpPage.urlParam('select')).toBe('srv-0');

      await mcpPage.closeRail();
      await expect(mcpPage.page.locator(mcpPage.selectors.rail('srv-0'))).toHaveCount(0);
      expect(mcpPage.searchParams().has('select')).toBe(false);
    });
  });
});
