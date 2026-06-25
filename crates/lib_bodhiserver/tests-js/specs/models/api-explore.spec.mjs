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
      await expect(modelsPage.page.locator(modelsPage.selectors.list)).toBeVisible();
    });

    await test.step('List renders the first page of model rows', async () => {
      expect(await modelsPage.getRowCount()).toBe(30);
      await expect(modelsPage.page.locator(modelsPage.selectors.row('anthropic', 'model-0'))).toContainText('Model 0');
    });

    await test.step('Numbered pager navigates to page 2', async () => {
      expect(await modelsPage.hasPagination()).toBe(true);
      await modelsPage.gotoPage(2);
      // 31 models, 30/page → page 2 has the single remaining row (wait for keepPreviousData to settle).
      await expect(modelsPage.page.locator(modelsPage.selectors.anyRow)).toHaveCount(1);
      await modelsPage.gotoPage(1);
      await expect(modelsPage.page.locator(modelsPage.selectors.anyRow)).toHaveCount(30);
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

    await test.step('Column picker hides the Family column', async () => {
      // Restore path (re-show) is covered deterministically by the component test; the E2E does a
      // single toggle to avoid a flaky Radix dropdown reopen.
      await expect(modelsPage.page.locator(modelsPage.selectors.sort('family'))).toBeVisible();
      await modelsPage.toggleColumn('family');
      await expect(modelsPage.page.locator(modelsPage.selectors.sort('family'))).toHaveCount(0);
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

    await test.step('Opening a model shows the rail with specs + Served-by, and writes ?select', async () => {
      await modelsPage.openModel('anthropic', 'model-0');
      const specs = modelsPage.page.locator(modelsPage.selectors.railSpecs);
      await expect(specs).toContainText('Context');
      await expect(specs).toContainText('Stable'); // null status → synthesized "Stable"
      await expect(modelsPage.page.locator(modelsPage.selectors.railServedBy)).toContainText('OpenRouter');
      // Selection is captured in the URL (composite slug/model_id).
      expect(modelsPage.urlParam('select')).toBe('anthropic/model-0');
    });

    await test.step('Reload restores the rail from ?select; closing strips it', async () => {
      await modelsPage.page.reload();
      await modelsPage.waitForSPAReady();
      await expect(modelsPage.page.locator(modelsPage.selectors.railSpecs)).toBeVisible();
      expect(modelsPage.urlParam('select')).toBe('anthropic/model-0');

      await modelsPage.closeRail();
      await expect(modelsPage.page.locator(modelsPage.selectors.railSpecs)).toHaveCount(0);
      expect(modelsPage.searchParams().has('select')).toBe(false);

      // Re-open so the following served-by / Add steps have the rail available.
      await modelsPage.openModel('anthropic', 'model-0');
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
      await expect(modelsPage.page.locator('[data-testid="base-url-input"]')).toHaveValue(
        'https://openrouter.ai/api/v1'
      );
    });

    await test.step('The Configure-in-Bodhi CTA is removed (the per-provider Add is the configure path)', async () => {
      await modelsPage.navigateToModels();
      await modelsPage.waitForListSettled();
      await modelsPage.openModel('anthropic', 'model-0');
      await expect(modelsPage.page.locator('[data-testid="cat-model-configure-cta"]')).toHaveCount(0);
    });

    await test.step('Served-by "All Models from Provider" filters in place via ?provider=', async () => {
      await modelsPage.expandServedBy('openrouter');
      await modelsPage.clickAllModelsFromProvider('openrouter');
      // Same route, now provider-filtered (the stub narrows by served_by → all 31 fixture models qualify).
      await expect(modelsPage.page).toHaveURL(/\/models\/explore\/api\/\?provider=/);
      await expect(modelsPage.page).toHaveURL(/openrouter/);
      await modelsPage.waitForListSettled();
      expect(await modelsPage.getRowCount()).toBeGreaterThan(0);
    });

    await test.step('Back restores the unfiltered URL and Forward re-applies the provider filter', async () => {
      await modelsPage.page.goBack();
      await modelsPage.waitForSPAReady();
      await expect(modelsPage.page).not.toHaveURL(/provider=/);
      await modelsPage.page.goForward();
      await modelsPage.waitForSPAReady();
      await expect(modelsPage.page).toHaveURL(/provider=/);
    });

    await test.step('Sort writes the URL and Back reverts the URL sort param', async () => {
      await modelsPage.navigateToModels();
      await modelsPage.waitForListSettled();
      await modelsPage.sortBy('price');
      await expect(modelsPage.page).toHaveURL(/sort=price/);
      await modelsPage.page.goBack();
      await modelsPage.waitForSPAReady();
      // Back reverts the URL sort param. (The header may still reflect the persisted sort preference,
      // which applies on a clean URL by design — so we assert on the URL, not the header state.)
      await expect(modelsPage.page).not.toHaveURL(/sort=price/);
    });

    await test.step('Served-by "View" opens the Providers page searching for the provider', async () => {
      await modelsPage.navigateToModels();
      await modelsPage.waitForListSettled();
      await modelsPage.openModel('anthropic', 'model-0');
      await modelsPage.expandServedBy('openrouter');
      await modelsPage.clickViewProvider('openrouter');
      await modelsPage.page.waitForURL(/\/models\/explore\/providers\//);
      await expect(modelsPage.page).toHaveURL(/q=OpenRouter/);
      // The providers search box is seeded from ?q=.
      await expect(modelsPage.page.locator('[data-testid="cat-prov-search"] input')).toHaveValue('OpenRouter');
    });

    await test.step('Search narrows the list and auto-ranks by relevance', async () => {
      await modelsPage.navigateToModels();
      await modelsPage.waitForListSettled();
      await modelsPage.searchFor('Model 7');
      // Wait for keepPreviousData to settle on the filtered result.
      await expect(modelsPage.page.locator(modelsPage.selectors.anyRow)).toHaveCount(1);
      await modelsPage.clearSearch();
      await expect(modelsPage.page.locator(modelsPage.selectors.anyRow)).toHaveCount(30);
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

    await test.step('Capability facet filters; toolbar reset clears it', async () => {
      await expect(modelsPage.page.locator(modelsPage.selectors.facets)).toBeVisible();
      await modelsPage.clickCapability('reasoning');
      await expect(modelsPage.page.locator(modelsPage.selectors.cap('reasoning'))).toHaveAttribute(
        'aria-pressed',
        'true'
      );
      await modelsPage.clearAllFilters();
      // The reset is always present (3-state); after clearing the only facet the pill is unpressed
      // and the reset returns to its inert 'none' state.
      await expect(modelsPage.page.locator(modelsPage.selectors.cap('reasoning'))).toHaveAttribute(
        'aria-pressed',
        'false'
      );
      await expect(modelsPage.page.locator(modelsPage.selectors.clearAll)).toHaveAttribute('data-test-state', 'none');
    });
  });
});
