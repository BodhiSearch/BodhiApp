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
    toolsetRowByType: (toolsetType) => `[data-testid-type="${toolsetType}"]`,
    toolsetStatus: (toolsetId) => `[data-testid="toolset-status-${toolsetId}"]`,
    toolsetEditButton: (toolsetId) => `[data-testid="toolset-edit-button-${toolsetId}"]`,
    toolsetEditButtonByType: (toolsetType) =>
      `[data-testid^="toolset-edit-button-"][data-testid-type="${toolsetType}"]`,
    // New page selectors
    newPageContainer: '[data-testid="new-toolset-page"]',
    toolsetTypeSelect: '[data-testid="toolset-type-select"]',
    toolsetTypeSelectTrigger: 'button[role="combobox"]',
    toolsetNameInput: '[data-testid="toolset-name-input"]',
    toolsetDescriptionInput: '[data-testid="toolset-description-input"]',
    createButton: '[data-testid="toolset-create-button"]',
    // Edit page selectors
    editPageContainer: '[data-testid="edit-toolset-page"]',
    apiKeyInput: '[data-testid="toolset-api-key-input"]',
    enabledToggle: '[data-testid="toolset-enabled-switch"]',
    clearApiKeyButton: '[data-testid="clear-api-key-button"]',
    saveButton: '[data-testid="toolset-save-button"]',
    // Admin page selectors
    adminPageContainer: '[data-testid="admin-toolsets-page"]',
    typeToggle: (typeId) => `[data-testid="type-toggle-${typeId}"]`,
    typeRow: (typeId) => `[data-testid="type-row-${typeId}"]`,
    toolsetEnabledSwitch: '[data-testid="toolset-enabled-switch"]',
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

  // New page methods
  async navigateToNewToolset() {
    await this.navigate('/ui/toolsets/new');
    await this.waitForSPAReady();
  }

  async expectNewToolsetPage() {
    await expect(this.page.locator(this.selectors.newPageContainer)).toBeVisible();
  }

  async createToolset(toolsetType, name, apiKey) {
    // Click the combobox by role within the new page
    await this.page.locator('button[role="combobox"]').click();
    // Click the option from the dropdown
    await this.page.click(`[data-testid="type-option-${toolsetType}"]`);

    await this.page.fill(this.selectors.toolsetNameInput, name);
    await this.page.fill(this.selectors.apiKeyInput, apiKey);
    await this.page.click(this.selectors.createButton);
    // Wait for redirect to list page
    await this.page.waitForURL(/\/ui\/toolsets(?!\/new)/);
    await this.waitForSPAReady();
  }

  async getToolsetRowByType(toolsetType) {
    return this.page.locator(this.selectors.toolsetRowByType(toolsetType)).first();
  }

  async getToolsetUuidByType(toolsetType) {
    const row = this.page.locator(this.selectors.toolsetRowByType(toolsetType)).first();
    return await row.getAttribute('data-test-uuid');
  }

  async clickEditByType(toolsetType) {
    await this.page.click(this.selectors.toolsetEditButtonByType(toolsetType));
    await this.page.waitForURL(/\/ui\/toolsets\/edit/);
    await this.waitForSPAReady();
  }

  // Edit page methods
  async navigateToToolsetEdit(toolsetId) {
    await this.navigate(`/ui/toolsets/edit?id=${toolsetId}`);
    await this.waitForSPAReady();
  }

  async expectToolsetEditPage() {
    // Increase timeout for page to load
    await expect(this.page.locator(this.selectors.editPageContainer)).toBeVisible();
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
      `${this.selectors.editPageContainer}[data-form-state="${state}"]`
    );
  }

  async clearApiKey() {
    await this.page.click(this.selectors.clearApiKeyButton);
    // Wait for confirmation dialog and confirm
    await this.page.click('button:has-text("Clear API Key")');
  }

  // Admin methods
  async navigateToAdmin() {
    await this.navigate('/ui/toolsets/admin');
    await this.waitForSPAReady();
  }

  async expectAdminPage() {
    await expect(this.page.locator(this.selectors.adminPageContainer)).toBeVisible();
  }

  async expectTypeToggle(typeId) {
    await this.expectVisible(this.selectors.typeToggle(typeId));
  }

  async enableToolsetTypeOnAdmin(typeId) {
    await this.navigateToAdmin();
    await this.expectAdminPage();

    const typeToggle = this.page.locator(this.selectors.typeToggle(typeId));
    const isEnabled = await typeToggle.getAttribute('data-state');
    if (isEnabled !== 'checked') {
      await typeToggle.click();
      await this.page.click('button:has-text("Enable")');
      await this.page.waitForSelector('button:has-text("Enable")', { state: 'hidden' });
      await this.page.waitForSelector(
        `${this.selectors.typeRow(typeId)}[data-test-state="enabled"]`
      );
    }
  }

  async toggleTypeEnabled(typeId) {
    await this.page.click(this.selectors.typeToggle(typeId));
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
  async configureToolsetWithApiKey(toolsetType, apiKey, toolsetName = null) {
    // Step 1: Ensure type is enabled on admin page
    await this.enableToolsetTypeOnAdmin(toolsetType);

    // Step 2: Create new toolset
    await this.navigateToNewToolset();
    await this.expectNewToolsetPage();

    const name = toolsetName || toolsetType;

    // Click the combobox by role
    await this.page.locator('button[role="combobox"]').click();
    // Click the option from the dropdown
    await this.page.click(`[data-testid="type-option-${toolsetType}"]`);

    await this.page.fill(this.selectors.toolsetNameInput, name);
    await this.page.fill(this.selectors.apiKeyInput, apiKey);

    // Enable the toolset by default (switch defaults to on, so no need to toggle)
    const enabledSwitch = this.page.locator(this.selectors.toolsetEnabledSwitch);
    const isEnabled = await enabledSwitch.getAttribute('data-state');
    if (isEnabled !== 'checked') {
      await enabledSwitch.click();
    }

    await this.page.click(this.selectors.createButton);

    // Wait for redirect to list page
    await this.page.waitForURL(/\/ui\/toolsets(?!\/new)/);
    await this.waitForSPAReady();
  }
}
