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

    await test.step('Load more appends the next page without duplicates', async () => {
      expect(await modelsPage.hasLoadMore()).toBe(true);
      await modelsPage.loadMore();
      await expect(modelsPage.page.locator(modelsPage.selectors.resultbar)).toContainText('Showing 31 of 31');
      expect(await modelsPage.getRowCount()).toBe(31);
      expect(await modelsPage.hasLoadMore()).toBe(false);
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
  });
});
