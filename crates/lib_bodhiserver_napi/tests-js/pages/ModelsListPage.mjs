import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

export class ModelsListPage extends BasePage {
  selectors = {
    content: '[data-testid="models-content"]',
    table: '[data-testid="table-list-models"]',
    newApiModelButton: 'button:has-text("New API Model")',
    newModelAliasButton: '[data-testid="new-model-alias-button"]',
    // Simplified API model selectors using consistent data attributes
    modelRow: (modelId) => `[data-model-id="${modelId}"]`,
    aliasCell: (modelId) => `[data-testid="alias-cell-${modelId}"]`,
    repoCell: (modelId) => `[data-testid="repo-cell-${modelId}"]`,
    filenameCell: (modelId) => `[data-testid="filename-cell-${modelId}"]`,
    prefixCell: (modelId) => `[data-testid="prefix-cell-${modelId}"]`,
    forwardAllCell: (modelId) => `[data-testid="forward-all-cell-${modelId}"]`,
    editButton: (modelId) => `[data-testid="edit-button-${modelId}"]:visible`,
    deleteButton: (modelId) => `[data-testid="delete-button-${modelId}"]:visible`,
    modelChatButton: (modelName) => `[data-testid="model-chat-button-${modelName}"]`,
    deleteConfirmDialog: 'text=Delete API Model',
    confirmDeleteButton: 'button:has-text("Delete")',
    // Local model alias selectors
    localAliasCell: (alias) => `[data-testid="alias-cell-${alias}"]`,
    localRepoCell: (alias) => `[data-testid="repo-cell-${alias}"]`,
    localFilenameCell: (alias) => `[data-testid="filename-cell-${alias}"]`,
    sourceBadge: (identifier) => `[data-testid="source-badge-${identifier}"]`,
    createAliasFromModelButton: (alias) => `[data-testid="create-alias-from-model-${alias}"]`,
    externalButton: (alias) => `[data-testid="external-button-${alias}"]`,
    chatButton: (alias) => `[data-testid="chat-button-${alias}"]`,
    // Preview modal selectors
    previewButton: (identifier) => `[data-testid="preview-button-${identifier}"]`,
    previewModal: '[data-testid="model-preview-modal"]',
    previewBasicAlias: '[data-testid="preview-basic-alias"]',
    previewBasicRepo: '[data-testid="preview-basic-repo"]',
    previewBasicFilename: '[data-testid="preview-basic-filename"]',
    previewBasicSnapshot: '[data-testid="preview-basic-snapshot"]',
    previewBasicSource: '[data-testid="preview-basic-source"]',
    previewCapabilityVision: '[data-testid="preview-capability-vision"]',
    previewCapabilityAudio: '[data-testid="preview-capability-audio"]',
    previewCapabilityThinking: '[data-testid="preview-capability-thinking"]',
    previewCapabilityFunctionCalling: '[data-testid="preview-capability-function-calling"]',
    previewCapabilityStructuredOutput: '[data-testid="preview-capability-structured-output"]',
    previewContextMaxInput: '[data-testid="preview-context-max-input"]',
    previewContextMaxOutput: '[data-testid="preview-context-max-output"]',
    previewArchitectureFormat: '[data-testid="preview-architecture-format"]',
    previewArchitectureFamily: '[data-testid="preview-architecture-family"]',
    previewArchitectureParameterCount: '[data-testid="preview-architecture-parameter-count"]',
    previewArchitectureQuantization: '[data-testid="preview-architecture-quantization"]',
    previewApiFormat: '[data-testid="preview-api-format"]',
    previewApiBaseUrl: '[data-testid="preview-api-base-url"]',
    previewApiPrefix: '[data-testid="preview-api-prefix"]',
    previewApiForwardAll: '[data-testid="preview-api-forward-all"]',
    previewApiModels: '[data-testid="preview-api-models"]',
    // Refresh button selectors
    refreshAllButton: '[data-testid="refresh-all-models-button"]',
    refreshButton: (alias) => `[data-testid="refresh-button-${alias}"]`,
  };

  async navigateToModels() {
    await this.navigate('/ui/models/');
    await this.waitForSelector(this.selectors.content);
  }

  async clickNewApiModel() {
    await this.expectVisible(this.selectors.newApiModelButton);
    await this.page.click(this.selectors.newApiModelButton);
    await this.waitForUrl('/ui/api-models/new/');
    await this.waitForSPAReady();
  }

