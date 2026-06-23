import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

/**
 * Explore · API Providers (screen-v2) page object.
 *
 * UNLIKE Explore · Local (which hits the live Reference API), this page object STUBS the external
 * catalog API via `page.route` so the E2E is deterministic — letting us assert exact provider names,
 * counts, and pagination. The stub mirrors the published catalog wire shape and the per-query
 * page/page_size pagination the real API uses.
 *
 * Call `stubCatalog()` BEFORE navigating so the route interceptor is installed when the page's
 * first request fires.
 */
export class ApiProvidersPage extends BasePage {
  selectors = {
    content: '[data-testid="explore-providers-content"]',
    resultbar: '[data-testid="cat-prov-resultbar"]',
    list: '[data-testid="cat-prov-list"]',
    anyRow: '[data-testid^="cat-prov-row-"]',
    row: (slug) => `[data-testid="cat-prov-row-${slug}"]`,
    empty: '[data-testid="cat-prov-empty"]',
    loadMore: '[data-testid="cat-prov-load-more"]',
  };

  /** Build N deterministic provider summaries (rank desc by model_count). */
  static makeProviders(n) {
    return Array.from({ length: n }, (_, i) => ({
      slug: `prov-${i}`,
      name: `Provider ${i}`,
      logo_url: `/api/v1/catalog/logos/prov-${i}.svg`,
      model_count: 1000 - i,
      rank: i + 1,
      api_base_url: `https://prov-${i}.example.com/v1`,
      provider_shape: 'openai-compatible',
      api_format_hint: 'openai',
      capabilities_summary: ['reasoning', 'tool_call', 'vision'],
      pricing_summary: { min_in_per_m: i === 0 ? 0 : 0.5, min_out_per_m: i === 0 ? 0 : 1.5 },
    }));
  }

  /**
   * Install the catalog stub. `providers` defaults to 31 rows so page-1 (page_size 30) leaves a
   * Load-more. Serves the page/page_size slice + total, mirroring the real API.
   */
  async stubCatalog({ providers = ApiProvidersPage.makeProviders(31) } = {}) {
    await this.page.route('**/api/v1/catalog/providers*', (route) => {
      const url = new URL(route.request().url());
      const page = Number(url.searchParams.get('page') ?? '1');
      const pageSize = Number(url.searchParams.get('page_size') ?? '30');
      const start = (page - 1) * pageSize;
      const slice = providers.slice(start, start + pageSize);
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        headers: { 'access-control-allow-origin': '*' },
        body: JSON.stringify({
          items: slice,
          page,
          page_size: pageSize,
          total: providers.length,
          facets: { capability: { reasoning: providers.length }, api_format: { openai: providers.length } },
        }),
      });
    });
  }

  async navigateToProviders() {
    // Kill the rail view-transition so nothing detaches mid-animation.
    await this.page.emulateMedia({ reducedMotion: 'reduce' });
    await this.navigate('/ui/models/explore/api-providers/');
    await this.waitForSPAReady();
    await this.expectVisible(this.selectors.content);
    await expect(this.page.locator(this.selectors.content)).toHaveAttribute('data-pagestatus', 'ready');
  }

  async waitForListSettled() {
    await this.page
      .locator(`${this.selectors.anyRow}, ${this.selectors.empty}`)
      .first()
      .waitFor({ state: 'visible' });
  }

  async getRowCount() {
    return this.page.locator(this.selectors.anyRow).count();
  }

  async loadMore() {
    await this.page.locator(this.selectors.loadMore).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async hasLoadMore() {
    return (await this.page.locator(this.selectors.loadMore).count()) > 0;
  }
}
