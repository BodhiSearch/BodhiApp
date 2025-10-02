import { expect } from '@playwright/test';
import { BasePage } from '@/pages/BasePage.mjs';

export class TokensPage extends BasePage {
  selectors = {
    // Page elements
    tokensPage: '[data-testid="tokens-page"]',
    tokenFormContainer: '[data-testid="token-form-container"]',
    tokenForm: '[data-testid="token-form"]',
    tokenNameInput: '[data-testid="token-name-input"]',
    generateButton: '[data-testid="generate-token-button"]',

    // Table elements
    tokensTableContainer: '[data-testid="tokens-table-container"]',
    tokensTable: '[data-testid="tokens-table"]',
    tokenName: (id) => `[data-testid="token-name-${id}"]`,
    statusSwitch: (id) => `[data-testid="token-status-switch-${id}"]`,
    tableRow: 'tbody tr',

    // Dialog elements
    tokenDialog: '[data-testid="token-dialog"]',
    tokenValueInput: '[data-testid="token-value-input"]',
    copyButton: '[data-testid="copy-content"]',
    doneButton: '[data-testid="token-dialog-done"]',

    // ShowHideInput button (generic selector for toggle)
    showHideButton: '[data-testid="toggle-show-content"]', // Matches both "Show" and "Hide"
  };

  /**
   * Navigate to tokens page
   */
  async navigateToTokens() {
    await this.navigate('/ui/tokens/');
    await this.waitForSPAReady();
    await this.page.waitForSelector('[data-testid="tokens-page"][data-pagestatus="ready"]');
  }

  /**
   * Verify we're on the tokens page
   */
  async expectTokensPage() {
    await expect(this.page.locator(this.selectors.tokensPage)).toBeVisible();
  }

  /**
   * Create a new token with optional name
   * @param {string} name - Optional token name
   */
  async createToken(name = '') {
    const nameInput = this.page.locator(this.selectors.tokenNameInput);
    const generateButton = this.page.locator(this.selectors.generateButton);

    if (name) {
      await nameInput.fill(name);
    }

    await generateButton.click();

    // Wait for dialog to appear
    await this.page.waitForSelector(this.selectors.tokenDialog);
    await expect(this.page.locator(this.selectors.tokenDialog)).toBeVisible();
  }

  /**
   * Verify token dialog is visible with expected content
   */
  async expectTokenDialog() {
    await expect(this.page.locator(this.selectors.tokenDialog)).toBeVisible();
    await expect(this.page.locator(this.selectors.tokenDialog)).toContainText(
      'API Token Generated'
    );
  }

  /**
   * Get the token value from the dialog
   * Requires token to be visible (after clicking show)
   */
  async getTokenValue() {
    const tokenContentDiv = this.page.locator('[data-testid="token-value-input-content"]');
    await expect(tokenContentDiv).toBeVisible();

    // Get the text content from the div element
    const tokenValue = await tokenContentDiv.textContent();
    return tokenValue || '';
  }

  /**
   * Toggle show/hide token in dialog
   */
  async toggleShowToken() {
    const showHideButton = this.page.locator(this.selectors.showHideButton).first();
    await expect(showHideButton).toBeVisible();
    await showHideButton.click();
  }

  /**
   * Verify token is hidden (shows dots)
   */
  async expectTokenHidden() {
    const tokenContentDiv = this.page.locator('[data-testid="token-value-input-content"]');
    const value = await tokenContentDiv.textContent();
    // When hidden, the value should be bullet characters (40 of them)
    expect(value).toMatch(/^[•●]+$/);
  }

  /**
   * Verify token is visible
   * @param {string} expectedToken - The expected token value
   */
  async expectTokenVisible(expectedToken) {
    const tokenContentDiv = this.page.locator('[data-testid="token-value-input-content"]');
    const value = await tokenContentDiv.textContent();
    expect(value).toBe(expectedToken);
  }