  async verifyApiModelInList(
    modelId,
    api_format = 'openai',
    baseUrl = 'https://api.openai.com/v1'
  ) {
    // Wait for table and data to load
    await this.waitForSelector(this.selectors.table);
    await this.waitForSelector(`${this.selectors.table} tbody tr`);

    // Use simplified model row selector for direct access (select first matching cell)
    const modelRow = this.page.locator(this.selectors.modelRow(modelId)).first();
    await expect(modelRow).toBeVisible();

    // Verify model data in table cells using consistent selectors
    await expect(this.page.locator(this.selectors.aliasCell(modelId))).toContainText(modelId);
    await expect(this.page.locator(this.selectors.repoCell(modelId))).toContainText(api_format);
    await expect(this.page.locator(this.selectors.filenameCell(modelId))).toContainText(baseUrl);
  }

  async editModel(modelId) {
    const editBtn = this.page.locator(this.selectors.editButton(modelId));
    await expect(editBtn).toBeVisible();
    await editBtn.click();
    await this.waitForUrl('/ui/api-models/edit/');
    await this.waitForSPAReady();

    // Verify we're on the edit page with correct model ID
    const currentUrl = new URL(this.page.url());
    expect(currentUrl.searchParams.get('id')).toBe(modelId);
  }

  async deleteModel(modelId) {
    const deleteBtn = this.page.locator(this.selectors.deleteButton(modelId));
    await expect(deleteBtn).toBeVisible();
    await deleteBtn.click();

    // Handle confirmation dialog
    await expect(this.page.locator(this.selectors.deleteConfirmDialog)).toBeVisible();
    await this.page.click(this.selectors.confirmDeleteButton);

    // Wait for success toast
    await this.waitForToast(`API model ${modelId} deleted successfully`);
  }

  async waitForModelsToLoad() {
    await this.waitForSelector(this.selectors.content);
    // Give time for models to load
    await this.page.waitForTimeout(1000);
  }

  async clickChatWithModel(modelName) {
    // Wait for the table to load first
    await this.waitForSelector(this.selectors.table);

    // Find the visible model chat button (handles responsive layout with multiple buttons)
    const modelChatButtons = this.page.locator(this.selectors.modelChatButton(modelName));
    const visibleButton = modelChatButtons.filter({ visible: true }).first();

    await expect(visibleButton).toBeVisible();
    await visibleButton.click();

    // Wait for navigation to chat with model pre-selected
    await this.page.waitForURL(
      (url) => url.pathname === '/ui/chat/' && url.searchParams.get('model') === modelName
    );
    await this.waitForSPAReady();
  }

  // Local Model Alias specific methods
  async clickNewModelAlias() {
    await this.expectVisible(this.selectors.newModelAliasButton);
    await this.page.click(this.selectors.newModelAliasButton);
    await this.waitForUrl('/ui/models/new/');
    await this.waitForSPAReady();
  }

  async verifyLocalModelInList(alias, repo, filename, source = 'user') {
    // Wait for table and data to load
    await this.waitForSelector(this.selectors.table);
    await this.waitForSelector(`${this.selectors.table} tbody tr`);

    // Wait for the specific model alias cell to appear (gives React Query time to reload data)
    await expect(this.page.locator(this.selectors.localAliasCell(alias))).toBeVisible({
      timeout: 10000,
    });

    // Verify model data in table cells
    await expect(this.page.locator(this.selectors.localAliasCell(alias))).toContainText(alias);
    await expect(this.page.locator(this.selectors.localRepoCell(alias))).toContainText(repo);
    await expect(this.page.locator(this.selectors.localFilenameCell(alias))).toContainText(
      filename
    );

    // Verify source badge
    await expect(this.page.locator(this.selectors.sourceBadge(alias))).toContainText(source);
  }

  async editLocalModel(alias) {
    // Get model UUID from the row's data-test-model-id attribute
    const aliasCell = this.page.locator(this.selectors.localAliasCell(alias));
    await expect(aliasCell).toBeVisible();
    const row = aliasCell.locator('xpath=ancestor::tr');
    const modelId = await row.getAttribute('data-test-model-id');

    const editBtn = this.page.locator(this.selectors.editButton(alias));
    await expect(editBtn).toBeVisible();
    await editBtn.click();
    await this.waitForUrl('/ui/models/edit/');
    await this.waitForSPAReady();

    // Verify we're on the edit page with correct model UUID
    const currentUrl = new URL(this.page.url());
    expect(currentUrl.searchParams.get('id')).toBe(modelId);
  }

