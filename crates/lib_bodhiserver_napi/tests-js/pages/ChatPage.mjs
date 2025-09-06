import { expect } from '@playwright/test';
import { BasePage } from './BasePage.mjs';

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
    newChatInlineButton: '[data-testid="new-chat-inline-button"]',
    emptyState: '[data-testid="empty-chat-state"]',

    // Model selection (in settings panel)
    modelSelectorLoaded: '[data-testid="model-selector-loaded"]',
    comboboxTrigger: '[data-testid="model-selector-trigger"]',
    comboboxOption: (modelName) => `[data-testid="combobox-option-${modelName}"]`,

    // Settings
    settingsSidebar: '[data-testid="settings-sidebar"]',
    settingsToggle: '[data-testid="settings-toggle-button"]',
    chatHistoryToggle: '[data-testid="chat-history-toggle"]',
  };

  /**
   * Navigate to chat page
   */
  async navigateToChat() {
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
  async sendMessage(message) {
    const sendButton = await this.sendMessageAndReturn(message);
    await expect(sendButton).toBeDisabled();
    await this.waitForLatestUserMessage();
  }

  async sendMessageAndReturn(message) {
    await this.page.fill(this.selectors.messageInput, message);
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
      await expect(this.page.locator(this.selectors.assistantMessage).last()).toContainText(
        expectedContent
      );
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
    await expect(lastAssistantMessage).toBeVisible({ timeout: 20000 }); // wait longer for chat messages
    await expect(this.page.locator(this.selectors.latestAiMessage)).toBeVisible();
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
    const visibleOption = modelOption.locator('visible=true').first();
    await expect(visibleOption).toBeVisible();
    await visibleOption.click();
  }

  /**
   * Verify that a model is selected
   */
  async verifyModelSelected(modelName) {
    await this.openSettingsPanel();

    // Check that the combobox shows the selected model
    const trigger = this.page.locator(this.selectors.comboboxTrigger);
    await expect(trigger).toContainText(modelName);
  }

  // Chat management

  /**
   * Start a new chat conversation
   */
  async startNewChat() {
    await this.page.click(this.selectors.newChatButton);
    await this.waitForSPAReady();
  }

  /**
   * Start a new chat using the inline button (+ button in chat input)
   */
  async startNewChatInline() {
    await this.page.click(this.selectors.newChatInlineButton);
    await this.waitForSPAReady();
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
    const messages = this.page.locator(`[data-testid="${role}-message"]`);
    let found = false;

    const count = await messages.count();
    for (let i = 0; i < count; i++) {
      const messageContent = await messages
        .nth(i)
        .locator(`[data-testid="${role}-message-content"]`)
        .textContent();
      if (messageContent && messageContent.includes(expectedContent)) {
        found = true;
        break;
      }
    }

    expect(found).toBe(true);
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
  async expectNetworkError() {
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
    await this.page.route('**/v1/chat/completions', (route) => route.abort());
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

    // The model selector should contain the expected model name
    await expect(modelSelector).toContainText(modelName);

    // Verify the message input is enabled (not showing "Please select a model first")
    const messageInput = this.page.locator(this.selectors.messageInput);
    await expect(messageInput).toBeVisible();
    await expect(messageInput).not.toHaveAttribute('placeholder', 'Please select a model first');
  }
}
