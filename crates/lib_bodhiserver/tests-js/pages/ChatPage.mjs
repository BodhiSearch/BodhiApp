import { McpFixtures } from '@/fixtures/mcpFixtures.mjs';
import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

export class ChatPage extends BasePage {
  selectors = {
    // Core elements
    chatContainer: '[data-testid="chat-ui"]',
    messageInput: '[data-testid="chat-input"]',
    sendButton: '[data-testid="send-button"]',
    messageList: '[data-testid="message-list"]',

    // Message elements (by data-testid)
    userMessage: '[data-testid="user-message"]',
    assistantMessage: '[data-testid="assistant-message"]',
    userMessageContent: '[data-testid="user-message-content"]',
    assistantMessageContent: '[data-testid="assistant-message-content"]',

    // Message state classes (deterministic from ChatMessage refactor)
    latestUserMessage: '.chat-user-message',
    archivedUserMessage: '.chat-user-message-archive',
    latestAiMessage: '.chat-ai-message',
    archivedAiMessage: '.chat-ai-archive',
    streamingAiMessage: '.chat-ai-streaming',
    completedMessage: '.message-completed',
    streamingMessage: '.message-streaming',

    // Chat management
    newChatButton: '[data-testid="new-chat-button"]',
    emptyState: '[data-testid="empty-chat-state"]',

    // Model selection (in settings panel)
    modelSelectorLoaded: '[data-testid="model-selector-loaded"]',
    comboboxTrigger: '[data-testid="model-selector-trigger"]',
    comboboxOption: modelName => `[data-testid="combobox-option-${modelName}"]`,

    // Settings
    settingsSidebar: '[data-testid="settings-sidebar"]',
    settingsToggle: '[data-testid="settings-toggle-button"]',
    chatHistoryToggle: '[data-testid="chat-history-toggle"]',
    maxTokensSliderToggle: '[data-testid="setting-max-tokens-toggle"]',

    // Tool call elements
    toolCallMessage: '[data-testid="tool-call-message"]',
    toolCallExpand: '[data-testid="tool-call-expand"]',
    toolCallStatus: '[data-testid="tool-call-status"]',
    toolCallContent: '[data-testid="tool-call-content"]',

    // MCP tools moved from the composer popover into the rail's "MCP servers" tab (V2).
    parametersTabTrigger: '[data-testid="chat-rail-tab-parameters"]',
    mcpsTabTrigger: '[data-testid="chat-rail-tab-mcp"]',
    mcpsPane: '[data-testid="mcp-servers-pane"]',
    // The rail-tab badge always renders the enabled-tool count (0 when none enabled).
    mcpsBadge: '[data-testid="chat-rail-mcp-count"]',
    mcpsEmptyState: '[data-testid="mcps-empty-state"]',
    // Servers are ADDED to the chat via a combobox, then appear as rows; tools toggle within a row.
    mcpAddTrigger: '[data-testid="mcp-add-trigger"]',
    mcpAddOption: id => `[data-testid="mcp-add-option-${id}"]`,
    mcpRemove: id => `[data-testid="mcp-remove-${id}"]`,
    mcpRow: id => `[data-testid="mcp-row-${id}"]`,
    mcpExpand: id => `[data-testid="mcp-expand-${id}"]`,
    mcpItem: id => `[data-testid="mcp-item-${id}"]`,
    mcpToolRow: (mcpId, toolName) => `[data-testid="mcp-tool-row-${mcpId}-${toolName}"]`,
    mcpToolCheckbox: (mcpId, toolName) => `[data-testid="mcp-tool-checkbox-${mcpId}-${toolName}"]`,

    chatFormModelSelected: '[data-test-state="model-selected"]',
  };

  /**
   * Navigate to chat page
   */
  async navigateToChat() {
    // Skip view-transitions so the rail/panel swaps don't detach mid-animation under test.
    await this.page.emulateMedia({ reducedMotion: 'reduce' });
    await this.navigate('/ui/chat/');
    await this.page.waitForSelector(this.selectors.chatContainer);
    await this.waitForSPAReady();
  }

  /**
   * Wait for chat page to load completely
   */
  async waitForChatPageLoad() {
    await this.page.waitForSelector(this.selectors.chatContainer);
    await this.waitForSPAReady();

    // Wait for model selector to be available
    await this.page.waitForSelector(this.selectors.modelSelectorLoaded);
  }

  // Core message operations

  /**
   * Send a message in the chat
   */
  async sendMessage(message, options = {}) {
    await this.waitForToastToHideOptional();
    await this.sendMessageAndReturn(message, options);
    await this.waitForLatestUserMessage();
  }

