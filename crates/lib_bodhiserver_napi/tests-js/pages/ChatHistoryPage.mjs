import { expect } from '@playwright/test';
import { BasePage } from './BasePage.mjs';

export class ChatHistoryPage extends BasePage {
  selectors = {
    // History container
    historyContainer: '[data-testid="chat-history-container"]',

    // Chat items
    chatHistoryItem: (id) => `[data-testid="chat-history-item-${id}"]`,
    chatHistoryButton: (id) => `[data-testid="chat-history-button-${id}"]`,
    deleteChatButton: (id) => `[data-testid="delete-chat-${id}"]`,

    // History groups
    todayGroup: 'text=TODAY',
    yesterdayGroup: 'text=YESTERDAY',
    previousGroup: 'text=PREVIOUS 7 DAYS',

    // Toggle
    historyToggle: '[data-testid="chat-history-toggle"]',

    // New chat button
    newChatButton: '[data-testid="new-chat-button"]',
  };

  /**
   * Open chat history sidebar if not already open
   */
  async openHistorySidebar() {
    const historyContainer = this.page.locator(this.selectors.historyContainer);
    const isVisible = await historyContainer.isVisible();

    if (!isVisible) {
      await this.page.click(this.selectors.historyToggle);
      await expect(historyContainer).toBeVisible();
    }
  }

  /**
   * Close chat history sidebar if open
   */
  async closeHistorySidebar() {
    const historyContainer = this.page.locator(this.selectors.historyContainer);
    const isVisible = await historyContainer.isVisible();

    if (isVisible) {
      await this.page.click(this.selectors.historyToggle);
      await expect(historyContainer).not.toBeVisible();
    }
  }

  /**
   * Select a chat from history by chat title
   */
  async selectChatByTitle(chatTitle) {
    await this.openHistorySidebar();

    // Look for chat button containing the title
    const chatButtons = this.page.locator('[data-testid^="chat-history-button-"]');
    const count = await chatButtons.count();

    for (let i = 0; i < count; i++) {
      const buttonText = await chatButtons.nth(i).textContent();
      if (buttonText && buttonText.includes(chatTitle)) {
        await chatButtons.nth(i).click();
        await this.waitForSPAReady();
        return;
      }
    }

    throw new Error(`Chat with title "${chatTitle}" not found in history`);
  }

  /**
   * Select a chat from history by chat ID
   */
  async selectChatById(chatId) {
    await this.openHistorySidebar();

    const chatButton = this.page.locator(this.selectors.chatHistoryButton(chatId));
    await expect(chatButton).toBeVisible();
    await chatButton.click();
    await this.waitForSPAReady();
  }

  /**
   * Delete a chat from history by chat ID
   */
  async deleteChatById(chatId) {
    await this.openHistorySidebar();

    // Find the chat item and hover to make delete button visible
    const chatItem = this.page.locator(this.selectors.chatHistoryItem(chatId));
    await expect(chatItem).toBeVisible();
    await chatItem.hover();

    // Click the delete button
    const deleteButton = this.page.locator(this.selectors.deleteChatButton(chatId));
    await expect(deleteButton).toBeVisible();
    await deleteButton.click();

    await this.waitForSPAReady();
  }

  /**
   * Delete a chat from history by title
   */
  async deleteChatByTitle(chatTitle) {
    await this.openHistorySidebar();

    // Find chat item by title, then get its ID for deletion
    const chatButtons = this.page.locator('[data-testid^="chat-history-button-"]');
    const count = await chatButtons.count();

    for (let i = 0; i < count; i++) {
      const buttonText = await chatButtons.nth(i).textContent();
      if (buttonText && buttonText.includes(chatTitle)) {
        const chatId = await chatButtons.nth(i).getAttribute('data-testid');
        const id = chatId.replace('chat-history-button-', '');

        await this.deleteChatById(id);
        return;
      }
    }

    throw new Error(`Chat with title "${chatTitle}" not found in history`);
  }

  /**
   * Verify that specific chats exist in the history
   */
  async verifyChatsInHistory(expectedChatTitles) {
    await this.openHistorySidebar();

    for (const expectedTitle of expectedChatTitles) {
      await this.verifyChatExistsInHistory(expectedTitle);
    }
  }

  /**
   * Verify that a specific chat exists in the history
   */
  async verifyChatExistsInHistory(chatTitle) {
    await this.openHistorySidebar();

    // Wait for the chat with the specific title to appear in history
    // This uses Playwright's built-in waiting with timeout
    const chatButton = this.page.locator(
      `[data-testid^="chat-history-button-"]:has-text("${chatTitle}")`
    );
    await expect(chatButton).toBeVisible();
  }

