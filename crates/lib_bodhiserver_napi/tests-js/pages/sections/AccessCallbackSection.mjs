export class AccessCallbackSection {
  selectors = {
    container: '[data-testid="div-access-callback"]',
    scopeInput: '[data-testid="input-scope"]',
    loginButton: '[data-testid="btn-login"]',
    errorSection: '[data-testid="error-section"]',
    // Terminal state: ready (approved, can login) OR error
    terminal:
      '[data-testid="div-access-callback"][data-test-state="ready"], [data-testid="div-access-callback"][data-test-state="error"]',
  };

  constructor(page) {
    this.page = page;
  }

  async waitForLoaded() {
    // Wait for terminal state (ready or error) instead of just hiding loading
    await this.page.locator(this.selectors.terminal).waitFor();
  }

  async getResourceScope() {
    return await this.page
      .locator('[data-test-resource-scope]')
      .getAttribute('data-test-resource-scope');
  }

  async getAccessRequestScope() {
    return await this.page
      .locator('[data-test-access-request-scope]')
      .getAttribute('data-test-access-request-scope');
  }

  async setScopes(value) {
    await this.page.fill(this.selectors.scopeInput, value);
  }

  async getScopeValue() {
    return await this.page.inputValue(this.selectors.scopeInput);
  }

  async clickLogin() {
    await this.page.click(this.selectors.loginButton);
  }

  async getState() {
    return await this.page.locator(this.selectors.container).getAttribute('data-test-state');
  }
}
