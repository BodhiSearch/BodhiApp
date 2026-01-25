import { randomPort } from '@/test-helpers.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { expect, test } from '@playwright/test';

import { ChatPage } from '@/pages/ChatPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ToolsetsPage } from '@/pages/ToolsetsPage.mjs';

/**
 * Chat UI Toolsets Integration E2E Tests
 *
 * These tests verify the toolsets integration in the chat UI:
 * - ToolsetsPopover component for selecting toolsets
 * - Max tool iterations setting in settings sidebar
 * - Tooltip display for disabled toolsets
 * - Badge count display for enabled toolsets
 *
 * NOTE: Tool execution tests require:
 * 1. INTEG_TEST_EXA_API_KEY environment variable for the Exa Web Search toolset
 * 2. A model that supports tool calling (e.g., GPT-4, Claude, etc.)
 */

const TOOLSET_NAME = 'builtin-exa-web-search';
const TOOLSET_SCOPE = 'scope_toolset-builtin-exa-web-search';

test.describe('Chat Interface - Toolsets Integration', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;
  let authClient;
  let resourceClient;
  let loginPage;
  let chatPage;
  let toolsetsPage;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    const port = randomPort();
    const serverUrl = `http://localhost:${port}`;

    authClient = createAuthServerTestClient(authServerConfig);
    resourceClient = await authClient.createResourceClient(serverUrl);
    await authClient.makeResourceAdmin(
      resourceClient.clientId,
      resourceClient.clientSecret,
      testCredentials.userId
    );

    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      port,
      host: 'localhost',
    });

    baseUrl = await serverManager.startServer();
  });

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
    chatPage = new ChatPage(page, baseUrl);
    toolsetsPage = new ToolsetsPage(page, baseUrl);
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('complete flow: configure toolset → verify in popover → enable → check persistence @integration', async ({
    page,
  }) => {
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').toBeDefined();

    await loginPage.performOAuthLogin();

    await test.step('Configure Exa Web Search toolset', async () => {
      await toolsetsPage.configureToolsetWithApiKey(TOOLSET_SCOPE, exaApiKey);
    });

    await test.step('Verify toolset in popover and enable', async () => {
      await chatPage.navigateToChat();
      await chatPage.openToolsetsPopover();
      await chatPage.waitForToolsetsToLoad();
      await chatPage.expectToolsetInPopover(TOOLSET_NAME);
      await chatPage.enableToolset(TOOLSET_NAME);
      await chatPage.closeToolsetsPopover();
      await chatPage.expectToolsetBadgeVisible(4);
    });

    await test.step('Verify selection persists after reopening popover', async () => {
      await chatPage.openToolsetsPopover();
      await chatPage.expectToolsetCheckboxChecked(TOOLSET_NAME);
    });
  });
});
