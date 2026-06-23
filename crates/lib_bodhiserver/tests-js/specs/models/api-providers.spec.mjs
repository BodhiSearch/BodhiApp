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

    await test.step('Load more appends the next page without duplicates', async () => {
      expect(await providersPage.hasLoadMore()).toBe(true);
      await providersPage.loadMore();
      await expect(providersPage.page.locator(providersPage.selectors.resultbar)).toContainText('Showing 31 of 31');
      expect(await providersPage.getRowCount()).toBe(31);
      // Last page consumed → Load more gone.
      expect(await providersPage.hasLoadMore()).toBe(false);
    });
  });
});
