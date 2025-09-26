import { BasePage } from '@/pages/BasePage.mjs';

export class OAuth2TestAppPage extends BasePage {
  constructor(page, testAppUrl) {
    super(page, testAppUrl);
  }

  selectors = {
    authServerUrlInput: '#auth-server-url',
    realmInput: '#realm',
    clientIdInput: '#client-id',
    redirectUriInput: '#redirect-uri',
    scopeInput: '#scope',
    submitButton: 'button[type="submit"]',
    consentYesButton: 'button:has-text("Yes")',
    successSection: '#success-section',
    accessToken: '#access-token',
  };

  async navigateToTestApp(redirectUri) {
    await this.page.goto(redirectUri);
    await this.page.waitForLoadState('networkidle');
  }

  async configureOAuthForm(authUrl, realm, clientId, redirectUri, scopes) {
    await this.page.fill(this.selectors.authServerUrlInput, authUrl);
    await this.page.fill(this.selectors.realmInput, realm);
    await this.page.fill(this.selectors.clientIdInput, clientId);
    await this.page.fill(this.selectors.redirectUriInput, redirectUri);
    await this.page.fill(this.selectors.scopeInput, scopes);
  }

  async startOAuthFlow() {
    await this.page.click(this.selectors.submitButton);
  }

  async waitForAuthServerRedirect(authServerUrl) {
    await this.page.waitForURL((url) => new URL(url).origin === authServerUrl);
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
}
