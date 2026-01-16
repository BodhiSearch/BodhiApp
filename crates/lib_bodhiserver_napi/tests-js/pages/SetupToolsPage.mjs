import { SetupBasePage } from '@/pages/SetupBasePage.mjs';
import { expect } from '@playwright/test';

/**
 * Page object for Tools setup operations on /ui/setup/tools page
 *
 * The setup tools page uses optimistic rendering:
 * - Form renders immediately without waiting for backend
 * - Enable for Server toggle defaults to OFF
 * - Form is disabled when Enable for Server is OFF
 * - Backend state is applied when fetched, overwriting local state
 */
export class SetupToolsPage extends SetupBasePage {
  selectors = {
    ...this.selectors,
    pageContainer: '[data-testid="tools-setup-page"]',
    toolConfigForm: '[data-testid="tool-config-form"]',
    apiKeyInput: '[data-testid="tool-api-key-input"]',
    enabledToggle: '[data-testid="tool-enabled-toggle"]',
    appEnabledToggle: '[data-testid="app-enabled-toggle"]',
    appDisabledMessage: '[data-testid="app-disabled-message"]',
    saveButton: '[data-testid="save-tool-config"]',
    skipButton: '[data-testid="skip-tools-setup"]',
    welcomeTitle: 'text=Configure Tools',
    exaLink: 'a[href*="exa.ai"]',
    // Dialog selectors for confirmation
    enableDialogTitle: 'text=Enable Tool for Server',
    disableDialogTitle: 'text=Disable Tool for Server',
    dialogEnableButton: 'button:has-text("Enable")',
    dialogDisableButton: 'button:has-text("Disable")',
    dialogCancelButton: 'button:has-text("Cancel")',
    // Badge for status
    enabledBadge: 'text=Enabled',
    disabledBadge: 'text=Disabled',
  };

  // Navigation and page state methods
  async navigateToToolsSetup() {
    await this.navigateToSetupStep('/ui/setup/tools/', 5);
  }

  async expectToolsPage() {
    await this.expectVisible(this.selectors.pageContainer);
    await this.expectVisible(this.selectors.welcomeTitle);
    await this.expectSetupStep(5, '/ui/setup/tools/');
  }

  async expectToBeOnToolsSetupPage() {
    await this.expectSetupStep(5, '/ui/setup/tools/');
  }

  async expectInitialFormState() {
    // Form should be visible immediately (optimistic rendering)
    await this.expectVisible(this.selectors.toolConfigForm);
    await this.expectVisible(this.selectors.appEnabledToggle);
    await this.expectVisible(this.selectors.apiKeyInput);
    await this.expectVisible(this.selectors.enabledToggle);
    await this.expectVisible(this.selectors.saveButton);

    // Verify external link to exa.ai
    await this.expectVisible(this.selectors.exaLink);
  }

  async expectAppEnabledToggle() {
    await this.expectVisible(this.selectors.appEnabledToggle);
  }

  async expectAppToggleOff() {
    const toggle = this.page.locator(this.selectors.appEnabledToggle);
    await expect(toggle).not.toBeChecked();
    await this.expectVisible(this.selectors.disabledBadge);
  }

  async expectAppToggleOn() {
    const toggle = this.page.locator(this.selectors.appEnabledToggle);
    await expect(toggle).toBeChecked();
    await this.expectVisible(this.selectors.enabledBadge);
  }

  async expectAppDisabledMessage() {
    await this.expectVisible(this.selectors.appDisabledMessage);
  }

  async expectNoAppDisabledMessage() {
    await expect(this.page.locator(this.selectors.appDisabledMessage)).not.toBeVisible();
  }

  async expectFormDisabled() {
    const input = this.page.locator(this.selectors.apiKeyInput);
    await expect(input).toBeDisabled();
  }

  async expectFormEnabled() {
    const input = this.page.locator(this.selectors.apiKeyInput);
    await expect(input).toBeEnabled();
  }

  // Form interaction methods
  async fillApiKey(apiKey) {
    await this.page.fill(this.selectors.apiKeyInput, apiKey);
  }

  async toggleEnabled() {
    await this.page.click(this.selectors.enabledToggle);
  }

  /**
   * Enable the tool for server (clicks toggle and confirms dialog)
   */
  async enableAppTool() {
    await this.page.click(this.selectors.appEnabledToggle);
    // Wait for confirmation dialog
    await this.expectVisible(this.selectors.enableDialogTitle);
    // Click Enable button in dialog
    await this.page.click(this.selectors.dialogEnableButton);
    // Wait for dialog to close and toast to appear
    await this.waitForToast('Tool enabled for server');
  }

  /**
   * Disable the tool for server (clicks toggle and confirms dialog)
   */
  async disableAppTool() {
    await this.page.click(this.selectors.appEnabledToggle);
    // Wait for confirmation dialog
    await this.expectVisible(this.selectors.disableDialogTitle);
    // Click Disable button in dialog
    await this.page.click(this.selectors.dialogDisableButton);
    // Wait for dialog to close and toast to appear
    await this.waitForToast('Tool disabled for server');
  }

  async submitForm() {
    await this.page.click(this.selectors.saveButton);
  }

  async skipToolsSetup() {
    await this.expectVisible(this.selectors.skipButton);
    await this.page.click(this.selectors.skipButton);
  }

  // Navigation expectations
  async expectNavigationToBrowserExtension() {
    await super.expectNavigationToBrowserExtension();
  }

  // Complete setup workflow
  async completeToolsSetup(options = {}) {
    const { apiKey = '', skipSetup = true, enableForServer = true } = options;

    if (skipSetup || !apiKey) {
      await this.skipToolsSetup();
      return;
    }

    // Wait for form to be ready
    await this.expectInitialFormState();

    // Enable for server if needed (opens confirmation dialog)
    if (enableForServer) {
      await this.enableAppTool();
    }

    // Fill API key (this auto-enables the tool toggle)
    await this.fillApiKey(apiKey);

    // Submit
    await this.submitForm();
  }
}
