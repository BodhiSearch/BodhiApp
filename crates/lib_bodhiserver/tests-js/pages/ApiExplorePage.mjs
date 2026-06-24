import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

/**
 * Explore · API Models (screen-v2) page object.
 *
 * Like ApiProvidersPage, this STUBS the external catalog API via page.route for determinism (exact
 * counts, rows, prefill values). `stubCatalog()` must be called BEFORE navigating.
 */
export class ApiExplorePage extends BasePage {
  selectors = {
    content: '[data-testid="explore-api-content"]',
    resultbar: '[data-testid="cat-model-resultbar"]',
    list: '[data-testid="cat-model-list"]',
    anyRow: '[data-testid^="cat-model-row-"]',
    row: (slug, modelId) => `[data-testid="cat-model-row-${slug}-${modelId}"]`,
    empty: '[data-testid="cat-model-empty"]',
    pagination: '[data-testid="pagination"]',
    pageBtn: (n) => `[data-testid="pagination-page-${n}"]`,
    pageNext: '[data-testid="pagination-next"]',
    // Detail rail.
    railSpecs: '[data-testid="cat-model-detail-specs"]',
    railServedBy: '[data-testid="cat-model-servedby"]',
    configureCta: '[data-testid="cat-model-configure-cta"]',
    detailClose: '[data-testid="cat-model-detail-close"]',
    servedByToggle: (slug) => `[data-testid="cat-model-servedby-toggle-${slug}"]`,
    servedByDetail: (slug) => `[data-testid="cat-model-servedby-detail-${slug}"]`,
    servedByAdd: (slug) => `[data-testid="cat-model-servedby-add-${slug}"]`,
    // Search / sort / columns / facets.
    search: '[data-testid="cat-model-search"] input',
    sort: (key) => `[data-testid="cat-model-sort-${key}"]`,
    columnsBtn: '[data-testid="cat-model-columns"]',
    column: (key) => `[data-testid="cat-model-col-${key}"]`,
    facets: '[data-testid="cat-model-facets"]',
    cap: (id) => `[data-testid="cat-model-cap-${id}"]`,
    status: (id) => `[data-testid="cat-model-status-${id}"]`,
    ow: (id) => `[data-testid="cat-model-ow-${id}"]`,
    pricingFree: '[data-testid="cat-model-pricing-free"]',
    providerTrigger: '[data-testid="cat-model-provider-trigger"]',
    providerChip: (slug) => `[data-testid="cat-model-provider-chip-${slug}"]`,
    clearAll: '[data-testid="cat-model-clear-all"]',
  };

  /** Build N deterministic catalog models. */
  static makeModels(n) {
    return Array.from({ length: n }, (_, i) => ({
      slug: 'anthropic',
      model_id: `model-${i}`,
      name: `Model ${i}`,
      family: 'claude',
      context_limit: 200000,
      output_limit: 64000,
      pricing: { input_per_m: 3, output_per_m: 15, cache_read_per_m: 0.3, cache_write_per_m: 3.75 },
      caps: ['reasoning', 'tool_call', 'vision'],
      status: null,
      open_weights: false,
      modalities_in: ['text', 'image'],
      modalities_out: ['text'],
      provider_count: 2,
      release_date: '2025-09-29',
      last_updated: '2025-10-15',
    }));
  }

  static facets(n) {
    return {
      capability: { reasoning: n, tool_call: n, structured_output: 0, attachment: 0, vision: n },
      modality: { text: n, image: n, audio: 0, video: 0, pdf: 0 },
      status: { stable: n, alpha: 0, beta: 0, deprecated: 0 },
      provider: { anthropic: n },
      family: { claude: n },
      open_weights: { open: 0, closed: n },
    };
  }

