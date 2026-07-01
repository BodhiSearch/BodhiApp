export class ConfigSection {
  selectors = {
    form: '[data-testid="div-config-form"]',
    bodhiServerUrl: '[data-testid="input-bodhi-server-url"]',
    authServerUrl: '[data-testid="input-auth-server-url"]',
    realm: '[data-testid="input-realm"]',
    clientId: '[data-testid="input-client-id"]',
    redirectUri: '[data-testid="input-redirect-uri"]',
    scope: '[data-testid="input-scope"]',
    requestedRole: '[data-testid="select-requested-role"]',
    flowType: '[data-testid="select-flow-type"]',
    requested: '[data-testid="input-requested"]',
    confidentialToggle: '[data-testid="toggle-confidential"]',
    clientSecret: '[data-testid="input-client-secret"]',
    submitButton: '[data-testid="btn-request-access"]',
    errorSection: '[data-testid="error-section"]',
    loading: '[data-testid="access-request-loading"]',
    formError: '[data-testid="div-config-form"][data-test-state="error"]',
  };

  constructor(page) {
    this.page = page;
  }

  async configureOAuthForm({
    bodhiServerUrl,
    authServerUrl,
    realm,
    clientId,
    redirectUri,
    scope,
    requestedRole,
    flowType,
    requested,
  }) {
    await this.page.fill(this.selectors.bodhiServerUrl, bodhiServerUrl);
    await this.page.fill(this.selectors.authServerUrl, authServerUrl);
    await this.page.fill(this.selectors.realm, realm);
    await this.page.fill(this.selectors.clientId, clientId);
    await this.page.fill(this.selectors.redirectUri, redirectUri);
    await this.page.fill(this.selectors.scope, scope);
    if (requestedRole) {
      await this.setRequestedRole(requestedRole);
    }
    if (flowType) {
      await this.setFlowType(flowType);
    }
    await this.page.fill(this.selectors.requested, requested || '');
  }

  async setRequestedRole(value) {
    await this.page.selectOption(this.selectors.requestedRole, value);
  }

  async setFlowType(value) {
    await this.page.selectOption(this.selectors.flowType, value);
  }

  async submitAccessRequest() {
    await this.page.click(this.selectors.submitButton);
  }

  async getFormState() {
    return await this.page.locator(this.selectors.form).getAttribute('data-test-state');
  }
}
