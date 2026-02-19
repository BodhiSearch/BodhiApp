/**
 * MCP Fixtures
 *
 * Provides test data factories for MCP server management tests.
 */

export class McpFixtures {
  static MCP_URL = 'https://mcp.deepwiki.com/mcp';
  static EXPECTED_TOOL = 'read_wiki_structure';
  static PLAYGROUND_TOOL = 'read_wiki_structure';
  static PLAYGROUND_PARAM = 'repoName';
  static PLAYGROUND_PARAMS = { repoName: 'facebook/react' };

  static TAVILY_URL = 'https://mcp.tavily.com/mcp/';
  static TAVILY_API_KEY = process.env.TAVILY_API_KEY;
  static TAVILY_EXPECTED_TOOL = 'tavily_search';
  static TAVILY_SEARCH_PARAMS = { query: 'BodhiApp AI', max_results: 1 };

  static createServerData() {
    const ts = Date.now();
    return {
      url: McpFixtures.MCP_URL,
      name: `DeepWiki-Server-${ts}`,
      description: 'DeepWiki MCP Server',
    };
  }

  static createLifecycleData() {
    const ts = Date.now();
    return {
      name: `DeepWiki-${ts}`,
      slug: `deepwiki-${ts}`,
      description: 'DeepWiki MCP instance',
    };
  }

  static createToolDiscoveryData() {
    const ts = Date.now();
    return {
      name: `DeepWiki-Tools-${ts}`,
      slug: `dw-tools-${ts}`,
    };
  }

  static createPlaygroundData() {
    const ts = Date.now();
    return {
      name: `Playground-${ts}`,
      slug: `pg-${ts}`,
      description: 'Playground test MCP',
    };
  }

  static createTavilyServerData() {
    const ts = Date.now();
    return {
      url: McpFixtures.TAVILY_URL,
      name: `Tavily-Server-${ts}`,
      description: 'Tavily MCP Server with header auth',
    };
  }

  static createTavilyInstanceData() {
    const ts = Date.now();
    return {
      name: `Tavily-${ts}`,
      slug: `tavily-${ts}`,
      description: 'Tavily search with header auth',
    };
  }
}
