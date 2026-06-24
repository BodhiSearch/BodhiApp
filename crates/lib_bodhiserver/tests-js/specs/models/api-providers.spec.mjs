import { ApiProvidersPage } from '@/pages/ApiProvidersPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

// Black-box E2E for Explore · API Providers (screen-v2). The provider catalog is served by the
// external Reference API — here we STUB it via page.route (deterministic) so we can assert exact
// names/counts/pagination. ONE test grows across phases via test.step (E2E runs are expensive).
// Phase B1: list + page-based Load more. Only runs standalone (multi_tenant excludes specs/models/).

test.describe('Explore · API Providers', () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  let loginPage;
  let providersPage;

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    providersPage = new ApiProvidersPage(page, sharedServerUrl);
  });

  test('browses the API-provider catalog @integration', async () => {
    await test.step('Login, stub the catalog, and open Explore · API Providers', async () => {
      await loginPage.performOAuthLogin();
      await providersPage.stubCatalog();
      await providersPage.navigateToProviders();
      await providersPage.waitForListSettled();
      await expect(providersPage.page.locator(providersPage.selectors.resultbar)).toBeVisible();
    });

    await test.step('List renders provider rows with "Showing X of TOTAL"', async () => {
      const count = await providersPage.getRowCount();
      expect(count).toBe(30);
      await expect(providersPage.page.locator(providersPage.selectors.resultbar)).toContainText('Showing 30 of 31');
      await expect(providersPage.page.locator(providersPage.selectors.row('prov-0'))).toContainText('Provider 0');
      // Model count + rank render on the row.
      await expect(providersPage.page.locator(providersPage.selectors.row('prov-0'))).toContainText('1000');
    });

    await test.step('Numbered pager navigates to page 2', async () => {
      expect(await providersPage.hasPagination()).toBe(true);
      await providersPage.nextPage();
      // 31 providers, 30/page → page 2 has the single remaining row.
      await expect(providersPage.page.locator(providersPage.selectors.resultbar)).toContainText('Showing 1 of 31');
      expect(await providersPage.getRowCount()).toBe(1);
      await providersPage.gotoPage(1);
    });

    await test.step('Opening a provider shows the rail with connection meta + models', async () => {
      await providersPage.openProvider('prov-0');
      const meta = providersPage.page.locator(providersPage.selectors.detailMeta);
      await expect(meta).toBeVisible();
      // Connection meta from the (stubbed) provider-detail fetch.
      await expect(meta).toContainText('PROV_0_API_KEY');
      await expect(meta).toContainText('prov-0.example.com');
      await expect(providersPage.page.locator(providersPage.selectors.docLink)).toBeVisible();
      // The provider's models render from the provider-models fetch.
      await expect(providersPage.page.locator(providersPage.selectors.detailModels)).toContainText('Model A');

      await providersPage.closeRail();
      await expect(providersPage.page.locator(providersPage.selectors.railPanel)).toHaveCount(0);
    });

    await test.step('Search narrows the provider list and counts reflect the query', async () => {
      // "Provider 7" matches exactly one stub provider (names are "Provider N").
      await providersPage.searchFor('Provider 7');
      await expect(providersPage.page.locator(providersPage.selectors.resultbar)).toContainText('Showing 1 of 1');
      await expect(providersPage.page.locator(providersPage.selectors.row('prov-7'))).toBeVisible();
      // Facet count reflects the filtered set.
      await expect(providersPage.page.locator(providersPage.selectors.cap('reasoning'))).toContainText('1');

      await providersPage.clearSearch();
      await expect(providersPage.page.locator(providersPage.selectors.resultbar)).toContainText('Showing 30 of 31');
    });

    await test.step('Sort by Models re-queries and marks the active control', async () => {
      await providersPage.sortBy('model_count');
      await expect(providersPage.page.locator(providersPage.selectors.sort('model_count'))).toHaveAttribute(
        'data-test-state',
        'active'
      );
      await expect(providersPage.page.locator(providersPage.selectors.resultbar)).toContainText('Models');
    });

    await test.step('Capability facet filters; Clear all resets', async () => {
      await expect(providersPage.page.locator(providersPage.selectors.facets)).toBeVisible();
      await providersPage.clickCapability('reasoning');
      await expect(providersPage.page.locator(providersPage.selectors.cap('reasoning'))).toHaveAttribute(
        'aria-pressed',
        'true'
      );
      await providersPage.clearAllFilters();
      await expect(providersPage.page.locator(providersPage.selectors.clearAll)).toHaveCount(0);
    });
  });
});