  async sendMessageAndReturn(message, options = {}) {
    await this.waitForToastToHideOptional();
    await this.waitForModelSelected();
    await this.page.fill(this.selectors.messageInput, message);
    if (!!options.maxTokens) {
      await this.page.locator(this.selectors.maxTokensSliderToggle).click();
    }
    const sendButton = this.page.locator(this.selectors.sendButton);
    await expect(sendButton).toBeEnabled();
    await sendButton.click();
    return sendButton;
  }

  /**
   * Wait for a response containing expected content
   */
  async waitForResponse(expectedContent) {
    if (expectedContent) {
      // Wait for assistant message with specific content
      await expect(this.page.locator(this.selectors.assistantMessage).last()).toContainText(expectedContent);
    } else {
      // Wait for any assistant message
      await expect(this.page.locator(this.selectors.assistantMessage).last()).toBeVisible();
    }
  }

  /**
   * Get the content of the last assistant message
   */
  async getLastAssistantMessage() {
    const lastAssistantMessage = this.page.locator(this.selectors.assistantMessage).last();
    await expect(lastAssistantMessage).toBeVisible();
    return await lastAssistantMessage.locator(this.selectors.assistantMessageContent).textContent();
  }

  /**
   * Read the served-model `data-served-model` attribute on the last completed
   * assistant message (the model the reply was produced under). Returns null if
   * the attribute is absent. Non-visible signal used to observe routing.
   */
  async getLastServedModel() {
    const lastAssistantMessage = this.page.locator(this.selectors.assistantMessage).last();
    await expect(lastAssistantMessage).toBeVisible();
    return await lastAssistantMessage.getAttribute('data-served-model');
  }

  /**
   * Wait for streaming to complete (streaming indicator disappears)
   */
  async waitForStreamingComplete() {
    // Wait for streaming AI message to appear
    await expect(this.page.locator(this.selectors.streamingAiMessage)).toBeVisible();

    // Wait for streaming to complete (streaming message becomes latest)
    await expect(this.page.locator(this.selectors.streamingAiMessage)).not.toBeVisible();
    await expect(this.page.locator(this.selectors.latestAiMessage)).toBeVisible();
  }

  /**
   * Wait for non-streaming response (when streaming is disabled)
   */
  async waitForNonStreamingResponse() {
    // Wait for latest AI message to appear (non-streaming)
    await expect(this.page.locator(this.selectors.latestAiMessage)).toBeVisible();

    // Ensure it's marked as completed
    const latestAiMessage = this.page.locator(this.selectors.latestAiMessage);
    await expect(latestAiMessage).toHaveClass(/message-completed/);
  }

  /**
   * Wait for any response to complete (streaming or non-streaming)
   */
  async waitForResponseComplete() {
    const lastAssistantMessage = this.page.locator(this.selectors.assistantMessage).last();
    await expect(lastAssistantMessage).toBeVisible({ timeout: 30000 }); // keep longer timeout, qwen going into thinking
    await expect(this.page.locator(this.selectors.latestAiMessage)).toBeVisible({ timeout: 30000 }); // wait longer for chat messages
    const latestAiMessage = this.page.locator(this.selectors.latestAiMessage);
    await expect(latestAiMessage).toHaveClass(/message-completed/);
  }

  /**
   * Wait for latest user message to appear
   */
  async waitForLatestUserMessage() {
    await expect(this.page.locator(this.selectors.latestUserMessage)).toBeVisible();
  }

  /**
   * Wait for latest AI message to appear (completed response)
   */
  async waitForLatestAiMessage() {
    await expect(this.page.locator(this.selectors.latestAiMessage)).toBeVisible();
  }

  /**
   * Wait for AI message to start streaming
   */
  async waitForStreamingAiMessage() {
    await expect(this.page.locator(this.selectors.streamingAiMessage)).toBeVisible();
  }

  // Model operations

  async waitForModelSelected() {
    await this.page.waitForSelector(this.selectors.chatFormModelSelected);
  }

  /**
   * Select a model from the settings panel
   */
  async selectModel(modelName) {
    // Open settings if not already open
    await this.openSettingsPanel();

    // Click the combobox trigger
    const trigger = this.page.locator(this.selectors.comboboxTrigger);
    await expect(trigger).toBeVisible();
    await trigger.click();

    // Select the model option (using visible element pattern from Phase 0)
    const modelOption = this.page.locator(this.selectors.comboboxOption(modelName));
    const visibleOption = modelOption.filter({ visible: true }).first();
    await expect(visibleOption).toBeVisible();
    await visibleOption.click();
  }

