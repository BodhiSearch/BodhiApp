import { expect } from '@playwright/test';
import { BasePage } from './BasePage.mjs';

export class ModelsListPage extends BasePage {
  selectors = {
    content: '[data-testid="models-content"]',
    table: '[data-testid="table-list-models"]',
    newApiModelButton: 'button:has-text("New API Model")',
    newModelAliasButton: '[data-testid="new-model-alias-button"]',
    tableRow: (index = 'first') =>
      `[data-testid="table-list-models"] tbody tr${index === 'first' ? '' : ':nth-child(' + index + ')'}`,
    // API model selectors
    aliasCell: (modelId) => `[data-testid="alias-cell-api_${modelId}"]`,
    repoCell: (modelId) => `[data-testid="repo-cell-api_${modelId}"]`,
    filenameCell: (modelId) => `[data-testid="filename-cell-api_${modelId}"]`,
    editButton: (modelId) => `[data-testid="edit-button-${modelId}"]:visible`,
    deleteButton: (modelId) => `[data-testid="delete-button-${modelId}"]:visible`,
    chatButton: (modelId) => `[data-testid="chat-button-${modelId}"]`,
    modelChatButton: (modelName) => `[data-testid="model-chat-button-${modelName}"]`,
    modelsDropdown: (modelId) => `[data-testid="models-dropdown-${modelId}"]`,
    deleteConfirmDialog: 'text=Delete API Model',
    confirmDeleteButton: 'button:has-text("Delete")',
    // Local model alias selectors
    localAliasCell: (alias) => `[data-testid="alias-cell-${alias}"]`,
    localRepoCell: (alias) => `[data-testid="repo-cell-${alias}"]`,
    localFilenameCell: (alias) => `[data-testid="filename-cell-${alias}"]`,
    sourceBadge: (identifier) => `[data-testid="source-badge-${identifier}"]`,
    createAliasFromModelButton: (alias) => `[data-testid="create-alias-from-model-${alias}"]`,
    externalButton: (alias) => `[data-testid="external-button-${alias}"]`,
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

  async verifyApiModelInList(modelId, provider = 'OpenAI', baseUrl = 'https://api.openai.com/v1') {
    // Wait for table and data to load
    await this.waitForSelector(this.selectors.table);
    await this.waitForSelector(`${this.selectors.table} tbody tr`);

    const firstRow = this.page.locator(this.selectors.tableRow('first'));
    await expect(firstRow).toBeVisible();

    // Verify model data in table cells
    await expect(this.page.locator(this.selectors.aliasCell(modelId))).toContainText(modelId);
    await expect(this.page.locator(this.selectors.repoCell(modelId))).toContainText(provider);
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

    // Verify model is removed from list
    const rowCount = await this.page.locator(`${this.selectors.table} tbody tr`).count();
    expect(rowCount).toBe(0);
  }

  async getRowCount() {
    try {
      return await this.page.locator(`${this.selectors.table} tbody tr`).count();
    } catch (error) {
      return 0;
    }
  }

  async waitForModelsToLoad() {
    await this.waitForSelector(this.selectors.content);
    // Give time for models to load
    await this.page.waitForTimeout(1000);
  }

  // Mobile-specific methods
  async clickModelsDropdown(modelId) {
    const dropdown = this.page.locator(this.selectors.modelsDropdown(modelId));
    await expect(dropdown.first()).toBeVisible();
    await dropdown.first().click();
  }

  async verifyDropdownModels(expectedCount = 2) {
    await expect(this.page.locator('[role="menuitem"]')).toHaveCount(expectedCount);
  }

  async selectModelFromDropdown(modelName) {
    await this.page.click(`[role="menuitem"]:has-text("${modelName}")`);
  }

  async clickChatWithModel(modelName) {
    // Wait for the table to load first
    await this.waitForSelector(this.selectors.table);

    // Find the visible model chat button (handles responsive layout with multiple buttons)
    const modelChatButtons = this.page.locator(this.selectors.modelChatButton(modelName));
    const visibleButton = modelChatButtons.locator('visible=true').first();

    await expect(visibleButton).toBeVisible();
    await visibleButton.click();

    // Wait for navigation to chat with model pre-selected
    await this.page.waitForURL(
      (url) => url.pathname === '/ui/chat/' && url.searchParams.get('model') === modelName
    );
    await this.waitForSPAReady();
  }

  // Responsive design helpers
  async setMobileViewport() {
    await this.page.setViewportSize({ width: 375, height: 667 });
  }

  async setTabletViewport() {
    await this.page.setViewportSize({ width: 768, height: 1024 });
  }

  async setDesktopViewport() {
    await this.page.setViewportSize({ width: 1920, height: 1080 });
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
    const editBtn = this.page.locator(this.selectors.editButton(alias));
    await expect(editBtn).toBeVisible();
    await editBtn.click();
    await this.waitForUrl('/ui/models/edit/');
    await this.waitForSPAReady();

    // Verify we're on the edit page with correct alias
    const currentUrl = new URL(this.page.url());
    expect(currentUrl.searchParams.get('alias')).toBe(alias);
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

  async getModelByAlias(alias) {
    // Get the row that contains the specified alias
    const rows = this.page.locator(`${this.selectors.table} tbody tr`);
    const rowCount = await rows.count();

    for (let i = 0; i < rowCount; i++) {
      const row = rows.nth(i);
      const aliasText = await row.locator('[data-testid*="alias-cell-"]').textContent();
      if (aliasText?.includes(alias)) {
        return row;
      }
    }

    throw new Error(`Model with alias "${alias}" not found in the list`);
  }

  async verifyModelNotInList(alias) {
    // Verify that a model with the given alias is not in the list
    await this.waitForSelector(this.selectors.table);

    try {
      await this.getModelByAlias(alias);
      throw new Error(`Model with alias "${alias}" should not be in the list`);
    } catch (error) {
      if (error.message.includes('not found in the list')) {
        // This is expected - the model should not be found
        return;
      }
      throw error;
    }
  }
}