  /**
   * Verify that a specific chat is currently selected/active
   */
  async verifyChatSelected(chatId) {
    await this.openHistorySidebar();

    const chatButton = this.page.locator(this.selectors.chatHistoryButton(chatId));

    // Check if the button has the active state (this might need adjustment based on actual CSS classes)
    await expect(chatButton).toHaveClass(/active|selected|bg-muted/);
  }

  /**
   * Get all chat titles from history
   */
  async getAllChatTitles() {
    await this.openHistorySidebar();

    const chatButtons = this.page.locator('[data-testid^="chat-history-button-"]');
    const count = await chatButtons.count();
    const titles = [];

    for (let i = 0; i < count; i++) {
      const title = await chatButtons.nth(i).textContent();
      if (title) {
        titles.push(title);
      }
    }

    return titles;
  }

  /**
   * Get count of chats in history
   */
  async getChatCount() {
    await this.openHistorySidebar();

    const chatButtons = this.page.locator('[data-testid^="chat-history-button-"]');
    return await chatButtons.count();
  }

  /**
   * Verify history is empty
   */
  async verifyHistoryEmpty() {
    await this.openHistorySidebar();

    const chatButtons = this.page.locator('[data-testid^="chat-history-button-"]');
    await expect(chatButtons).toHaveCount(0);
  }

  /**
   * Clear all chats from history (bulk delete - might need UI implementation)
   */
  async clearAllChats() {
    const titles = await this.getAllChatTitles();

    for (const title of titles) {
      await this.deleteChatByTitle(title);
    }
  }

  /**
   * Create a new chat using the history panel button
   */
  async createNewChatFromHistory() {
    await this.openHistorySidebar();

    await this.page.click(this.selectors.newChatButton);
    await this.waitForSPAReady();
  }

  /**
   * Verify chat groups are organized correctly (TODAY, YESTERDAY, PREVIOUS)
   */
  async verifyChatGrouping() {
    await this.openHistorySidebar();

    // Check if groups are present (they may not all be visible depending on chat ages)
    const groups = [
      this.selectors.todayGroup,
      this.selectors.yesterdayGroup,
      this.selectors.previousGroup,
    ];

    for (const group of groups) {
      const groupElement = this.page.locator(group);
      if (await groupElement.isVisible()) {
        // If group is visible, verify it has chats under it
        // This would need more specific implementation based on the HTML structure
      }
    }
  }

  /**
   * Test responsive behavior of chat history
   */
  async testResponsiveHistory(viewportWidth) {
    if (viewportWidth < 768) {
      // Mobile: history should be in drawer/overlay
      await this.page.click(this.selectors.historyToggle);

      // Verify history appears as overlay/drawer
      const historyContainer = this.page.locator(this.selectors.historyContainer);
      await expect(historyContainer).toBeVisible();

      // Should be able to close by clicking toggle again or outside
      await this.page.click(this.selectors.historyToggle);
      await expect(historyContainer).not.toBeVisible();
    } else {
      // Desktop: history should be a sidebar
      const historyContainer = this.page.locator(this.selectors.historyContainer);

      // May be visible by default on desktop
      if (!(await historyContainer.isVisible())) {
        await this.page.click(this.selectors.historyToggle);
        await expect(historyContainer).toBeVisible();
      }
    }
  }

  /**
   * Verify chat deletion with confirmation (if UI has confirmation dialog)
   */
  async deleteChatWithConfirmation(chatId) {
    await this.openHistorySidebar();

    const chatItem = this.page.locator(this.selectors.chatHistoryItem(chatId));
    await chatItem.hover();

    const deleteButton = this.page.locator(this.selectors.deleteChatButton(chatId));
    await deleteButton.click();

    // If there's a confirmation dialog, handle it
    const confirmButton = this.page.locator(
      'button:has-text("Delete"), button:has-text("Confirm")'
    );
    if (await confirmButton.isVisible()) {
      await confirmButton.click();
    }

    await this.waitForSPAReady();
  }

  /**
   * Verify chat search functionality (if implemented)
   */
  async searchChats(searchTerm) {
    await this.openHistorySidebar();

    // Look for search input (this might not be implemented yet)
    const searchInput = this.page.locator(
      '[data-testid="chat-search"], input[placeholder*="search" i]'
    );
    if (await searchInput.isVisible()) {
      await searchInput.fill(searchTerm);
      await this.page.waitForTimeout(500); // Wait for search results

      // Return filtered chat titles
      return await this.getAllChatTitles();
    }

    // If no search functionality, return all titles
    return await this.getAllChatTitles();
  }
}
