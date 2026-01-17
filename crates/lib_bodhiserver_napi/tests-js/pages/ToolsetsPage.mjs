import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

/**
 * Page object for Toolsets management on /ui/toolsets pages
 */
export class ToolsetsPage extends BasePage {
  selectors = {
    pageContainer: '[data-testid="toolsets-page"]',
    toolsetsTable: '[data-testid="toolsets-table"]',
    toolsetRow: (toolsetId) => `[data-testid="toolset-name-${toolsetId}"]`,
    toolsetStatus: (toolsetId) => `[data-testid="toolset-status-${toolsetId}"]`,
    toolsetEditButton: (toolsetId) => `[data-testid="toolset-edit-button-${toolsetId}"]`,
    // Edit page selectors
    editPageContainer: '[data-testid="toolset-edit-page"]',
    toolsetConfigForm: '[data-testid="toolset-config-form"]',
    apiKeyInput: '[data-testid="toolset-api-key-input"]',
    enabledToggle: '[data-testid="toolset-enabled-toggle"]',
    appEnabledToggle: '[data-testid="app-enabled-toggle"]',
    clearApiKeyButton: '[data-testid="clear-api-key-button"]',
    saveButton: '[data-testid="save-toolset-config"]',
    // Admin
    appEnableCheckbox: '[data-testid="app-enable-checkbox"]',
    // Badges
    enabledBadge: 'text=Enabled',
    configuredBadge: 'text=Configured',
    notConfiguredBadge: 'text=Not Configured',
    appDisabledBadge: 'text=App Disabled',
  };

  // List page methods
  async navigateToToolsetsList() {
    await this.navigate('/ui/toolsets/');
    await this.waitForSPAReady();
  }

  async expectToolsetsListPage() {
    // Increase timeout for page to load after OAuth redirect
    await expect(this.page.locator(this.selectors.pageContainer)).toBeVisible({ timeout: 15000 });
  }

  async expectToolsetListed(toolsetId, expectedStatus = null) {
    await this.expectVisible(this.selectors.toolsetRow(toolsetId));

    if (expectedStatus) {
      const statusCell = this.page.locator(this.selectors.toolsetStatus(toolsetId));
      await expect(statusCell).toContainText(expectedStatus);
    }
  }

  async clickEditToolset(toolsetId) {
    await this.page.click(this.selectors.toolsetEditButton(toolsetId));
    await this.page.waitForURL(/\/ui\/toolsets\/edit/);
    await this.waitForSPAReady();
  }

  // Edit page methods
  async navigateToToolsetEdit(toolsetId) {
    await this.navigate(`/ui/toolsets/edit?toolset_id=${toolsetId}`);
    await this.waitForSPAReady();
  }

  async expectToolsetEditPage() {
    // Increase timeout for page to load
    await expect(this.page.locator(this.selectors.editPageContainer)).toBeVisible();
    await expect(this.page.locator(this.selectors.toolsetConfigForm)).toBeVisible();
  }

  async expectFormLoaded() {
    await this.expectVisible(this.selectors.apiKeyInput);
    await this.expectVisible(this.selectors.enabledToggle);
    await this.expectVisible(this.selectors.saveButton);
  }

  async fillApiKey(apiKey) {
    await this.page.fill(this.selectors.apiKeyInput, apiKey);
  }

  async toggleEnabled() {
    await this.page.click(this.selectors.enabledToggle);
  }

  async saveConfig() {
    await this.page.click(this.selectors.saveButton);
  }

  async waitForFormState(state) {
    await this.page.waitForSelector(
      `${this.selectors.toolsetConfigForm}[data-form-state="${state}"]`
    );
  }

  async clearApiKey() {
    await this.page.click(this.selectors.clearApiKeyButton);
    // Wait for confirmation dialog and confirm
    await this.page.click('button:has-text("Clear API Key")');
  }

  // Admin methods
  async expectAdminToggle() {
    await this.expectVisible(this.selectors.appEnabledToggle);
  }

  async toggleAppEnabled() {
    await this.page.click(this.selectors.appEnabledToggle);
  }

  async confirmAppEnable() {
    await this.page.click('button:has-text("Enable")');
  }

  async confirmAppDisable() {
    await this.page.click('button:has-text("Disable")');
  }

  // Status expectations
  async expectToolsetEnabled(toolsetId) {
    const statusCell = this.page.locator(this.selectors.toolsetStatus(toolsetId));
    await expect(statusCell).toContainText('Enabled');
  }

  async expectToolsetConfigured(toolsetId) {
    const statusCell = this.page.locator(this.selectors.toolsetStatus(toolsetId));
    await expect(statusCell).toContainText('Configured');
  }

  async expectToolsetNotConfigured(toolsetId) {
    const statusCell = this.page.locator(this.selectors.toolsetStatus(toolsetId));
    await expect(statusCell).toContainText('Not Configured');
  }

  async expectToolsetAppDisabled(toolsetId) {
    const statusCell = this.page.locator(this.selectors.toolsetStatus(toolsetId));
    await expect(statusCell).toContainText('App Disabled');
  }

  // Complete workflow methods
  async configureToolsetWithApiKey(toolsetId, apiKey) {
    await this.navigateToToolsetEdit(toolsetId);
    await this.expectToolsetEditPage();
    await this.expectFormLoaded();

    const appToggle = this.page.locator(this.selectors.appEnabledToggle);
    const isAppEnabled = await appToggle.getAttribute('data-state');
    if (isAppEnabled !== 'checked') {
      await this.toggleAppEnabled();
      await this.confirmAppEnable();
    }

    const userToggle = this.page.locator(this.selectors.enabledToggle);
    const isUserEnabled = await userToggle.getAttribute('data-state');
    if (isUserEnabled !== 'checked') {
      await this.toggleEnabled();
    }

    await this.fillApiKey(apiKey);
    await this.saveConfig();
  }
}
