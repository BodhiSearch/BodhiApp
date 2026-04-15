export class ChatPage {
  selectors = {
    page: '[data-testid="page-chat"]',
    pageLoaded: '[data-testid="page-chat"][data-test-state="loaded"]',
    section: '[data-testid="section-chat"]',
    modelSelect: '[data-testid="chat-model-select"]',
    modelsLoaded: '[data-testid="chat-model-select"][data-test-state="loaded"]',
    chatInput: '[data-testid="chat-input"]',
    sendButton: '[data-testid="btn-chat-send"]',
    messages: '[data-testid="chat-messages"]',
    statusBadge: '[data-testid="chat-status"]',
    error: '[data-testid="chat-error"]',
    terminal:
      '[data-testid="section-chat"][data-test-state="idle"], [data-testid="section-chat"][data-test-state="error"]',
    navLink: '[data-testid="nav-chat"]',
  };

  constructor(page) {
    this.page = page;
  }

  async navigateTo() {
    await this.page.click(this.selectors.navLink);
    await this.page.locator(this.selectors.pageLoaded).waitFor();
  }

  async selectModel(modelId) {
    await this.page.locator(this.selectors.modelSelect).scrollIntoViewIfNeeded();
    await this.page.selectOption(this.selectors.modelSelect, modelId);
  }

  async waitForModelsLoaded() {
    await this.page.locator(this.selectors.modelsLoaded).waitFor();
  }

  async getModels() {
    return await this.page.evaluate(() => {
      const select = document.querySelector('[data-testid="chat-model-select"]');
      return Array.from(select.options)
        .filter((o) => o.value)
        .map((o) => o.value);
    });
  }

  async sendMessage(text) {
    const input = this.page.locator(this.selectors.chatInput);
    await input.scrollIntoViewIfNeeded();
    await input.fill(text);
    await input.press('Enter');
  }

  async waitForResponse() {
    await this.page.locator(this.selectors.terminal).waitFor();
  }

  async getLastResponse() {
    const messages = await this.page
      .locator('[data-testid="chat-messages"] .justify-start p.whitespace-pre-wrap')
      .all();
    if (messages.length === 0) return null;
    return await messages[messages.length - 1].textContent();
  }

  async getStatus() {
    return await this.page.locator(this.selectors.section).getAttribute('data-test-state');
  }
}
