import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

export class LocalModelFormPage extends BasePage {
  selectors = {
    aliasInput: '[data-testid="alias-input"]',
    repoSelect: '[data-testid="repo-select"]',
    filenameSelect: '[data-testid="filename-select"]',
    snapshotSelect: '[data-testid="snapshot-select"]',
    contextParamsTextarea: '[data-testid="context-params"]',
    requestParamsToggle: '[data-testid="request-params-toggle"]',
    submitButton: '[data-testid="submit-alias-form"]',
    // Request parameter fields
    temperatureInput: '[data-testid="request-param-temperature"]',
    maxTokensInput: '[data-testid="request-param-max_tokens"]',
    topPInput: '[data-testid="request-param-top_p"]',
    seedInput: '[data-testid="request-param-seed"]',
    stopInput: '[data-testid="request-param-stop"]',
    frequencyPenaltyInput: '[data-testid="request-param-frequency_penalty"]',
    presencePenaltyInput: '[data-testid="request-param-presence_penalty"]',
    userInput: '[data-testid="request-param-user"]',
    // ComboBox selectors
    comboboxTrigger: (testId) => `[data-testid="${testId}"]`,
    comboboxOption: (value) => `[data-testid="combobox-option-${value}"]`,
  };

  async waitForFormReady() {
    await this.waitForSelector(this.selectors.aliasInput);
    await this.waitForSPAReady();
  }

  async fillBasicInfo(alias, repo, filename, snapshot = null) {
    // Fill alias
    await this.fillTestId('alias-input', alias);

    // Select repo from combobox
    if (repo) {
      await this.selectFromCombobox('repo-select', repo);
    }

    // Select filename from combobox
    if (filename) {
      await this.selectFromCombobox('filename-select', filename);
    }

    // Wait for snapshot options to load after repo and filename selection
    if (repo && filename) {
      await this.waitForSnapshotToLoad();

      // Select specific snapshot if provided, otherwise auto-selected snapshot will be used
      if (snapshot) {
        await this.selectFromCombobox('snapshot-select', snapshot);
      }
    }
  }

  async fillContextParams(contextParams) {
    if (contextParams) {
      await this.page.fill(this.selectors.contextParamsTextarea, contextParams);
    }
  }

  async expandRequestParams() {
    const toggle = this.page.locator(this.selectors.requestParamsToggle);
    // Check if request params section is collapsed
    const isExpanded = await toggle.getAttribute('aria-expanded');
    if (isExpanded !== 'true') {
      await toggle.click();
      // Wait for section to expand
      await this.page.waitForTimeout(500);
    }
  }

  async fillRequestParams(params = {}) {
    // Expand request params section if needed
    await this.expandRequestParams();

    // Fill individual parameter fields
    if (params.temperature !== undefined) {
      await this.page.fill(this.selectors.temperatureInput, params.temperature.toString());
    }
    if (params.max_tokens !== undefined) {
      await this.page.fill(this.selectors.maxTokensInput, params.max_tokens.toString());
    }
    if (params.top_p !== undefined) {
      await this.page.fill(this.selectors.topPInput, params.top_p.toString());
    }
    if (params.seed !== undefined) {
      await this.page.fill(this.selectors.seedInput, params.seed.toString());
    }
    if (params.stop !== undefined) {
      const stopValue = Array.isArray(params.stop) ? params.stop.join(',') : params.stop;
      await this.page.fill(this.selectors.stopInput, stopValue);
    }
    if (params.frequency_penalty !== undefined) {
      await this.page.fill(
        this.selectors.frequencyPenaltyInput,
        params.frequency_penalty.toString()
      );
    }
    if (params.presence_penalty !== undefined) {
      await this.page.fill(this.selectors.presencePenaltyInput, params.presence_penalty.toString());
    }
    if (params.user !== undefined) {
      await this.page.fill(this.selectors.userInput, params.user);
    }
  }

  async selectFromCombobox(testId, value) {
    // Click the combobox trigger to open it
    const trigger = this.page.locator(this.selectors.comboboxTrigger(testId));
    await expect(trigger).toBeVisible();
    await trigger.click();

    // Wait for options to appear and select the specific option
    const option = this.page.locator(this.selectors.comboboxOption(value));
    await expect(option).toBeVisible();
    await option.click();
  }

  async waitForSnapshotToLoad() {
    // Wait for snapshot combobox to become enabled
    const snapshotSelect = this.page.locator(this.selectors.snapshotSelect);
    await expect(snapshotSelect).toBeVisible();

    // Wait for it to not be disabled (snapshot options should load after repo/filename selection)
    await this.page.waitForFunction(() => {
      const snapshotElement = document.querySelector('[data-testid="snapshot-select"]');
      return snapshotElement && !snapshotElement.disabled;
    });

    // Small delay to ensure snapshot options are fully loaded
    await this.page.waitForTimeout(500);
  }

  async getSelectedSnapshot() {
    const snapshotSelect = this.page.locator(this.selectors.snapshotSelect);
    const snapshotValue = await snapshotSelect.textContent();
    return snapshotValue && !snapshotValue.includes('Select') ? snapshotValue.trim() : '';
  }

  async createAlias() {
    const submitBtn = this.page.locator(this.selectors.submitButton);
    await expect(submitBtn).toBeVisible();
    await expect(submitBtn).toBeEnabled();
    await submitBtn.click();

    // Wait for navigation back to models list
    await this.waitForUrl('/ui/models/');
    await this.waitForSPAReady();
  }

  async updateAlias() {
    const submitBtn = this.page.locator(this.selectors.submitButton);
    await expect(submitBtn).toBeVisible();
    await expect(submitBtn).toBeEnabled();
    await expect(submitBtn).toContainText('Update');
    await submitBtn.click();

    // Wait for navigation back to models list
    await this.waitForUrl('/ui/models/');
    await this.waitForSPAReady();
  }

  async isEditMode() {
    const submitBtn = this.page.locator(this.selectors.submitButton);
    const buttonText = await submitBtn.textContent();
    return buttonText?.includes('Update');
  }

  async getFormData() {
    const alias = await this.page.locator(this.selectors.aliasInput).inputValue();
    const contextParams = await this.page
      .locator(this.selectors.contextParamsTextarea)
      .inputValue();

    // Get selected values from comboboxes - the selectors themselves are the buttons
    const repoValue = await this.page.locator(this.selectors.repoSelect).textContent();
    const filenameValue = await this.page.locator(this.selectors.filenameSelect).textContent();
    const snapshotValue = await this.getSelectedSnapshot();

    return {
      alias,
      repo: repoValue && repoValue !== 'Select repo' ? repoValue.trim() : '',
      filename: filenameValue && filenameValue !== 'Select filename' ? filenameValue.trim() : '',
      snapshot: snapshotValue,
      contextParams,
    };
  }
}
