import { expect } from '@playwright/test';

export class OAuthSection {
  selectors = {
    // Keycloak login form selectors
    usernameField: '#username',
    passwordField: '#password',
    signInButton: 'button:has-text("Sign In")',
    consentYesButton: 'button:has-text("Yes")',
    // Post-login landing page (REST page)
    restPageLoaded: '[data-testid="page-rest"][data-test-state="loaded"]',
  };

  constructor(page) {
    this.page = page;
  }

  async waitForAccessRequestRedirect(bodhiServerUrl) {
    await this.page.waitForURL((url) => new URL(url).origin === new URL(bodhiServerUrl).origin);
  }

  async waitForAccessRequestCallback(testAppUrl) {
    await this.page.waitForURL((url) => {
      const parsed = new URL(url);
      return parsed.origin === new URL(testAppUrl).origin && parsed.pathname === '/access-callback' && parsed.searchParams.has('id');
    });
  }

  async waitForAuthServerRedirect(authServerUrl) {
    await this.page.waitForURL((url) => new URL(url).origin === authServerUrl);
  }

  async handleLogin(username, password) {
    await expect(this.page.locator(this.selectors.usernameField)).toBeVisible();
    await this.page.fill(this.selectors.usernameField, username);
    await this.page.fill(this.selectors.passwordField, password);
    await this.page.click(this.selectors.signInButton);
  }

  async handleConsent() {
    await expect(this.page.locator(this.selectors.consentYesButton)).toBeVisible();
    await this.page.click(this.selectors.consentYesButton);
  }

  async waitForTokenExchange(testAppUrl) {
    // Wait for redirect to /rest on the test app (default post-login landing)
    await this.page.waitForURL((url) => {
      const parsed = new URL(url);
      return parsed.origin === new URL(testAppUrl).origin && parsed.pathname === '/rest';
    });
    // Wait for REST page to be fully loaded via data-test-state
    await this.page.locator(this.selectors.restPageLoaded).waitFor();
  }

  async expectOAuthError(expectedError = 'invalid_scope') {
    // The React app redirects errors to /?error=...
    await this.page.waitForURL((url) => {
      const parsed = new URL(url);
      return parsed.searchParams.has('error');
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
