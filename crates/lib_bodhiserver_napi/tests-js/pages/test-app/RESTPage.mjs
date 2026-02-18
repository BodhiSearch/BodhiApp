export class RESTPage {
  selectors = {
    page: '[data-testid="page-rest"]',
    pageLoaded: '[data-testid="page-rest"][data-test-state="loaded"]',
    section: '[data-testid="section-rest-client"]',
    serverUrl: '[data-testid="rest-server-url"]',
    methodSelect: '[data-testid="select-rest-method"]',
    urlInput: '[data-testid="input-rest-url"]',
    headersInput: '[data-testid="input-rest-headers"]',
    bodyInput: '[data-testid="input-rest-body"]',
    authToggle: '[data-testid="toggle-rest-auth"]',
    jsonToggle: '[data-testid="toggle-rest-json"]',
    sendButton: '[data-testid="btn-rest-send"]',
    responseStatus: '[data-testid="rest-response-status"]',
    response: '[data-testid="rest-response"]',
    error: '[data-testid="rest-error"]',
    terminal:
      '[data-testid="section-rest-client"][data-test-state="success"], [data-testid="section-rest-client"][data-test-state="error"]',
    navLink: '[data-testid="nav-rest"]',
  };

  constructor(page) {
    this.page = page;
  }

  async navigateTo() {
    await this.page.click(this.selectors.navLink);
    await this.page.locator(this.selectors.pageLoaded).waitFor();
  }

  async waitForLoaded() {
    await this.page.locator(this.selectors.pageLoaded).waitFor();
  }

  async getServerUrl() {
    return await this.page.locator(this.selectors.serverUrl).textContent();
  }

  async sendRequest({ method, url, headers, body, useAuth = true, json = true }) {
    if (method) {
      await this.page.selectOption(this.selectors.methodSelect, method);
    }
    await this.page.fill(this.selectors.urlInput, url);
    if (headers) {
      await this.page.fill(this.selectors.headersInput, headers);
    }
    if (body) {
      await this.page.fill(
        this.selectors.bodyInput,
        typeof body === 'string' ? body : JSON.stringify(body)
      );
    }
    const authCheckbox = this.page.locator(this.selectors.authToggle);
    const authChecked = await authCheckbox.isChecked();
    if (useAuth !== authChecked) {
      await authCheckbox.click();
    }
    const jsonCheckbox = this.page.locator(this.selectors.jsonToggle);
    const jsonChecked = await jsonCheckbox.isChecked();
    if (json !== jsonChecked) {
      await jsonCheckbox.click();
    }
    await this.page.click(this.selectors.sendButton);
    await this.page.locator(this.selectors.terminal).waitFor();
  }

  async getResponseStatus() {
    const text = await this.page.locator(this.selectors.responseStatus).textContent();
    const match = text.match(/Status:\s*(\d+)/);
    return match ? parseInt(match[1]) : null;
  }

  async getResponse() {
    const text = await this.page.locator(this.selectors.response).textContent();
    try {
      return JSON.parse(text);
    } catch {
      return text;
    }
  }

  async getState() {
    return await this.page.locator(this.selectors.section).getAttribute('data-test-state');
  }
}
