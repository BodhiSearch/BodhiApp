import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

/**
 * Page object for MCP server management on /ui/mcps pages
 */
export class McpsPage extends BasePage {
  selectors = {
    // List page
    pageContainer: '[data-testid="mcps-page"]',
    pageLoading: '[data-testid="mcps-page-loading"]',
    tableContainer: '[data-testid="mcps-table-container"]',
    newButton: '[data-testid="mcp-new-button"]',
    mcpRow: (id) => `[data-testid="mcp-row-${id}"]`,
    mcpRowByName: (name) => `[data-test-mcp-name="${name}"]`,
    mcpStatus: (id) => `[data-testid="mcp-status-${id}"]`,
    mcpEditButton: (id) => `[data-testid="mcp-edit-button-${id}"]`,
    mcpDeleteButton: (id) => `[data-testid="mcp-delete-button-${id}"]`,
    emptyState: 'text=No MCP servers configured',

    // New/Edit page
    newPageContainer: '[data-testid="new-mcp-page"]',
    urlInput: '[data-testid="mcp-url-input"]',
    checkUrlButton: '[data-testid="mcp-check-url-button"]',
    urlEnabled: '[data-testid="mcp-url-enabled"]',
    urlNotEnabled: '[data-testid="mcp-url-not-enabled"]',
    enableServerButton: '[data-testid="mcp-enable-server-button"]',
    confirmEnableButton: '[data-testid="mcp-confirm-enable-button"]',
    nameInput: '[data-testid="mcp-name-input"]',
    slugInput: '[data-testid="mcp-slug-input"]',
    descriptionInput: '[data-testid="mcp-description-input"]',
    enabledSwitch: '[data-testid="mcp-enabled-switch"]',
    createButton: '[data-testid="mcp-create-button"]',
    updateButton: '[data-testid="mcp-update-button"]',
    cancelButton: '[data-testid="mcp-cancel-button"]',
    doneButton: '[data-testid="mcp-done-button"]',
    backButton: '[data-testid="mcp-back-button"]',

    // Tools section
    toolsSection: '[data-testid="mcp-tools-section"]',
    fetchToolsButton: '[data-testid="mcp-fetch-tools-button"]',
    toolsLoading: '[data-testid="mcp-tools-loading"]',
    toolsList: '[data-testid="mcp-tools-list"]',
    toolItem: (name) => `[data-testid="mcp-tool-${name}"]`,
    toolCheckbox: (name) => `[data-testid="mcp-tool-checkbox-${name}"]`,
    selectAllButton: '[data-testid="mcp-select-all-tools"]',
    deselectAllButton: '[data-testid="mcp-deselect-all-tools"]',
    noTools: '[data-testid="mcp-no-tools"]',

    // Playground page
    mcpPlaygroundButton: (id) => `[data-testid="mcp-playground-button-${id}"]`,
    playgroundPage: '[data-testid="mcp-playground-page"]',
    playgroundLoading: '[data-testid="mcp-playground-loading"]',
    playgroundToolSidebar: '[data-testid="mcp-playground-tool-sidebar"]',
    playgroundToolList: '[data-testid="mcp-playground-tool-list"]',
    playgroundTool: (name) => `[data-testid="mcp-playground-tool-${name}"]`,
    playgroundRefreshButton: '[data-testid="mcp-playground-refresh-button"]',
    playgroundToolName: '[data-testid="mcp-playground-tool-name"]',
    playgroundNotWhitelistedWarning: '[data-testid="mcp-playground-not-whitelisted-warning"]',
    playgroundInputModeForm: '[data-testid="mcp-playground-input-mode-form"]',
    playgroundInputModeJson: '[data-testid="mcp-playground-input-mode-json"]',
    playgroundParam: (name) => `[data-testid="mcp-playground-param-${name}"]`,
    playgroundJsonEditor: '[data-testid="mcp-playground-json-editor"]',
    playgroundExecuteButton: '[data-testid="mcp-playground-execute-button"]',
    playgroundResultSection: '[data-testid="mcp-playground-result-section"]',
    playgroundResultStatus: '[data-testid="mcp-playground-result-status"]',
    playgroundResultTabResponse: '[data-testid="mcp-playground-result-tab-response"]',
    playgroundResultTabRaw: '[data-testid="mcp-playground-result-tab-raw"]',
    playgroundResultTabRequest: '[data-testid="mcp-playground-result-tab-request"]',
    playgroundResultContent: '[data-testid="mcp-playground-result-content"]',
    playgroundCopyButton: '[data-testid="mcp-playground-copy-button"]',
    playgroundBackButton: '[data-testid="mcp-playground-back-button"]',
  };

