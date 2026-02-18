/**
 * MCP Fixtures
 *
 * Provides test data factories for MCP server management tests.
 */

export class McpFixtures {
  static MCP_URL = 'https://mcp.deepwiki.com/mcp';
  static EXPECTED_TOOL = 'read_wiki_structure';

  static createLifecycleData() {
    const ts = Date.now();
    return {
      url: McpFixtures.MCP_URL,
      name: `DeepWiki-${ts}`,
      slug: `deepwiki-${ts}`,
      description: 'DeepWiki MCP Server',
    };
  }

  static createToolDiscoveryData() {
    const ts = Date.now();
    return {
      url: McpFixtures.MCP_URL,
      name: `DeepWiki-Tools-${ts}`,
      slug: `dw-tools-${ts}`,
    };
  }
}
