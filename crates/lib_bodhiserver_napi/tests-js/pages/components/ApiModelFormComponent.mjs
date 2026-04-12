import { expect } from '@playwright/test';

/**
 * Shared component for API Model form interactions
 * Used by both ApiModelFormPage and SetupApiModelsPage to eliminate duplication
 */
export class ApiModelFormComponent {
  constructor(page) {
    this.page = page;
  }

  selectors = {
    apiFormatSelect: '[data-testid="api-format-selector"]',
    baseUrlInput: '[data-testid="base-url-input"]',
    apiKeyInput: '[data-testid="api-key-input"]',
    useApiKeyCheckbox: '[data-testid="api-key-input-checkbox"]',
    usePrefixCheckbox: '[data-testid="prefix-input-checkbox"]',
    prefixInput: '[data-testid="prefix-input"]',
    forwardModeSelector: '[data-testid="forward-mode-selector"]',
    forwardAllRadio: '[data-testid="forward-mode-selector-forward-all"]',
    forwardSelectedRadio: '[data-testid="forward-mode-selector-forward-selected"]',
    fetchModelsButton: '[data-testid="fetch-models-button"]',
    testConnectionButton: '[data-testid="test-connection-button"]',
    createButton: '[data-testid="create-api-model-button"]',
    updateButton: '[data-testid="update-api-model-button"]',
    modelOption: (model) => `[data-testid="available-model-${model}"]`,
    modelsScrollArea: '.scroll-area',
    modelSearchInput: '[data-testid="model-search-input"]',
    successToast: '[data-state="open"]',
  };

  // API format display name mapping (must match API_FORMAT_PRESETS in apiModel.ts)
  static FORMAT_DISPLAY_NAMES = {
    openai: 'OpenAI - Completions',
    openai_responses: 'OpenAI - Responses',
    anthropic: 'Anthropic',
    anthropic_oauth: 'Anthropic (Claude Code OAuth)',
  };

  static getFormatDisplayName(format) {
    return ApiModelFormComponent.FORMAT_DISPLAY_NAMES[format] || format.toUpperCase();
  }

  // API Format Selection
  async selectApiFormat(format = 'openai') {
    await this.page.click(this.selectors.apiFormatSelect);

    const formatDisplayText = ApiModelFormComponent.getFormatDisplayName(format);

    // Use exact match to avoid ambiguity between similar option names
    await this.page.getByRole('option', { name: formatDisplayText, exact: true }).click();

    // Wait for base URL to be auto-populated for known formats
    if (format === 'openai' || format === 'openai_responses') {
      await expect(this.page.locator(this.selectors.baseUrlInput)).toHaveValue(
        'https://api.openai.com/v1'
      );
    } else if (format === 'anthropic' || format === 'anthropic_oauth') {
      await expect(this.page.locator(this.selectors.baseUrlInput)).toHaveValue(
        'https://api.anthropic.com/v1'
      );
    }
  }

  // Form Field Interactions
  async fillApiKey(apiKey) {
    const isChecked = await this.page.locator(this.selectors.useApiKeyCheckbox).isChecked();
    if (!isChecked) {
      await this.page.check(this.selectors.useApiKeyCheckbox);
      await expect(this.page.locator(this.selectors.apiKeyInput)).toBeEnabled();
    }
    await this.page.fill(this.selectors.apiKeyInput, apiKey);
  }

  async clearApiKey() {
    await this.page.fill(this.selectors.apiKeyInput, '');
  }

  async checkUseApiKey() {
    await this.page.check(this.selectors.useApiKeyCheckbox);
    await expect(this.page.locator(this.selectors.apiKeyInput)).toBeEnabled();
  }

  async uncheckUseApiKey() {
    await this.page.uncheck(this.selectors.useApiKeyCheckbox);
    await expect(this.page.locator(this.selectors.apiKeyInput)).toBeDisabled();
  }

  async isUseApiKeyChecked() {
    return await this.page.locator(this.selectors.useApiKeyCheckbox).isChecked();
  }

  async fillBaseUrl(baseUrl) {
    await this.page.fill(this.selectors.baseUrlInput, baseUrl);
  }

  async setCustomBaseUrl(url) {
    await this.page.fill(this.selectors.baseUrlInput, url);
  }

