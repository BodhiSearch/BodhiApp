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
    expect(types.every((t) => t.trim() === 'Local File')).toBe(true);

    // Open the first local-file row's detail rail. Local files are read-only, so the footer is a
    // single "Chat with Model" CTA (no Edit). Its href targets the chat route with the alias
    // pre-selected via ?model=, and clicking it navigates there.
    await modelsPage.openRow();
    await modelsPage.expectVisible(modelsPage.selectors.railChat);
    await expect(modelsPage.page.locator(modelsPage.selectors.railEdit)).toHaveCount(0);

    const chatHref = await modelsPage.page
      .locator(modelsPage.selectors.railChat)
      .getAttribute('href');
    expect(chatHref).toContain('/ui/chat/?');
    expect(chatHref).toContain('model=');

    await modelsPage.page.locator(modelsPage.selectors.railChat).click();
    await modelsPage.page.waitForURL(/\/ui\/chat\/\?.*model=/);

    await modelsPage.navigateToModels();
    await modelsPage.expectModelsPageV2();

    // Filtering to API models (none configured) yields the empty state — proves server-side filtering.
    // (Page was reloaded above, so no prior type filter is set.)
    await modelsPage.filterByType('api_model');
    await expect(
      modelsPage.page
        .locator(`${modelsPage.selectors.empty}, ${modelsPage.selectors.anyRow}`)
        .first()
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

  test('V2 parity: table layout, URL-synced filters/sort/select, back/forward, reset', async () => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();

    await test.step('renders a semantic table with the universal columns (no count heading)', async () => {
      await modelsPage.expectVisible(modelsPage.selectors.listhead);
      await expect(modelsPage.page.locator(modelsPage.selectors.listhead)).toContainText('NAME');
      await expect(modelsPage.page.locator(modelsPage.selectors.listhead)).toContainText(
        'PROVIDER / REPO'
      );
      // The count heading was removed to save vertical space.
      await expect(modelsPage.page.locator(modelsPage.selectors.heading)).toHaveCount(0);
      expect(
        await modelsPage.page.locator(`tr${modelsPage.selectors.anyRow}`).first().isVisible()
      ).toBe(true);
    });

    await test.step('a filter is pushed to the URL and Back reverts it', async () => {
      await modelsPage.filterByType('local_file');
      expect(modelsPage.searchParams().get('type')).toContain('local_file');
      await modelsPage.page.goBack();
      await modelsPage.waitForSPAReady();
      expect(modelsPage.searchParams().has('type')).toBe(false);
    });

    await test.step('the column picker hides the Provider column', async () => {
      await modelsPage.page.locator(modelsPage.selectors.columnsBtn).click();
      await modelsPage.page.locator(modelsPage.selectors.columnToggle('provider')).click();
      await expect(modelsPage.page.locator(modelsPage.selectors.listhead)).not.toContainText(
        'PROVIDER / REPO'
      );
      await modelsPage.page.keyboard.press('Escape');
    });

    await test.step('clicking the Name header writes ?sort=name and marks it active', async () => {
      await modelsPage.sortBy('name');
      expect(modelsPage.searchParams().get('sort')).toBe('name');
      await expect(
        modelsPage.page.locator(modelsPage.selectors.sortHeader('name'))
      ).toHaveAttribute('data-test-state', 'active');
    });

    await test.step('selecting a row writes ?select (replace) and reload restores the rail', async () => {
      const firstRow = modelsPage.page.locator(modelsPage.selectors.anyRow).first();
      const id = await modelsPage.getModelIdFromRow(firstRow);
      await firstRow.click();
      await expect.poll(() => modelsPage.searchParams().get('select')).toBe(id);
      await modelsPage.page.reload();
      await modelsPage.waitForSPAReady();
      await modelsPage.expectVisible(modelsPage.selectors.rail(id));
    });

    await test.step('arrow-down moves selection through rows', async () => {
      await modelsPage.page.locator('body').click();
      await modelsPage.page.keyboard.press('ArrowDown');
      await expect.poll(() => modelsPage.searchParams().has('select')).toBe(true);
    });

    await test.step('toolbar reset waterfalls filters → disabled', async () => {
      await modelsPage.navigateToModels();
      await modelsPage.filterByType('local_file');
      await expect(modelsPage.page.locator(modelsPage.selectors.clearAll)).toHaveAttribute(
        'data-test-state',
        'filters'
      );
      await modelsPage.page.locator(modelsPage.selectors.clearAll).click();
      await modelsPage.waitForSPAReady();
      await expect(modelsPage.page.locator(modelsPage.selectors.clearAll)).toHaveAttribute(
        'data-test-state',
        'none'
      );
      await expect(modelsPage.page.locator(modelsPage.selectors.clearAll)).toBeDisabled();
    });
  });
});
