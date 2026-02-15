import { randomPort } from '@/test-helpers.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { expect, test } from '@playwright/test';

import { ChatPage } from '@/pages/ChatPage.mjs';
import { ChatSettingsPage } from '@/pages/ChatSettingsPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ToolsetsPage } from '@/pages/ToolsetsPage.mjs';

/**
 * Agentic Chat E2E Tests
 *
 * Tests the complete agentic chat flow:
 * 1. Configure Exa toolset with API key
 * 2. Enable toolset in chat
 * 3. Send message that triggers tool call
 * 4. Verify tool execution and final response
 *
 * Requires:
 * - INTEG_TEST_EXA_API_KEY environment variable
 * - Qwen3 model with tool calling support (configured via selectModelQwen)
 */

const TOOLSET_TYPE = 'builtin-exa-search';
const TOOLSET_SLUG = 'exa-web-search';

test.describe('Chat Interface - Agentic Flow', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;
  let authClient;
  let resourceClient;
  let loginPage;
  let chatPage;
  let chatSettingsPage;
  let toolsetsPage;

  test.beforeAll(async () => {
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').toBeDefined();

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
    chatSettingsPage = new ChatSettingsPage(page, baseUrl);
    toolsetsPage = new ToolsetsPage(page, baseUrl);
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('agentic chat with Exa web search executes tool and generates response @integration', async ({
    page,
  }) => {
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').toBeDefined();
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').not.toBeNull();

    await loginPage.performOAuthLogin();

    await toolsetsPage.configureToolsetWithApiKey(TOOLSET_TYPE, exaApiKey);

    await chatPage.navigateToChat();
    await chatPage.waitForChatPageLoad();
    await chatSettingsPage.selectModelQwen();

    await chatPage.openToolsetsPopover();
    await chatPage.waitForToolsetsToLoad();
    await chatPage.enableToolset(TOOLSET_SLUG);
    await chatPage.closeToolsetsPopover();

    await chatPage.sendMessage('What is the latest news about AI from San Francisco?');

    await chatPage.waitForAgenticResponseComplete();

    await chatPage.expandToolCall();
    const toolArgs = await chatPage.getToolCallArguments();

    expect(toolArgs).toContain('San Francisco');
    expect(toolArgs).toContain('AI');

    const finalResponse = await chatPage.getLastAssistantMessage();
    expect(finalResponse).toBeTruthy();

    const responseLower = finalResponse.toLowerCase();
    expect(
      responseLower.includes('san francisco') ||
        responseLower.includes('artificial intelligence') ||
        responseLower.includes(' ai ')
    ).toBe(true);
  });
});
