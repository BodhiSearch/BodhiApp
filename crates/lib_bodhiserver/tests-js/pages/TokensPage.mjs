import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

export class TokensPage extends BasePage {
  selectors = {
    // Page elements
    tokensPage: '[data-testid="tokens-page"]',

    // New API Token page (dialog → page in V2)
    newTokenPage: '[data-testid="new-token-page"]',
    tokenForm: '[data-testid="token-form"]',
    tokenNameInput: '[data-testid="token-name-input"]',
    tokenScopeSelect: '[data-testid="token-scope-select"]',
    generateButton: '[data-testid="generate-token-button"]',

    // List elements (V2 list rows; no <table>)
    tokensTable: '[data-testid="tokens-table"]',
    tokenRow: (id) => `[data-testid="token-row-${id}"]`,
    tokenName: (id) => `[data-testid="token-name-${id}"]`,
    tokenScope: (id) => `[data-testid="token-scope-${id}"]`,
    statusSwitch: (id) => `[data-testid="token-status-switch-${id}"]`,
    listRow: '[data-testid^="token-row-"]',

    // Token reveal dialog (after creation) — unchanged across the migration
    tokenDialog: '[data-testid="token-dialog"]',
    tokenValueInput: '[data-testid="token-value-input"]',
    copyButton: '[data-testid="copy-content"]',
    doneButton: '[data-testid="token-dialog-done"]',
    showHideButton: '[data-testid="toggle-show-content"]',
  };

  /**
   * Navigate to tokens page (via the shell nav when already inside the app, or by URL).
   */
  async navigateToTokens() {
    await this.navigate('/ui/tokens/');
    await this.waitForSPAReady();
    await this.page.waitForSelector('[data-testid="tokens-page"][data-pagestatus="ready"]');
  }

  /**
   * Navigate to the Access Tokens section / API Tokens via the shell nav (black-box nav).
   */
  async navigateToTokensViaShell() {
    await this.navViaShell('api-keys', 'api-tokens');
    await this.page.waitForSelector('[data-testid="tokens-page"][data-pagestatus="ready"]');
  }

  async expectTokensPage() {
    await expect(this.page.locator(this.selectors.tokensPage)).toBeVisible();
  }

  /**
   * Open the New API Token page. "New API Token" lives in the Access Tokens shell nav
   * (the header button was removed to match the design), so navigate via the nav.
   */
  async openNewTokenPage() {
    await this.navViaShell('api-keys', 'new-token');
    await this.page.waitForURL(/\/ui\/tokens\/new\/?$/);
    await this.page.waitForSelector(this.selectors.tokenForm);
  }

  /**
   * Select token scope from dropdown.
   * @param {string} scope - 'scope_token_user' or 'scope_token_power_user'
   */
  async selectScope(scope = 'scope_token_user') {
    const scopeSelect = this.page.locator(this.selectors.tokenScopeSelect);
    await expect(scopeSelect).toBeVisible();
    await scopeSelect.click();

    const scopeOption = this.page.locator(`[data-testid="scope-option-${scope}"]`);
    await expect(scopeOption).toBeVisible();
    await scopeOption.click();
  }

  /**
   * Create a new token with optional name and scope, ending on the reveal dialog.
   * @param {string} name - Optional token name
   * @param {string} scope - Token scope
   */
  async createToken(name = '', scope = 'scope_token_user') {
    await this.openNewTokenPage();

    await this.selectScope(scope);

    if (name) {
      await this.page.locator(this.selectors.tokenNameInput).fill(name);
    }

    await this.page.locator(this.selectors.generateButton).click();

    await this.page.waitForSelector(this.selectors.tokenDialog);
    await expect(this.page.locator(this.selectors.tokenDialog)).toBeVisible();
  }

  async expectTokenDialog() {
    await expect(this.page.locator(this.selectors.tokenDialog)).toBeVisible();
    await expect(this.page.locator(this.selectors.tokenDialog)).toContainText(
      'API Token Generated'
    );
  }

  /**
   * Get the token value from the dialog (requires token shown).
   */
  async getTokenValue() {
    const tokenContentDiv = this.page.locator('[data-testid="token-value-input-content"]');
    await expect(tokenContentDiv).toBeVisible();
    const tokenValue = await tokenContentDiv.textContent();
    return tokenValue || '';
  }

  async toggleShowToken() {
    const showHideButton = this.page.locator(this.selectors.showHideButton).first();
    await expect(showHideButton).toBeVisible();
    await showHideButton.click();
  }

  async expectTokenHidden() {
    const tokenContentDiv = this.page.locator('[data-testid="token-value-input-content"]');
    const value = await tokenContentDiv.textContent();
    expect(value).toMatch(/^[•●]+$/);
  }

