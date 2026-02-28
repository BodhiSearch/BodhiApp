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
  static TAVILY_API_KEY = process.env.INTEG_TEST_TAVILY_API_KEY;
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

  static OAUTH_MCP_PORT = process.env.TEST_MCP_OAUTH_PORT || '55174';
  static OAUTH_MCP_URL = `http://localhost:${McpFixtures.OAUTH_MCP_PORT}/mcp`;
  static OAUTH_SERVER_BASE = `http://localhost:${McpFixtures.OAUTH_MCP_PORT}`;
  static OAUTH_CLIENT_ID = process.env.TEST_MCP_OAUTH_CLIENT_ID || 'test-mcp-client-id';
  static OAUTH_CLIENT_SECRET = process.env.TEST_MCP_OAUTH_CLIENT_SECRET || 'test-mcp-client-secret';
  static OAUTH_EXPECTED_TOOL = 'echo';

  static createOAuthServerData() {
    const ts = Date.now();
    return {
      url: McpFixtures.OAUTH_MCP_URL,
      name: `OAuth-MCP-Server-${ts}`,
      description: 'Test MCP OAuth Server',
    };
  }

  static createOAuthInstanceData() {
    const ts = Date.now();
    return {
      name: `OAuth-MCP-${ts}`,
      slug: `oauth-mcp-${ts}`,
      description: 'OAuth-authenticated MCP instance',
    };
  }

  static createOAuthConfigData() {
    return {
      name: 'OAuth Pre-Reg Config',
      client_id: McpFixtures.OAUTH_CLIENT_ID,
      client_secret: McpFixtures.OAUTH_CLIENT_SECRET,
      authorization_endpoint: `${McpFixtures.OAUTH_SERVER_BASE}/authorize`,
      token_endpoint: `${McpFixtures.OAUTH_SERVER_BASE}/token`,
      scopes: 'mcp:tools',
      registration_type: 'pre_registered',
    };
  }

  static OAUTH_DCR_PORT = process.env.TEST_MCP_OAUTH_DCR_PORT || '55175';
  static OAUTH_DCR_MCP_URL = `http://localhost:${McpFixtures.OAUTH_DCR_PORT}/mcp`;
  static OAUTH_DCR_SERVER_BASE = `http://localhost:${McpFixtures.OAUTH_DCR_PORT}`;
  static OAUTH_DCR_EXPECTED_TOOL = 'echo';

  static createDcrServerData() {
    const ts = Date.now();
    return {
      url: McpFixtures.OAUTH_DCR_MCP_URL,
      name: `DCR-MCP-Server-${ts}`,
      description: 'Test MCP OAuth DCR Server',
    };
  }

  static createDcrInstanceData() {
    const ts = Date.now();
    return {
      name: `DCR-MCP-${ts}`,
      slug: `dcr-mcp-${ts}`,
      description: 'DCR-authenticated MCP instance',
    };
  }
}
