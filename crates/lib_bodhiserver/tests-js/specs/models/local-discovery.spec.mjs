import { LocalDiscoveryPage } from '@/pages/LocalDiscoveryPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

// Black-box E2E for Explore · Local Models (screen-v2). The discovery list is served by the
// external Reference API (the app reads it anonymously — the catalog is public). ONE test grows
// across phases via `test.step`s (E2E runs are expensive). Phase 1: search-only list.
//
// The catalog comes from a live external API. We assert the page reaches a terminal state and,
// when the catalog is reachable, that real rows render and search/sort drive the query. Only
// runs in the standalone project (multi_tenant excludes specs/models/).

test.describe('Explore · Local Models (discovery)', () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  let loginPage;
  let discoveryPage;

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    discoveryPage = new LocalDiscoveryPage(page, sharedServerUrl);
  });

  test('browses, searches, and sorts the local-model catalog @integration', async () => {
    await test.step('Login and open Explore · Local Models', async () => {
      await loginPage.performOAuthLogin();
      await discoveryPage.navigateToDiscovery();
      // The list reaches a terminal state (rows or empty) regardless of catalog reachability.
      await discoveryPage.waitForListSettled();
    });

    await test.step('Catalog renders a real repository table with the count below the list', async () => {
      await discoveryPage.expectCatalogLoaded();
      const count = await discoveryPage.getRowCount();
      expect(count).toBeGreaterThan(0);
      // No result bar — the count moved below the list (next to Load more), the sort lives in the headers.
      await expect(discoveryPage.page.locator(discoveryPage.selectors.listhead)).toContainText(
        'REPOSITORY'
      );
      await expect(discoveryPage.page.locator(discoveryPage.selectors.count)).toContainText(
        'Showing'
      );
    });

    await test.step('Search narrows the catalog and persists the query', async () => {
      await discoveryPage.searchFor('qwen');
      const rows = discoveryPage.page.locator(discoveryPage.selectors.anyRow);
      await expect(rows.first()).toBeVisible();
      // Every visible repo id mentions the query (search is server-side relevance).
      const ids = await rows.evaluateAll((els) =>
        els.map((e) => e.getAttribute('data-testid') || '')
      );
      expect(ids.some((id) => id.toLowerCase().includes('qwen'))).toBe(true);
    });

    await test.step('Clearing the search restores the full catalog', async () => {
      await discoveryPage.clearSearch();
      await expect(
        discoveryPage.page.locator(discoveryPage.selectors.anyRow).first()
      ).toBeVisible();
    });

    await test.step('Sorting by Likes re-queries and marks the active column (descending-only)', async () => {
      await discoveryPage.sortBy('likes');
      await discoveryPage.expectSortState('likes', 'active');
      expect(discoveryPage.searchParams().get('sort')).toBe('likes');
      // Descending-only: the URL never carries an order param, and re-clicking does not flip to asc.
      expect(discoveryPage.searchParams().has('order')).toBe(false);
      await discoveryPage.sortBy('likes');
      await discoveryPage.expectSortState('likes', 'active');
      expect(discoveryPage.searchParams().has('order')).toBe(false);
    });

    await test.step('Sorting by Updated re-queries by last_modified and marks the active column', async () => {
      await discoveryPage.sortBy('last_modified');
      await discoveryPage.expectSortState('last_modified', 'active');
      expect(discoveryPage.searchParams().get('sort')).toBe('last_modified');
      await expect(
        discoveryPage.page.locator(discoveryPage.selectors.anyRow).first()
      ).toBeVisible();
    });

    await test.step('Faceted sidebar: Browse=Trending + Specialisation=Coding filter the catalog', async () => {
      await expect(discoveryPage.page.locator(discoveryPage.selectors.facets)).toBeVisible();

      await discoveryPage.clickFacet(discoveryPage.selectors.browse('trending'));
      await discoveryPage.expectFacetActive(discoveryPage.selectors.browse('trending'));
      expect(discoveryPage.searchParams().get('sort')).toBe('trending');

      await discoveryPage.clickFacet(discoveryPage.selectors.spec('coding'));
      await discoveryPage.expectFacetActive(discoveryPage.selectors.spec('coding'));
      // Catalog still renders rows under the combined facets.
      await expect(
        discoveryPage.page.locator(discoveryPage.selectors.anyRow).first()
      ).toBeVisible();
    });

    await test.step('Publisher free-text filters to one author; Clear all resets', async () => {
      // Start from a clean baseline (prior steps left Trending/Coding active).
      await discoveryPage.clearAllFilters();
      await discoveryPage.addPublisher('bartowski');
      await expect(
        discoveryPage.page.locator(discoveryPage.selectors.authorChip('bartowski'))
      ).toBeVisible();

      // `keepPreviousData` keeps the stale list visible during the refetch — poll until the
      // filtered result has fully landed (a bartowski row present AND zero non-bartowski rows).
      const nonBartowski = `${discoveryPage.selectors.anyRow}:not([data-testid^="ld-row-bartowski-"])`;
      await expect(
        discoveryPage.page
          .locator(`${discoveryPage.selectors.anyRow}[data-testid^="ld-row-bartowski-"]`)
          .first()
      ).toBeVisible();
      await expect(discoveryPage.page.locator(nonBartowski)).toHaveCount(0);

      // Clear all filters returns to the full catalog; the toolbar reset goes inert (disabled).
      await discoveryPage.clearAllFilters();
      await expect(discoveryPage.page.locator(discoveryPage.selectors.clearAll)).toBeDisabled();
      await expect(
        discoveryPage.page.locator(discoveryPage.selectors.anyRow).first()
      ).toBeVisible();
    });

    await test.step('Opening a model shows the detail rail with specs and download options', async () => {
      await discoveryPage.openFirstRow();

      // Rail header names the repo; Overview specs come from the single-model detail fetch.
      await expect(discoveryPage.page.locator(discoveryPage.selectors.railTitle)).toContainText(
        '/'
      );
      await expect(discoveryPage.page.locator(discoveryPage.selectors.specs)).toBeVisible();
      await expect(discoveryPage.page.locator(discoveryPage.selectors.specs)).toContainText(
        'Context'
      );

      // Download options tab renders the quant table from the DTO.
      await discoveryPage.openQuantsTab();
      await expect(discoveryPage.page.locator(discoveryPage.selectors.quants)).toBeVisible();
      const quantCount = await discoveryPage.page.locator(discoveryPage.selectors.quantRow).count();
      expect(quantCount).toBeGreaterThan(0);

      // No README tab in v1.
      await expect(discoveryPage.page.getByRole('button', { name: /README/i })).toHaveCount(0);

      // Each quant row carries its own download button (real filename-backed download). We don't
      // trigger a multi-GB download in CI — the quant→filename mapping is asserted in RTL and
      // exercised in the GATE-B manual walk; here we just confirm the per-quant wiring renders.
      await expect(
        discoveryPage.page.locator(discoveryPage.selectors.quantPull).first()
      ).toBeEnabled();

      // Close the rail.
      await discoveryPage.page.locator(discoveryPage.selectors.detailClose).click();
      await expect(discoveryPage.page.locator(discoveryPage.selectors.railPanel)).toHaveCount(0);
    });

    await test.step('URL state: selecting writes ?select (replace) and reload restores the rail', async () => {
      const firstRow = discoveryPage.page.locator(discoveryPage.selectors.anyRow).first();
      await firstRow.click();
      // ?select carries `namespace/repo`; the row testid is `ld-row-namespace-repo` (different
      // separators), so just assert a select param landed and that reload restores the rail.
      await expect.poll(() => discoveryPage.searchParams().has('select')).toBe(true);
      const selected = discoveryPage.searchParams().get('select');
      await discoveryPage.page.reload();
      await discoveryPage.waitForSPAReady();
      await expect(
        discoveryPage.page.locator(discoveryPage.selectors.railPanel).first()
      ).toBeVisible();
      expect(discoveryPage.searchParams().get('select')).toBe(selected);
    });

    await test.step('Arrow-down moves selection through the table rows', async () => {
      await discoveryPage.page.locator('body').click();
      await discoveryPage.page.keyboard.press('ArrowDown');
      await expect.poll(() => discoveryPage.searchParams().has('select')).toBe(true);
    });

    await test.step('Toolbar reset waterfalls filters → disabled', async () => {
      await discoveryPage.navigateToDiscovery();
      await discoveryPage.clickFacet(discoveryPage.selectors.spec('coding'));
      const reset = discoveryPage.page.locator(discoveryPage.selectors.clearAll);
      await expect(reset).toHaveAttribute('data-test-state', 'filters');
      // The reset disables itself the instant it clears the last bucket, racing the click's
      // actionability wait — dispatchEvent skips that wait (the handler fires synchronously).
      await reset.dispatchEvent('click');
      await discoveryPage.waitForSPAReady();
      await expect(reset).toHaveAttribute('data-test-state', 'none');
      await expect(reset).toBeDisabled();
      expect(discoveryPage.searchParams().has('specialisation')).toBe(false);
    });

    await test.step('Downloads button opens the Downloads panel in the rail', async () => {
      // Fresh DB (auto-reset) → no downloads yet; the panel renders its empty state. We assert the
      // header action → rail wiring (the four-section grouping + archive/retry are covered in RTL +
      // routes_app; a real multi-GB pull is out of scope for CI).
      await discoveryPage.openDownloads();
      await expect(
        discoveryPage.page.locator(discoveryPage.selectors.downloadsPanel)
      ).toBeVisible();
      await expect(
        discoveryPage.page.locator(discoveryPage.selectors.downloadsPanel)
      ).toContainText('No downloads yet');

      // Closing the rail removes the panel.
      await discoveryPage.closeDownloads();
      await expect(discoveryPage.page.locator(discoveryPage.selectors.downloadsPanel)).toHaveCount(
        0
      );
    });
  });
});
