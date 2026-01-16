import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

/**
 * Page object for Tools management on /ui/tools pages
 */
export class ToolsPage extends BasePage {
  selectors = {
    pageContainer: '[data-testid="tools-page"]',
    toolsTable: '[data-testid="tools-table"]',
    toolRow: (toolId) => `[data-testid="tool-name-${toolId}"]`,
    toolStatus: (toolId) => `[data-testid="tool-status-${toolId}"]`,
    toolEditButton: (toolId) => `[data-testid="tool-edit-button-${toolId}"]`,
    // Edit page selectors
    editPageContainer: '[data-testid="tool-edit-page"]',
    toolConfigForm: '[data-testid="tool-config-form"]',
    apiKeyInput: '[data-testid="tool-api-key-input"]',
    enabledToggle: '[data-testid="tool-enabled-toggle"]',
    appEnabledToggle: '[data-testid="app-enabled-toggle"]',
    clearApiKeyButton: '[data-testid="clear-api-key-button"]',
    saveButton: '[data-testid="save-tool-config"]',
    // Admin
    appEnableCheckbox: '[data-testid="app-enable-checkbox"]',
    // Badges
    enabledBadge: 'text=Enabled',
    configuredBadge: 'text=Configured',
    notConfiguredBadge: 'text=Not Configured',
    appDisabledBadge: 'text=App Disabled',
  };

  // List page methods
  async navigateToToolsList() {
    await this.navigate('/ui/tools/');
    await this.waitForSPAReady();
  }

  async expectToolsListPage() {
    // Increase timeout for page to load after OAuth redirect
    await expect(this.page.locator(this.selectors.pageContainer)).toBeVisible({ timeout: 15000 });
  }

  async expectToolListed(toolId, expectedStatus = null) {
    await this.expectVisible(this.selectors.toolRow(toolId));

    if (expectedStatus) {
      const statusCell = this.page.locator(this.selectors.toolStatus(toolId));
      await expect(statusCell).toContainText(expectedStatus);
    }
  }

  async clickEditTool(toolId) {
    await this.page.click(this.selectors.toolEditButton(toolId));
    await this.page.waitForURL(/\/ui\/tools\/edit/);
    await this.waitForSPAReady();
  }

  // Edit page methods
  async navigateToToolEdit(toolId) {
    await this.navigate(`/ui/tools/edit?toolid=${toolId}`);
    await this.waitForSPAReady();
  }

  async expectToolEditPage() {
    // Increase timeout for page to load
    await expect(this.page.locator(this.selectors.editPageContainer)).toBeVisible();
    await expect(this.page.locator(this.selectors.toolConfigForm)).toBeVisible();
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
    await this.page.waitForSelector(`${this.selectors.toolConfigForm}[data-form-state="${state}"]`);
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
  async expectToolEnabled(toolId) {
    const statusCell = this.page.locator(this.selectors.toolStatus(toolId));
    await expect(statusCell).toContainText('Enabled');
  }

  async expectToolConfigured(toolId) {
    const statusCell = this.page.locator(this.selectors.toolStatus(toolId));
    await expect(statusCell).toContainText('Configured');
  }

  async expectToolNotConfigured(toolId) {
    const statusCell = this.page.locator(this.selectors.toolStatus(toolId));
    await expect(statusCell).toContainText('Not Configured');
  }

  async expectToolAppDisabled(toolId) {
    const statusCell = this.page.locator(this.selectors.toolStatus(toolId));
    await expect(statusCell).toContainText('App Disabled');
  }

  // Complete workflow methods
  async configureToolWithApiKey(toolId, apiKey) {
    await this.navigateToToolEdit(toolId);
    await this.expectToolEditPage();
    await this.expectFormLoaded();
    await this.fillApiKey(apiKey);
    await this.toggleEnabled();
    await this.saveConfig();
  }
}
