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

    await test.step('Opening a server shows the rail with description + server spec, and writes ?select', async () => {
      await mcpPage.openServer('srv-0');
      const spec = mcpPage.page.locator(mcpPage.selectors.railServer);
      await expect(spec).toContainText('Streamable HTTP');
      await expect(spec).toContainText('mcp.example.com');
      await expect(mcpPage.page.locator(mcpPage.selectors.railDescription)).toBeVisible();
      expect(mcpPage.urlParam('select')).toBe('srv-0');
    });

    await test.step('Reload restores the rail from ?select; closing strips it', async () => {
      await mcpPage.page.reload();
      await mcpPage.waitForSPAReady();
      await expect(mcpPage.page.locator(mcpPage.selectors.railServer)).toBeVisible();
      expect(mcpPage.urlParam('select')).toBe('srv-0');

      await mcpPage.closeRail();
      await expect(mcpPage.page.locator(mcpPage.selectors.rail('srv-0'))).toHaveCount(0);
      expect(mcpPage.searchParams().has('select')).toBe(false);
    });

    await test.step('Auth facet (data-driven) filters server-side; reset clears it', async () => {
      await expect(mcpPage.page.locator(mcpPage.selectors.facets)).toBeVisible();
      // Category rail is hidden (facets.category is empty); the auth rail shows the single http chip.
      await expect(mcpPage.page.locator(mcpPage.selectors.auth('http'))).toBeVisible();
      await mcpPage.clickAuth('http');
      await expect(mcpPage.page.locator(mcpPage.selectors.auth('http'))).toHaveAttribute('aria-pressed', 'true');
      // Reset is in 'filters' state and clears the auth filter back to the inert 'none' state.
      await mcpPage.clearAllFilters();
      await expect(mcpPage.page.locator(mcpPage.selectors.auth('http'))).toHaveAttribute('aria-pressed', 'false');
      await expect(mcpPage.page.locator(mcpPage.selectors.clearAll)).toHaveAttribute('data-test-state', 'none');
    });

    await test.step('Verified facet filters client-side (no verified API param)', async () => {
      // No stub server is verified → Verified yields an empty list.
      await mcpPage.toggleVerified();
      await expect(mcpPage.page.locator(mcpPage.selectors.verified)).toHaveAttribute('aria-pressed', 'true');
      await expect(mcpPage.page.locator(mcpPage.selectors.empty)).toBeVisible();
      await mcpPage.clearAllFilters();
    });

    await test.step('Column picker hides the Auth column', async () => {
      await expect(mcpPage.page.locator(mcpPage.selectors.sort('name'))).toBeVisible();
      await mcpPage.toggleColumn('auth');
      // The Auth header is gone after hiding the optional column.
      await expect(mcpPage.page.locator('[data-testid="cat-listhead"]')).not.toContainText('AUTH');
    });

    await test.step('Status column reflects the instance join (none configured → Not installed)', async () => {
      // No instances configured in the test DB → every catalog row joins to "Not installed".
      await expect(mcpPage.page.locator(mcpPage.selectors.install('srv-0'))).toContainText('Not installed');
      await expect(mcpPage.page.locator(mcpPage.selectors.installedFacet('not_installed'))).toBeVisible();
    });

    await test.step('Rail of an unregistered catalog server offers admin a Connect-Server footer', async () => {
      await mcpPage.openServer('srv-0');
      // V2 rail has no Status section; admin sees a "not configured" note + a Connect-Server footer
      // deep-linking to the New-Server form with the catalog url/name (+ auth_type) prefilled.
      await expect(mcpPage.page.locator(mcpPage.selectors.railNotConfiguredAdmin)).toBeVisible();
      const connect = mcpPage.page.locator(mcpPage.selectors.railConnectServer);
      await expect(connect).toBeVisible();
      await expect(connect).toHaveAttribute('href', /\/mcps\/servers\/new\/\?url=/);
    });
  });
});
