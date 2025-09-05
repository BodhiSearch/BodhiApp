import { expect } from '@playwright/test';
import { BasePage } from './BasePage.mjs';

export class ChatPage extends BasePage {
  constructor(page, baseUrl) {
    super(page, baseUrl);
  }

  selectors = {
    ...this.selectors,
    chatUI: '[data-testid="chat-ui"]',
    chatInput: '[data-testid="chat-input"]',
    sendButton: '[data-testid="send-button"]',
    messageContainer: '.space-y-2',
    userMessage: '[data-testid="user-message"]',
    assistantMessage: '[data-testid="assistant-message"]',
    assistantMessageContent: '[data-testid="assistant-message-content"]',
    modelSelectorLoaded: '[data-testid="model-selector-loaded"]',
    modelSelectorCombo: '#model-selector',
    newChatButton: 'button:has-text("New chat")',
    settingsButton: 'button[aria-label="Toggle settings"]',
    welcomeMessage: 'text=Welcome to Chat',
    emptyState: '.text-center'
  };

  async navigateToChat() {
    await this.navigate('/ui/chat/');
    await this.waitForSPAReady();
  }

  async expectChatPageReady() {
    // Wait for page to load
    await this.page.waitForLoadState('networkidle');
    
    // Check if we're redirected somewhere unexpected
    const currentUrl = this.page.url();
    if (!currentUrl.includes('/ui/chat')) {
      throw new Error(`Expected to be on chat page but got: ${currentUrl}`);
    }
    
    await this.expectVisible(this.selectors.chatUI);
    await this.expectVisible(this.selectors.chatInput);
    await this.expectVisible(this.selectors.sendButton);
  }

  async expectChatPageWithModel(modelName) {
    await this.expectChatPageReady();
    // Verify the URL contains the model parameter
    const url = new URL(this.page.url());
    expect(url.searchParams.get('model')).toBe(modelName);
    // The model should be automatically selected in the chat settings
  }

  async openSettings() {
    await this.expectVisible(this.selectors.settingsButton);
    await this.page.click(this.selectors.settingsButton);
    await this.page.waitForTimeout(500); // Wait for sidebar animation
  }

  async selectModel(modelName) {
    // First open settings panel
    await this.openSettings();
    
    // Wait for model selector to load
    await this.expectVisible(this.selectors.modelSelectorLoaded);
    
    // Click on the combo box
    await this.page.click(this.selectors.modelSelectorCombo);
    
    // Wait for dropdown and select the model
    await this.page.waitForSelector(`text=${modelName}`, { timeout: 5000 });
    await this.page.click(`text=${modelName}`);
    
    // Close settings panel
    await this.page.click(this.selectors.settingsButton);
    await this.page.waitForTimeout(500);
  }

  async sendMessage(message) {
    await this.expectVisible(this.selectors.chatInput);
    await this.page.fill(this.selectors.chatInput, message);
    
    const sendButton = this.page.locator(this.selectors.sendButton);
    await expect(sendButton).toBeVisible();
    await expect(sendButton).toBeEnabled();
    await sendButton.click();
  }

  async waitForAssistantResponse(timeout = 30000) {
    // Wait for assistant message to appear
    await expect(this.page.locator(this.selectors.assistantMessage).last()).toBeVisible({ timeout });
    
    // Wait for streaming to complete - give some time for response to stabilize
    await this.page.waitForTimeout(2000);
  }

  async getLastAssistantMessage() {
    const assistantMessageContent = this.page.locator(this.selectors.assistantMessageContent);
    const lastMessage = assistantMessageContent.last();
    await expect(lastMessage).toBeVisible();
    return await lastMessage.textContent();
  }
}