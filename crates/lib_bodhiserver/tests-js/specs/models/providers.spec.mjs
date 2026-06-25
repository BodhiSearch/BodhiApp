import { LoginPage } from '@/pages/LoginPage.mjs';
import { ProvidersPage } from '@/pages/ProvidersPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

// Black-box E2E for Explore · API Providers (screen-v2). The provider catalog is served by the
// external Reference API — here we STUB it via page.route (deterministic) so we can assert exact
// names/counts/pagination. ONE test grows across phases via test.step (E2E runs are expensive).
// Only runs standalone (multi_tenant excludes specs/models/).

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
    providersPage = new ProvidersPage(page, sharedServerUrl);
  });

  test('browses the API-provider catalog with URL-synced filters @integration', async () => {
    await test.step('Login, stub the catalog, and open Explore · API Providers', async () => {
      await loginPage.performOAuthLogin();
      await providersPage.stubCatalog();
      await providersPage.navigateToProviders();
      await providersPage.waitForListSettled();
      // The route lives at /providers/ now (not /api-providers/).
      expect(providersPage.page.url()).toContain('/models/explore/providers/');
    });

    await test.step('List renders rows; there is no result bar (count is in the pager)', async () => {
      expect(await providersPage.getRowCount()).toBe(30);
      await expect(providersPage.page.locator('[data-testid="cat-prov-resultbar"]')).toHaveCount(0);
      await expect(providersPage.page.locator(providersPage.selectors.row('prov-0'))).toContainText('Provider 0');
      await expect(providersPage.page.locator(providersPage.selectors.row('prov-0'))).toContainText('1000');
    });

    await test.step('Numbered pager navigates to page 2 and writes ?page=2', async () => {
      expect(await providersPage.hasPagination()).toBe(true);
      await providersPage.nextPage();
      expect(providersPage.searchParams().get('page')).toBe('2');
      expect(await providersPage.getRowCount()).toBe(1);
      await providersPage.gotoPage(1);
    });

    await test.step('Opening a provider shows the rail with connection meta + models', async () => {
      await providersPage.openProvider('prov-0');
      const meta = providersPage.page.locator(providersPage.selectors.detailMeta);
      await expect(meta).toBeVisible();
      await expect(meta).toContainText('PROV_0_API_KEY');
      await expect(meta).toContainText('prov-0.example.com');
      await expect(providersPage.page.locator(providersPage.selectors.docLink)).toBeVisible();
      await expect(providersPage.page.locator(providersPage.selectors.detailModels)).toContainText('Model A');

      await providersPage.closeRail();
      await expect(providersPage.page.locator(providersPage.selectors.railPanel)).toHaveCount(0);
    });

    await test.step('Search narrows the list and writes ?q to the URL', async () => {
      await providersPage.searchFor('Provider 7');
      expect(providersPage.searchParams().get('q')).toBe('Provider 7');
      await expect(providersPage.page.locator(providersPage.selectors.row('prov-7'))).toBeVisible();
      await expect(providersPage.page.locator(providersPage.selectors.cap('reasoning'))).toContainText('1');

      await providersPage.clearSearch();
      expect(providersPage.searchParams().has('q')).toBe(false);
      await expect(providersPage.page.locator(providersPage.selectors.row('prov-0'))).toBeVisible();
    });

    await test.step('Sort by the MODELS column writes ?sort and marks the header active', async () => {
      await providersPage.sortBy('model_count');
      expect(providersPage.searchParams().get('sort')).toBe('model_count');
      await expect(providersPage.page.locator(providersPage.selectors.sort('model_count'))).toHaveAttribute(
        'data-test-state',
        'active'
      );
    });

    await test.step('Sort by the FORMAT column (api_format); rank + cheapest sorts are gone', async () => {
      await providersPage.sortBy('api_format');
      expect(providersPage.searchParams().get('sort')).toBe('api_format');
      await expect(providersPage.page.locator(providersPage.selectors.sort('rank'))).toHaveCount(0);
      await expect(providersPage.page.locator(providersPage.selectors.sort('pricing'))).toHaveCount(0);
    });

    await test.step('Back/Forward revert and re-apply the sort + URL', async () => {
      // Currently sort=api_format; Back returns to sort=model_count from the prior step.
      await providersPage.goBack();
      expect(providersPage.searchParams().get('sort')).toBe('model_count');
      await providersPage.goForward();
      expect(providersPage.searchParams().get('sort')).toBe('api_format');
    });

    await test.step('Labs-only toggle filters to labs and writes ?is_lab=true', async () => {
      await expect(providersPage.page.locator(providersPage.selectors.labs)).toHaveAttribute('aria-pressed', 'false');
      await providersPage.clickLabs();
      expect(providersPage.searchParams().get('is_lab')).toBe('true');
      await expect(providersPage.page.locator(providersPage.selectors.labs)).toHaveAttribute('aria-pressed', 'true');
      // Every 4th stub provider is a lab → 8 of 31 (prov-0,4,…,28).
      expect(await providersPage.getRowCount()).toBe(8);
      await providersPage.clickLabs();
      expect(providersPage.searchParams().has('is_lab')).toBe(false);
    });

    await test.step('Free/Paid pricing toggle is single-select and writes ?pricing', async () => {
      await providersPage.clickPricing('free');
      expect(providersPage.searchParams().get('pricing')).toBe('free');
      await providersPage.clickPricing('free');
      expect(providersPage.searchParams().has('pricing')).toBe(false);
    });

    await test.step('Toolbar reset waterfalls: clears facets first, then the query', async () => {
      const reset = providersPage.page.locator(providersPage.selectors.clearAll);
      // Reset lives in the toolbar, not the sidebar facet panel.
      await expect(
        providersPage.page.locator(`${providersPage.selectors.facets} ${providersPage.selectors.clearAll}`)
      ).toHaveCount(0);

      await providersPage.searchFor('Provider');
      await providersPage.clickCapability('reasoning');
      await expect(reset).toHaveAttribute('data-test-state', 'filters');

      await providersPage.clickReset();
      // Facets cleared, query kept.
      await expect(providersPage.page.locator(providersPage.selectors.cap('reasoning'))).toHaveAttribute(
        'aria-pressed',
        'false'
      );
      expect(providersPage.searchParams().get('q')).toBe('Provider');
      await expect(reset).toHaveAttribute('data-test-state', 'query');

      await providersPage.clickReset();
      expect(providersPage.searchParams().has('q')).toBe(false);
      await expect(reset).toHaveAttribute('data-test-state', 'none');
    });
  });
});
