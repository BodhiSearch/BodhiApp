import { expect } from '@playwright/test';
import { SetupBasePage } from '@/pages/SetupBasePage.mjs';
import { ApiModelFormComponent } from '@/pages/components/ApiModelFormComponent.mjs';

/**
 * Page object for API Models setup operations on /ui/setup/api-models pages
 * Uses composition with ApiModelFormComponent to eliminate duplication
 */
export class SetupApiModelsPage extends SetupBasePage {
  constructor(page, baseUrl) {
    super(page, baseUrl);
    this.form = new ApiModelFormComponent(page);
  }

  selectors = {
    ...this.selectors,
    pageContainer: '[data-testid="api-models-setup-page"]',
    setupProgress: '[data-testid="setup-progress"]',
    setupForm: '[data-testid="setup-api-model-form"]',
    skipButton: '[data-testid="skip-api-setup"]',
    welcomeTitle: 'text=☁️Setup API Models',
    helpSection: "text=Don't have an API key?",
  };

  // Navigation and page state methods
  async navigateToApiModelsSetup() {
    await this.navigateToSetupStep('/ui/setup/api-models/', 4);
  }

  async expectApiModelsPage() {
    await this.expectVisible(this.selectors.pageContainer);
    await this.expectVisible(this.selectors.welcomeTitle);
    await this.expectSetupStep(4, '/ui/setup/api-models/');
  }

  async expectToBeOnApiModelsSetupPage() {
    await this.expectSetupStep(4, '/ui/setup/api-models/');
  }

  async expectInitialFormState() {
    // Verify form is present
    await this.expectVisible(this.selectors.setupForm);

    // Verify initial empty state (setup mode should not pre-select OpenAI)
    const apiFormatSelect = this.page.locator(this.form.selectors.apiFormatSelect);
    await expect(apiFormatSelect).toContainText('Select an API format');

    // Verify empty inputs
    await expect(this.page.locator(this.form.selectors.baseUrlInput)).toHaveValue('');
    await expect(this.page.locator(this.form.selectors.apiKeyInput)).toHaveValue('');

    // Verify disabled buttons initially
    await this.form.expectTestConnectionButtonDisabled();
    await this.form.expectFetchModelsButtonDisabled();

    // Verify correct button text
    await expect(this.page.locator(this.form.selectors.createButton)).toContainText(
      'Create API Model'
    );
  }

  // Form interaction methods (delegated to form component)
  async selectApiFormat(format = 'openai') {
    await this.form.selectApiFormat(format);
  }

  async fillApiKey(apiKey) {
    await this.form.fillApiKey(apiKey);
  }

  async testConnection() {
    await this.form.expectButtonEnabled('testConnectionButton');
    await this.page.click(this.form.selectors.testConnectionButton);
  }

  async expectConnectionSuccess() {
    await this.form.expectTestConnectionSuccess();
    await this.form.expectFetchModelsButtonEnabled();
  }

  async fetchModels() {
    await this.form.expectFetchModelsButtonEnabled();
    await this.page.click(this.form.selectors.fetchModelsButton);
  }

  async expectModelsLoaded(expectedModels = ['gpt-4', 'gpt-3.5-turbo']) {
    // Check if models are available
    for (const model of expectedModels) {
      await this.form.verifyModelAvailable(model);
    }
  }

  async selectModels(models = ['gpt-4']) {
    for (const model of models) {
      await this.form.selectSpecificModel(model);
    }
  }

  async submitForm() {
    await this.expectVisible(this.form.selectors.createButton);
    await this.page.click(this.form.selectors.createButton);
  }

  // Setup-specific methods
  async skipApiSetup() {
    await this.expectVisible(this.selectors.skipButton);
    await this.page.click(this.selectors.skipButton);
  }

  async expectSuccessToast(message = 'API Model Created') {
    await this.page.waitForSelector(`text=${message}`, { timeout: 5000 });
  }

  async expectErrorToast(message = 'Failed to Create API Model') {
    await this.page.waitForSelector(`text=${message}`, { timeout: 5000 });
  }

  async expectNavigationToBrowserExtension() {
    await super.expectNavigationToBrowserExtension();
  }

  async expectHelpSection() {
    await this.expectVisible(this.selectors.helpSection);
    await expect(this.page.locator('text=you can skip this step')).toBeVisible();
  }

  // Setup-specific workflow methods
  async completeApiModelSetup(options = {}) {
    const {
      apiFormat = 'openai',
      apiKey = 'sk-test-key-123',
      models = ['gpt-4'],
      skipSetup = false,
    } = options;

    if (skipSetup) {
      await this.skipApiSetup();
      return;
    }

    // Complete the full API model setup flow
    await this.selectApiFormat(apiFormat);
    await this.fillApiKey(apiKey);
    await this.testConnection();
    await this.expectConnectionSuccess();
    await this.fetchModels();
    await this.expectModelsLoaded(models);
    await this.selectModels(models);
    await this.submitForm();
    await this.expectSuccessToast();
  }

  // Real API integration methods for happy path testing (delegated to form component)
  async fetchAndSelectModels(models = ['gpt-3.5-turbo'], maxRetries = 1) {
    await this.form.fetchAndSelectModels(models, maxRetries);
  }

  async searchAndSelectModel(modelName) {
    await this.form.searchAndSelectModel(modelName);
  }

  async testConnectionWithRetry(maxRetries = 1) {
    await this.form.testConnection(maxRetries);
  }

  async createModelAndNavigateToBrowserExtension() {
    await this.page.click(this.form.selectors.createButton);

    // Wait for successful navigation to browser extension page (setup flow)
    await this.expectNavigationToBrowserExtension();
  }
}
