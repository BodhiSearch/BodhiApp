import { ApiExplorePage } from '@/pages/ApiExplorePage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

// Black-box E2E for Explore · API Models (screen-v2). Catalog STUBBED via page.route for
// determinism. ONE test grows across phases via test.step. Phase A1: list + page-based Load more.
// Standalone-only (multi_tenant excludes specs/models/).

test.describe('Explore · API Models', () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  let loginPage;
  let modelsPage;

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    modelsPage = new ApiExplorePage(page, sharedServerUrl);
  });

  test('browses the API-model catalog @integration', async () => {
    await test.step('Login, stub the catalog, and open Explore · API Models', async () => {
      await loginPage.performOAuthLogin();
      await modelsPage.stubCatalog();
      await modelsPage.navigateToModels();
      await modelsPage.waitForListSettled();
      await expect(modelsPage.page.locator(modelsPage.selectors.resultbar)).toBeVisible();
    });

    await test.step('List renders model rows with "Showing X of TOTAL"', async () => {
      expect(await modelsPage.getRowCount()).toBe(30);
      await expect(modelsPage.page.locator(modelsPage.selectors.resultbar)).toContainText('Showing 30 of 31');
      await expect(modelsPage.page.locator(modelsPage.selectors.row('anthropic', 'model-0'))).toContainText('Model 0');
    });

    await test.step('Numbered pager navigates to page 2', async () => {
      expect(await modelsPage.hasPagination()).toBe(true);
      await modelsPage.gotoPage(2);
      // 31 models, 30/page → page 2 has the single remaining row.
      await expect(modelsPage.page.locator(modelsPage.selectors.resultbar)).toContainText('Showing 1 of 31');
      expect(await modelsPage.getRowCount()).toBe(1);
      await modelsPage.gotoPage(1);
    });

    await test.step('Opening a model shows the rail with specs + Served-by', async () => {
      await modelsPage.openModel('anthropic', 'model-0');
      const specs = modelsPage.page.locator(modelsPage.selectors.railSpecs);
      await expect(specs).toContainText('Context');
      await expect(specs).toContainText('Stable'); // null status → synthesized "Stable"
      await expect(modelsPage.page.locator(modelsPage.selectors.railServedBy)).toContainText('Anthropic');
    });

    await test.step('Configure in Bodhi prefills the create form from the bridge', async () => {
      await modelsPage.clickConfigure();
      // Lands on the New API Model page with the bridge prefill applied.
      await modelsPage.page.waitForURL(/\/models\/api\/new\//);
      await expect(modelsPage.page.locator('[data-testid="new-api-model-page"]')).toBeVisible();
      // base_url prefilled from the stub bridge (anthropic), api_key left empty.
      await expect(modelsPage.page.locator('[data-testid="base-url-input"]')).toHaveValue(
        'https://api.anthropic.com/v1'
      );
    });

    await test.step('Back on the catalog: search narrows and clearing restores the list', async () => {
      await modelsPage.navigateToModels();
      await modelsPage.waitForListSettled();
      await modelsPage.searchFor('Model 7');
      await expect(modelsPage.page.locator(modelsPage.selectors.resultbar)).toContainText('Showing 1 of 1');
      await modelsPage.clearSearch();
      await expect(modelsPage.page.locator(modelsPage.selectors.resultbar)).toContainText('Showing 30 of 31');
    });

    await test.step('Sort by Input price marks the active column and toggles asc/desc', async () => {
      await modelsPage.sortBy('price');
      await expect(modelsPage.page.locator(modelsPage.selectors.sort('price'))).toHaveAttribute(
        'data-test-state',
        'active'
      );
      // price is naturally ascending; re-clicking the active column flips to descending.
      await expect(modelsPage.page.locator(modelsPage.selectors.resultbar)).toContainText('(asc)');
      await modelsPage.sortBy('price');
      await expect(modelsPage.page.locator(modelsPage.selectors.resultbar)).toContainText('(desc)');
    });

    await test.step('Capability facet filters; Clear all resets', async () => {
      await expect(modelsPage.page.locator(modelsPage.selectors.facets)).toBeVisible();
      await modelsPage.clickCapability('reasoning');
      await expect(modelsPage.page.locator(modelsPage.selectors.cap('reasoning'))).toHaveAttribute(
        'aria-pressed',
        'true'
      );
      await modelsPage.clearAllFilters();
      await expect(modelsPage.page.locator(modelsPage.selectors.clearAll)).toHaveCount(0);
    });
  });
});