  /**
   * Copy token from dialog using the copy button
   * @returns {Promise<string>} The copied token value from clipboard
   */
  async copyTokenFromDialog() {
    // Click copy button to copy the token to clipboard
    const copyButton = this.page.locator(this.selectors.copyButton);
    await expect(copyButton).toBeVisible();
    await copyButton.click();

    // Wait a moment for copy operation to complete
    await this.page.waitForTimeout(100);

    // Read the actual token value from clipboard
    const tokenValue = await this.page.evaluate(() => window.clipboardData);
    return tokenValue;
  }

  /**
   * Close token dialog
   */
  async closeTokenDialog() {
    const doneButton = this.page.locator(this.selectors.doneButton);
    await expect(doneButton).toBeVisible();
    await doneButton.click();

    // Wait for dialog to close
    await this.page.waitForSelector(this.selectors.tokenDialog, { state: 'hidden' });
  }

  /**
   * Verify dialog is closed
   */
  async expectDialogClosed() {
    await expect(this.page.locator(this.selectors.tokenDialog)).not.toBeVisible();
  }

  /**
   * Find a token in the list by name
   * @param {string} name - Token name to find
   * @returns {Promise<Object|null>} Token data or null if not found
   */
  async findTokenByName(name) {
    const rows = this.page.locator(this.selectors.tableRow);
    const count = await rows.count();

    for (let i = 0; i < count; i++) {
      const row = rows.nth(i);
      const rowText = await row.textContent();

      if (rowText && rowText.includes(name)) {
        // Found the row, extract data
        const cells = row.locator('td');
        const tokenName = await cells.nth(0).textContent();
        const statusCell = cells.nth(1);
        const statusSwitch = statusCell.locator('[role="switch"]');
        const isActive = await statusSwitch.isChecked();

        return {
          row,
          name: tokenName?.trim() || '',
          status: isActive ? 'active' : 'inactive',
          statusSwitch,
        };
      }
    }

    return null;
  }

  /**
   * Verify token exists in list
   * @param {string} name - Token name
   * @param {string} expectedStatus - Expected status ('active' or 'inactive')
   */
  async expectTokenInList(name, expectedStatus = 'active') {
    const token = await this.findTokenByName(name);
    expect(token).not.toBeNull();
    expect(token.status).toBe(expectedStatus);
  }

  /**
   * Verify token does not exist in list
   * @param {string} name - Token name
   */
  async expectTokenNotInList(name) {
    const token = await this.findTokenByName(name);
    expect(token).toBeNull();
  }

  /**
   * Toggle token status
   * @param {string} name - Token name
   */
  async toggleTokenStatus(name) {
    const token = await this.findTokenByName(name);
    expect(token).not.toBeNull();

    await token.statusSwitch.click();
    await this.waitForSPAReady();
    // Wait a moment for the toggle animation to complete
    await this.page.waitForTimeout(100);
  }

  /**
   * Verify empty state (no tokens)
   */
  async expectEmptyTokensList() {
    const rows = this.page.locator(this.selectors.tableRow);
    const count = await rows.count();
    // Should only have header row, no data rows
    expect(count).toBe(0);
  }

  /**
   * Get count of tokens in list
   */
  async getTokenCount() {
    const rows = this.page.locator(this.selectors.tableRow);
    return await rows.count();
  }

  /**
   * Wait for success toast after token creation
   */
  async waitForTokenCreationSuccess() {
    // Toast should appear with success message
    const toast = this.page.locator('[data-state="open"]');
    await expect(toast).toBeVisible();
    await expect(toast).toContainText('API token successfully generated');
  }

  /**
   * Wait for success toast after token update
   */
  async waitForTokenUpdateSuccess(status) {
    const toast = this.page.locator('[data-state="open"]');
    await expect(toast).toBeVisible();
    await expect(toast).toContainText(`Token status changed to ${status}`);
  }

  /**
   * Verify token name in list
   * @param {string} name - Token name to verify
   */
  async expectTokenName(name) {
    const token = await this.findTokenByName(name);
    expect(token).not.toBeNull();
    expect(token.name).toBe(name);
  }

  /**
   * Verify token status
   * @param {string} name - Token name
   * @param {string} expectedStatus - Expected status
   */
  async expectTokenStatus(name, expectedStatus) {
    const token = await this.findTokenByName(name);
    expect(token).not.toBeNull();
    expect(token.status).toBe(expectedStatus);
  }
}
