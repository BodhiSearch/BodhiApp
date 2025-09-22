import { expect } from '@playwright/test';
import { BasePage } from './BasePage.mjs';

export class ApiModelFormPage extends BasePage {
  selectors = {
    apiFormatSelect: '[data-testid="api-model-format"]',
    baseUrlInput: '[data-testid="api-model-base-url"]',
    apiKeyInput: '[data-testid="api-model-api-key"]',
    usePrefixCheckbox: '[data-testid="api-model-use-prefix"]',
    prefixInput: '[data-testid="api-model-prefix"]',
    fetchModelsButton: '[data-testid="fetch-models-button"]',
    testConnectionButton: '[data-testid="test-connection-button"]',
    createButton: '[data-testid="create-api-model-button"]',
    updateButton: '[data-testid="update-api-model-button"]',
    modelOption: (model) => `[data-testid="available-model-${model}"]`,
    modelsScrollArea: '.scroll-area', // Container for model selection
    modelSearchInput: '[data-testid="model-search-input"]', // Search input for filtering models
    successToast: '[data-state="open"]',
  };

  async fillBasicInfo(apiKey, baseUrl = 'https://api.openai.com/v1') {
    // Fill form fields
    await this.fillTestId('api-model-base-url', baseUrl);
    await this.fillTestId('api-model-api-key', apiKey);
  }

  async fetchAndSelectModels(models = ['gpt-4', 'gpt-3.5-turbo'], maxRetries = 1) {
    let attempt = 0;
    const maxAttempts = maxRetries + 1;

    while (attempt < maxAttempts) {
      try {
        attempt++;

        await this.expectVisible(this.selectors.fetchModelsButton);
        await this.page.click(this.selectors.fetchModelsButton);

        await this.page.waitForFunction(() => {
          const fetchContainer = document.querySelector('[data-testid="fetch-models-container"]');
          if (!fetchContainer) return false;
          const fetchState = fetchContainer.getAttribute('data-fetch-state');
          return fetchState === 'success' || fetchState === 'error';
        });

        const fetchResult = await this.page.evaluate(() => {
          const fetchContainer = document.querySelector('[data-testid="fetch-models-container"]');
          if (!fetchContainer) return { success: false, hasModels: false };

          const fetchState = fetchContainer.getAttribute('data-fetch-state');
          const hasModels = fetchContainer.getAttribute('data-models-fetched') === 'true';

          return {
            success: fetchState === 'success',
            hasModels: hasModels,
            fetchState: fetchState,
          };
        });

        if (fetchResult.success && fetchResult.hasModels) {
          await this.page.waitForFunction(() => {
            const searchInput = document.querySelector('[data-testid="model-search-input"]');
            return searchInput && !searchInput.disabled;
          });
          await this.waitForToast(/Models Fetched Successfully/i);
          for (const model of models) {
            await this.searchAndSelectModel(model);
          }
          return;
        }

        if (attempt < maxAttempts) {
          await this.page.waitForTimeout(1000);
        } else {
          throw new Error(
            `Failed to fetch models after ${maxAttempts} attempts. Final state: ${fetchResult.fetchState}, hasModels: ${fetchResult.hasModels}`
          );
        }
      } catch (error) {
        if (attempt < maxAttempts) {
          await this.page.waitForTimeout(1000);
        } else {
          throw error;
        }
      }
    }
  }

  async searchAndSelectModel(modelName) {
    // Clear search and type the model name to filter
    await this.page.fill(this.selectors.modelSearchInput, modelName);

    // Wait for the filtered model to appear
    await this.waitForSelector(this.selectors.modelOption(modelName));

    // Click on the model to select it
    await this.page.click(this.selectors.modelOption(modelName));

    // Clear the search to see all models again for next selection
    await this.page.fill(this.selectors.modelSearchInput, '');
  }

  async testConnection(maxRetries = 1) {
    let attempt = 0;
    const maxAttempts = maxRetries + 1;

    while (attempt < maxAttempts) {
      try {
        attempt++;

        await expect(this.page.locator(this.selectors.testConnectionButton)).toBeEnabled();
        await this.page.click(this.selectors.testConnectionButton);
        await expect(this.page.locator(this.selectors.testConnectionButton)).toBeEnabled();
        await this.waitForToast(/Connection Test Successful/i);
        return; // Success, exit the function
      } catch (error) {
        if (attempt < maxAttempts) {
          console.log(`Test connection attempt ${attempt} failed, retrying...`);
          await this.page.waitForTimeout(1000); // Wait 1 second before retry
        } else {
          throw new Error(
            `Failed to test connection after ${maxAttempts} attempts. Last error: ${error.message}`
          );
        }
      }
    }
  }