  async expectTokenVisible(expectedToken) {
    const tokenContentDiv = this.page.locator('[data-testid="token-value-input-content"]');
    const value = await tokenContentDiv.textContent();
    expect(value).toBe(expectedToken);
  }

  async copyTokenFromDialog() {
    const copyButton = this.page.locator(this.selectors.copyButton);
    await expect(copyButton).toBeVisible();
    await copyButton.click();
    await this.page.waitForTimeout(100);
    const tokenValue = await this.page.evaluate(() => window.clipboardData);
    return tokenValue;
  }

  /**
   * Close the reveal dialog with Done. In V2 this also returns to the tokens list.
   */
  async closeTokenDialog() {
    const doneButton = this.page.locator(this.selectors.doneButton);
    await expect(doneButton).toBeVisible();
    await doneButton.click();
    await this.page.waitForSelector(this.selectors.tokenDialog, { state: 'hidden' });
    await this.page.waitForSelector('[data-testid="tokens-page"][data-pagestatus="ready"]');
  }

  async expectDialogClosed() {
    await expect(this.page.locator(this.selectors.tokenDialog)).not.toBeVisible();
  }

  /**
   * Find a token row in the list by name.
   * @param {string} name - Token name to find
   * @param {{ waitFor?: boolean }} [opts] - waitFor (default true) auto-waits for a row to
   *   appear before scanning, absorbing the brief route view-transition cross-fade.
   * @returns {Promise<Object|null>} Token data or null if not found
   */
  async findTokenByName(name, { waitFor = true } = {}) {
    const rows = this.page.locator(this.selectors.listRow);
    if (waitFor) {
      // After create/navigate, the list refetches behind a route view-transition; wait for
      // THIS token's row to actually land (not just any row) so the scan doesn't race the
      // transition + refetch.
      await rows
        .filter({ has: this.page.locator(`[data-testid^="token-name-"]`, { hasText: name }) })
        .first()
        .waitFor({ state: 'visible' })
        .catch(() => {});
    }
    const count = await rows.count();

    for (let i = 0; i < count; i++) {
      const row = rows.nth(i);
      const nameText =
        (await row.locator('[data-testid^="token-name-"]').textContent())?.trim() || '';
      if (nameText === name || nameText.includes(name)) {
        const scopeText =
          (await row.locator('[data-testid^="token-scope-"]').textContent())?.trim() || '';
        const statusSwitch = row.locator('[role="switch"]');
        const isActive = await statusSwitch.isChecked();
        return {
          row,
          name: nameText,
          scope: scopeText,
          status: isActive ? 'active' : 'inactive',
          statusSwitch,
        };
      }
    }
    return null;
  }

  async expectTokenInList(name, expectedStatus = 'active') {
    // Wait for the specific row to appear (absorbs the post-create/navigate view
    // transition + list refetch) before reading its status.
    await this.page
      .locator(this.selectors.listRow)
      .filter({ has: this.page.locator(`[data-testid^="token-name-"]`, { hasText: name }) })
      .first()
      .waitFor({ state: 'visible' });
    const token = await this.findTokenByName(name);
    expect(token).not.toBeNull();
    expect(token.status).toBe(expectedStatus);
  }

  async expectTokenNotInList(name) {
    const token = await this.findTokenByName(name, { waitFor: false });
    expect(token).toBeNull();
  }

  async toggleTokenStatus(name) {
    const token = await this.findTokenByName(name);
    expect(token).not.toBeNull();
    await token.statusSwitch.click();
    await this.waitForSPAReady();
    await this.page.waitForTimeout(100);
  }

  async expectEmptyTokensList() {
    const rows = this.page.locator(this.selectors.listRow);
    expect(await rows.count()).toBe(0);
  }

  async getTokenCount() {
    const rows = this.page.locator(this.selectors.listRow);
    return await rows.count();
  }

  async waitForTokenCreationSuccess() {
    const toast = this.page.locator('[data-state="open"]');
    await expect(toast).toBeVisible();
    await expect(toast).toContainText('API token successfully generated');
  }

  async waitForTokenUpdateSuccess(status) {
    const toast = this.page.locator('[data-state="open"]');
    await expect(toast).toBeVisible();
    await expect(toast).toContainText(`Token status changed to ${status}`);
  }

  async expectTokenName(name) {
    const token = await this.findTokenByName(name);
    expect(token).not.toBeNull();
    expect(token.name).toBe(name);
  }

  async expectTokenStatus(name, expectedStatus) {
    // Poll: a status toggle invalidates + refetches the list, re-rendering the row;
    // read the live switch state until it settles to the expected value.
    await expect
      .poll(async () => (await this.findTokenByName(name))?.status, {
        message: `token "${name}" should be ${expectedStatus}`,
      })
      .toBe(expectedStatus);
  }
}
