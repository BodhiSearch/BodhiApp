import { BasePage } from '@/pages/BasePage.mjs';

export class OAuth2TestAppPage extends BasePage {
  selectors = {
    bodhiServerUrlInput: '#bodhi-server-url',
    authServerUrlInput: '#auth-server-url',
    realmInput: '#realm',
    clientIdInput: '#client-id',
    redirectUriInput: '#redirect-uri',
    scopeInput: '#scope',
    requestedToolsetsInput: '#requested-toolsets',
    submitButton: 'button[type="submit"]',
    // Loading sections
    accessRequestLoading: '#access-request-loading',
    accessCallbackLoading: '#access-callback-loading',
    // Keycloak login form selectors
    usernameField: '#username',
    passwordField: '#password',
    signInButton: 'button:has-text("Sign In")',
    // Consent screen selectors
    consentYesButton: 'button:has-text("Yes")',
    successSection: '#success-section',
    accessToken: '#access-token',
    // Scope status
    scopeStatusMessage: '#scope-status-message',
  };

  async navigateToTestApp(redirectUri) {
    await this.page.goto(redirectUri);
    await this.page.waitForLoadState('networkidle');
  }

  async configureOAuthForm(bodhiServerUrl, authUrl, realm, clientId, redirectUri, scopes, requestedToolsets) {
    await this.page.fill(this.selectors.bodhiServerUrlInput, bodhiServerUrl);
    await this.page.fill(this.selectors.authServerUrlInput, authUrl);
    await this.page.fill(this.selectors.realmInput, realm);
    await this.page.fill(this.selectors.clientIdInput, clientId);
    await this.page.fill(this.selectors.redirectUriInput, redirectUri);
    await this.page.fill(this.selectors.scopeInput, scopes);
    if (requestedToolsets) {
      await this.page.fill(this.selectors.requestedToolsetsInput, requestedToolsets);
    }
  }

  async submitAccessRequest() {
    await this.page.click(this.selectors.submitButton);
  }

  async waitForLoginReady() {
    await this.page.waitForSelector('button[data-test-state="login"]');
  }

  async clickLogin() {
    await this.waitForLoginReady();
    await this.page.click(this.selectors.submitButton);
  }

  async setScopes(value) {
    await this.page.fill(this.selectors.scopeInput, value);
  }

  async getResourceScope() {
    return await this.page.locator(this.selectors.scopeStatusMessage).getAttribute('data-test-resource-scope');
  }

  async getAccessRequestScope() {
    return await this.page.locator(this.selectors.scopeStatusMessage).getAttribute('data-test-access-request-scope');
  }

  async waitForAccessRequestRedirect(bodhiServerUrl) {
    await this.page.waitForURL((url) => new URL(url).origin === new URL(bodhiServerUrl).origin);
  }

  async waitForAccessRequestCallback(testAppUrl) {
    await this.page.waitForURL((url) => {
      var parsed = new URL(url);
      return parsed.origin === new URL(testAppUrl).origin && parsed.searchParams.has('id');
    });
  }

  async waitForAuthServerRedirect(authServerUrl) {
    await this.page.waitForURL((url) => new URL(url).origin === authServerUrl);
  }

  /**
   * Handle Keycloak login form - fill credentials and submit
   * @param {string} username - Username for login
   * @param {string} password - Password for login
   */
  async handleLogin(username, password) {
    await this.expectVisible(this.selectors.usernameField);
    await this.page.fill(this.selectors.usernameField, username);
    await this.page.fill(this.selectors.passwordField, password);
    await this.page.click(this.selectors.signInButton);
  }

  async handleConsent() {
    await this.expectVisible(this.selectors.consentYesButton);
    await this.page.click(this.selectors.consentYesButton);
  }

  async waitForTokenExchange(testAppUrl) {
    await this.page.waitForURL((url) => new URL(url).origin === new URL(testAppUrl).origin);
    await this.page.waitForLoadState('networkidle');
    await this.expectVisible(this.selectors.successSection);
  }

  async getAccessToken() {
    const tokenElement = await this.page.locator(this.selectors.accessToken);
    return await tokenElement.textContent();
  }

  /**
   * Wait for and verify OAuth error redirect
   * When Keycloak rejects an OAuth request (e.g., invalid_scope), it redirects
   * back to the redirect_uri with error params in the URL
   * @param {string} expectedError - Expected error code (default: 'invalid_scope')
   * @returns {Promise<{error: string, errorDescription: string}>} Error details from redirect
   */
  async expectOAuthError(expectedError = 'invalid_scope') {
    // Wait for redirect back to test app with error params
    await this.page.waitForURL((url) => {
      const params = new URL(url).searchParams;
      return params.has('error');
    });

    const url = new URL(this.page.url());
    const error = url.searchParams.get('error');
    const errorDescription = url.searchParams.get('error_description');

    if (expectedError && error !== expectedError) {
      throw new Error(`Expected OAuth error '${expectedError}' but got '${error}'`);
    }

    return { error, errorDescription };
  }
}
