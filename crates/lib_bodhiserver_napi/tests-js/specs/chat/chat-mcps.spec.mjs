import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { McpFixtures } from '@/fixtures/mcpFixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { ChatSettingsPage } from '@/pages/ChatSettingsPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { McpsPage } from '@/pages/McpsPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { registerApiModelViaUI } from '@/utils/api-model-helpers.mjs';

/**
 * Chat UI MCP Integration E2E Tests
 *
 * Tests the complete MCP-chat integration flow:
 * 1. Create MCP server + instance (Exa, public/no-auth)
 * 2. Verify MCPs popover in chat, enable tools, verify badge
 * 3. Verify selection persistence
 * 4. Execute agentic chat with MCP tool call
 *
 * Requires:
 * - INTEG_TEST_OPENAI_API_KEY environment variable
 */

test.describe('Chat Interface - MCP Integration', () => {
  let authServerConfig;
  let testCredentials;
  let loginPage;
  let mcpsPage;
  let modelsPage;
  let apiModelFormPage;
  let chatPage;
  let chatSettingsPage;
  let testApiKey;

  test.beforeAll(async () => {
    testApiKey = ApiModelFixtures.getRequiredEnvVars().apiKey;
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    mcpsPage = new McpsPage(page, sharedServerUrl);
    modelsPage = new ModelsListPage(page, sharedServerUrl);
    apiModelFormPage = new ApiModelFormPage(page, sharedServerUrl);
    chatPage = new ChatPage(page, sharedServerUrl);
    chatSettingsPage = new ChatSettingsPage(page, sharedServerUrl);
  });

  test('configure MCP → verify in popover → enable → check persistence → execute via chat @integration', async ({
    page,
  }) => {
    const serverData = McpFixtures.createExaServerData();
    const instanceData = McpFixtures.createExaInstanceData();
    let mcpId;

    await test.step('Login', async () => {
      await loginPage.performOAuthLogin();
    });

    await test.step('Register API model for chat', async () => {
      await registerApiModelViaUI(modelsPage, apiModelFormPage, testApiKey);
    });

    await test.step('Create Exa MCP server', async () => {
      await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
      const row = page.locator(`[data-test-server-name="${serverData.name}"]`).first();
      await expect(row).toBeVisible();
    });

    await test.step('Create Exa MCP instance with tools', async () => {
      await mcpsPage.createMcpInstance(
        serverData.name,
        instanceData.name,
        instanceData.slug,
        instanceData.description
      );
      await mcpsPage.expectMcpsListPage();
      const row = await mcpsPage.getMcpRowByName(instanceData.name);
      await expect(row).toBeVisible();
      mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
      expect(mcpId).toBeTruthy();
    });

    await test.step('Navigate to chat and verify MCPs popover', async () => {
      await chatPage.navigateToChat();
      await chatPage.waitForChatPageLoad();
      await chatPage.expectMcpsPopoverTriggerVisible();
    });

    await test.step('Open MCPs popover and enable MCP', async () => {
      await chatPage.openMcpsPopover();
      await chatPage.expectMcpsPopoverOpen();
      await chatPage.waitForMcpsToLoad();
      await chatPage.expectMcpInPopover(mcpId);
      await chatPage.enableMcp(mcpId);
      await chatPage.closeMcpsPopover();
    });

    await test.step('Verify badge count', async () => {
      // Exa has multiple tools; enableMcp toggles all on
      const badge = page.locator(chatPage.selectors.mcpsBadge);
      await expect(badge).toBeVisible();
    });

    await test.step('Verify selection persists after reopening popover', async () => {
      await chatPage.openMcpsPopover();
      await chatPage.expectMcpCheckboxChecked(mcpId);
      await chatPage.closeMcpsPopover();
    });

    await test.step('Verify selection persists in new chat', async () => {
      await chatPage.startNewChat();
      await chatPage.waitForChatPageLoad();
      const badge = page.locator(chatPage.selectors.mcpsBadge);
      await expect(badge).toBeVisible();
      await chatPage.openMcpsPopover();
      await chatPage.expectMcpCheckboxChecked(mcpId);
      await chatPage.closeMcpsPopover();
    });

    await test.step('Select model and send message triggering MCP tool call', async () => {
      await chatSettingsPage.selectModel(ApiModelFixtures.OPENAI_MODEL);
      await chatPage.sendMessage('What is the latest news about AI from San Francisco?');
    });

    await test.step('Verify agentic loop: tool call → execution → response', async () => {
      await chatPage.waitForAgenticResponseComplete();

      await chatPage.expandToolCall();
      const toolArgs = await chatPage.getToolCallArguments();
      expect(toolArgs).toBeTruthy();

      const finalResponse = await chatPage.getLastAssistantMessage();
      expect(finalResponse).toBeTruthy();
    });
  });
});
