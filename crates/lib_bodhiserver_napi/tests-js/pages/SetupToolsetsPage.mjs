import { SetupBasePage } from '@/pages/SetupBasePage.mjs';
import { expect } from '@playwright/test';

/**
 * Page object for Toolsets setup operations on /ui/setup/toolsets page
 *
 * The setup toolsets page uses optimistic rendering:
 * - Form renders immediately without waiting for backend
 * - Enable for Server toggle defaults to OFF
 * - Form is disabled when Enable for Server is OFF
 * - Backend state is applied when fetched, overwriting local state
 */
export class SetupToolsetsPage extends SetupBasePage {
  selectors = {
    ...this.selectors,
    pageContainer: '[data-testid="toolsets-setup-page"]',
    toolsetConfigForm: '[data-testid="setup-toolset-form"]',
    apiKeyInput: '[data-testid="toolset-api-key-input"]',
    enabledToggle: '[data-testid="toolset-enabled-toggle"]',
    appEnabledToggle: '[data-testid="app-enabled-toggle"]',
    appDisabledMessage: '[data-testid="app-disabled-message"]',
    saveButton: '[data-testid="create-toolset-button"]',
    skipButton: '[data-testid="skip-toolsets-setup"]',
    welcomeTitle: 'text=Configure Toolsets',
    exaLink: 'a[href*="exa.ai"]',
    // Dialog selectors for confirmation
    enableDialogTitle: 'text=Enable Toolset for Server',
    disableDialogTitle: 'text=Disable Toolset for Server',
    dialogEnableButton: 'button:has-text("Enable")',
    dialogDisableButton: 'button:has-text("Disable")',
    dialogCancelButton: 'button:has-text("Cancel")',
    // Badge for status
    enabledBadge: 'text=Enabled',
    disabledBadge: 'text=Disabled',
  };

  // Navigation and page state methods
  async navigateToToolsetsSetup() {
    await this.navigateToSetupStep('/ui/setup/toolsets/', 5);
  }

  async expectToolsetsPage() {
    await this.expectVisible(this.selectors.pageContainer);
    await this.expectVisible(this.selectors.welcomeTitle);
    await this.expectSetupStep(5, '/ui/setup/toolsets/');
  }

  async expectToBeOnToolsetsSetupPage() {
    await this.expectSetupStep(5, '/ui/setup/toolsets/');
  }

  async expectInitialFormState() {
    // Form should be visible immediately (optimistic rendering)
    await this.expectVisible(this.selectors.toolsetConfigForm);
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
    // Scope the badge selector to the form to avoid matching toast messages
    const form = this.page.locator(this.selectors.toolsetConfigForm);
    await expect(form.locator(this.selectors.enabledBadge)).toBeVisible();
  }

  async expectAppDisabledMessage() {
    const message = this.page.locator(this.selectors.appDisabledMessage);
    await expect(message).toBeVisible();
    await expect(message).toHaveAttribute('data-test-state', 'disabled');
  }

  async expectNoAppDisabledMessage() {
    const message = this.page.locator(this.selectors.appDisabledMessage);
    await expect(message).toBeVisible();
    await expect(message).toHaveAttribute('data-test-state', 'enabled');
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
   * Enable the toolset for server (clicks toggle and confirms dialog)
   */
  async enableAppToolset() {
    await this.page.click(this.selectors.appEnabledToggle);
    // Wait for confirmation dialog
    await this.expectVisible(this.selectors.enableDialogTitle);
    // Click Enable button in dialog
    await this.page.click(this.selectors.dialogEnableButton);
    // Wait for dialog to close
    await this.page.waitForSelector(this.selectors.enableDialogTitle, { state: 'hidden' });
  }

  /**
   * Disable the toolset for server (clicks toggle and confirms dialog)
   */
  async disableAppToolset() {
    await this.page.click(this.selectors.appEnabledToggle);
    // Wait for confirmation dialog
    await this.expectVisible(this.selectors.disableDialogTitle);
    // Click Disable button in dialog
    await this.page.click(this.selectors.dialogDisableButton);
    // Wait for dialog to close
    await this.page.waitForSelector(this.selectors.disableDialogTitle, { state: 'hidden' });
  }

  async submitForm() {
    await this.page.click(this.selectors.saveButton);
  }

  async skipToolsetsSetup() {
    await this.expectVisible(this.selectors.skipButton);
    await this.page.click(this.selectors.skipButton);
  }

  // Navigation expectations
  async expectNavigationToBrowserExtension() {
    await super.expectNavigationToBrowserExtension();
  }

  // Complete setup workflow
  async completeToolsetsSetup(options = {}) {
    const { apiKey = '', skipSetup = true, enableForServer = true } = options;

    if (skipSetup || !apiKey) {
      await this.skipToolsetsSetup();
      return;
    }

    // Wait for form to be ready
    await this.expectInitialFormState();

    // Enable for server if needed (opens confirmation dialog)
    if (enableForServer) {
      await this.enableAppToolset();
    }

    // Fill API key (this auto-enables the toolset toggle)
    await this.fillApiKey(apiKey);

    // Submit
    await this.submitForm();
  }
}