  async deleteLocalModel(alias) {
    // Note: Local models with source='model' cannot be deleted, only 'user' aliases can be deleted
    const deleteBtn = this.page.locator(this.selectors.deleteButton(alias));
    await expect(deleteBtn).toBeVisible();
    await deleteBtn.click();

    // Handle confirmation dialog (if applicable)
    // For now, local model deletion might not have confirmation dialogs
    await this.waitForToast(`Model alias ${alias} deleted successfully`);
  }

  async chatWithLocalModel(alias) {
    const chatBtn = this.page.locator(this.selectors.chatButton(alias));
    await expect(chatBtn).toBeVisible();
    await chatBtn.click();

    // Wait for navigation to chat with model pre-selected
    await this.page.waitForURL(
      (url) => url.pathname === '/ui/chat/' && url.searchParams.get('model') === alias
    );
    await this.waitForSPAReady();
  }

  async createAliasFromModel(modelAlias) {
    const createBtn = this.page.locator(this.selectors.createAliasFromModelButton(modelAlias));
    await expect(createBtn).toBeVisible();
    await createBtn.click();

    // Wait for navigation to new alias form with pre-populated data
    await this.waitForUrl('/ui/models/new/');
    await this.waitForSPAReady();
  }

  async openExternalLink(alias) {
    // This opens in a new tab/window - we'll need to handle this in tests
    const externalBtn = this.page.locator(this.selectors.externalButton(alias));
    await expect(externalBtn).toBeVisible();
    return externalBtn; // Return the element for the test to handle the new page
  }

  async verifyModelTypeBadge(identifier, expectedType) {
    const badge = this.page.locator(this.selectors.sourceBadge(identifier));
    await expect(badge).toBeVisible();
    await expect(badge).toContainText(expectedType);
  }

  // Simplified helper method for API model verification
  async verifyApiModelExists(modelId) {
    await this.waitForSelector(this.selectors.table);
    const modelRow = this.page.locator(this.selectors.modelRow(modelId)).first();
    await expect(modelRow).toBeVisible();
  }

  // Property-based verification methods (CI-friendly)
  async getModelCount() {
    await this.waitForSelector(this.selectors.table);
    await this.waitForSelector(`${this.selectors.table} tbody tr`);
    const rows = await this.page.locator(`${this.selectors.table} tbody tr`).count();
    return rows;
  }

  async findModelByProperties(baseUrl, apiFormat) {
    await this.waitForSelector(this.selectors.table);
    await this.waitForSelector(`${this.selectors.table} tbody tr`);

    const rows = await this.page.locator(`${this.selectors.table} tbody tr`).all();
    for (const row of rows) {
      try {
        // Get text from URL and format columns (adjust indices as needed)
        const urlText = await row.locator('td:nth-child(3)').textContent();
        const formatText = await row.locator('td:nth-child(2)').textContent();

        if (urlText?.includes(baseUrl) && formatText && formatText.includes(apiFormat)) {
          return row;
        }
      } catch {}
    }
    return null;
  }

  async verifyModelByProperties(baseUrl, apiFormat, expectedModels = []) {
    const modelRow = await this.findModelByProperties(baseUrl, apiFormat);
    expect(modelRow).not.toBeNull();

    // Optionally verify selected models if provided
    if (expectedModels.length > 0) {
      const modelText = await modelRow.textContent();
      for (const model of expectedModels) {
        expect(modelText).toContain(model);
      }
    }
  }

  async editModelByProperties(baseUrl, apiFormat) {
    const modelRow = await this.findModelByProperties(baseUrl, apiFormat);
    expect(modelRow).not.toBeNull();

    const editBtn = modelRow.locator('[data-testid*="edit"], button:has-text("Edit")').first();
    await expect(editBtn).toBeVisible();
    await editBtn.click();

    await this.waitForUrl('/ui/api-models/edit/');
    await this.waitForSPAReady();
  }

  async getLatestModel() {
    await this.waitForSelector(this.selectors.table);
    await this.waitForSelector(`${this.selectors.table} tbody tr`);

    // Get the first row (assuming models are sorted with newest first)
    const firstRow = this.page.locator(`${this.selectors.table} tbody tr`).first();
    await expect(firstRow).toBeVisible();
    return firstRow;
  }

  async getModelIdFromRow(row) {
    // Try to extract ID from the first column (alias/ID column)
    try {
      const idText = await row.locator('td:first-child').textContent();
      return idText?.trim();
    } catch {
      return null;
    }
  }

