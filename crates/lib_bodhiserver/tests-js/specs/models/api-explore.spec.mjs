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

    await test.step('Family + Updated are their own sortable columns; capabilities render as a subheading', async () => {
      const row = modelsPage.page.locator(modelsPage.selectors.row('anthropic', 'model-0'));
      await expect(row).toContainText('claude'); // Family column value
      await expect(row).toContainText('Reasoning'); // capabilities subheading under the name
      // Updated is a sortable header; Family adopts its natural ascending direction on first click.
      await modelsPage.sortBy('family');
      await expect(modelsPage.page.locator(modelsPage.selectors.sort('family'))).toHaveAttribute(
        'data-test-state',
        'active'
      );
      await modelsPage.sortBy('updated');
      await expect(modelsPage.page.locator(modelsPage.selectors.sort('updated'))).toHaveAttribute(
        'data-test-state',
        'active'
      );
    });

    await test.step('Column picker hides and restores the Family column', async () => {
      await expect(modelsPage.page.locator(modelsPage.selectors.sort('family'))).toBeVisible();
      await modelsPage.toggleColumn('family');
      await expect(modelsPage.page.locator(modelsPage.selectors.sort('family'))).toHaveCount(0);
      await modelsPage.toggleColumn('family');
      await expect(modelsPage.page.locator(modelsPage.selectors.sort('family'))).toBeVisible();
    });

    await test.step('Input-price column toggles asc/desc and marks the active header', async () => {
      await modelsPage.sortBy('price');
      const priceHeader = modelsPage.page.locator(modelsPage.selectors.sort('price'));
      await expect(priceHeader).toHaveAttribute('data-test-state', 'active');
      // price is naturally ascending → arrow-up; re-click flips to descending → arrow-down.
      await expect(priceHeader.locator('svg.lucide-arrow-up')).toBeVisible();
      await modelsPage.sortBy('price');
      await expect(priceHeader.locator('svg.lucide-arrow-down')).toBeVisible();
    });

    await test.step('Opening a model shows the rail with specs + Served-by', async () => {
      await modelsPage.openModel('anthropic', 'model-0');
      const specs = modelsPage.page.locator(modelsPage.selectors.railSpecs);
      await expect(specs).toContainText('Context');
      await expect(specs).toContainText('Stable'); // null status → synthesized "Stable"
      await expect(modelsPage.page.locator(modelsPage.selectors.railServedBy)).toContainText('OpenRouter');
    });

    await test.step('Served-by provider reveals inline connection detail (no navigation)', async () => {
      await modelsPage.expandServedBy('openrouter');
      const detail = modelsPage.page.locator(modelsPage.selectors.servedByDetail('openrouter'));
      await expect(detail).toContainText('Base URL');
      await expect(detail).toContainText('API format');
      // Still on the Explore page — no route change to the Providers page.
      await expect(modelsPage.page).toHaveURL(/\/models\/explore\/api\//);
    });

    await test.step('Per-provider Add jumps to the create form (api_format=openai, provider base_url)', async () => {
      await modelsPage.clickServedByAdd('openrouter');
      await modelsPage.page.waitForURL(/\/models\/api\/new\//);
      await expect(modelsPage.page).toHaveURL(/api_format=openai/);
      await expect(modelsPage.page).toHaveURL(/base_url=https%3A%2F%2Fopenrouter\.ai%2Fapi%2Fv1/);
      await expect(modelsPage.page.locator('[data-testid="base-url-input"]')).toHaveValue('https://openrouter.ai/api/v1');
    });

    await test.step('Configure in Bodhi prefills the create form from the bridge', async () => {
      await modelsPage.navigateToModels();
      await modelsPage.waitForListSettled();
      await modelsPage.openModel('anthropic', 'model-0');
      await modelsPage.clickConfigure();
      await modelsPage.page.waitForURL(/\/models\/api\/new\//);
      await expect(modelsPage.page.locator('[data-testid="new-api-model-page"]')).toBeVisible();
      // base_url prefilled from the stub bridge (anthropic), api_key left empty.
      await expect(modelsPage.page.locator('[data-testid="base-url-input"]')).toHaveValue(
        'https://api.anthropic.com/v1'
      );
    });

    await test.step('Search narrows the list and auto-ranks by relevance', async () => {
      await modelsPage.navigateToModels();
      await modelsPage.waitForListSettled();
      await modelsPage.searchFor('Model 7');
      await expect(modelsPage.page.locator(modelsPage.selectors.resultbar)).toContainText('Showing 1 of 1');
      await modelsPage.clearSearch();
      await expect(modelsPage.page.locator(modelsPage.selectors.resultbar)).toContainText('Showing 30 of 31');
    });

    await test.step('Free pins pricing=free and pares the list to free models', async () => {
      // Fixture models are paid ($3/$15) → Free yields an empty list (the stub has no $0 models).
      await modelsPage.toggleFree();
      await expect(modelsPage.page.locator(modelsPage.selectors.pricingFree)).toHaveAttribute('aria-pressed', 'true');
      await expect(modelsPage.page.locator(modelsPage.selectors.empty)).toBeVisible();
      await modelsPage.toggleFree();
      await modelsPage.waitForListSettled();
    });

    await test.step('Provider autocomplete selects from facet options and shows a removable chip', async () => {
      await modelsPage.selectProvider('anthropic');
      await expect(modelsPage.page.locator(modelsPage.selectors.providerChip('anthropic'))).toBeVisible();
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
