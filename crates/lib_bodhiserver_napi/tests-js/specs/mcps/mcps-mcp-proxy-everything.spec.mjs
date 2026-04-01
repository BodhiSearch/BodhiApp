import { McpFixtures } from '@/fixtures/mcpFixtures.mjs';
import { AccessRequestReviewPage } from '@/pages/AccessRequestReviewPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { McpInspectorPage } from '@/pages/McpInspectorPage.mjs';
import { McpsPage } from '@/pages/McpsPage.mjs';
import { OAuthTestApp } from '@/pages/OAuthTestApp.mjs';
import {
  getAuthServerConfig,
  getPreConfiguredAppClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';
import { SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';

/**
 * MCP Proxy E2E Tests via MCP Inspector + Everything Server
 *
 * Drives the MCP Inspector UI in Direct connection mode to exercise
 * the Bodhi MCP proxy against the everything reference server.
 * Tests CORS, session management, and full MCP protocol passthrough
 * at the browser level — true black-box E2E testing.
 *
 * Flow:
 * 1. Login to Bodhi → create everything MCP server + instance via UI
 * 2. OAuth test app → request access → approve → get Bearer token
 * 3. MCP Inspector → Direct mode → connect to /bodhi/v1/apps/mcps/{id}/mcp
 * 4. Exercise MCP features via Inspector UI
 */
test.describe(
  'MCP Proxy — Everything Server via Inspector',
  { tag: ['@mcps', '@mcp-proxy', '@everything'] },
  () => {
    let authServerConfig;
    let testCredentials;

    test.beforeAll(async () => {
      authServerConfig = getAuthServerConfig();
      testCredentials = getTestCredentials();
    });

    test('MCP proxy full protocol journey — tools, resources, prompts via Inspector Direct mode', async ({
      page,
      sharedServerUrl,
    }) => {
      const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
      const mcpsPage = new McpsPage(page, sharedServerUrl);
      const inspector = new McpInspectorPage(page, sharedServerUrl);
      const serverData = McpFixtures.createEverythingServerData();
      const instanceData = McpFixtures.createEverythingInstanceData();

      let mcpId;
      let accessToken;

      // ── Phase 1: Create MCP server + instance via Bodhi UI ──

      await test.step('Login and create everything MCP server + instance', async () => {
        await loginPage.performOAuthLogin('/ui/chat/');
        await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
        await mcpsPage.createMcpInstance(
          serverData.name,
          instanceData.name,
          instanceData.slug,
          instanceData.description
        );
        await mcpsPage.expectMcpsListPage();
        mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
        expect(mcpId).toBeTruthy();
      });

      // ── Phase 2: OAuth access request + token via test-oauth-app ──

      const appClient = getPreConfiguredAppClient();
      const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
      const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

      await test.step('Configure OAuth form with everything MCP request', async () => {
        await app.navigate();
        await app.config.configureOAuthForm({
          bodhiServerUrl: sharedServerUrl,
          authServerUrl: authServerConfig.authUrl,
          realm: authServerConfig.authRealm,
          clientId: appClient.clientId,
          redirectUri,
          scope: 'openid profile email',
          requested: JSON.stringify({
            version: '1',
            mcp_servers: [{ url: McpFixtures.EVERYTHING_SERVER_MCP_URL }],
          }),
        });
      });

      await test.step('Submit access request, approve with MCP, and login', async () => {
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);

        const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
        await reviewPage.approveWithMcps([
          { url: McpFixtures.EVERYTHING_SERVER_MCP_URL, instanceId: mcpId },
        ]);

        await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
        await app.accessCallback.waitForLoaded();
        await app.accessCallback.clickLogin();
        await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
      });

      await test.step('Get access token from dashboard', async () => {
        await app.dashboard.navigateTo();
        accessToken = await app.dashboard.getAccessToken();
        expect(accessToken).toBeTruthy();
        expect(accessToken.startsWith('eyJ')).toBe(true);
      });

      // ── Phase 3: Connect via MCP Inspector Direct mode ──

      await test.step('Open Inspector and configure Direct connection', async () => {
        await inspector.configureDirectConnection({
          inspectorUrl: McpFixtures.INSPECTOR_URL,
          serverUrl: sharedServerUrl,
          mcpId,
          accessToken,
        });
      });

      await test.step('Connect to MCP proxy', async () => {
        await inspector.clickConnect();
        await inspector.expectConnected();
      });

      await test.step('Verify initialize and logging/setLevel in history', async () => {
        await inspector.expectHistoryEntry('initialize');
        await inspector.expectHistoryEntry('logging/setLevel');
      });

      // ── Phase 4: Exercise MCP features via Inspector UI ──

      // ── Tools ──

      await test.step('Tools — list tools', async () => {
        await inspector.switchToToolsTab();
        await inspector.listTools();
        await expect(page.getByText('Echo Tool')).toBeVisible({ timeout: 10000 });
      });

      await test.step('Tools — call echo', async () => {
        await inspector.selectTool('Echo Tool');
        await inspector.fillToolTextInput('proxy-e2e-hello');
        await inspector.executeSelectedTool();
        await inspector.expectToolResult('proxy-e2e-hello');
      });

      await test.step('Tools — call get-sum with number params', async () => {
        await inspector.selectTool('Returns the sum of two numbers');
        await inspector.fillToolNumberInputs([7, 13]);
        await inspector.executeSelectedTool();
        await inspector.expectToolResult('20');
      });

      await test.step('Tools — call get-tiny-image returns image', async () => {
        await inspector.selectTool('Returns a tiny MCP logo image');
        await inspector.executeSelectedTool();
        await inspector.expectToolResultImage();
      });

      // ── Resources ──

      await test.step('Resources — list and read', async () => {
        await inspector.switchToResourcesTab();
        await inspector.listResources();
        await inspector.expectResourceContent('architecture.md');

        // Click a resource to read it — Inspector auto-reads on click
        await inspector.selectResource('architecture.md');
        // Content panel shows resource mime type
        await inspector.expectResourceContent('text/markdown');
      });

      await test.step('Resources — list templates', async () => {
        await inspector.listTemplates();
        await expect(page.getByText('Dynamic').first()).toBeVisible({ timeout: 10000 });
      });

      // ── Prompts ──

      await test.step('Prompts — list', async () => {
        await inspector.switchToPromptsTab();
        await inspector.listPrompts();
        await inspector.expectPromptContent('simple-prompt');
        await inspector.expectPromptContent('args-prompt');
      });

      await test.step('Prompts — get simple-prompt', async () => {
        await inspector.selectPrompt('simple-prompt');
        await inspector.getSelectedPrompt();
        // Result shows prompt messages with role "user"
        await inspector.expectPromptContent('user');
      });

      await test.step('Prompts — get args-prompt with city argument', async () => {
        await inspector.selectPrompt('args-prompt');
        await inspector.fillPromptCombobox('Enter city', 'TestCity');
        await inspector.getSelectedPrompt();
        await inspector.expectPromptContent('TestCity');
      });

      // ── Ping ──

      await test.step('Ping — verify server responds', async () => {
        await inspector.switchToPingTab();
        await inspector.executePing();
        await inspector.expectPingSuccess();
      });

      // ── Disconnect ──

      await test.step('Disconnect from proxy', async () => {
        await inspector.clickDisconnect();
        await inspector.expectDisconnected();
      });
    });

    test('MCP proxy — disabled instance shows error', async ({ page, sharedServerUrl }) => {
      const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
      const mcpsPage = new McpsPage(page, sharedServerUrl);
      const inspector = new McpInspectorPage(page, sharedServerUrl);
      const serverData = McpFixtures.createEverythingServerData();
      const instanceData = McpFixtures.createEverythingInstanceData();

      let mcpId;
      let accessToken;

      // ── Phase 1: Create MCP server + instance, then disable the instance ──

      await test.step('Login and create everything MCP server + instance', async () => {
        await loginPage.performOAuthLogin('/ui/chat/');
        await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
        await mcpsPage.createMcpInstance(
          serverData.name,
          instanceData.name,
          instanceData.slug,
          instanceData.description
        );
        await mcpsPage.expectMcpsListPage();
        mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
        expect(mcpId).toBeTruthy();
      });

      await test.step('Disable the MCP instance via API', async () => {
        // Use page.evaluate to PUT the update with enabled: false
        const result = await page.evaluate(
          async ({ baseUrl, mcpId, instanceData }) => {
            const resp = await fetch(`${baseUrl}/bodhi/v1/mcps/${mcpId}`, {
              method: 'PUT',
              headers: { 'Content-Type': 'application/json' },
              credentials: 'include',
              body: JSON.stringify({
                name: instanceData.name,
                slug: instanceData.slug,
                description: instanceData.description,
                enabled: false,
              }),
            });
            if (!resp.ok) throw new Error(`HTTP ${resp.status}: ${await resp.text()}`);
            return await resp.json();
          },
          { baseUrl: sharedServerUrl, mcpId, instanceData }
        );
        expect(result.enabled).toBe(false);
      });

      // ── Phase 2: Get OAuth token ──

      const appClient = getPreConfiguredAppClient();
      const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
      const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

      await test.step('Obtain access token via OAuth flow', async () => {
        await app.navigate();
        await app.config.configureOAuthForm({
          bodhiServerUrl: sharedServerUrl,
          authServerUrl: authServerConfig.authUrl,
          realm: authServerConfig.authRealm,
          clientId: appClient.clientId,
          redirectUri,
          scope: 'openid profile email',
          requested: JSON.stringify({
            version: '1',
            mcp_servers: [{ url: McpFixtures.EVERYTHING_SERVER_MCP_URL }],
          }),
        });
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);

        const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
        await reviewPage.approveWithMcps([
          { url: McpFixtures.EVERYTHING_SERVER_MCP_URL, instanceId: mcpId },
        ]);

        await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
        await app.accessCallback.waitForLoaded();
        await app.accessCallback.clickLogin();
        await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);

        await app.dashboard.navigateTo();
        accessToken = await app.dashboard.getAccessToken();
        expect(accessToken).toBeTruthy();
      });

      // ── Phase 3: Try to connect via Inspector — expect failure ──

      await test.step('Connect to disabled MCP instance and expect error', async () => {
        await inspector.configureDirectConnection({
          inspectorUrl: McpFixtures.INSPECTOR_URL,
          serverUrl: sharedServerUrl,
          mcpId,
          accessToken,
        });
        await inspector.clickConnect();
        // The proxy returns 403 for a disabled instance; Inspector should not reach Connected
        await inspector.expectConnectionError();
      });
    });

    test('MCP proxy — invalid token rejected', async ({ page, sharedServerUrl }) => {
      const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
      const mcpsPage = new McpsPage(page, sharedServerUrl);
      const inspector = new McpInspectorPage(page, sharedServerUrl);
      const serverData = McpFixtures.createEverythingServerData();
      const instanceData = McpFixtures.createEverythingInstanceData();

      let mcpId;

      // ── Phase 1: Create MCP server + instance (need a valid mcpId) ──

      await test.step('Login and create everything MCP server + instance', async () => {
        await loginPage.performOAuthLogin('/ui/chat/');
        await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
        await mcpsPage.createMcpInstance(
          serverData.name,
          instanceData.name,
          instanceData.slug,
          instanceData.description
        );
        await mcpsPage.expectMcpsListPage();
        mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
        expect(mcpId).toBeTruthy();
      });

      // ── Phase 2: Try Inspector with garbage token — expect failure ──

      await test.step('Connect with invalid token and expect error', async () => {
        const garbageToken = 'invalid-not-a-real-jwt-token';
        await inspector.configureDirectConnection({
          inspectorUrl: McpFixtures.INSPECTOR_URL,
          serverUrl: sharedServerUrl,
          mcpId,
          accessToken: garbageToken,
        });
        await inspector.clickConnect();
        // The proxy rejects invalid tokens; Inspector should not reach Connected
        await inspector.expectConnectionError();
      });
    });
  }
);