  // ========== List Page Methods ==========

  async navigateToMcpsList() {
    await this.navigate('/ui/mcps/');
    await this.waitForSPAReady();
  }

  async expectMcpsListPage() {
    await expect(this.page.locator(this.selectors.pageContainer)).toBeVisible({
      timeout: 15000,
    });
  }

  async expectEmptyState() {
    await expect(this.page.locator(this.selectors.emptyState)).toBeVisible();
  }

  async clickNewMcp() {
    await this.page.click(this.selectors.newButton);
    await this.page.waitForURL(/\/ui\/mcps\/new/);
    await this.waitForSPAReady();
  }

  async getMcpRowByName(name) {
    return this.page.locator(this.selectors.mcpRowByName(name)).first();
  }

  async getMcpUuidByName(name) {
    const row = this.page.locator(this.selectors.mcpRowByName(name)).first();
    return await row.getAttribute('data-test-uuid');
  }

  async clickEditById(id) {
    await this.page.click(this.selectors.mcpEditButton(id));
    await this.page.waitForURL(/\/ui\/mcps\/new\/?\?id=/);
    await this.waitForSPAReady();
  }

  async clickDeleteById(id) {
    await this.page.click(this.selectors.mcpDeleteButton(id));
  }

  async confirmDelete() {
    await this.page.click('button:has-text("Delete")');
  }

  async expectMcpStatus(id, statusText) {
    const statusCell = this.page.locator(this.selectors.mcpStatus(id));
    await expect(statusCell).toContainText(statusText);
  }

  // ========== New/Edit Page Methods ==========

  async navigateToNewMcp() {
    await this.navigate('/ui/mcps/new');
    await this.waitForSPAReady();
  }

  async expectNewMcpPage() {
    await expect(this.page.locator(this.selectors.newPageContainer)).toBeVisible();
  }

  async fillUrl(url) {
    await this.page.fill(this.selectors.urlInput, url);
  }

  async checkUrl() {
    await this.page.click(this.selectors.checkUrlButton);
  }

  async expectUrlEnabled() {
    await expect(this.page.locator(this.selectors.urlEnabled)).toBeVisible({
      timeout: 15000,
    });
  }

  async expectUrlNotEnabled() {
    await expect(this.page.locator(this.selectors.urlNotEnabled)).toBeVisible({
      timeout: 15000,
    });
  }

  async clickEnableServer() {
    await this.page.click(this.selectors.enableServerButton);
  }

  async confirmEnableServer() {
    await this.page.click(this.selectors.confirmEnableButton);
  }

  async fillName(name) {
    await this.page.fill(this.selectors.nameInput, name);
  }

  async fillSlug(slug) {
    await this.page.fill(this.selectors.slugInput, slug);
  }

  async fillDescription(description) {
    await this.page.fill(this.selectors.descriptionInput, description);
  }

  async clickCreate() {
    await this.page.click(this.selectors.createButton);
  }

  async clickUpdate() {
    await this.page.click(this.selectors.updateButton);
    await this.page.waitForURL(/\/ui\/mcps(?!\/new)/);
    await this.waitForSPAReady();
  }

  async clickDone() {
    await this.page.click(this.selectors.doneButton);
  }

  async clickBackToList() {
    await this.page.click(this.selectors.backButton);
  }

  // ========== Tools Section Methods ==========

  async expectToolsSection() {
    await expect(this.page.locator(this.selectors.toolsSection)).toBeVisible();
  }

  async clickFetchTools() {
    await this.page.click(this.selectors.fetchToolsButton);
  }

  async expectToolsLoading() {
    await expect(this.page.locator(this.selectors.toolsLoading)).toBeVisible();
  }

  async expectToolsList() {
    await expect(this.page.locator(this.selectors.toolsList)).toBeVisible({
      timeout: 30000,
    });
  }

  async expectToolItem(toolName) {
    await expect(this.page.locator(this.selectors.toolItem(toolName))).toBeVisible();
  }

  async toggleTool(toolName) {
    await this.page.click(this.selectors.toolCheckbox(toolName));
  }

  async clickSelectAll() {
    await this.page.click(this.selectors.selectAllButton);
  }