  async stubCatalog({ models = ApiExplorePage.makeModels(31) } = {}) {
    const json = (route, body) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        headers: { 'access-control-allow-origin': '*' },
        body: JSON.stringify(body),
      });

    await this.page.route(/\/api\/v1\/catalog\/models/, (route) => {
      const url = new URL(route.request().url());
      const path = url.pathname;
      const segments = path.split('/').filter(Boolean);
      const modelsIdx = segments.indexOf('models');
      const hasDetail = segments.length > modelsIdx + 1; // /models/{slug}/{model_id}

      // Model detail: /models/{slug}/{model_id}
      if (hasDetail) {
        const slug = segments[modelsIdx + 1];
        const modelId = segments.slice(modelsIdx + 2).join('/');
        const src = models.find((m) => m.slug === slug && m.model_id === modelId) ?? models[0];
        return json(route, ApiExplorePage.detailFor(src));
      }

      // Model list: /models
      const q = url.searchParams.get('q')?.toLowerCase();
      let filtered = models;
      if (q) filtered = filtered.filter((m) => `${m.model_id} ${m.name}`.toLowerCase().includes(q));
      const pricing = url.searchParams.get('pricing');
      if (pricing === 'free') filtered = filtered.filter((m) => m.pricing.input_per_m === 0 && m.pricing.output_per_m === 0);
      if (pricing === 'paid') filtered = filtered.filter((m) => m.pricing.input_per_m > 0 || m.pricing.output_per_m > 0);
      const page = Number(url.searchParams.get('page') ?? '1');
      const pageSize = Number(url.searchParams.get('page_size') ?? '30');
      const start = (page - 1) * pageSize;
      return json(route, {
        items: filtered.slice(start, start + pageSize),
        page,
        page_size: pageSize,
        total: filtered.length,
        facets: ApiExplorePage.facets(filtered.length),
      });
    });

    // Provider detail backs the served-by inline-detail expansion in the rail.
    await this.page.route(/\/api\/v1\/catalog\/providers\/[^/]+$/, (route) => {
      const slug = new URL(route.request().url()).pathname.split('/').filter(Boolean).pop();
      return json(route, {
        slug,
        name: slug === 'openrouter' ? 'OpenRouter' : 'Anthropic',
        logo_url: null,
        model_count: 10,
        env: ['ANTHROPIC_API_KEY'],
        npm: '@anthropic-ai/sdk',
        doc_url: 'https://docs.anthropic.com',
        api_base_url: 'https://api.anthropic.com/v1',
        provider_shape: 'native',
        bridge: {
          api_format: 'anthropic',
          base_url: 'https://api.anthropic.com/v1',
          base_url_source: 'modelsdev_api',
          base_url_requires_substitution: false,
        },
      });
    });
  }

  static detailFor(m) {
    return {
      slug: m.slug,
      model_id: m.model_id,
      name: m.name,
      family: m.family,
      status: m.status,
      reasoning: m.caps.includes('reasoning'),
      tool_call: m.caps.includes('tool_call'),
      structured_output: m.caps.includes('structured_output'),
      attachment: m.caps.includes('attachment'),
      open_weights: m.open_weights,
      temperature: true,
      reasoning_options: null,
      context_limit: m.context_limit,
      output_limit: m.output_limit,
      modalities_in: m.modalities_in,
      modalities_out: m.modalities_out,
      release_date: m.release_date,
      last_updated: m.last_updated,
      knowledge_cutoff: '2025-03',
      pricing: {
        currency: 'USD',
        input_per_m: m.pricing.input_per_m,
        output_per_m: m.pricing.output_per_m,
        cache_read_per_m: m.pricing.cache_read_per_m,
        cache_write_per_m: m.pricing.cache_write_per_m,
        reasoning_per_m: null,
        input_audio_per_m: null,
        output_audio_per_m: null,
        pricing_source: 'modelsdev',
      },
      license: null,
      links: null,
      weights: null,
      benchmarks: null,
      served_by: [
        {
          slug: m.slug,
          name: 'Anthropic',
          logo_url: null,
          base_url: 'https://api.anthropic.com/v1',
          pricing: m.pricing,
        },
        {
          slug: 'openrouter',
          name: 'OpenRouter',
          logo_url: null,
          base_url: 'https://openrouter.ai/api/v1',
          pricing: m.pricing,
        },
      ],
      bridge: {
        api_format: 'anthropic',
        base_url: 'https://api.anthropic.com/v1',
        base_url_source: 'modelsdev_api',
        base_url_requires_substitution: false,
      },
    };
  }

  async navigateToModels() {
    await this.page.emulateMedia({ reducedMotion: 'reduce' });
    await this.navigate('/ui/models/explore/api/');
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

  async gotoPage(n) {
    await this.page.locator(this.selectors.pageBtn(n)).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async hasPagination() {
    return (await this.page.locator(this.selectors.pagination).count()) > 0;
  }

  /** Open the detail rail for a model row; waits for the spec grid to render. */
  async openModel(slug, modelId) {
    await this.page.locator(this.selectors.row(slug, modelId)).click();
    await this.waitForSPAReady();
    await this.page.locator(this.selectors.railSpecs).waitFor({ state: 'visible' });
  }

  async clickConfigure() {
    await this.page.locator(this.selectors.configureCta).click();
    await this.waitForSPAReady();
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

  async toggleColumn(key) {
    await this.page.locator(this.selectors.columnsBtn).click();
    await this.page.locator(this.selectors.column(key)).click();
    await this.page.keyboard.press('Escape');
  }

  async toggleFree() {
    await this.page.locator(this.selectors.pricingFree).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  /** Expand a served-by provider's inline connection detail (no navigation). */
  async expandServedBy(slug) {
    await this.page.locator(this.selectors.servedByToggle(slug)).click();
    await this.page.locator(this.selectors.servedByDetail(slug)).waitFor({ state: 'visible' });
  }

  async clickServedByAdd(slug) {
    await this.page.locator(this.selectors.servedByAdd(slug)).click();
    await this.waitForSPAReady();
  }

  /** Open the provider autocomplete and select an option by its slug (accessible name). */
  async selectProvider(slug) {
    await this.page.locator(this.selectors.providerTrigger).click();
    await this.page.getByRole('option', { name: slug }).click();
    await this.page.keyboard.press('Escape');
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async clearAllFilters() {
    await this.page.locator(this.selectors.clearAll).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }
}