  async getModelPrefix(modelId) {
    await this.waitForSelector(this.selectors.table);
    const prefixCell = this.page.locator(this.selectors.prefixCell(modelId));
    await expect(prefixCell).toBeVisible();
    const text = await prefixCell.textContent();
    return text?.trim();
  }

  async getModelForwardAll(modelId) {
    await this.waitForSelector(this.selectors.table);
    const forwardAllCell = this.page.locator(this.selectors.forwardAllCell(modelId));
    await expect(forwardAllCell).toBeVisible();
    const text = await forwardAllCell.textContent();
    return text?.trim();
  }

  async verifyForwardAllModel(modelId, expectedPrefix) {
    // Verify prefix column
    const prefix = await this.getModelPrefix(modelId);
    expect(prefix).toBe(expectedPrefix);

    // Verify forward_all column shows "Yes"
    const forwardAll = await this.getModelForwardAll(modelId);
    expect(forwardAll).toBe('Yes');
  }

  async getModelRow(modelId) {
    await this.waitForSelector(this.selectors.table);
    const modelRow = this.page.locator(this.selectors.modelRow(modelId)).first();
    await expect(modelRow).toBeVisible();

    const id = await this.page.locator(this.selectors.aliasCell(modelId)).textContent();
    const api_format = await this.page.locator(this.selectors.repoCell(modelId)).textContent();
    const base_url = await this.page.locator(this.selectors.filenameCell(modelId)).textContent();
    const prefix = await this.page.locator(this.selectors.prefixCell(modelId)).textContent();
    const forward_all = await this.page
      .locator(this.selectors.forwardAllCell(modelId))
      .textContent();

    return {
      id: id?.trim() || '',
      api_format: api_format?.trim() || '',
      base_url: base_url?.trim() || '',
      prefix: prefix?.trim() || '',
      forward_all: forward_all?.trim() || '',
    };
  }

  // Preview modal methods
  async clickPreviewButton(identifier) {
    const previewBtn = this.page.locator(this.selectors.previewButton(identifier));
    await expect(previewBtn).toBeVisible();
    await previewBtn.click();
    await expect(this.page.locator(this.selectors.previewModal)).toBeVisible();
  }

  async closePreviewModal() {
    await this.page.keyboard.press('Escape');
    await expect(this.page.locator(this.selectors.previewModal)).not.toBeVisible();
  }

  async verifyPreviewBasicInfo(expectedValues) {
    await expect(this.page.locator(this.selectors.previewModal)).toBeVisible();

    if (expectedValues.alias) {
      await expect(this.page.locator(this.selectors.previewBasicAlias)).toContainText(
        expectedValues.alias
      );
    }
    if (expectedValues.repo) {
      await expect(this.page.locator(this.selectors.previewBasicRepo)).toContainText(
        expectedValues.repo
      );
    }
    if (expectedValues.filename) {
      await expect(this.page.locator(this.selectors.previewBasicFilename)).toContainText(
        expectedValues.filename
      );
    }
    if (expectedValues.snapshot) {
      await expect(this.page.locator(this.selectors.previewBasicSnapshot)).toContainText(
        expectedValues.snapshot
      );
    }
    if (expectedValues.source) {
      await expect(this.page.locator(this.selectors.previewBasicSource)).toContainText(
        expectedValues.source
      );
    }
  }

  async verifyPreviewCapability(capabilityName, expectedValue) {
    const selectorMap = {
      vision: this.selectors.previewCapabilityVision,
      audio: this.selectors.previewCapabilityAudio,
      thinking: this.selectors.previewCapabilityThinking,
      function_calling: this.selectors.previewCapabilityFunctionCalling,
      structured_output: this.selectors.previewCapabilityStructuredOutput,
    };

    const selector = selectorMap[capabilityName];
    const element = this.page.locator(selector);

    if (expectedValue === true) {
      await expect(element).toContainText('Supported');
    } else if (expectedValue === false) {
      await expect(element).toContainText('Not supported');
    } else {
      // If undefined, the capability should not be visible
      await expect(element).not.toBeVisible();
    }
  }

  async verifyPreviewContext(maxInput, maxOutput) {
    if (maxInput) {
      await expect(this.page.locator(this.selectors.previewContextMaxInput)).toContainText(
        maxInput.toLocaleString()
      );
    }
    if (maxOutput) {
      await expect(this.page.locator(this.selectors.previewContextMaxOutput)).toContainText(
        maxOutput.toLocaleString()
      );
    }
  }

