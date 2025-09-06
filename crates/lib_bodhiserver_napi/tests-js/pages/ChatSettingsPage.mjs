import { expect } from '@playwright/test';
import { BasePage } from './BasePage.mjs';

export class ChatSettingsPage extends BasePage {
  selectors = {
    // Settings panel
    settingsSidebar: '[data-testid="settings-sidebar"]',
    settingsToggle: '[data-testid="settings-toggle-button"]',

    // Model selection
    modelSelectorLoaded: '[data-testid="model-selector-loaded"]',
    comboboxTrigger: '[data-testid="model-selector-trigger"]', // Desktop version (no prefix)
    comboboxOption: (modelName) => `[data-testid="combobox-option-${modelName}"]`,

    // Settings controls
    streamModeSwitch: '#stream-mode',
    apiTokenEnabledSwitch: '#api-token-enabled',
    apiTokenInput: '#api-token',
    seedEnabledSwitch: '#seed-enabled',
    seedInput: '#seed',

    // Sliders (these might need adjustment based on actual implementation)
    temperatureSlider: '[data-testid="temperature-slider"]',
    maxTokensSlider: '[data-testid="max-tokens-slider"]',
    topPSlider: '[data-testid="top-p-slider"]',

    // System prompt
    systemPromptToggle: '[data-testid="system-prompt-enabled"]',
    systemPromptTextarea: '[data-testid="system-prompt-textarea"]',

    // Stop words
    stopWordsToggle: '[data-testid="stop-words-enabled"]',
    stopWordsInput: '[data-testid="stop-words-input"]',
  };

  /**
   * Open settings panel (settings are open by default in current UI)
   */
  async openSettings() {
    const settingsPanel = this.page.locator(this.selectors.settingsSidebar);
    await expect(settingsPanel).toBeVisible();
    await this.waitForSPAReady();
  }

  /**
   * Close settings panel (Note: Currently settings panel doesn't close properly in UI)
   */
  async closeSettings() {
    await this.page.click(this.selectors.settingsToggle);
    await this.waitForSPAReady();
    // Note: Settings panel stays visible due to current UI implementation
  }

  // Model selection

  /**
   * Select a model from the dropdown
   */
  async selectModel(modelName) {
    await this.openSettings();

    // Click the combobox trigger to open dropdown
    const trigger = this.page.locator(this.selectors.comboboxTrigger);
    await expect(trigger).toBeVisible();
    await trigger.click();

    // Select the model option (using visible element pattern from Phase 0)
    const modelOption = this.page.locator(this.selectors.comboboxOption(modelName));
    const visibleOption = modelOption.locator('visible=true').first();
    await expect(visibleOption).toBeVisible();
    await visibleOption.click();

    await this.waitForSPAReady();
  }

  /**
   * Verify that a model is selected
   */
  async verifyModelSelected(modelName) {
    await this.openSettings();

    const trigger = this.page.locator(this.selectors.comboboxTrigger);
    await expect(trigger).toContainText(modelName);
  }

  /**
   * Get currently selected model
   */
  async getCurrentModel() {
    await this.openSettings();

    const trigger = this.page.locator(this.selectors.comboboxTrigger);
    return await trigger.textContent();
  }

  // Streaming settings

  /**
   * Set streaming state (streaming is enabled by default)
   */
  async setStreaming(enabled) {
    await this.openSettings();

    const streamSwitch = this.page.locator(this.selectors.streamModeSwitch);

    // For deterministic testing - just verify current state matches expected
    if (enabled) {
      await expect(streamSwitch).toBeChecked();
    } else {
      // If we need streaming off, click to disable it
      await streamSwitch.click();
      await expect(streamSwitch).not.toBeChecked();
    }

    await this.waitForSPAReady();
  }

  /**
   * Verify streaming setting
   */
  async verifyStreamingSetting(enabled) {
    await this.openSettings();

    const streamSwitch = this.page.locator(this.selectors.streamModeSwitch);

    if (enabled) {
      await expect(streamSwitch).toBeChecked();
    } else {
      await expect(streamSwitch).not.toBeChecked();
    }
  }

  // API Token settings

