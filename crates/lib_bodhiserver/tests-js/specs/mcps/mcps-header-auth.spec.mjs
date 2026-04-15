import { McpFixtures } from '@/fixtures/mcpFixtures.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { McpsPage } from '@/pages/McpsPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

test.describe('MCP Header/Query Auth E2E', { tag: ['@mcps', '@auth'] }, () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test('single header auth - create via form with credentials and execute', async ({
    page,
    sharedServerUrl,
  }) => {
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, sharedServerUrl);

    await loginPage.performOAuthLogin('/ui/chat/');

    // 1. Create server pointing to AUTH_HEADER_MCP_URL
    const ts = Date.now();
    const srvName = `Hdr-Srv-${ts}`;
    await mcpsPage.createMcpServer(McpFixtures.AUTH_HEADER_MCP_URL, srvName, 'Header auth test');
    const srvId = await mcpsPage.getServerUuidByName(srvName);
    expect(srvId).toBeTruthy();

    // 2. Create auth config with single header key definition
    const authConfig = await mcpsPage.createAuthConfigViaApi(srvId, {
      name: 'Header Auth',
      entries: [{ param_type: 'header', param_key: McpFixtures.AUTH_HEADER_KEY }],
    });
    expect(authConfig.id).toBeTruthy();

    // 3. Create MCP instance via form: select config, fill credential, create
    const mcpSlug = `mcp-hdr-${ts}`;
    await mcpsPage.createMcpInstanceWithHeaderAuth({
      serverName: srvName,
      name: `MCP Header Auth ${ts}`,
      slug: mcpSlug,
      authConfigId: authConfig.id,
      credentials: [
        { param_key: McpFixtures.AUTH_HEADER_KEY, value: McpFixtures.AUTH_HEADER_VALUE },
      ],
    });

    // 4. Verify MCP was created by finding it in the list
    const mcpId = await mcpsPage.getMcpUuidByName(`MCP Header Auth ${ts}`);
    expect(mcpId).toBeTruthy();

    // 5. Execute echo tool via playground
    await mcpsPage.clickPlaygroundById(mcpId);
    await mcpsPage.expectPlaygroundPage();
    await mcpsPage.selectPlaygroundTool('echo');
    await mcpsPage.expectPlaygroundToolSelected('echo');
    await mcpsPage.fillPlaygroundParam('text', 'hello-header');
    await mcpsPage.clickPlaygroundExecute();
    await mcpsPage.expectPlaygroundResultSuccess();
  });

  test('single query param auth - create via form with credentials', async ({
    page,
    sharedServerUrl,
  }) => {
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, sharedServerUrl);

    await loginPage.performOAuthLogin('/ui/chat/');

    const ts = Date.now();
    const srvName = `Qry-Srv-${ts}`;
    await mcpsPage.createMcpServer(McpFixtures.AUTH_QUERY_MCP_URL, srvName, 'Query auth test');
    const srvId = await mcpsPage.getServerUuidByName(srvName);
    expect(srvId).toBeTruthy();

    const authConfig = await mcpsPage.createAuthConfigViaApi(srvId, {
      name: 'Query Auth',
      entries: [{ param_type: 'query', param_key: McpFixtures.AUTH_QUERY_KEY }],
    });
    expect(authConfig.id).toBeTruthy();

    const mcpSlug = `mcp-qry-${ts}`;
    await mcpsPage.createMcpInstanceWithHeaderAuth({
      serverName: srvName,
      name: `MCP Query Auth ${ts}`,
      slug: mcpSlug,
      authConfigId: authConfig.id,
      credentials: [{ param_key: McpFixtures.AUTH_QUERY_KEY, value: McpFixtures.AUTH_QUERY_VALUE }],
    });

    const mcpId = await mcpsPage.getMcpUuidByName(`MCP Query Auth ${ts}`);
    expect(mcpId).toBeTruthy();
  });

  test('mixed auth - header + query params via form, verify credential values', async ({
    page,
    sharedServerUrl,
  }) => {
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, sharedServerUrl);

    await loginPage.performOAuthLogin('/ui/chat/');

    const ts = Date.now();
    const srvName = `Mix-Srv-${ts}`;
    await mcpsPage.createMcpServer(McpFixtures.AUTH_MIXED_MCP_URL, srvName, 'Mixed auth test');
    const srvId = await mcpsPage.getServerUuidByName(srvName);
    expect(srvId).toBeTruthy();

    const entries = [
      ...McpFixtures.AUTH_MIXED_HEADERS.map((h) => ({ param_type: 'header', param_key: h.key })),
      ...McpFixtures.AUTH_MIXED_QUERIES.map((q) => ({ param_type: 'query', param_key: q.key })),
    ];

    const authConfig = await mcpsPage.createAuthConfigViaApi(srvId, {
      name: 'Mixed Auth',
      entries,
    });
    expect(authConfig.id).toBeTruthy();

    const credentials = [
      ...McpFixtures.AUTH_MIXED_HEADERS.map((h) => ({ param_key: h.key, value: h.value })),
      ...McpFixtures.AUTH_MIXED_QUERIES.map((q) => ({ param_key: q.key, value: q.value })),
    ];

    const mcpSlug = `mcp-mix-${ts}`;
    await mcpsPage.createMcpInstanceWithHeaderAuth({
      serverName: srvName,
      name: `MCP Mixed Auth ${ts}`,
      slug: mcpSlug,
      authConfigId: authConfig.id,
      credentials,
    });

    const mcpId = await mcpsPage.getMcpUuidByName(`MCP Mixed Auth ${ts}`);
    expect(mcpId).toBeTruthy();

    // Execute get_auth_info via playground to verify all params sent
    await mcpsPage.clickPlaygroundById(mcpId);
    await mcpsPage.expectPlaygroundPage();
    // Wait for MCP client to connect via proxy
    await mcpsPage.expectPlaygroundConnected();
    await mcpsPage.selectPlaygroundTool('get_auth_info');
    await mcpsPage.expectPlaygroundToolSelected('get_auth_info');
    await mcpsPage.clickPlaygroundExecute();
    await mcpsPage.expectPlaygroundResultSuccess();

    // Verify credential values in the tool execution result
    await test.step('Verify credential values flow through encryption/decryption', async () => {
      // Switch to raw tab to get the full response JSON
      await mcpsPage.clickPlaygroundResultTab('raw');
      const rawContent = await mcpsPage.getPlaygroundResultContent();
      expect(rawContent).toBeTruthy();

      // The raw response contains { content: [{ type: 'text', text: '...' }], isError: false }
      const rawJson = JSON.parse(rawContent);
      const textContent = rawJson.content[0].text;
      const authInfo = JSON.parse(textContent);

      // Assert exact header key-value pairs match what was entered in the form
      for (const h of McpFixtures.AUTH_MIXED_HEADERS) {
        expect(authInfo.headers[h.key.toLowerCase()]).toBe(h.value);
      }

      // Assert exact query param key-value pairs match what was entered
      for (const q of McpFixtures.AUTH_MIXED_QUERIES) {
        expect(authInfo.query[q.key]).toBe(q.value);
      }
    });
  });
});
