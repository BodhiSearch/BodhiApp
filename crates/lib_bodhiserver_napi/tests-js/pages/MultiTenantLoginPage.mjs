import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

export class MultiTenantLoginPage extends BasePage {
  constructor(page, baseUrl, authServerConfig, testCredentials) {
    super(page, baseUrl);
    this.authServerConfig = authServerConfig;
    this.testCredentials = testCredentials;
  }

  selectors = {
    usernameField: '#username',
    passwordField: '#password',
    signInButton: 'button:has-text("Sign In")',
    loginPage: '[data-testid="login-page"]',
  };

  /**
   * Perform dashboard OAuth login (platform login).
   * Clicks "Login to Bodhi Platform" → fills KC credentials → waits for redirect.
   * Handles SSO (Keycloak auto-redirect without login form) and
   * already-authenticated (dashboard token valid, no KC redirect) scenarios.
   */
  async performDashboardLogin(credentials = null) {
    const creds = credentials || this.testCredentials;

    if (!this.page.url().includes('/ui/login')) {
      await this.navigate('/ui/login');
    }

    // Wait for React to render a meaningful state (loading → actual state)
    // data-test-state appears after API calls complete and the page decides which state to show
    try {
      await this.page
        .locator('[data-test-state]')
        .first()
        .waitFor({ state: 'visible', timeout: 15000 });
    } catch {
      // Page might have redirected (e.g., setup status redirect) — no login needed
      return;
    }

    // Check if it's State A (login button visible)
    const isLoginState = await this.page.locator('[data-test-state="login"]').isVisible();
    if (!isLoginState) {
      // Already authenticated (tenant selection, welcome, etc.)
      return;
    }

    // Click login and intercept the initiate API response to determine flow
    const loginButton = this.page.locator('[data-test-action="Login to Bodhi Platform"]');
    const [initiateResponse] = await Promise.all([
      this.page.waitForResponse((resp) => resp.url().includes('/bodhi/v1/auth/dashboard/initiate')),
      loginButton.click(),
    ]);

    if (initiateResponse.status() === 200) {
      // Dashboard token already valid — no OAuth redirect needed
      return;
    }

    // OAuth flow initiated (status 201) — browser will navigate to KC
    // Wait for URL to leave the login page
    await this.page.waitForURL((url) => !url.pathname.startsWith('/ui/login'), { timeout: 30000 });

    // Determine if we're at KC login form or SSO redirected past it
    const currentOrigin = new URL(this.page.url()).origin;
    if (currentOrigin === this.authServerConfig.authUrl) {
      // KC login form — fill credentials and submit
      await this.page.fill(this.selectors.usernameField, creds.username);
      await this.page.fill(this.selectors.passwordField, creds.password);
      await this.page.click(this.selectors.signInButton);
    }

    // Wait for the full redirect chain to complete (KC/callback → non-auth app page)
    const pastAuthCallback = (url) =>
      url.origin === this.baseUrl && !url.pathname.includes('/auth/');
    await this.page.waitForURL(pastAuthCallback, { timeout: 30000 });
    await this.waitForSPAReady();
  }

  /**
   * Wait for tenant selection state (State B2).
   */
  async waitForTenantSelection() {
    await expect(this.page.locator('[data-test-state="select"]')).toBeVisible();
  }

  /**
   * Select a tenant by its display name.
   */
  async selectTenant(tenantName) {
    const tenantButton = this.page.locator(`[data-test-action="${tenantName}"]`);
    await tenantButton.click();
  }

  /**
   * Switch to a different tenant by name (from State C).
   */
  async switchToTenant(tenantName) {
    const switchButton = this.page.locator(`[data-test-action="Switch to ${tenantName}"]`);
    await switchButton.click();
  }

  /**
   * Assert State A (login button visible, no session).
   */
  async expectStateA() {
    await expect(this.page.locator('[data-test-state="login"]')).toBeVisible();
  }

  /**
   * Assert State C (welcome, fully authenticated).
   */
  async expectStateC() {
    await expect(this.page.locator('[data-test-state="welcome"]')).toBeVisible();
  }

  /**
   * Click logout from State C.
   */
  async logout() {
    const logoutButton = this.page.locator('[data-test-action="Log Out"]');
    await logoutButton.click();
  }

  /**
   * Wait for SSO auto-completion and redirect to chat.
   */
  async waitForAutoLogin() {
    await this.page.waitForURL(
      (url) => url.origin === this.baseUrl && url.pathname === '/ui/chat/',
      { timeout: 30000 }
    );
    await this.waitForSPAReady();
  }

  /**
   * Wait for redirect to setup/tenants page (no tenants registered).
   */
  async waitForTenantSetup() {
    await this.page.waitForURL(
      (url) => url.origin === this.baseUrl && url.pathname.includes('/ui/setup/tenants'),
      { timeout: 30000 }
    );
    await this.waitForSPAReady();
  }
}