  /**
   * Open the model combobox dropdown
   */
  async toggleModelCombobox() {
    await this.openSettingsPanel();
    const trigger = this.page.locator(this.selectors.comboboxTrigger);
    await expect(trigger).toBeVisible();
    await trigger.click();
  }

  /**
   * Expect a model option to be visible in the combobox
   */
  async expectModelOptionVisible(modelName) {
    const option = this.page.locator(this.selectors.comboboxOption(modelName));
    await expect(option).toBeVisible();
  }

  /**
   * Expect a model option to not be visible in the combobox
   */
  async expectModelOptionNotVisible(modelName) {
    const option = this.page.locator(this.selectors.comboboxOption(modelName));
    await expect(option).not.toBeVisible();
  }

  /**
   * Verify that a model is selected
   */
  async verifyModelSelected(modelName) {
    await this.openSettingsPanel();

    // The combobox trigger is a free-text <input>; the selected model lives in `value`.
    const trigger = this.page.locator(this.selectors.comboboxTrigger);
    await expect(trigger).toHaveValue(modelName);
  }

  // Chat management

  /**
   * Start a new chat conversation
   */
  async startNewChat() {
    await this.page.click(this.selectors.newChatButton);
    await this.waitForSPAReady();
    await expect(this.page.locator(this.selectors.emptyState)).toBeVisible({ timeout: 10000 });
  }


  async expectChatPage() {
    await this.page.waitForURL(url => url.pathname === '/ui/chat/');
    await this.waitForChatPageLoad();
  }

  /**
   * Wait for the API format label to contain the expected text.
   * Ensures the AliasSelector's useEffect has synced the apiFormat setting
   * from the model's configuration, preventing a race where sendMessage fires
   * before the format propagates through React state.
   */
  async waitForApiFormat(expectedFormatText) {
    await this.openSettingsPanel();
    await expect(this.page.locator('[data-testid="api-format-label"]')).toContainText(expectedFormatText);
  }

  /**
   * Verify that chat is empty (shows empty state)
   */
  async verifyChatEmpty() {
    // Wait for the chat page to fully load and stabilize
    await this.page.waitForSelector(this.selectors.chatContainer);
    await this.waitForSPAReady();

    // Wait a bit more for any async state to settle
    await this.page.waitForTimeout(1000);

    // Check for "Welcome to Chat" heading as the empty state indicator
    const welcomeHeading = this.page.locator('h3:has-text("Welcome to Chat")');
    await expect(welcomeHeading).toBeVisible();

    // Also verify no messages are present
    const userMessages = await this.page.locator(this.selectors.userMessage).count();
    const assistantMessages = await this.page.locator(this.selectors.assistantMessage).count();
    expect(userMessages).toBe(0);
    expect(assistantMessages).toBe(0);
  }

  /**
   * Verify that a message exists in chat history
   */
  async verifyMessageInHistory(role, expectedContent) {
    await expect(
      this.page.locator(`[data-testid="${role}-message-content"]`).filter({ hasText: expectedContent }).first()
    ).toBeVisible();
  }

  // Validation and error handling

  /**
   * Verify that send button is disabled when input is empty
   */
  async verifySendButtonDisabled() {
    await expect(this.page.locator(this.selectors.sendButton)).toBeDisabled();
  }

  /**
   * Verify that send button is disabled for empty messages
   */
  async verifySendButtonDisabledForEmpty() {
    // Clear input and verify button is disabled
    await this.page.fill(this.selectors.messageInput, '');
    await expect(this.page.locator(this.selectors.sendButton)).toBeDisabled();
  }

  /**
   * Expect error when no model is selected
   */
  async expectModelNotSelectedError() {
    // Look for error toast or message about model selection
    await expect(this.page.locator('[data-state="open"]')).toContainText('No Model Selected');
  }

  /**
   * Expect validation error message
   */
  async expectValidationError(errorMessage) {
    await expect(this.page.locator('[data-state="open"]')).toContainText(errorMessage);
  }

  /**
   * Expect network error
   */
  async expectError() {
    await expect(this.page.locator('[data-state="open"]')).toContainText(/error|failed|network/i);
  }

  // Settings panel operations

  /**
   * Open settings panel
   */
  async openSettingsPanel() {
    const settingsPanel = this.page.locator(this.selectors.settingsSidebar);
    const isVisible = await settingsPanel.isVisible();

    if (!isVisible) {
      await this.page.click(this.selectors.settingsToggle);
      await expect(settingsPanel).toBeVisible();
    }
  }

