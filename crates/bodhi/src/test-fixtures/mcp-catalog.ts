import type { ListMcpServersResponse, McpServerDetail, McpServerSummary } from '@bodhiapp/reference-api-types';

/** The reference-catalog list row shape. */
export function createMcpServerSummary(overrides: Partial<McpServerSummary> = {}): McpServerSummary {
  return {
    id: 'notion',
    slug: 'notion',
    name: 'Notion',
    description: 'Pages, databases, comments, workspace search',
    logo_url: 'https://svgl.app/library/notion.svg',
    endpoint_url: 'https://mcp.notion.com/mcp',
    transport: 'streamable-http',
    auth_type: 'http',
    category: null,
    external_link: 'https://claude.com/connectors/notion',
    verified: false,
    featured: true,
    ...overrides,
  };
}

export function createMcpServerList(): McpServerSummary[] {
  return [
    createMcpServerSummary(),
    createMcpServerSummary({
      id: 'linear',
      slug: 'linear',
      name: 'Linear',
      description: 'Issue tracking and project management',
      logo_url: 'https://svgl.app/library/linear.svg',
      endpoint_url: 'https://mcp.linear.app/mcp',
      external_link: 'https://claude.com/connectors/linear',
      featured: false,
    }),
    createMcpServerSummary({
      id: 'exa',
      slug: 'exa',
      name: 'Exa Search',
      description: 'AI-powered web search',
      logo_url: null,
      endpoint_url: 'https://mcp.exa.ai/mcp',
      external_link: 'https://claude.com/connectors/exa',
      verified: true,
      featured: false,
    }),
  ];
}

/** Envelope + facets. */
export function createMcpServersListResponse(overrides: Partial<ListMcpServersResponse> = {}): ListMcpServersResponse {
  const items = overrides.items ?? createMcpServerList();
  return {
    items,
    facets: { category: [], auth: ['http'] },
    page: 1,
    page_size: 50,
    total: items.length,
    ...overrides,
  };
}

/** List fields + enrichment fields, all null in v1. */
export function createMcpServerDetail(overrides: Partial<McpServerDetail> = {}): McpServerDetail {
  return {
    ...createMcpServerSummary(),
    details: 'Search, read and write pages & databases across your Notion workspace.',
    publisher: null,
    tools: null,
    license: null,
    repo: null,
    source: 'mcpservers.org',
    sources: ['mcpservers.org'],
    first_seen_at: 1782400000000,
    last_scraped_at: 1782400000000,
    ...overrides,
  };
}
