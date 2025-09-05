import { expect } from '@playwright/test';
import { BasePage } from './BasePage.mjs';

export class ApiModelFormPage extends BasePage {
  selectors = {
    modelIdInput: '[data-testid="api-model-id"]',
    providerSelect: '[data-testid="api-model-provider"]',
    baseUrlInput: '[data-testid="api-model-base-url"]',
    apiKeyInput: '[data-testid="api-model-api-key"]',
    fetchModelsButton: 'button:has-text("Fetch Models")',
    testConnectionButton: '[data-testid="test-connection-button"]',
    createButton: '[data-testid="create-api-model-button"]',
    updateButton: '[data-testid="update-api-model-button"]',
    modelOption: (model) => `.cursor-pointer >> text="${model}"`,
    modelsScrollArea: '.scroll-area', // Container for model selection
    successToast: '[data-state="open"]',
  };

  async fillBasicInfo(modelId, apiKey, baseUrl = 'https://api.openai.com/v1') {
    // Fill form fields
    await this.fillTestId('api-model-id', modelId);
    await this.fillTestId('api-model-base-url', baseUrl);
    await this.fillTestId('api-model-api-key', apiKey);
  }

  async fetchAndSelectModels(models = ['gpt-4', 'gpt-3.5-turbo']) {
    // Fetch models from API
    await this.expectVisible(this.selectors.fetchModelsButton);
    await this.page.click(this.selectors.fetchModelsButton);

    // Wait for models to load - check for specific model
    await this.waitForSelector('text=gpt-4');

    // Select specified models from the available models list
    for (const model of models) {
      await this.page.click(this.selectors.modelOption(model));
    }
  }

  async testConnection() {
    // Wait for the button to be enabled first
    await expect(this.page.locator(this.selectors.testConnectionButton)).toBeEnabled();
    await this.page.click(this.selectors.testConnectionButton);
    // Wait for connection test to complete - may take a few seconds
    await this.waitForToast(/Connection Test Successful/i);
  }

  async createModel() {
    await this.page.click(this.selectors.createButton);
    await this.waitForUrl('/ui/models/');
    await this.waitForSPAReady();
  }

  async updateModel() {
    await this.page.click(this.selectors.updateButton);
    await this.waitForUrl('/ui/models/');
    await this.waitForSPAReady();
  }

  async verifyFormPreFilled(modelId, provider = 'OpenAI', baseUrl = 'https://api.openai.com/v1') {
    // Verify form fields are pre-filled with existing data
    await this.expectValue(this.selectors.modelIdInput, modelId);
    await this.expectText(this.selectors.providerSelect, provider);
    await this.expectValue(this.selectors.baseUrlInput, baseUrl);

    // API key should be empty (masked for security)
    await this.expectValue(this.selectors.apiKeyInput, '');
  }

  async waitForFormReady() {
    // Wait for form to be fully loaded
    await this.waitForSelector(this.selectors.modelIdInput);
    await this.waitForSelector(this.selectors.baseUrlInput);
    await this.waitForSelector(this.selectors.apiKeyInput);
  }

  async isCreateMode() {
    try {
      await this.waitForSelector(this.selectors.createButton, { timeout: 1000 });
      return true;
    } catch {
      return false;
    }
  }

  async isUpdateMode() {
    try {
      await this.waitForSelector(this.selectors.updateButton, { timeout: 1000 });
      return true;
    } catch {
      return false;
    }
  }

  async selectSpecificModel(modelName) {
    await this.page.click(this.selectors.modelOption(modelName));
  }

  async verifyModelAvailable(modelName) {
    await expect(this.page.locator(`text=${modelName}`)).toBeVisible();
  }

  async getSelectedModels() {
    // This would need to be implemented based on the UI design
    // for showing which models are currently selected
    return [];
  }

  async clearApiKey() {
    await this.page.fill(this.selectors.apiKeyInput, '');
  }

  async fillApiKey(apiKey) {
    await this.page.fill(this.selectors.apiKeyInput, apiKey);
  }

  async expectTestConnectionSuccess() {
    await this.waitForToast(/Connection Test Successful/i);
  }

  async expectTestConnectionFailure() {
    await this.waitForToast(/Connection Test Failed/i);
  }

  // Provider-specific methods
  async selectProvider(provider) {
    if (provider !== 'OpenAI') {
      // This would need implementation for other providers
      throw new Error(`Provider ${provider} not yet implemented in test framework`);
    }
    // OpenAI is default, so no action needed for now
  }

  async setCustomBaseUrl(url) {
    await this.page.fill(this.selectors.baseUrlInput, url);
  }
}