  async fillBasicInfo(apiKey, baseUrl = 'https://api.openai.com/v1') {
    await this.fillBaseUrl(baseUrl);
    await this.fillApiKey(apiKey);
  }

  // Extra Headers / Extra Body methods
  async fillExtraHeaders(jsonString) {
    const locator = this.page.locator('[data-testid="extra-headers-input"]');
    await locator.clear();
    await locator.fill(jsonString);
  }

  async fillExtraBody(jsonString) {
    const locator = this.page.locator('[data-testid="extra-body-input"]');
    await locator.clear();
    await locator.fill(jsonString);
  }

  async getExtraHeaders() {
    return await this.page.locator('[data-testid="extra-headers-input"]').inputValue();
  }

  async getExtraBody() {
    return await this.page.locator('[data-testid="extra-body-input"]').inputValue();
  }

  async expectExtraHeadersError(substring) {
    await expect(this.page.locator('[data-testid="extra-headers-input-error"]')).toContainText(
      substring
    );
  }

  async expectExtraBodyError(substring) {
    await expect(this.page.locator('[data-testid="extra-body-input-error"]')).toContainText(
      substring
    );
  }

  async expectExtrasVisible(visible = true) {
    const headersLocator = this.page.locator('[data-testid="extra-headers-input"]');
    const bodyLocator = this.page.locator('[data-testid="extra-body-input"]');
    if (visible) {
      await expect(headersLocator).toBeVisible();
      await expect(bodyLocator).toBeVisible();
    } else {
      await expect(headersLocator).toBeHidden();
      await expect(bodyLocator).toBeHidden();
    }
  }

  async expectExtrasPrefilledFor(formatConfig) {
    const headersValue = await this.getExtraHeaders();
    const bodyValue = await this.getExtraBody();
    const parsedHeaders = JSON.parse(headersValue);
    const parsedBody = JSON.parse(bodyValue);
    expect(parsedHeaders).toEqual(formatConfig.extraHeaders);
    expect(parsedBody).toEqual(formatConfig.extraBody);
  }

  // Prefix-related methods
  async enablePrefix() {
    await this.page.check(this.selectors.usePrefixCheckbox);
    await expect(this.page.locator(this.selectors.prefixInput)).toBeEnabled();
  }

  async disablePrefix() {
    await this.page.uncheck(this.selectors.usePrefixCheckbox);
    await expect(this.page.locator(this.selectors.prefixInput)).toBeDisabled();
  }

  async setPrefix(prefix) {
    await this.enablePrefix();
    await this.page.fill(this.selectors.prefixInput, prefix);
  }

