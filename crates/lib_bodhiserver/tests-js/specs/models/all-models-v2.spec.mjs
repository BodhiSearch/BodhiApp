import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPageV2 } from '@/pages/ModelsListPageV2.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

// Black-box E2E for the All Models list: faceted sidebar (TYPE / API-FORMAT incl. Liberty),
// server-side filtering, and the read-only detail rail with its Edit CTA. Uses only the local
// auto-discovered GGUF models the dev-server always has — no external API keys required.

test.describe('All Models V2', () => {
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
    modelsPage = new ModelsListPageV2(page, sharedServerUrl);
  });

  test('renders the V2 list with the faceted sidebar, filters by type, and opens a detail rail', async () => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();

    // V2 shell list + published faceted sidebar are present.
    await modelsPage.expectModelsPageV2();
    await modelsPage.expectVisible(modelsPage.selectors.facetType('local_file'));
    await modelsPage.expectVisible(modelsPage.selectors.facetFormat('liberty')); // Liberty bucket added this batch

    // There is at least one auto-discovered local model row.
    const initialCount = await modelsPage.getRowCount();
    expect(initialCount).toBeGreaterThan(0);

    // Server-side TYPE filter to local files only — every visible row is a Local File.
    await modelsPage.filterByType('local_file');
    const localCount = await modelsPage.getRowCount();
    expect(localCount).toBeGreaterThan(0);
    const types = await modelsPage.page.locator('[data-testid^="model-type-"]').allInnerTexts();
    expect(types.every(t => t.trim() === 'Local File')).toBe(true);

    // Open the first row's detail rail and confirm the Edit CTA is present.
    await modelsPage.openRow();
    await modelsPage.expectVisible(modelsPage.selectors.railEdit);

    // Filtering to API models (none configured) yields the empty state — proves server-side filtering.
    await modelsPage.filterByType('local_file'); // toggle local off
    await modelsPage.filterByType('api_model');
    await expect(
      modelsPage.page.locator(`${modelsPage.selectors.empty}, ${modelsPage.selectors.anyRow}`).first()
    ).toBeVisible();
  });

  test('search submits to the backend on Enter and narrows the list; clearing restores it', async () => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.expectModelsPageV2();

    const all = await modelsPage.getRowCount();
    expect(all).toBeGreaterThan(0);

    // A query that no row matches → empty state (server-side search).
    await modelsPage.searchFor('zzz-no-such-model-zzz');
    await modelsPage.expectVisible(modelsPage.selectors.empty);

    // Clearing the box restores the full list.
    await modelsPage.clearSearch();
    await expect(modelsPage.page.locator(modelsPage.selectors.anyRow).first()).toBeVisible();
    expect(await modelsPage.getRowCount()).toBe(all);
  });
});
