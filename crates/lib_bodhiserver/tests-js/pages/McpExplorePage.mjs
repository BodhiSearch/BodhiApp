import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

/**
 * Explore · MCP Servers (screen-v2) page object.
 *
 * Like ApiExplorePage, this STUBS the external MCP-server catalog (`/api/v1/mcp-servers`) via
 * page.route for determinism (exact counts, rows). `stubCatalog()` must be called BEFORE navigating.
 * Grows across phases (list → rail → facets → join) — Phase 1 covers list + search + pagination.
 */
export class McpExplorePage extends BasePage {
  selectors = {
    content: '[data-testid="explore-mcp-content"]',
    list: '[data-testid="cat-mcp-list"]',
    anyRow: '[data-testid^="cat-mcp-row-"]',
    row: id => `[data-testid="cat-mcp-row-${id}"]`,
    empty: '[data-testid="cat-mcp-empty"]',
    pagination: '[data-testid="pagination"]',
    pageBtn: n => `[data-testid="pagination-page-${n}"]`,
    search: '[data-testid="cat-mcp-search"] input',
    sort: key => `[data-testid="cat-mcp-sort-${key}"]`,
    // Detail rail.
    rail: id => `[data-testid="cat-mcp-detail-${id}"]`,
    railDescription: '[data-testid="cat-mcp-detail-description"]',
    railConnection: '[data-testid="cat-mcp-detail-connection"]',
    detailClose: '[data-testid="cat-mcp-detail-close"]',
    // Facets / reset / columns.
    facets: '[data-testid="cat-mcp-facets"]',
    auth: value => `[data-testid="cat-mcp-auth-${value}"]`,
    verified: '[data-testid="cat-mcp-verified"]',
    clearAll: '[data-testid="cat-mcp-clear-all"]',
    columnsBtn: '[data-testid="cat-mcp-columns"]',
    column: key => `[data-testid="cat-mcp-col-${key}"]`,
    empty: '[data-testid="cat-mcp-empty"]',
  };

  /** Build N deterministic catalog servers. */
  static makeServers(n) {
    return Array.from({ length: n }, (_, i) => ({
      id: `srv-${i}`,
      slug: `srv-${i}`,
      name: `Server ${i}`,
      description: `Description for server ${i}`,
      logo_url: null,
      endpoint_url: `https://mcp.example.com/srv-${i}/mcp`,
      transport: 'streamable-http',
      auth_type: 'http',
      category: null,
      external_link: `https://claude.com/connectors/srv-${i}`,
      verified: false,
      featured: i === 0,
    }));
  }

  async stubCatalog({ servers = McpExplorePage.makeServers(60) } = {}) {
    const json = (route, body) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        headers: { 'access-control-allow-origin': '*' },
        body: JSON.stringify(body),
      });

    await this.page.route(/\/api\/v1\/mcp-servers/, route => {
      const url = new URL(route.request().url());
      const segments = url.pathname.split('/').filter(Boolean);
      const idx = segments.indexOf('mcp-servers');
      const hasDetail = segments.length > idx + 1; // /mcp-servers/{id}

      if (hasDetail) {
        const id = segments[idx + 1];
        const src = servers.find(s => s.id === id) ?? servers[0];
        return json(route, McpExplorePage.detailFor(src));
      }

      const q = url.searchParams.get('q')?.toLowerCase();
      let filtered = servers;
      if (q) filtered = filtered.filter(s => `${s.name} ${s.description ?? ''}`.toLowerCase().includes(q));
      const auths = url.searchParams.getAll('auth');
      if (auths.length) filtered = filtered.filter(s => auths.includes(s.auth_type));
      const page = Number(url.searchParams.get('page') ?? '1');
      const pageSize = Number(url.searchParams.get('page_size') ?? '50');
      const start = (page - 1) * pageSize;
      return json(route, {
        items: filtered.slice(start, start + pageSize),
        page,
        page_size: pageSize,
        total: filtered.length,
        facets: { category: [], auth: ['http'] },
      });
    });
  }

  static detailFor(s) {
    return {
      ...s,
      details: `${s.name} — long description.`,
      publisher: null,
      tools: null,
      license: null,
      repo: null,
    };
  }

  async navigateToExplore() {
    await this.page.emulateMedia({ reducedMotion: 'reduce' });
    await this.navigate('/ui/mcps/explore/');
    await this.waitForSPAReady();
    await this.expectVisible(this.selectors.content);
    await expect(this.page.locator(this.selectors.content)).toHaveAttribute('data-pagestatus', 'ready');
  }

  async waitForListSettled() {
    await this.page.locator(`${this.selectors.anyRow}, ${this.selectors.empty}`).first().waitFor({ state: 'visible' });
  }

  async getRowCount() {
    return this.page.locator(this.selectors.anyRow).count();
  }

  async hasPagination() {
    return (await this.page.locator(this.selectors.pagination).count()) > 0;
  }

  async gotoPage(n) {
    await this.page.locator(this.selectors.pageBtn(n)).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
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

  searchParams() {
    return new URL(this.page.url()).searchParams;
  }

  /** Open the detail rail for a server row; waits for the connection grid to render. */
  async openServer(id) {
    await this.page.locator(this.selectors.row(id)).click();
    await this.waitForSPAReady();
    await this.page.locator(this.selectors.railConnection).waitFor({ state: 'visible' });
  }

  async closeRail() {
    await this.page.locator(this.selectors.detailClose).click();
    await this.waitForSPAReady();
  }

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

  async clickAuth(value) {
    await this.page.locator(this.selectors.auth(value)).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async toggleVerified() {
    await this.page.locator(this.selectors.verified).click();
    await this.waitForSPAReady();
  }

  async clearAllFilters() {
    await this.page.locator(this.selectors.clearAll).click();
    await this.waitForSPAReady();
    await this.waitForListSettled();
  }

  async toggleColumn(key) {
    await this.page.locator(this.selectors.columnsBtn).click();
    const item = this.page.locator(this.selectors.column(key));
    await item.waitFor({ state: 'visible' });
    await item.click();
    await this.page.keyboard.press('Escape');
  }
}
