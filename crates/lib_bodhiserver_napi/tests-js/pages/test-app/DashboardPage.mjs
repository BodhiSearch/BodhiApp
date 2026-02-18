export class DashboardPage {
  selectors = {
    page: '[data-testid="page-dashboard"]',
    pageLoaded: '[data-testid="page-dashboard"][data-test-state="loaded"]',
    tokenSection: '[data-testid="section-token"]',
    accessToken: '[data-testid="access-token"]',
    // User info section
    userInfoSection: '[data-testid="section-user-info"]',
    fetchUserButton: '[data-testid="btn-fetch-user"]',
    userInfoResponse: '[data-testid="user-info-response"]',
    userInfoError: '[data-testid="user-info-error"]',
    userInfoTerminal:
      '[data-testid="section-user-info"][data-test-state="success"], [data-testid="section-user-info"][data-test-state="error"]',
    // Navigation
    navLink: '[data-testid="nav-dashboard"]',
  };

  constructor(page) {
    this.page = page;
  }

  async navigateTo() {
    await this.page.click(this.selectors.navLink);
    await this.waitForLoaded();
  }

  async waitForLoaded() {
    await this.page.locator(this.selectors.pageLoaded).waitFor();
  }

  async getAccessToken() {
    const tokenElement = this.page.locator(this.selectors.accessToken);
    return await tokenElement.textContent();
  }

  async fetchUserInfo() {
    await this.page.click(this.selectors.fetchUserButton);
    await this.page.locator(this.selectors.userInfoTerminal).waitFor();
    const state = await this.page
      .locator(this.selectors.userInfoSection)
      .getAttribute('data-test-state');
    if (state === 'error') {
      const errorText = await this.page.locator(this.selectors.userInfoError).textContent();
      throw new Error(`User info fetch failed: ${errorText}`);
    }
    const text = await this.page.locator(this.selectors.userInfoResponse).textContent();
    return JSON.parse(text);
  }

  async getUserInfoResponse() {
    const text = await this.page.locator(this.selectors.userInfoResponse).textContent();
    return JSON.parse(text);
  }
}
