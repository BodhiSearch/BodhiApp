import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

/**
 * Explore · Local Models (discovery) page object — screen-v2.
 *
 * The discovery view is a search-driven catalog of downloadable GGUF repos served by the external
 * Reference API (the app calls it directly with the user's id_token). It is a Models nav sub-page
 * (no feature flag). Phase 1 covers the search-only list (rows, sort, "Showing N", Load more).
 */
export class LocalDiscoveryPage extends BasePage {
  selectors = {
    content: '[data-testid="local-discovery-content"]',
    listhead: '[data-testid="cat-listhead"]',
    count: '[data-testid="ld-count"]',
    list: '[data-testid="ld-list"]',
    anyRow: '[data-testid^="ld-row-"]',
    empty: '[data-testid="ld-empty"]',
    search: '[data-testid="ld-search"] input',
    sortDownloads: '[data-testid="ld-sort-downloads"]',
    sortLikes: '[data-testid="ld-sort-likes"]',
    sortUpdated: '[data-testid="ld-sort-last_modified"]',
    loadMore: '[data-testid="ld-load-more"]',
    facets: '[data-testid="ld-facets"]',
    browse: (id) => `[data-testid="ld-browse-${id}"]`,
    spec: (id) => `[data-testid="ld-spec-${id}"]`,
    task: (id) => `[data-testid="ld-task-${id}"]`,
    license: (id) => `[data-testid="ld-license-${id}"]`,
    authorInput: '[data-testid="ld-author-input"]',
    authorChip: (name) => `[data-testid="ld-author-chip-${name}"]`,
    clearAll: '[data-testid="ld-clear-all"]',
    // Detail rail (the shell renders the published rail header + panel into the real DOM).
    railPanel: '[data-testid^="ld-detail-"]',
    detailClose: '[data-testid="ld-detail-close"]',
    railTitle: '.dp-head-title',
    tabOverview: '[data-testid="ld-tab-overview"]',
    tabQuants: '[data-testid="ld-tab-quants"]',
    specs: '[data-testid="ld-detail-specs"]',
    quants: '[data-testid="ld-quants"]',
    quantRow: '[data-testid^="ld-quant-"]:not([data-testid^="ld-quant-pull-"])',
    quantPull: '[data-testid^="ld-quant-pull-"]',
    // Downloads panel (header action → right rail).
    downloadsButton: '[data-testid="ld-downloads-button"]',
    downloadsBadge: '[data-testid="ld-downloads-badge"]',
    downloadsPanel: '[data-testid="ld-downloads-panel"]',
    downloadsClose: '[data-testid="ld-downloads-close"]',
  };

  async navigateToDiscovery() {
    // Kill the rail view-transition so nothing detaches mid-animation (memory carry-forward).
    await this.page.emulateMedia({ reducedMotion: 'reduce' });
    await this.navigate('/ui/models/explore/local/');
    await this.waitForSPAReady();
    await this.expectVisible(this.selectors.content);
    await expect(this.page.locator(this.selectors.content)).toHaveAttribute('data-pagestatus', 'ready');
  }

  /** Current ?-search params on the Local Models URL. */
  searchParams() {
    return new URL(this.page.url()).searchParams;
  }

  /** Wait for the list to settle on rows OR the empty state (avoids racing the in-flight query). */
  async waitForListSettled() {
    await this.page
      .locator(`${this.selectors.anyRow}, ${this.selectors.empty}`)
      .first()
      .waitFor({ state: 'visible' });
  }

  async expectCatalogLoaded() {
    await this.waitForListSettled();
    await expect(this.page.locator(this.selectors.anyRow).first()).toBeVisible();
  }

  async getRowCount() {
    return this.page.locator(this.selectors.anyRow).count();
  }

  /** Type a query and submit (Enter → `q` param; search disables cursor pagination). */
  async searchFor(query) {
    const input = this.page.locator(this.selectors.search);
    await input.click();
    await input.fill(query);
    await input.press('Enter');
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async clearSearch() {
    await this.page.locator(this.selectors.search).fill('');
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  /** Maps a column id to its sort-header selector. */
  sortSelector(column) {
    if (column === 'likes') return this.selectors.sortLikes;
    if (column === 'last_modified') return this.selectors.sortUpdated;
    return this.selectors.sortDownloads;
  }

  /** Click a sortable column header; waits for the list to re-settle. */
  async sortBy(column) {
    await this.page.locator(this.sortSelector(column)).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async expectSortState(column, state) {
    await expect(this.page.locator(this.sortSelector(column))).toHaveAttribute('data-test-state', state);
  }

  async loadMore() {
    await this.page.locator(this.selectors.loadMore).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async hasLoadMore() {
    return (await this.page.locator(this.selectors.loadMore).count()) > 0;
  }

  /** Click a sidebar facet pill (selector locator) and wait for the list to re-settle. */
  async clickFacet(locator) {
    await this.page.locator(locator).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async expectFacetActive(locator, active = true) {
    await expect(this.page.locator(locator)).toHaveAttribute('aria-pressed', String(active));
  }

  /** Type a publisher into the free-text Publisher input and commit (Enter → author chip). */
  async addPublisher(name) {
    const input = this.page.locator(this.selectors.authorInput);
    await input.click();
    await input.fill(name);
    await input.press('Enter');
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async clearAllFilters() {
    await this.page.locator(this.selectors.clearAll).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  /** Open the detail rail for the first row (or a given row locator). */
  async openFirstRow() {
    await this.page.locator(this.selectors.anyRow).first().click();
    await this.waitForSPAReady();
  }

  async openQuantsTab() {
    await this.page.locator(this.selectors.tabQuants).click();
  }

  /** Open the Downloads panel via the header action; waits for the rail panel to render. */
  async openDownloads() {
    await this.page.locator(this.selectors.downloadsButton).click();
    await this.waitForSPAReady();
    await this.page.locator(this.selectors.downloadsPanel).waitFor({ state: 'visible' });
  }

  async closeDownloads() {
    await this.page.locator(this.selectors.downloadsClose).click();
  }
}