  async fillBasicInfoWithPrefix(apiKey, prefix = null, baseUrl = 'https://api.openai.com/v1') {
    await this.fillBasicInfo(apiKey, baseUrl);
    if (prefix) {
      await this.setPrefix(prefix);
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

  // Forward All Mode methods
  async enableForwardAll() {
    await this.page.click(this.selectors.forwardAllRadio);
    await expect(this.page.locator(this.selectors.forwardAllRadio)).toBeChecked();
  }

  async selectModelsMode() {
    await this.page.click(this.selectors.forwardSelectedRadio);
    await expect(this.page.locator(this.selectors.forwardSelectedRadio)).toBeChecked();
  }

  async isForwardAllEnabled() {
    return await this.page.locator(this.selectors.forwardAllRadio).isChecked();
  }

  async expectForwardAllDisabled() {
    await expect(this.page.locator(this.selectors.forwardAllRadio)).toBeDisabled();
  }

  async expectForwardAllEnabled() {
    await expect(this.page.locator(this.selectors.forwardAllRadio)).toBeEnabled();
  }

  async expectModelSelectionState(state) {
    // state: 'enabled' | 'disabled'
    await expect(this.page.locator('[data-testid="model-selection-section"]')).toHaveAttribute(
      'data-teststate',
      state
    );
  }

  // Model Management
  async clickFetchModels() {
    await expect(this.page.locator(this.selectors.fetchModelsButton)).toBeVisible();
    await this.page.click(this.selectors.fetchModelsButton);
  }

  async expectFetchError() {
    await this.waitForToast(/failed.*fetch|fetch.*failed/i);
  }

  async expectFetchSuccess() {
    await this.waitForToast(/Models Fetched Successfully/i);
  }

  // Guard after expectFetchSuccess: makes empty-list upstream failures produce a
  // clear assertion instead of an obscure selector error in searchAndSelectModel.
  async expectAtLeastOneModelFetched() {
    const firstModel = this.page.locator('[data-testid^="available-model-"]').first();
    await expect(firstModel).toBeVisible();
  }

  async fetchAndSelectModels(models = ['gpt-4', 'gpt-3.5-turbo'], maxRetries = 1) {
    let attempt = 0;
    const maxAttempts = maxRetries + 1;

    while (attempt < maxAttempts) {
      try {
        attempt++;

        await expect(this.page.locator(this.selectors.fetchModelsButton)).toBeVisible();
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
    await this.page.waitForSelector(this.selectors.modelOption(modelName), { state: 'visible' });

    // Click on the model to select it
    await this.page.click(this.selectors.modelOption(modelName));

    // Clear the search to see all models again for next selection
    await this.page.fill(this.selectors.modelSearchInput, '');
  }

  async selectSpecificModel(modelName) {
    await this.page.click(this.selectors.modelOption(modelName));
  }

  async verifyModelAvailable(modelName) {
    await expect(this.page.locator(`text=${modelName}`)).toBeVisible();
  }

  // Connection Testing
  async testConnection(maxRetries = 1) {
    let attempt = 0;
    const maxAttempts = maxRetries + 1;

    while (attempt < maxAttempts) {
      try {
        attempt++;

        await expect(this.page.locator(this.selectors.testConnectionButton)).toBeEnabled();
        await this.page.click(this.selectors.testConnectionButton);
        await expect(this.page.locator(this.selectors.testConnectionButton)).toBeEnabled({
          timeout: 20000,
        });
        await this.waitForToast(/Connection Test Successful/i);
        return; // Success, exit the function
      } catch (error) {
        if (attempt < maxAttempts) {
          console.log(`Test connection attempt ${attempt} failed, retrying...`);
          await this.page.waitForTimeout(1000);
        } else {
          throw new Error(
            `Failed to test connection after ${maxAttempts} attempts. Last error: ${error.message}`
          );
        }
      }
    }
  }

  async expectTestConnectionSuccess() {
    await this.waitForToast(/Connection Test Successful/i);
  }

  async expectTestConnectionFailure() {
    await this.waitForToast(/Connection Test Failed/i);
  }

  // Form Validation and State
  async expectButtonEnabled(buttonSelector) {
    await expect(this.page.locator(this.selectors[buttonSelector] || buttonSelector)).toBeEnabled();
  }

  async expectButtonDisabled(buttonSelector) {
    await expect(
      this.page.locator(this.selectors[buttonSelector] || buttonSelector)
    ).toBeDisabled();
  }

  async expectFetchModelsButtonEnabled() {
    await expect(this.page.locator(this.selectors.fetchModelsButton)).toBeEnabled();
  }

  async expectFetchModelsButtonDisabled() {
    await expect(this.page.locator(this.selectors.fetchModelsButton)).toBeDisabled();
  }

  async expectTestConnectionButtonEnabled() {
    await expect(this.page.locator(this.selectors.testConnectionButton)).toBeEnabled();
  }

  async expectTestConnectionButtonDisabled() {
    await expect(this.page.locator(this.selectors.testConnectionButton)).toBeDisabled();
  }

  // Form Pre-population Verification
  async verifyFormPreFilled(api_format = 'openai', baseUrl = 'https://api.openai.com/v1') {
    await this.expectText(
      this.selectors.apiFormatSelect,
      ApiModelFormComponent.getFormatDisplayName(api_format)
    );
    await this.expectValue(this.selectors.baseUrlInput, baseUrl);
    // API key should be empty (masked for security)
    await this.expectValue(this.selectors.apiKeyInput, '');
  }

  async verifyFormPreFilledWithPrefix(
    api_format = 'openai',
    baseUrl = 'https://api.openai.com/v1',
    prefix = null
  ) {
    await this.verifyFormPreFilled(api_format, baseUrl);

    if (prefix) {
      await expect(this.page.locator(this.selectors.usePrefixCheckbox)).toBeChecked();
      await expect(this.page.locator(this.selectors.prefixInput)).toBeEnabled();
      await this.expectValue(this.selectors.prefixInput, prefix);
    } else {
      await expect(this.page.locator(this.selectors.usePrefixCheckbox)).not.toBeChecked();
      await expect(this.page.locator(this.selectors.prefixInput)).toBeDisabled();
    }
  }

  async waitForFormReady() {
    await this.page.waitForSelector(this.selectors.baseUrlInput);
    await this.page.waitForSelector(this.selectors.apiKeyInput);
  }

  async isCreateMode() {
    try {
      await this.page.waitForSelector(this.selectors.createButton);
      return true;
    } catch {
      return false;
    }
  }

  async isUpdateMode() {
    try {
      await this.page.waitForSelector(this.selectors.updateButton);
      return true;
    } catch {
      return false;
    }
  }

  async getSelectedModels() {
    // This would need to be implemented based on the UI design
    // for showing which models are currently selected
    return [];
  }

  // Toast and Success Handling (using shared logic from BasePage)
  async waitForToast(message, options = {}) {
    if (message instanceof RegExp) {
      await expect(this.page.locator(this.selectors.successToast)).toContainText(message, options);
    } else {
      await expect(this.page.locator(this.selectors.successToast)).toContainText(message, options);
    }
  }

  async waitForToastOptional(message, options = {}) {
    try {
      const timeout = process.env.CI ? 1000 : 5000;
      const finalOptions = { timeout, ...options };

      if (message instanceof RegExp) {
        await expect(this.page.locator(this.selectors.successToast)).toContainText(
          message,
          finalOptions
        );
      } else {
        await expect(this.page.locator(this.selectors.successToast)).toContainText(
          message,
          finalOptions
        );
      }
    } catch (error) {
      console.log(`Toast check skipped (CI=${!!process.env.CI}):`, message);
    }
  }

  async waitForToastAndExtractId(messagePattern) {
    await this.waitForToast(messagePattern);

    const toastText = await this.page.locator(this.selectors.successToast).textContent();
    const ulidPattern = /([0-9A-HJKMNP-TV-Z]{26})/i;
    const match = toastText.match(ulidPattern);

    if (!match) {
      throw new Error(`Failed to extract ULID from toast message: "${toastText}"`);
    }

    return match[1];
  }

  async waitForToastToHideOptional() {
    try {
      const toastLocator = this.page.locator(this.selectors.successToast);
      if (await toastLocator.isVisible({ timeout: 500 })) {
        const closeButton = this.page.locator('[toast-close]').first();
        if (await closeButton.isVisible({ timeout: 500 })) {
          await closeButton.click();
        }
        await expect(toastLocator).toBeHidden({ timeout: 2000 });
      }
    } catch {
      // Silent fail - toast hiding is optional
    }
  }

  // Helper methods for assertions
  async expectText(selector, text) {
    if (text instanceof RegExp) {
      await expect(this.page.locator(selector)).toContainText(text);
    } else {
      await expect(this.page.locator(selector)).toHaveText(text);
    }
  }

  async expectValue(selector, value) {
    await expect(this.page.locator(selector)).toHaveValue(value);
  }

  async expectVisible(selector) {
    await expect(this.page.locator(selector)).toBeVisible();
  }

  async expectBaseUrlValue(expectedValue) {
    await this.expectValue(this.selectors.baseUrlInput, expectedValue);
  }

  async expectApiKeyValue(expectedValue) {
    await this.expectValue(this.selectors.apiKeyInput, expectedValue);
  }

  async verifyForwardAllModeSelected() {
    const forwardAllRadio = this.page.locator(this.selectors.forwardAllRadio);
    await expect(forwardAllRadio).toBeVisible();
    await expect(forwardAllRadio).toBeChecked();
  }

  async verifyForwardSelectedModeSelected() {
    const forwardSelectedRadio = this.page.locator(this.selectors.forwardSelectedRadio);
    await expect(forwardSelectedRadio).toBeVisible();
    await expect(forwardSelectedRadio).toBeChecked();
  }

  async verifyPrefixValue(expectedPrefix) {
    await this.expectValue(this.selectors.prefixInput, expectedPrefix);
  }
}