  async createModel() {
    await this.page.click(this.selectors.createButton);
    await this.waitForToastToHide();
    await this.waitForUrl('/ui/models/');
    await this.waitForSPAReady();
  }

  async createModelAndCaptureId() {
    await this.page.click(this.selectors.createButton);

    // Capture the generated ID from the success toast
    const generatedId = await this.waitForToastAndExtractId(/Successfully created API model/i);
    await this.waitForToastToHide();

    // Navigate to models page
    await this.waitForUrl('/ui/models/');
    await this.waitForSPAReady();

    return generatedId;
  }

  async updateModel() {
    await this.page.click(this.selectors.updateButton);
    await this.waitForToastToHide();
    await this.waitForUrl('/ui/models/');
    await this.waitForSPAReady();
  }

  async verifyFormPreFilled(api_format = 'openai', baseUrl = 'https://api.openai.com/v1') {
    // Verify form fields are pre-filled with existing data (no ID field since it's auto-generated)
    await this.expectText(this.selectors.apiFormatSelect, api_format);
    await this.expectValue(this.selectors.baseUrlInput, baseUrl);

    // API key should be empty (masked for security)
    await this.expectValue(this.selectors.apiKeyInput, '');
  }

  async waitForFormReady() {
    // Wait for form to be fully loaded (no ID field since it's auto-generated)
    await this.waitForSelector(this.selectors.baseUrlInput);
    await this.waitForSelector(this.selectors.apiKeyInput);
  }

  async isCreateMode() {
    try {
      await this.waitForSelector(this.selectors.createButton);
      return true;
    } catch {
      return false;
    }
  }

  async isUpdateMode() {
    try {
      await this.waitForSelector(this.selectors.updateButton);
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

  async selectApiFormat(api_format) {
    // For Shadcn Select component (combobox), need to click trigger then select option
    await this.page.click(this.selectors.apiFormatSelect);
    await this.page.click(`text=${api_format}`);
  }

  async setCustomBaseUrl(url) {
    await this.page.fill(this.selectors.baseUrlInput, url);
  }

  // Prefix-related methods
  async enablePrefix() {
    await this.page.check(this.selectors.usePrefixCheckbox);
    // Prefix input should become enabled
    await expect(this.page.locator(this.selectors.prefixInput)).toBeEnabled();
  }

  async disablePrefix() {
    await this.page.uncheck(this.selectors.usePrefixCheckbox);
    // Prefix input should become disabled but remain visible
    await expect(this.page.locator(this.selectors.prefixInput)).toBeDisabled();
  }

  async setPrefix(prefix) {
    await this.enablePrefix();
    await this.page.fill(this.selectors.prefixInput, prefix);
  }

  async fillBasicInfoWithPrefix(apiKey, prefix = null, baseUrl = 'https://api.openai.com/v1') {
    // Fill basic info
    await this.fillBasicInfo(apiKey, baseUrl);

    // Handle prefix if provided
    if (prefix) {
      await this.setPrefix(prefix);
    }
  }

  async verifyFormPreFilledWithPrefix(
    api_format = 'openai',
    baseUrl = 'https://api.openai.com/v1',
    prefix = null
  ) {
    // Verify basic form fields
    await this.verifyFormPreFilled(api_format, baseUrl);

    // Verify prefix state
    if (prefix) {
      await expect(this.page.locator(this.selectors.usePrefixCheckbox)).toBeChecked();
      await expect(this.page.locator(this.selectors.prefixInput)).toBeEnabled();
      await this.expectValue(this.selectors.prefixInput, prefix);
    } else {
      await expect(this.page.locator(this.selectors.usePrefixCheckbox)).not.toBeChecked();
      await expect(this.page.locator(this.selectors.prefixInput)).toBeDisabled();
    }
  }

  async isPrefixEnabled() {
    return await this.page.locator(this.selectors.usePrefixCheckbox).isChecked();
  }

  async getPrefixValue() {
    const isEnabled = await this.isPrefixEnabled();
    if (!isEnabled) {
      return null;
    }
    return await this.page.locator(this.selectors.prefixInput).inputValue();
  }
}