  async verifyPreviewArchitecture(expectedValues) {
    if (expectedValues.format) {
      await expect(this.page.locator(this.selectors.previewArchitectureFormat)).toContainText(
        expectedValues.format
      );
    }
    if (expectedValues.family) {
      await expect(this.page.locator(this.selectors.previewArchitectureFamily)).toContainText(
        expectedValues.family
      );
    }
    if (expectedValues.parameter_count) {
      await expect(
        this.page.locator(this.selectors.previewArchitectureParameterCount)
      ).toContainText(expectedValues.parameter_count.toLocaleString());
    }
    if (expectedValues.quantization) {
      await expect(this.page.locator(this.selectors.previewArchitectureQuantization)).toContainText(
        expectedValues.quantization
      );
    }
  }

  async verifyPreviewApiConfig(expectedValues) {
    await expect(this.page.locator(this.selectors.previewModal)).toBeVisible();

    if (expectedValues.api_format) {
      await expect(this.page.locator(this.selectors.previewApiFormat)).toContainText(
        expectedValues.api_format
      );
    }
    if (expectedValues.base_url) {
      await expect(this.page.locator(this.selectors.previewApiBaseUrl)).toContainText(
        expectedValues.base_url
      );
    }
    if (expectedValues.prefix) {
      await expect(this.page.locator(this.selectors.previewApiPrefix)).toContainText(
        expectedValues.prefix
      );
    }
    if (expectedValues.forward_all !== undefined) {
      const expectedText = expectedValues.forward_all ? 'Enabled' : 'Disabled';
      await expect(this.page.locator(this.selectors.previewApiForwardAll)).toContainText(
        expectedText
      );
    }
  }

  // Refresh metadata methods
  async clickRefreshAll() {
    const refreshBtn = this.page.locator(this.selectors.refreshAllButton);
    await expect(refreshBtn).toBeVisible();
    await expect(refreshBtn).toBeEnabled();
    await refreshBtn.click();

    // Wait for toast notification
    await this.waitForToast('Metadata refresh queued');
  }

  async waitForQueueIdle(maxWaitMs = 180000) {
    // Wait for the success toast which indicates frontend has processed queue completion
    await this.waitForToast('Metadata refresh completed', { timeout: maxWaitMs });
    // Wait a bit more for the models list to refetch
    await this.page.waitForTimeout(500);
  }

  async verifyRefreshButtonState(expectedState) {
    const refreshBtn = this.page.locator(this.selectors.refreshAllButton);
    await expect(refreshBtn).toBeVisible();

    if (expectedState === 'disabled') {
      await expect(refreshBtn).toBeDisabled();
    } else if (expectedState === 'enabled') {
      await expect(refreshBtn).toBeEnabled();
    }
  }

  // Modal refresh method (replaces per-row refresh)
  async clickRefreshButton(alias) {
    // Open preview modal first
    await this.clickPreviewButton(alias);

    // Wait for modal to be visible
    await expect(this.page.locator('[data-testid="model-preview-modal"]')).toBeVisible();

    // Try header button first (if metadata exists), fallback to body button
    const headerBtn = this.page.locator('[data-testid="preview-modal-refresh-button-header"]');
    const bodyBtn = this.page.locator('[data-testid="preview-modal-refresh-button-body"]');

    const refreshBtn = (await headerBtn.isVisible()) ? headerBtn : bodyBtn;
    await expect(refreshBtn).toBeVisible();
    await expect(refreshBtn).toBeEnabled();
    await refreshBtn.click();

    // Close modal first to avoid toast selector conflicts
    await this.closePreviewModal();

    // Wait for toast notification after modal is closed
    await this.waitForToast('Metadata refreshed successfully');
  }

  async verifyRefreshButtonStateForModel(alias, expectedState) {
    const refreshBtn = this.page.locator(this.selectors.refreshButton(alias));
    await expect(refreshBtn).toBeVisible();

    if (expectedState === 'disabled') {
      await expect(refreshBtn).toBeDisabled();
    } else if (expectedState === 'enabled') {
      await expect(refreshBtn).toBeEnabled();
    } else if (expectedState === 'loading') {
      // Verify spinner is showing
      await expect(refreshBtn).toBeDisabled();
      const spinner = refreshBtn.locator('svg.animate-spin');
      await expect(spinner).toBeVisible();
    }
  }
}