  /**
   * Close settings panel
   */
  async closeSettingsPanel() {
    const settingsPanel = this.page.locator(this.selectors.settingsSidebar);
    const isVisible = await settingsPanel.isVisible();

    if (isVisible) {
      await this.page.click(this.selectors.settingsToggle);
      await expect(settingsPanel).not.toBeVisible();
    }
  }

  // Streaming operations

  /**
   * Verify streaming has started
   */
  async verifyStreamingStarted() {
    await expect(this.page.locator(this.selectors.streamingAiMessage)).toBeVisible();
  }

  /**
   * Stop streaming (if there's a stop button - this might need adjustment based on UI)
   */
  async stopStreaming() {
    // This may need to be implemented based on actual streaming UI
    // For now, we'll assume there's a stop button or we can interrupt by clicking send again
    const stopButton = this.page.locator('[data-testid="stop-streaming-button"]');
    if (await stopButton.isVisible()) {
      await stopButton.click();
    }
  }

  /**
   * Verify streaming has stopped
   */
  async verifyStreamingStopped() {
    await expect(this.page.locator(this.selectors.streamingAiMessage)).not.toBeVisible();
  }

  /**
   * Verify partial response is saved in history
   */
  async verifyPartialResponseInHistory() {
    await expect(this.page.locator(this.selectors.assistantMessage)).toBeVisible();
  }

  // Network simulation helpers

  /**
   * Simulate network failure
   */
  async simulateNetworkFailure() {
    await this.page.route('**/v1/chat/completions', route => route.abort());
  }

  /**
   * Restore network connection
   */
  async restoreNetworkConnection() {
    await this.page.unroute('**/v1/chat/completions');
  }

  /**
   * Retry last message (this might need UI implementation)
   */
  async retryLastMessage() {
    // This would depend on UI having a retry mechanism
    // For now, we'll assume we can click the send button again
    const lastInput = await this.page.locator(this.selectors.messageInput).inputValue();
    if (lastInput) {
      await this.page.click(this.selectors.sendButton);
    }
  }

  // Responsive layout helpers

  /**
   * Verify responsive layout for given viewport width
   */
  async verifyResponsiveLayout(viewportWidth) {
    if (viewportWidth < 768) {
      // Mobile: verify mobile-specific elements are visible
      await expect(this.page.locator(this.selectors.chatHistoryToggle)).toBeVisible();
      await expect(this.page.locator(this.selectors.settingsToggle)).toBeVisible();
    } else {
      // Desktop: verify desktop layout
      // Settings and history panels might be visible by default
    }
  }

  /**
   * Test responsive chatting functionality
   */
  async testResponsiveChatting(viewportWidth) {
    if (viewportWidth < 768) {
      // Mobile: settings should be in drawer/modal
      await this.openSettingsPanel();
      await this.selectModel('gpt-4');
      await this.closeSettingsPanel();
    } else {
      // Desktop: settings in sidebar
      await this.selectModel('gpt-4');
    }

    await this.sendMessage('Test responsive message');
    await this.waitForResponse();
  }

