import { expect } from '@playwright/test';
import { SetupBasePage } from '@/pages/SetupBasePage.mjs';

export class SetupResourceAdminPage extends SetupBasePage {
  constructor(page, baseUrl, authServerConfig, testCredentials) {
    super(page, baseUrl);
    this.authServerConfig = authServerConfig;
    this.testCredentials = testCredentials;
  }

  selectors = {
    ...this.selectors,
    adminSetupTitle: 'text=Admin Setup',
    continueWithLoginButton: 'button:has-text("Continue with Login")',
    // Auth server selectors
    signInTitle: 'text=Sign in to your account',
    usernameInput: 'input[name="username"]',
    passwordInput: 'input[name="password"]',
    submitButton: 'button[type="submit"]',
  };

  async navigateToResourceAdmin() {
    await this.navigate('/ui/setup/resource-admin/');
    await this.waitForSetupPage();
  }

  async expectResourceAdminPage() {
    await this.page.waitForURL((url) => url.pathname === '/ui/setup/resource-admin/');
    await this.expectVisible(this.selectors.adminSetupTitle);
    await this.expectStepIndicator(2);
    await this.expectVisible(this.selectors.continueWithLoginButton);
  }

  async clickContinueWithLogin() {
    await this.expectVisible(this.selectors.continueWithLoginButton);
    await this.page.click(this.selectors.continueWithLoginButton);
    // Wait for redirect to auth server
    await this.page.waitForURL((url) => url.origin === this.authServerConfig.authUrl);
  }

  async expectAuthServerLogin() {
    await this.expectVisible(this.selectors.signInTitle);
    await this.expectVisible(this.selectors.usernameInput);
    await this.expectVisible(this.selectors.passwordInput);
    await this.expectVisible(this.selectors.submitButton);
  }

  async fillAuthCredentials(username = null, password = null) {
    const user = username || this.testCredentials.username;
    const pass = password || this.testCredentials.password;

    await this.page.fill(this.selectors.usernameInput, user);
    await this.page.fill(this.selectors.passwordInput, pass);
  }

  async submitLogin() {
    await this.page.click(this.selectors.submitButton);
    // Wait for redirect back to app
    await this.page.waitForURL((url) => url.origin === this.baseUrl);
  }

  async performCompleteLogin() {
    await this.clickContinueWithLogin();
    await this.expectAuthServerLogin();
    await this.fillAuthCredentials();
    await this.submitLogin();
    await this.page.waitForURL((url) => !url.pathname.includes('/ui/auth/callback/'), {
      timeout: 60000,
    });
  }
}
