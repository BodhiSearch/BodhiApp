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
export class ProvidersPage extends BasePage {
  selectors = {
    content: '[data-testid="explore-providers-content"]',
    list: '[data-testid="cat-prov-list"]',
    anyRow: '[data-testid^="cat-prov-row-"]',
    row: (slug) => `[data-testid="cat-prov-row-${slug}"]`,
    empty: '[data-testid="cat-prov-empty"]',
    pagination: '[data-testid="pagination"]',
    pageBtn: (n) => `[data-testid="pagination-page-${n}"]`,
    pageNext: '[data-testid="pagination-next"]',
    search: '[data-testid="cat-prov-search"] input',
    sort: (key) => `[data-testid="cat-prov-sort-${key}"]`,
    facets: '[data-testid="cat-prov-facets"]',
    cap: (id) => `[data-testid="cat-prov-cap-${id}"]`,
    fmt: (id) => `[data-testid="cat-prov-fmt-${id}"]`,
    pricing: (id) => `[data-testid="cat-prov-pricing-${id}"]`,
    labs: '[data-testid="cat-prov-labs"]',
    clearAll: '[data-testid="cat-prov-clear-all"]',
    // Detail rail. railPanel keys off the meta block (unique) to avoid matching the close button /
    // skeleton, which also share the `cat-prov-detail-` prefix.
    railPanel: '[data-testid="cat-prov-detail-meta"]',
    detailMeta: '[data-testid="cat-prov-detail-meta"]',
    detailModels: '[data-testid="cat-prov-models"]',
    docLink: '[data-testid="cat-prov-doc-link"]',
    detailClose: '[data-testid="cat-prov-detail-close"]',
  };

  /** Build N deterministic provider summaries (by model_count desc). Every 4th is a lab. */
  static makeProviders(n) {
    return Array.from({ length: n }, (_, i) => ({
      slug: `prov-${i}`,
      name: `Provider ${i}`,
      logo_url: `/api/v1/catalog/logos/prov-${i}.svg`,
      model_count: 1000 - i,
      rank: i + 1,
      is_lab: i % 4 === 0,
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
  async stubCatalog({ providers = ProvidersPage.makeProviders(31) } = {}) {
    const json = (route, body) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        headers: { 'access-control-allow-origin': '*' },
        body: JSON.stringify(body),
      });

    // Single route keyed off the catalog path; dispatch by URL shape so glob overlap can't
    // mis-route (Playwright's `?` matches any char, so a `providers?*` glob would also swallow
    // `/providers/{slug}/models`). One matcher, explicit branching = no ordering hazard.
    await this.page.route(/\/api\/v1\/catalog\/providers/, (route) => {
      const url = new URL(route.request().url());
      const path = url.pathname;
      const segments = path.split('/').filter(Boolean); // [..., providers, {slug?}, {models?}]
      const providersIdx = segments.indexOf('providers');
      const slug = segments[providersIdx + 1];
      const isModels = segments[providersIdx + 2] === 'models';

      // Provider list: /providers (no slug segment). Honors q + page/page_size; facet counts
      // reflect the filtered set so the sidebar shows per-query counts.
      if (!slug) {
        const q = url.searchParams.get('q')?.toLowerCase();
        let filtered = providers;
        if (q) filtered = filtered.filter((p) => `${p.slug} ${p.name}`.toLowerCase().includes(q));
        if (url.searchParams.get('is_lab') === 'true') filtered = filtered.filter((p) => p.is_lab);
        const page = Number(url.searchParams.get('page') ?? '1');
        const pageSize = Number(url.searchParams.get('page_size') ?? '30');
        const start = (page - 1) * pageSize;
        const slice = filtered.slice(start, start + pageSize);
        return json(route, {
          items: slice,
          page,
          page_size: pageSize,
          total: filtered.length,
          facets: {
            capability: { reasoning: filtered.length, tool_call: filtered.length, vision: filtered.length },
            api_format: { openai: filtered.length, anthropic: 0 },
          },
        });
      }

      // Provider models: /providers/{slug}/models
      if (isModels) {
        const src = providers.find((p) => p.slug === slug) ?? providers[0];
        return json(route, {
          items: [
            {
              model_id: `${src.slug}/model-a`,
              name: 'Model A',
              caps: ['reasoning', 'tool_call'],
              context_limit: 200000,
              output_limit: 64000,
              pricing: { input_per_m: 3, output_per_m: 15, cache_read_per_m: null, cache_write_per_m: null },
              status: null,
              modalities_in: ['text'],
              modalities_out: ['text'],
            },
          ],
          total: 1,
        });
      }
      const src = providers.find((p) => p.slug === slug) ?? providers[0];
      return json(route, {
        slug: src.slug,
        name: src.name,
        logo_url: src.logo_url,
        model_count: src.model_count,
        env: [`${src.slug.toUpperCase().replace(/-/g, '_')}_API_KEY`],
        npm: '@ai-sdk/openai-compatible',
        doc_url: `https://docs.${src.slug}.example.com`,
        api_base_url: src.api_base_url,
        provider_shape: src.provider_shape,
        bridge: {
          api_format: 'openai',
          base_url: src.api_base_url,
          base_url_source: 'modelsdev_api',
          base_url_requires_substitution: false,
        },
      });
    });
  }

  async navigateToProviders() {
    // Kill the rail view-transition so nothing detaches mid-animation.
    await this.page.emulateMedia({ reducedMotion: 'reduce' });
    await this.navigate('/ui/models/explore/providers/');
    await this.waitForSPAReady();
    await this.expectVisible(this.selectors.content);
    await expect(this.page.locator(this.selectors.content)).toHaveAttribute('data-pagestatus', 'ready');
  }

  /** The current URL's query string (without the leading '?'). */
  searchParams() {
    return new URL(this.page.url()).searchParams;
  }

  /**
   * A single URL search param, decoded. TanStack Router's default serializer JSON-encodes values, so
   * string scalars arrive quoted in the URL (is_lab="true", q="nano"). Unwrap the JSON so assertions
   * compare against the plain value; non-JSON values (already-bare) pass through.
   */
  urlParam(key) {
    const raw = this.searchParams().get(key);
    if (raw == null) return null;
    try {
      const parsed = JSON.parse(raw);
      return typeof parsed === 'string' ? parsed : String(parsed);
    } catch {
      return raw;
    }
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

  async gotoPage(n) {
    await this.page.locator(this.selectors.pageBtn(n)).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async nextPage() {
    await this.page.locator(this.selectors.pageNext).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async hasPagination() {
    return (await this.page.locator(this.selectors.pagination).count()) > 0;
  }

  /** Open the detail rail for a provider row; waits for the rail panel to render. */
  async openProvider(slug) {
    await this.page.locator(this.selectors.row(slug)).click();
    await this.waitForSPAReady();
    await this.page.locator(this.selectors.railPanel).waitFor({ state: 'visible' });
  }

  async closeRail() {
    await this.page.locator(this.selectors.detailClose).click();
  }

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

  async sortBy(key) {
    await this.page.locator(this.selectors.sort(key)).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async clickCapability(id) {
    await this.page.locator(this.selectors.cap(id)).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async clickPricing(id) {
    await this.page.locator(this.selectors.pricing(id)).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async clickLabs() {
    await this.page.locator(this.selectors.labs).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  /** Click the toolbar reset (3-state waterfall: filters → query → none). */
  async clickReset() {
    await this.page.locator(this.selectors.clearAll).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async clearAllFilters() {
    await this.clickReset();
  }

  async goBack() {
    await this.page.goBack();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async goForward() {
    await this.page.goForward();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }
}
