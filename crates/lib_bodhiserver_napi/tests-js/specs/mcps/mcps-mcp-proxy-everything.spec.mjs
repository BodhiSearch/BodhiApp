import { McpFixtures } from '@/fixtures/mcpFixtures.mjs';
import { AccessRequestReviewPage } from '@/pages/AccessRequestReviewPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
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
        // Workaround for Playwright bug #20439/#29521: extraHTTPHeaders overrides
        // the Accept header set by in-page fetch() at the CDP level.
        await page.route(`${sharedServerUrl}/**/mcp`, async (route) => {
          const headers = { ...route.request().headers() };
          if (!headers['accept']?.includes('text/event-stream')) {
            headers['accept'] = 'text/event-stream, application/json';
          }
          await route.continue({ headers });
        });

        await page.goto(McpFixtures.INSPECTOR_URL);
        await page.waitForLoadState('networkidle');

        // Configure: Streamable HTTP + Direct + URL + Auth header
        await page.getByRole('combobox', { name: 'Transport Type' }).click();
        await page.getByRole('option', { name: 'Streamable HTTP' }).click();

        await page
          .getByRole('textbox', { name: 'URL' })
          .fill(`${sharedServerUrl}/bodhi/v1/apps/mcps/${mcpId}/mcp`);

        await page.getByRole('combobox', { name: 'Connection Type' }).click();
        await page.getByRole('option', { name: 'Direct' }).click();

        await page.getByRole('button', { name: 'Authentication' }).click();
        await page.getByRole('textbox', { name: 'Header Value' }).fill(`Bearer ${accessToken}`);

        // Ensure header toggle is enabled
        const headerSwitch = page.getByRole('switch').first();
        const switchState = await headerSwitch.getAttribute('data-state');
        if (switchState !== 'checked') {
          await headerSwitch.click();
        }
      });

      await test.step('Connect to MCP proxy', async () => {
        await page.getByRole('button', { name: 'Connect' }).click();
        await expect(page.getByText('Connected')).toBeVisible({ timeout: 30000 });
      });

      await test.step('Verify initialize and logging/setLevel in history', async () => {
        await expect(page.getByText('initialize')).toBeVisible();
        await expect(page.getByText('logging/setLevel')).toBeVisible();
      });

      // ── Phase 4: Exercise MCP features via Inspector UI ──

      // ── Tools ──

      await test.step('Tools — list tools', async () => {
        await page.getByRole('tab', { name: 'Tools' }).click();
        await page.waitForURL(/#tools/);
        await page.getByRole('button', { name: 'List Tools' }).click();
        await expect(page.getByText('Echo Tool')).toBeVisible({ timeout: 10000 });
      });

      await test.step('Tools — call echo', async () => {
        await page.getByText('Echo Tool').first().click();
        await page.locator('textarea').first().fill('proxy-e2e-hello');
        await page.getByRole('button', { name: 'Run Tool' }).click();
        await expect(page.getByText('Success')).toBeVisible({ timeout: 10000 });
        await expect(page.getByText('proxy-e2e-hello').nth(1)).toBeVisible();
      });

      await test.step('Tools — call get-sum with number params', async () => {
        await page.getByText('Returns the sum of two numbers').first().click();
        const numberInputs = page.locator('input[type="number"]');
        await numberInputs.first().fill('7');
        await numberInputs.nth(1).fill('13');
        await page.getByRole('button', { name: 'Run Tool' }).click();
        await expect(page.getByText('Success')).toBeVisible({ timeout: 10000 });
        await expect(page.getByText('20')).toBeVisible();
      });

      await test.step('Tools — call get-tiny-image returns image', async () => {
        await page.getByText('Returns a tiny MCP logo image').first().click();
        await page.getByRole('button', { name: 'Run Tool' }).click();
        await expect(page.getByText('Success')).toBeVisible({ timeout: 10000 });
        // Image content renders as an img element
        await expect(
          page.locator('img[src^="data:image"]').or(page.getByText('image/'))
        ).toBeVisible();
      });

      // ── Resources ──

      await test.step('Resources — list and read', async () => {
        await page.getByRole('tab', { name: 'Resources' }).click();
        await page.waitForURL(/#resources/);
        await page.getByRole('button', { name: 'List Resources' }).click();
        await expect(page.getByText('architecture.md').first()).toBeVisible({ timeout: 10000 });

        // Click a resource to read it — Inspector auto-reads on click
        await page.getByText('architecture.md').first().click();
        // Content panel shows resource mime type
        await expect(page.getByText('text/markdown').first()).toBeVisible({ timeout: 10000 });
      });

      await test.step('Resources — list templates', async () => {
        await page.getByRole('button', { name: 'List Templates' }).click();
        await expect(page.getByText('Dynamic').first()).toBeVisible({ timeout: 10000 });
      });

      // ── Prompts ──

      await test.step('Prompts — list', async () => {
        await page.getByRole('tab', { name: 'Prompts' }).click();
        await page.waitForURL(/#prompts/);
        await page.getByRole('button', { name: 'List Prompts' }).click();
        await expect(page.getByText('simple-prompt').first()).toBeVisible({ timeout: 10000 });
        await expect(page.getByText('args-prompt').first()).toBeVisible();
      });

      await test.step('Prompts — get simple-prompt', async () => {
        await page.getByText('simple-prompt').first().click();
        await page.getByRole('button', { name: 'Get Prompt' }).click();
        // Result shows prompt messages with role "user"
        await expect(page.getByText('user').first()).toBeVisible({ timeout: 10000 });
      });

      await test.step('Prompts — get args-prompt with city argument', async () => {
        await page.getByText('args-prompt').first().click();
        // args-prompt has a 'city' combobox — click it, type value
        const cityCombobox = page.getByRole('combobox').filter({ hasText: 'Enter city' });
        await cityCombobox.click();
        await page.keyboard.type('TestCity');
        await page.getByRole('button', { name: 'Get Prompt' }).click();
        await expect(page.getByText('TestCity')).toBeVisible({ timeout: 10000 });
      });

      // ── Ping ──

      await test.step('Ping — verify server responds', async () => {
        await page.getByRole('tab', { name: 'Ping' }).click();
        await page.waitForURL(/#ping/);
        await page.getByRole('button', { name: 'Ping Server' }).click();
        // Verify ping appears in history
        await expect(page.getByText('ping').last()).toBeVisible({ timeout: 5000 });
      });

      // ── Disconnect ──

      await test.step('Disconnect from proxy', async () => {
        await page.getByRole('button', { name: 'Disconnect' }).click();
        await expect(page.getByText('Disconnected')).toBeVisible({ timeout: 5000 });
      });
    });
  }
);