  /**
   * Verify we're on chat page with specific model pre-selected
   * @param {string} modelName - The expected model name
   */
  async expectChatPageWithModel(modelName) {
    // Wait for chat page to load
    await this.waitForChatPageLoad();

    // Check that we're on the chat page
    await expect(this.page).toHaveURL(/\/ui\/chat\//);

    // Wait for model selector to be loaded and check if the model is selected
    await this.page.waitForSelector(this.selectors.modelSelectorLoaded);

    // Check the selected model in the combobox
    const modelSelector = this.page.locator(this.selectors.comboboxTrigger);
    await expect(modelSelector).toBeVisible();

    // The model selector is a free-text autocomplete <input>; its selected model lives in `value`
    // (not textContent), so assert on the input value.
    await expect(modelSelector).toHaveValue(modelName);

    // Verify the message input is enabled (not showing "Please select a model first")
    const messageInput = this.page.locator(this.selectors.messageInput);
    await expect(messageInput).toBeVisible();
    await expect(messageInput).not.toHaveAttribute('placeholder', 'Please select a model first');
  }

  // Tool call operations

  /**
   * Wait for tool call message to appear in chat
   */
  async waitForToolCall() {
    await expect(this.page.locator(this.selectors.toolCallMessage).first()).toBeVisible({
      timeout: 30000,
    });
  }

  /**
   * Expand the first tool call collapsed section
   */
  async expandToolCall() {
    const toolCallExpand = this.page.locator(this.selectors.toolCallExpand).first();
    await toolCallExpand.click();
    await expect(this.page.locator(this.selectors.toolCallContent).first()).toBeVisible();
  }

  /**
   * Get the arguments JSON from expanded tool call
   */
  async getToolCallArguments() {
    const content = this.page.locator(this.selectors.toolCallContent).first();
    const argumentsPre = content.locator('pre').first();
    return await argumentsPre.textContent();
  }

  /**
   * Wait for tool call to complete (status changes from "Calling..." to "Completed")
   */
  async waitForToolCallComplete() {
    const status = this.page.locator(this.selectors.toolCallStatus).first();
    await expect(status).toContainText('Completed', { timeout: 60000 });
  }

  /**
   * Wait for agentic response to complete (includes tool execution + final response)
   */
  async waitForAgenticResponseComplete() {
    // First wait for tool call to appear
    await this.waitForToolCall();

    // Wait for tool call to complete
    await this.waitForToolCallComplete();

    // Wait for final response from model
    await this.waitForResponseComplete();
  }

  // MCP servers rail-tab interactions (was the composer popover before V2)

  async openMcpsPopover() {
    await this.openSettingsPanel();
    await this.page.locator(this.selectors.mcpsTabTrigger).click();
    await expect(this.page.locator(this.selectors.mcpsPane)).toBeVisible();
  }

  async closeMcpsPopover() {
    // Switching back to the Parameters tab hides the MCP pane.
    await this.page.locator(this.selectors.parametersTabTrigger).click();
    await expect(this.page.locator(this.selectors.mcpsPane)).not.toBeVisible();
  }

  async expectMcpsPopoverTriggerVisible() {
    await this.openSettingsPanel();
    await expect(this.page.locator(this.selectors.mcpsTabTrigger)).toBeVisible();
  }

  async expectMcpsPopoverOpen() {
    await expect(this.page.locator(this.selectors.mcpsPane)).toBeVisible();
  }

  // A not-yet-added server is offered inside the add combobox.
  async expectMcpInPopover(mcpId) {
    await this.page.click(this.selectors.mcpAddTrigger);
    await expect(this.page.locator(this.selectors.mcpAddOption(mcpId))).toBeVisible();
  }

  async expandMcp(mcpId) {
    await this.page.locator(this.selectors.mcpExpand(mcpId)).click();
  }

  // Add a server to the chat (enables all its tools). Opens the combobox if it isn't already open.
  async enableMcp(mcpId) {
    const option = this.page.locator(this.selectors.mcpAddOption(mcpId));
    if (!(await option.isVisible())) {
      await this.page.click(this.selectors.mcpAddTrigger);
    }
    await expect(option).toBeVisible();
    await option.click();
    // The server now appears as an added row.
    await expect(this.page.locator(this.selectors.mcpItem(mcpId))).toBeVisible();
  }

  async enableMcpTool(mcpId, toolName) {
    const checkbox = this.page.locator(this.selectors.mcpToolCheckbox(mcpId, toolName));
    await expect(checkbox).toBeEnabled();
    await checkbox.click();
  }

  // "Added to this chat" → the server is shown as a row (no parent checkbox in the add-server model).
  async expectMcpCheckboxChecked(mcpId) {
    await expect(this.page.locator(this.selectors.mcpItem(mcpId))).toBeVisible();
  }

  async expectMcpBadgeVisible(count) {
    await this.openSettingsPanel();
    const badge = this.page.locator(this.selectors.mcpsBadge);
    await expect(badge).toBeVisible();
    await expect(badge).toHaveText(count.toString());
  }

  // The rail badge always renders; "has tools" means the count is > 0.
  async waitForMcpToolsBadge(timeout = McpFixtures.MCP_CONNECTION_TIMEOUT) {
    await this.openSettingsPanel();
    const badge = this.page.locator(this.selectors.mcpsBadge);
    await expect(badge).toBeVisible({ timeout });
    await expect(badge).not.toHaveText('0', { timeout });
  }

  async expectMcpBadgeNotVisible() {
    await this.openSettingsPanel();
    // No popover badge anymore; the rail-tab count reads 0 when nothing is enabled.
    await expect(this.page.locator(this.selectors.mcpsBadge)).toHaveText('0');
  }

  async expectMcpsEmptyState() {
    await expect(this.page.locator(this.selectors.mcpsEmptyState)).toBeVisible();
  }

  // Servers are loaded once they can be added (the combobox trigger) or are already added (a row).
  async waitForMcpsToLoad() {
    await this.page.waitForSelector(`${this.selectors.mcpAddTrigger}, [data-testid^="mcp-item-"]`, {
      timeout: 15000,
    });
  }
}
