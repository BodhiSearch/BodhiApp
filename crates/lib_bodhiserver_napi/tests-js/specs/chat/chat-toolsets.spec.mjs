import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';
import { SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';

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

const TOOLSET_TYPE = 'builtin-exa-search';
const TOOLSET_SLUG = 'exa-web-search';

test.describe('Chat Interface - Toolsets Integration', () => {
  let authServerConfig;
  let testCredentials;
  let loginPage;
  let chatPage;
  let toolsetsPage;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();

    // Use shared server started by Playwright webServer
  });

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    chatPage = new ChatPage(page, sharedServerUrl);
    toolsetsPage = new ToolsetsPage(page, sharedServerUrl);
  });

  test('complete flow: configure toolset → verify in popover → enable → check persistence @integration', async ({
    page,
  }) => {
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').toBeDefined();

    await loginPage.performOAuthLogin();

    await test.step('Configure Exa Web Search toolset', async () => {
      await toolsetsPage.configureToolsetWithApiKey(TOOLSET_TYPE, exaApiKey);
    });

    await test.step('Verify toolset in popover and enable', async () => {
      await chatPage.navigateToChat();
      await chatPage.openToolsetsPopover();
      await chatPage.waitForToolsetsToLoad();
      await chatPage.expectToolsetInPopover(TOOLSET_SLUG);
      await chatPage.enableToolset(TOOLSET_SLUG);
      await chatPage.closeToolsetsPopover();
      await chatPage.expectToolsetBadgeVisible(4);
    });

    await test.step('Verify selection persists after reopening popover', async () => {
      await chatPage.openToolsetsPopover();
      await chatPage.expectToolsetCheckboxChecked(TOOLSET_SLUG);
    });
  });
});