  /**
   * Enable/disable API token and set value
   */
  async setApiToken(enabled, token = '') {
    await this.openSettings();

    const apiTokenToggle = this.page.locator(this.selectors.apiTokenEnabledSwitch);
    const isEnabled = await apiTokenToggle.isChecked();

    if (isEnabled !== enabled) {
      await apiTokenToggle.click();
    }

    if (enabled && token) {
      const apiTokenInput = this.page.locator(this.selectors.apiTokenInput);
      await apiTokenInput.fill(token);
    }

    await this.waitForSPAReady();
  }

  /**
   * Verify API token settings
   */
  async verifyApiTokenSettings(enabled, hasValue = false) {
    await this.openSettings();

    const apiTokenToggle = this.page.locator(this.selectors.apiTokenEnabledSwitch);
    const apiTokenInput = this.page.locator(this.selectors.apiTokenInput);

    if (enabled) {
      await expect(apiTokenToggle).toBeChecked();
      await expect(apiTokenInput).toBeEnabled();

      if (hasValue) {
        const value = await apiTokenInput.inputValue();
        expect(value.length).toBeGreaterThan(0);
      }
    } else {
      await expect(apiTokenToggle).not.toBeChecked();
      await expect(apiTokenInput).toBeDisabled();
    }
  }

  // Parameter settings

  /**
   * Set temperature value
   */
  async setTemperature(value) {
    await this.openSettings();

    const temperatureSlider = this.page.locator(this.selectors.temperatureSlider);
    if (await temperatureSlider.isVisible()) {
      // This might need adjustment based on actual slider implementation
      await temperatureSlider.fill(value.toString());
    }

    await this.waitForSPAReady();
  }

  /**
   * Set max tokens value
   */
  async setMaxTokens(value) {
    await this.openSettings();

    const maxTokensSlider = this.page.locator(this.selectors.maxTokensSlider);
    if (await maxTokensSlider.isVisible()) {
      await maxTokensSlider.fill(value.toString());
    }

    await this.waitForSPAReady();
  }

  /**
   * Set top-p value
   */
  async setTopP(value) {
    await this.openSettings();

    const topPSlider = this.page.locator(this.selectors.topPSlider);
    if (await topPSlider.isVisible()) {
      await topPSlider.fill(value.toString());
    }

    await this.waitForSPAReady();
  }

  // System prompt settings

  /**
   * Enable system prompt and set content
   */
  async setSystemPrompt(enabled, prompt = '') {
    await this.openSettings();

    const systemPromptToggle = this.page.locator(this.selectors.systemPromptToggle);
    if (await systemPromptToggle.isVisible()) {
      const isEnabled = await systemPromptToggle.isChecked();

      if (isEnabled !== enabled) {
        await systemPromptToggle.click();
      }

      if (enabled && prompt) {
        const systemPromptTextarea = this.page.locator(this.selectors.systemPromptTextarea);
        await systemPromptTextarea.fill(prompt);
      }
    }

    await this.waitForSPAReady();
  }

  /**
   * Verify system prompt settings
   */
  async verifySystemPromptSettings(enabled, expectedPrompt = '') {
    await this.openSettings();

    const systemPromptToggle = this.page.locator(this.selectors.systemPromptToggle);
    if (await systemPromptToggle.isVisible()) {
      if (enabled) {
        await expect(systemPromptToggle).toBeChecked();

        if (expectedPrompt) {
          const systemPromptTextarea = this.page.locator(this.selectors.systemPromptTextarea);
          await expect(systemPromptTextarea).toHaveValue(expectedPrompt);
        }
      } else {
        await expect(systemPromptToggle).not.toBeChecked();
      }
    }
  }

  // Stop words settings

  /**
   * Enable stop words and set values
   */
  async setStopWords(enabled, stopWords = []) {
    await this.openSettings();

    const stopWordsToggle = this.page.locator(this.selectors.stopWordsToggle);
    if (await stopWordsToggle.isVisible()) {
      const isEnabled = await stopWordsToggle.isChecked();

      if (isEnabled !== enabled) {
        await stopWordsToggle.click();
      }

      if (enabled && stopWords.length > 0) {
        const stopWordsInput = this.page.locator(this.selectors.stopWordsInput);
        await stopWordsInput.fill(stopWords.join(', '));
      }
    }

    await this.waitForSPAReady();
  }

  // Comprehensive settings operations

