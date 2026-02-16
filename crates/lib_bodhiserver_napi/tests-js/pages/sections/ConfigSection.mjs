export class ConfigSection {
  selectors = {
    form: '[data-testid="div-config-form"]',
    bodhiServerUrl: '[data-testid="input-bodhi-server-url"]',
    authServerUrl: '[data-testid="input-auth-server-url"]',
    realm: '[data-testid="input-realm"]',
    clientId: '[data-testid="input-client-id"]',
    redirectUri: '[data-testid="input-redirect-uri"]',
    scope: '[data-testid="input-scope"]',
    requestedToolsets: '[data-testid="input-requested-toolsets"]',
    confidentialToggle: '[data-testid="toggle-confidential"]',
    clientSecret: '[data-testid="input-client-secret"]',
    submitButton: '[data-testid="btn-request-access"]',
    errorSection: '[data-testid="error-section"]',
    loading: '[data-testid="access-request-loading"]',
    // State-based selectors
    formLogin: '[data-testid="div-config-form"][data-test-state="login"]',
    formError: '[data-testid="div-config-form"][data-test-state="error"]',
    // Terminal state after submitting access request: login (approved) or error
    terminal: '[data-testid="div-config-form"][data-test-state="login"], [data-testid="div-config-form"][data-test-state="error"]',
    buttonLogin: '[data-testid="btn-request-access"][data-test-state="login"]',
  };

  constructor(page) {
    this.page = page;
  }

  async configureOAuthForm({ bodhiServerUrl, authServerUrl, realm, clientId, redirectUri, scope, requestedToolsets }) {
    await this.page.fill(this.selectors.bodhiServerUrl, bodhiServerUrl);
    await this.page.fill(this.selectors.authServerUrl, authServerUrl);
    await this.page.fill(this.selectors.realm, realm);
    await this.page.fill(this.selectors.clientId, clientId);
    await this.page.fill(this.selectors.redirectUri, redirectUri);
    await this.page.fill(this.selectors.scope, scope);
    await this.page.fill(this.selectors.requestedToolsets, requestedToolsets || '');
  }

  async submitAccessRequest() {
    await this.page.click(this.selectors.submitButton);
  }

  async waitForLoginReady() {
    // Wait for form state to transition to "login" (auto-approved)
    await this.page.locator(this.selectors.buttonLogin).waitFor();
  }

  async clickLogin() {
    await this.waitForLoginReady();
    await this.page.click(this.selectors.submitButton);
  }

  async setScopes(value) {
    await this.page.fill(this.selectors.scope, value);
  }

  async getScopeValue() {
    return await this.page.inputValue(this.selectors.scope);
  }

  async getResourceScope() {
    return await this.page.locator('[data-test-resource-scope]').getAttribute('data-test-resource-scope');
  }

  async getAccessRequestScope() {
    return await this.page.locator('[data-test-access-request-scope]').getAttribute('data-test-access-request-scope');
  }

  async getFormState() {
    return await this.page.locator(this.selectors.form).getAttribute('data-test-state');
  }
}
