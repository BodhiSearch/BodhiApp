import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

/**
 * All Models (V2) page object. The V2 screen is an AppShell list with a published faceted
 * sidebar (TYPE / CAPABILITY / SIZE / API-FORMAT incl. Liberty), selectable rows that open a
 * read-only detail rail (Local File / Model Alias / API Model / Fallback variants), and an Edit
 * CTA on the rail that navigates to the (still-V1) form routes.
 *
 * The V2 screen is gated by the `models` per-screen flag (default off). Enable it via
 * {@link enableV2Flag} (an init script) BEFORE the first navigation.
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
    rowType: (id) => `[data-testid="model-type-${id}"]`,
    anyRow: '[data-testid^="model-row-"]',
    rail: (id) => `[data-testid="model-detail-${id}"]`,
    railClose: '[data-testid="model-detail-close"]',
    railEdit: '[data-testid="model-detail-edit"]',
    empty: '[data-testid="no-models"]',
    search: '[data-testid="models-search"] input',
    download: '[data-testid="models-download"]',
  };

  /** Set the `models` V2 flag in localStorage before any page script runs. */
  async enableV2Flag() {
    await this.page.addInitScript(() => {
      window.localStorage.setItem('bodhi.ui-v2.models', 'true');
    });
  }

  async navigateToModels() {
    // Skip the rail view-transition so the close button doesn't detach mid-animation.
    await this.page.emulateMedia({ reducedMotion: 'reduce' });
    // Direct navigation (more robust than the shell-nav dropdown, which the scroll-area can intercept).
    await this.navigate('/ui/models/');
    await this.waitForSPAReady();
    await this.expectVisible(this.selectors.content);
    await expect(this.page.locator(this.selectors.content)).toHaveAttribute('data-pagestatus', 'ready');
  }

  async expectModelsPageV2() {
    await this.expectVisible(this.selectors.content);
    await this.expectVisible(this.selectors.facets);
  }

  /** Click a TYPE facet pill; the list re-queries server-side. Waits for the list to settle. */
  async filterByType(id) {
    await this.page.locator(this.selectors.facetType(id)).click();
    await this.waitForSPAReady();
    // The facet triggers a server-side refetch; wait for the list to settle on rows OR the
    // empty state before the caller reads counts (avoids racing the in-flight query).
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

  /** Open the detail rail for a given alias id (or the first row if omitted). */
  async openRow(id) {
    const row = id ? this.page.locator(this.selectors.row(id)) : this.page.locator(this.selectors.anyRow).first();
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

  async getRowCount() {
    return this.page.locator(this.selectors.anyRow).count();
  }
}