  async clickDeselectAll() {
    await this.page.click(this.selectors.deselectAllButton);
  }

  // ========== Complete Workflow Methods ==========

  /**
   * Create a new MCP server with admin enable flow
   */
  async createMcpWithAdminEnable(url, name, slug, description = '') {
    await this.navigateToNewMcp();
    await this.expectNewMcpPage();

    // Enter URL and check
    await this.fillUrl(url);
    await this.checkUrl();

    // If not enabled, enable it (admin only)
    const notEnabled = this.page.locator(this.selectors.urlNotEnabled);
    const enabled = this.page.locator(this.selectors.urlEnabled);

    await expect(notEnabled.or(enabled)).toBeVisible({ timeout: 15000 });

    if (await notEnabled.isVisible()) {
      await this.clickEnableServer();
      await this.confirmEnableServer();
      await this.expectUrlEnabled();
    }

    // Fill form
    await this.fillName(name);
    await this.fillSlug(slug);
    if (description) {
      await this.fillDescription(description);
    }

    // Create
    await this.clickCreate();

    // Wait for tools section to appear (MCP created)
    await this.expectToolsSection();
  }

  // ========== Playground Page Methods ==========

  async clickPlaygroundById(id) {
    await this.page.click(this.selectors.mcpPlaygroundButton(id));
    await this.page.waitForURL(/\/ui\/mcps\/playground\/?\?id=/);
    await this.waitForSPAReady();
  }

  async expectPlaygroundPage() {
    await expect(this.page.locator(this.selectors.playgroundPage)).toBeVisible({
      timeout: 15000,
    });
  }

  async selectPlaygroundTool(name) {
    await this.page.click(this.selectors.playgroundTool(name));
  }

  async expectPlaygroundToolSelected(name) {
    const toolName = this.page.locator(this.selectors.playgroundToolName);
    await expect(toolName).toContainText(name);
  }

  async expectNotWhitelistedWarning() {
    await expect(this.page.locator(this.selectors.playgroundNotWhitelistedWarning)).toBeVisible();
  }

  async expectNoWhitelistedWarning() {
    await expect(
      this.page.locator(this.selectors.playgroundNotWhitelistedWarning)
    ).not.toBeVisible();
  }

  async clickPlaygroundRefresh() {
    await this.page.click(this.selectors.playgroundRefreshButton);
  }

  async fillPlaygroundParam(name, value) {
    const paramContainer = this.page.locator(this.selectors.playgroundParam(name));
    const input = paramContainer.locator('input, textarea').first();
    await input.fill(value);
  }

  async switchToJsonMode() {
    await this.page.click(this.selectors.playgroundInputModeJson);
  }

  async switchToFormMode() {
    await this.page.click(this.selectors.playgroundInputModeForm);
  }

  async fillPlaygroundJson(json) {
    await this.page.fill(this.selectors.playgroundJsonEditor, json);
  }

  async getPlaygroundJsonContent() {
    return await this.page.locator(this.selectors.playgroundJsonEditor).inputValue();
  }

  async clickPlaygroundExecute() {
    await this.page.click(this.selectors.playgroundExecuteButton);
  }

  async expectPlaygroundResultSuccess() {
    const status = this.page.locator(this.selectors.playgroundResultStatus);
    await expect(status).toBeVisible({ timeout: 30000 });
    await expect(status).toHaveAttribute('data-test-state', 'success');
  }

  async expectPlaygroundResultError() {
    const status = this.page.locator(this.selectors.playgroundResultStatus);
    await expect(status).toBeVisible({ timeout: 30000 });
    await expect(status).toHaveAttribute('data-test-state', 'error');
  }

  async clickPlaygroundResultTab(tab) {
    const selector = {
      response: this.selectors.playgroundResultTabResponse,
      raw: this.selectors.playgroundResultTabRaw,
      request: this.selectors.playgroundResultTabRequest,
    }[tab];
    await this.page.click(selector);
  }

  async getPlaygroundResultContent() {
    return await this.page.locator(this.selectors.playgroundResultContent).textContent();
  }

  async clickPlaygroundCopy() {
    await this.page.click(this.selectors.playgroundCopyButton);
  }

  async clickPlaygroundBack() {
    await this.page.click(this.selectors.playgroundBackButton);
    await this.page.waitForURL(/\/ui\/mcps(?!\/playground)/);
    await this.waitForSPAReady();
  }
}
