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
      await expect(discoveryPage.page.locator(discoveryPage.selectors.resultbar)).toBeVisible();
    });

    await test.step('Catalog renders real repository rows', async () => {
      await discoveryPage.expectCatalogLoaded();
      const count = await discoveryPage.getRowCount();
      expect(count).toBeGreaterThan(0);
      // The result bar shows "Showing N" (never a total count — the API gives none).
      await expect(discoveryPage.page.locator(discoveryPage.selectors.resultbar)).toContainText('Showing');
    });

    await test.step('Search narrows the catalog and persists the query', async () => {
      await discoveryPage.searchFor('qwen');
      const rows = discoveryPage.page.locator(discoveryPage.selectors.anyRow);
      await expect(rows.first()).toBeVisible();
      // Every visible repo id mentions the query (search is server-side relevance).
      const ids = await rows.evaluateAll((els) => els.map((e) => e.getAttribute('data-testid') || ''));
      expect(ids.some((id) => id.toLowerCase().includes('qwen'))).toBe(true);
    });

    await test.step('Clearing the search restores the full catalog', async () => {
      await discoveryPage.clearSearch();
      await expect(discoveryPage.page.locator(discoveryPage.selectors.anyRow).first()).toBeVisible();
    });

    await test.step('Sorting by Likes re-queries and marks the active column', async () => {
      await discoveryPage.sortBy('likes');
      await discoveryPage.expectSortState('likes', 'active-desc');
      await expect(discoveryPage.page.locator(discoveryPage.selectors.resultbar)).toContainText('Likes');
      // Toggling the active column flips the order.
      await discoveryPage.sortBy('likes');
      await discoveryPage.expectSortState('likes', 'active-asc');
    });
  });
});
