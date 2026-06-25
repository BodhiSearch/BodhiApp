import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

/**
 * All Models page object (the V2 AppShell list — now the only Models screen).
 *
 * The screen is a master-detail list: a published faceted sidebar (TYPE / CAPABILITY / SIZE /
 * API-FORMAT incl. Liberty), selectable rows that open a read-only detail rail (Local File /
 * Model Alias / API Model / Fallback variants), and an Edit CTA on the rail that navigates to the
 * form routes. Creation entry points live in the left-sidebar nav (shell sub-pages), not on the
 * list itself.
 *
 * NOTE: the V2 screen has no inline delete, chat-from-list, preview modal, or refresh-metadata
 * affordances — that coverage is deferred (see docs/claude-plans/202606/screen-v2/techdebt.md).
 * Specs that need to create-then-chat select the model directly via ChatPage.
 */
export class ModelsListPageV2 extends BasePage {
  selectors = {
    content: '[data-testid="models-content"]',
    list: '[data-testid="table-list-models"]',
    facets: '[data-testid="models-facets"]',
    facetType: (id) => `[data-testid="models-facet-type-${id}"]`,
    facetCapability: (id) => `[data-testid="models-facet-capability-${id}"]`,
    facetFormat: (id) => `[data-testid="models-facet-format-${id}"]`,
    facetSize: '[data-testid="models-facet-size"]',
    row: (id) => `[data-testid="model-row-${id}"]`,
    rowTitle: (id) => `[data-testid="model-title-${id}"]`,
    rowType: (id) => `[data-testid="model-type-${id}"]`,
    anyRow: '[data-testid^="model-row-"]',
    rail: (id) => `[data-testid="model-detail-${id}"]`,
    railClose: '[data-testid="model-detail-close"]',
    railEdit: '[data-testid="model-detail-edit"]',
    railChat: '[data-testid="model-detail-chat"]',
    empty: '[data-testid="no-models"]',
    search: '[data-testid="models-search"] input',
    download: '[data-testid="models-downloads-button"]',
    heading: '[data-testid="models-heading"]',
    listhead: '[data-testid="cat-listhead"]',
    sortHeader: (col) => `[data-testid="cat-mymodel-sort-${col}"]`,
    columnsBtn: '[data-testid="cat-mymodel-columns"]',
    columnToggle: (key) => `[data-testid="cat-mymodel-col-${key}"]`,
    clearAll: '[data-testid="cat-mymodel-clear-all"]',
  };

  /** Current ?-search params on the My Models URL. */
  searchParams() {
    return new URL(this.page.url()).searchParams;
  }

  /** Click a sortable column header and wait for the list to settle. */
  async sortBy(col) {
    await this.page.locator(this.selectors.sortHeader(col)).click();
    await this.waitForSPAReady();
    await this.page
      .locator(`${this.selectors.anyRow}, ${this.selectors.empty}`)
      .first()
      .waitFor({ state: 'visible' });
  }

  async navigateToModels() {
    // Skip the rail view-transition so the close button doesn't detach mid-animation.
    await this.page.emulateMedia({ reducedMotion: 'reduce' });
    // Direct navigation (more robust than the shell-nav dropdown, which the scroll-area can intercept).
    await this.navigate('/ui/models/');
    await this.waitForSPAReady();
    await this.expectVisible(this.selectors.content);
    await expect(this.page.locator(this.selectors.content)).toHaveAttribute(
      'data-pagestatus',
      'ready'
    );
  }

  async expectModelsPageV2() {
    await this.expectVisible(this.selectors.content);
    await this.expectVisible(this.selectors.facets);
  }

  async expectToBeOnModelsListPage() {
    await this.expectToBeOnPage('/ui/models/');
  }

  // ── Create entry points (V2: sidebar-nav sub-pages; forms also work by direct URL) ──────────

  /** Go to the New API Model form. Drives the shell nav so the wiring stays covered. */
  async clickNewApiModel() {
    await this.navViaShell('models', 'new-api-model');
    await this.waitForUrl('/ui/models/api/new/');
    await this.waitForSPAReady();
  }

  /** Go to the New Local Model (alias) form. */
  async clickNewModelAlias() {
    await this.navViaShell('models', 'new-local-model');
    await this.waitForUrl('/ui/models/alias/new/');
    await this.waitForSPAReady();
  }

  /** Go to the New Model Router form. */
  async clickNewModelRouter() {
    await this.navViaShell('models', 'new-fallback-model');
    await this.waitForUrl('/ui/models/router/new/');
    await this.waitForSPAReady();
  }

  async waitForModelsToLoad() {
    await this.expectVisible(this.selectors.content);
    await this.page
      .locator(`${this.selectors.anyRow}, ${this.selectors.empty}`)
      .first()
      .waitFor({ state: 'visible' });
  }

  // ── List verification ──────────────────────────────────────────────────────────────────────

  /** Assert a row for the given alias id (api/router) or alias name (local) is present. */
  async expectModelInList(id) {
    await this.waitForSelector(this.selectors.row(id));
    await this.expectVisible(this.selectors.row(id));
  }

