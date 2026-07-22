import { BasePage } from '@/pages/BasePage.mjs';
import { withRetry } from '@/utils/retry.mjs';
import { expect } from '@playwright/test';

export class LoginPage extends BasePage {
  constructor(page, baseUrl, authServerConfig, testCredentials) {
    super(page, baseUrl);
    this.authServerConfig = authServerConfig;
    this.testCredentials = testCredentials;
  }

  selectors = {
    loginButton: 'button:has-text("Login")',
    usernameField: '#username',
    passwordField: '#password',
    signInButton: 'button:has-text("Sign In")',
  };

  async performOAuthLoginFromSession() {
    const loginButton = this.page.locator(this.selectors.loginButton);
    await loginButton.first().click();
    await this.waitForSPAReady();
    await this.page.waitForURL((url) => url.origin === this.baseUrl);
  }

  async performOAuthLogin(expectedRedirectPath = '/ui/chat/') {
    await withRetry((attempt) => this.oauthLoginAttempt(expectedRedirectPath, attempt), {
      label: 'OAuth login',
    });
  }

  async oauthLoginAttempt(expectedRedirectPath, attempt) {
    // Retries re-navigate to reset the flow; the first attempt reuses the login page if already there.
    if (attempt > 1 || !this.page.url().includes('/ui/login')) {
      await this.navigate('/ui/login');
    }

    const loginButton = this.page.locator(this.selectors.loginButton);
    await loginButton.first().click();

    // `commit` (not `load`): the auth server's login page can stall its load event on a
    // slow subresource while still interactive; the fill() below auto-waits for #username.
    await this.page.waitForURL((url) => url.origin === this.authServerConfig.authUrl, {
      waitUntil: 'commit',
    });

    await this.page.fill(this.selectors.usernameField, this.testCredentials.username);
    await this.page.fill(this.selectors.passwordField, this.testCredentials.password);
    await this.page.click(this.selectors.signInButton);

    if (expectedRedirectPath) {
      await this.page.waitForURL(
        (url) => url.origin === this.baseUrl && url.pathname === expectedRedirectPath
      );
    } else {
      await this.page.waitForURL((url) => url.origin === this.baseUrl);
    }

    await this.waitForSPAReady();
  }

  async navigateToLogin() {
    await this.navigate('/ui/login');
  }

  async expectLoginPage() {
    await this.expectVisible(this.selectors.loginButton);
  }

  async expectLoginPageVisible() {
    const loginButton = this.page.locator(
      'button:has-text("Log In"), button:has-text("Login"), button:has-text("Sign In"), button[type="submit"]'
    );
    await expect(loginButton.first()).toBeVisible();
  }

  async clickLogin() {
    await this.page.click(this.selectors.loginButton);
  }

  async waitForAuthServer() {
    // `commit`: the auth server's login page can stall its load event on a slow subresource.
    await this.page.waitForURL((url) => url.origin === this.authServerConfig.authUrl, {
      waitUntil: 'commit',
    });
  }

  async fillCredentials(username = null, password = null) {
    const user = username || this.testCredentials.username;
    const pass = password || this.testCredentials.password;

    await this.page.fill(this.selectors.usernameField, user);
    await this.page.fill(this.selectors.passwordField, pass);
  }

  async submitLogin() {
    await this.page.click(this.selectors.signInButton);
  }

  async waitForSuccessfulLogin() {
    await this.page.waitForURL(
      (url) => url.origin === this.baseUrl && url.pathname === '/ui/chat/'
    );
  }
}
