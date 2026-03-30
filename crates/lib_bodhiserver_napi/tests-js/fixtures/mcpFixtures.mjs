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

  static EXA_MCP_URL = 'https://mcp.exa.ai/mcp';
  static EXA_EXPECTED_TOOL = 'web_search_exa';

  static createExaServerData() {
    const ts = Date.now();
    return {
      url: McpFixtures.EXA_MCP_URL,
      name: `Exa-Server-${ts}`,
      description: 'Exa MCP Server (public, no auth)',
    };
  }

  static createExaInstanceData() {
    const ts = Date.now();
    return {
      name: `Exa-${ts}`,
      slug: `exa-${ts}`,
      description: 'Exa search MCP instance',
    };
  }

  // Header auth test server (port 55176)
  static AUTH_HEADER_PORT = process.env.TEST_MCP_AUTH_HEADER_PORT || '55176';
  static AUTH_HEADER_MCP_URL = `http://localhost:${McpFixtures.AUTH_HEADER_PORT}/mcp`;
  static AUTH_HEADER_KEY = 'Authorization';
  static AUTH_HEADER_VALUE = 'Bearer test-header-key';
  static AUTH_HEADER_EXPECTED_TOOL = 'echo';

  // Query param auth test server (port 55177)
  static AUTH_QUERY_PORT = process.env.TEST_MCP_AUTH_QUERY_PORT || '55177';
  static AUTH_QUERY_MCP_URL = `http://localhost:${McpFixtures.AUTH_QUERY_PORT}/mcp`;
  static AUTH_QUERY_KEY = 'api_key';
  static AUTH_QUERY_VALUE = 'test-query-key';
  static AUTH_QUERY_EXPECTED_TOOL = 'echo';

  // Mixed auth test server (port 55178)
  static AUTH_MIXED_PORT = process.env.TEST_MCP_AUTH_MIXED_PORT || '55178';
  static AUTH_MIXED_MCP_URL = `http://localhost:${McpFixtures.AUTH_MIXED_PORT}/mcp`;
  static AUTH_MIXED_HEADERS = [
    { key: 'X-Auth-1', value: 'header-val-1' },
    { key: 'X-Auth-2', value: 'header-val-2' },
  ];
  static AUTH_MIXED_QUERIES = [
    { key: 'q_key_1', value: 'query-val-1' },
    { key: 'q_key_2', value: 'query-val-2' },
  ];
  static AUTH_MIXED_EXPECTED_TOOL = 'echo';

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

  // Everything MCP reference server (port 55180) — full MCP spec surface
  static EVERYTHING_SERVER_PORT = process.env.TEST_MCP_EVERYTHING_PORT || '55180';
  static EVERYTHING_SERVER_MCP_URL = `http://localhost:${McpFixtures.EVERYTHING_SERVER_PORT}/mcp`;
  static EVERYTHING_EXPECTED_TOOLS = [
    'echo',
    'get-sum',
    'get-tiny-image',
    'get-env',
    'get-annotated-message',
    'get-resource-links',
    'get-resource-reference',
    'get-structured-content',
    'gzip-file-as-resource',
    'toggle-simulated-logging',
    'toggle-subscriber-updates',
    'trigger-long-running-operation',
  ];
  static EVERYTHING_EXPECTED_PROMPTS = [
    'simple-prompt',
    'args-prompt',
    'completable-prompt',
    'resource-prompt',
  ];
  static EVERYTHING_EXPECTED_RESOURCE_TEMPLATES = [
    'demo://resource/dynamic/text/{resourceId}',
    'demo://resource/dynamic/blob/{resourceId}',
  ];

  // MCP Inspector (DANGEROUSLY_OMIT_AUTH=true, no token needed)
  static INSPECTOR_URL = 'http://localhost:6274';

  static createEverythingServerData() {
    const ts = String(Date.now()).slice(-8);
    return {
      url: McpFixtures.EVERYTHING_SERVER_MCP_URL,
      name: `Everything-MCP-Server-${ts}`,
      description: 'MCP Everything reference server',
    };
  }

  static createEverythingInstanceData() {
    const ts = String(Date.now()).slice(-8);
    return {
      name: `Everything-MCP-${ts}`,
      slug: `ev-mcp-${ts}`,
      description: 'Everything MCP server instance',
    };
  }
}