  /**
   * Verify an API model appears as a V2 row. Rows key on the alias id; the type slot shows the
   * provider (api_format) as a badge. The legacy alias/repo/filename cells do not exist in V2,
   * so this asserts the row + provider badge (and optionally the visible title).
   */
  async verifyApiModelInList(modelId, api_format = 'openai', _baseUrl = null, name = null) {
    await this.expectModelInList(modelId);
    await expect(this.page.locator(this.selectors.rowType(modelId))).toContainText(
      api_format.toUpperCase()
    );
    if (name) {
      await expect(this.page.locator(this.selectors.rowTitle(modelId))).toContainText(name);
    }
  }

  /** Verify an API model row exists (by id) without asserting provider/name. */
  async verifyApiModelExists(modelId) {
    await this.expectModelInList(modelId);
  }

  /**
   * Verify a model-router appears in the list. Router rows key on the router id; the type badge
   * reads "Router". When only the alias name is known, match the row by its visible title.
   */
  async verifyModelRouterInList(alias) {
    await this.waitForModelsToLoad();
    const row = this.page
      .locator(this.selectors.anyRow)
      .filter({ has: this.page.getByText(alias, { exact: true }) })
      .first();
    await expect(row).toBeVisible();
    await expect(row.locator('[data-testid^="model-type-"]')).toContainText('Router');
  }

  /**
   * Verify a local model alias appears in the list. Local rows key on the alias name; the row
   * subtitle shows the filename and the type badge reads "Local File" or "Model Alias".
   */
  async verifyLocalModelInList(alias, _repo, filename, source = 'user') {
    // V2 local rows show the filename as the subtitle; repo is not rendered on the row (it's in the
    // detail rail). The type badge distinguishes auto-discovered files from user aliases.
    await this.expectModelInList(alias);
    const row = this.page.locator(this.selectors.row(alias));
    await expect(row).toContainText(filename);
    const expectedBadge = source === 'model' ? 'Local File' : 'Model Alias';
    await expect(this.page.locator(this.selectors.rowType(alias))).toContainText(expectedBadge);
  }

  // ── Detail rail + edit ───────────────────────────────────────────────────────────────────────

  /** Open the detail rail for a given alias id (or the first row if omitted). */
  async openRow(id) {
    const row = id
      ? this.page.locator(this.selectors.row(id))
      : this.page.locator(this.selectors.anyRow).first();
    await row.click();
  }

  async expectRailVisible(id) {
    await this.expectVisible(this.selectors.rail(id));
  }

  /** Click the rail's Edit CTA and return after the SPA navigates to the form route. */
  async clickRailEdit() {
    await this.page.locator(this.selectors.railEdit).click();
    await this.waitForSPAReady();
  }

  /**
   * Edit an API model: open its row → rail → Edit CTA → API edit route. Asserts the id is carried
   * through on the edit URL (mirrors the legacy editModel contract).
   */
  async editModel(modelId) {
    await this.openRow(modelId);
    await this.expectRailVisible(modelId);
    await this.clickRailEdit();
    await this.waitForUrl('/ui/models/api/edit/');
    await this.waitForSPAReady();
    const currentUrl = new URL(this.page.url());
    expect(currentUrl.searchParams.get('id')).toBe(modelId);
  }

  /** Edit a local model alias via the rail; the alias row keys on its name. */
  async editLocalModel(alias) {
    await this.openRow(alias);
    await this.expectRailVisible(alias);
    await this.clickRailEdit();
    await this.waitForUrl('/ui/models/alias/edit/');
    await this.waitForSPAReady();
  }

  // ── Search / filter ──────────────────────────────────────────────────────────────────────────

  /** Click a TYPE facet pill; the list re-queries server-side. Waits for the list to settle. */
  async filterByType(id) {
    await this.page.locator(this.selectors.facetType(id)).click();
    await this.waitForSPAReady();
    await this.page
      .locator(`${this.selectors.anyRow}, ${this.selectors.empty}`)
      .first()
      .waitFor({ state: 'visible' });
  }

  /** Type a query into the always-visible search and submit it (Enter → backend `search`). */
  async searchFor(query) {
    const input = this.page.locator(this.selectors.search);
    await input.click();
    await input.fill(query);
    await input.press('Enter');
    await this.waitForSPAReady();
    await this.page
      .locator(`${this.selectors.anyRow}, ${this.selectors.empty}`)
      .first()
      .waitFor({ state: 'visible' });
  }

  /** Clear the search box (live-resets the server search to the full list). */
  async clearSearch() {
    const input = this.page.locator(this.selectors.search);
    await input.fill('');
    await this.waitForSPAReady();
  }

  async getRowCount() {
    return this.page.locator(this.selectors.anyRow).count();
  }

  // ── Parity helpers for ApiModelFormPage's create-id fallback ──────────────────────────────────

  /** Return the first list row (the V2 list is sorted by alias asc, not newest-first). */
  async getLatestModel() {
    await this.waitForModelsToLoad();
    const firstRow = this.page.locator(this.selectors.anyRow).first();
    await expect(firstRow).toBeVisible();
    return firstRow;
  }

  /** Extract the alias id from a row's `data-testid="model-row-<id>"`. */
  async getModelIdFromRow(row) {
    const testId = await row.getAttribute('data-testid');
    return testId?.replace(/^model-row-/, '') ?? null;
  }
}