  /**
   * Configure all settings at once
   */
  async configureSettings(settings) {
    await this.openSettings();

    if (settings.model) {
      await this.selectModel(settings.model);
    }

    if (settings.streaming !== undefined) {
      await this.toggleStreaming(settings.streaming);
    }

    if (settings.apiToken !== undefined) {
      await this.setApiToken(settings.apiToken.enabled, settings.apiToken.value);
    }

    if (settings.temperature !== undefined) {
      await this.setTemperature(settings.temperature);
    }

    if (settings.maxTokens !== undefined) {
      await this.setMaxTokens(settings.maxTokens);
    }

    if (settings.topP !== undefined) {
      await this.setTopP(settings.topP);
    }

    if (settings.systemPrompt !== undefined) {
      await this.setSystemPrompt(settings.systemPrompt.enabled, settings.systemPrompt.content);
    }

    if (settings.stopWords !== undefined) {
      await this.setStopWords(settings.stopWords.enabled, settings.stopWords.words);
    }
  }

  /**
   * Verify all settings match expected configuration
   */
  async verifySettings(expectedSettings) {
    await this.openSettings();

    if (expectedSettings.model) {
      await this.verifyModelSelected(expectedSettings.model);
    }

    if (expectedSettings.streaming !== undefined) {
      await this.verifyStreamingSetting(expectedSettings.streaming);
    }

    if (expectedSettings.apiToken !== undefined) {
      await this.verifyApiTokenSettings(
        expectedSettings.apiToken.enabled,
        expectedSettings.apiToken.hasValue
      );
    }

    if (expectedSettings.systemPrompt !== undefined) {
      await this.verifySystemPromptSettings(
        expectedSettings.systemPrompt.enabled,
        expectedSettings.systemPrompt.content
      );
    }
  }

  /**
   * Reset all settings to defaults (if UI provides this functionality)
   */
  async resetToDefaults() {
    await this.openSettings();

    // Look for reset button (this might not be implemented)
    const resetButton = this.page.locator(
      '[data-testid="reset-settings"], button:has-text("Reset")'
    );
    if (await resetButton.isVisible()) {
      await resetButton.click();

      // Handle confirmation if needed
      const confirmButton = this.page.locator(
        'button:has-text("Confirm"), button:has-text("Reset")'
      );
      if (await confirmButton.isVisible()) {
        await confirmButton.click();
      }

      await this.waitForSPAReady();
    }
  }

  /**
   * Test responsive settings panel
   */
  async testResponsiveSettings(viewportWidth) {
    if (viewportWidth < 768) {
      // Mobile: settings should be in drawer/modal
      await this.page.click(this.selectors.settingsToggle);

      const settingsPanel = this.page.locator(this.selectors.settingsSidebar);
      await expect(settingsPanel).toBeVisible();

      // Should be able to close by clicking toggle or outside
      await this.page.click(this.selectors.settingsToggle);
      await expect(settingsPanel).not.toBeVisible();
    } else {
      // Desktop: settings should be a sidebar
      const settingsPanel = this.page.locator(this.selectors.settingsSidebar);

      // May be visible by default on desktop
      if (!(await settingsPanel.isVisible())) {
        await this.page.click(this.selectors.settingsToggle);
        await expect(settingsPanel).toBeVisible();
      }
    }
  }

  /**
   * Verify settings persistence after page reload
   */
  async verifySettingsPersistence(expectedSettings) {
    // First set the settings
    await this.configureSettings(expectedSettings);

    // Reload the page
    await this.page.reload();
    await this.waitForSPAReady();

    // Verify settings are still there
    await this.verifySettings(expectedSettings);
  }

  /**
   * Test model switching functionality
   */
  async testModelSwitching(models) {
    for (const model of models) {
      await this.selectModel(model);
      await this.verifyModelSelected(model);

      // Small delay between switches
      await this.page.waitForTimeout(500);
    }
  }

  /**
   * Verify model options are available
   */
  async verifyModelOptionsAvailable(expectedModels) {
    await this.openSettings();

    // Open the dropdown
    const trigger = this.page.locator(this.selectors.comboboxTrigger);
    await trigger.click();

    // Check each expected model option exists
    for (const modelName of expectedModels) {
      const option = this.page.locator(this.selectors.comboboxOption(modelName));
      await expect(option.first()).toBeVisible();
    }

    // Close dropdown by clicking trigger again
    await trigger.click();
  }
}
