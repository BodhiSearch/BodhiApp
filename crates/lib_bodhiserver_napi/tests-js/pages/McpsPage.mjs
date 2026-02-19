import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

/**
 * Page object for MCP management on /ui/mcps and /ui/mcp-servers pages
 */
export class McpsPage extends BasePage {
  selectors = {
    // Management tabs
    managementTabs: '[data-testid="mcp-management-tabs"]',
    tabMcps: '[data-testid="mcp-tab-mcps"]',
    tabServers: '[data-testid="mcp-tab-mcp-servers"]',

    // MCP Servers list page
    serversPage: '[data-testid="mcp-servers-page"]',
    serverNewButton: '[data-testid="mcp-server-new-button"]',
    serverRow: (id) => `[data-testid="server-row-${id}"]`,
    serverRowByName: (name) => `[data-test-server-name="${name}"]`,
    serverToggle: (id) => `[data-testid="server-toggle-${id}"]`,
    serverEditButton: (id) => `[data-testid="server-edit-button-${id}"]`,
    serverViewButton: (id) => `[data-testid="server-view-button-${id}"]`,

    // MCP Server new/edit page
    newServerPage: '[data-testid="new-mcp-server-page"]',
    editServerPage: '[data-testid="edit-mcp-server-page"]',
    serverUrlInput: '[data-testid="mcp-server-url-input"]',
    serverNameInput: '[data-testid="mcp-server-name-input"]',
    serverDescriptionInput: '[data-testid="mcp-server-description-input"]',
    serverEnabledSwitch: '[data-testid="mcp-server-enabled-switch"]',
    serverSaveButton: '[data-testid="mcp-server-save-button"]',

    // MCPs list page
    pageContainer: '[data-testid="mcps-page"]',
    pageLoading: '[data-testid="mcps-page-loading"]',
    tableContainer: '[data-testid="mcps-table-container"]',
    newButton: '[data-testid="mcp-new-button"]',
    mcpRow: (id) => `[data-testid="mcp-row-${id}"]`,
    mcpRowByName: (name) => `[data-test-mcp-name="${name}"]`,
    mcpStatus: (id) => `[data-testid="mcp-status-${id}"]`,
    mcpEditButton: (id) => `[data-testid="mcp-edit-button-${id}"]`,
    mcpDeleteButton: (id) => `[data-testid="mcp-delete-button-${id}"]`,

    // New/Edit MCP instance page
    newPageContainer: '[data-testid="new-mcp-page"]',
    serverCombobox: '[data-testid="mcp-server-combobox"]',
    serverSearch: '[data-testid="mcp-server-search"]',
    serverOption: (id) => `[data-testid="mcp-server-option-${id}"]`,
    serverAddNew: '[data-testid="mcp-server-add-new"]',
    nameInput: '[data-testid="mcp-name-input"]',
    slugInput: '[data-testid="mcp-slug-input"]',
    descriptionInput: '[data-testid="mcp-description-input"]',
    enabledSwitch: '[data-testid="mcp-enabled-switch"]',
    createButton: '[data-testid="mcp-create-button"]',
    updateButton: '[data-testid="mcp-update-button"]',
    cancelButton: '[data-testid="mcp-cancel-button"]',
    doneButton: '[data-testid="mcp-done-button"]',
    backButton: '[data-testid="mcp-back-button"]',

    // Auth config dropdown (replaces old inline auth forms)
    authSection: '[data-testid="mcp-auth-section"]',
    authConfigSelect: '[data-testid="auth-config-select"]',
    authConfigOptionPublic: '[data-testid="auth-config-option-public"]',
    authConfigOption: (id) => `[data-testid="auth-config-option-${id}"]`,
    authConfigOptionNew: '[data-testid="auth-config-option-new"]',
    authConfigHeaderSummary: '[data-testid="auth-config-header-summary"]',
    authConfigOAuthConnect: '[data-testid="auth-config-oauth-connect"]',
    authConfigNewRedirect: '[data-testid="auth-config-new-redirect"]',

    // OAuth connected state (unchanged)
    oauthConnectedCard: '[data-testid="oauth-connected-card"]',
    oauthConnectedBadge: '[data-testid="oauth-connected-badge"]',
    oauthDisconnectButton: '[data-testid="oauth-disconnect-button"]',
    oauthConnectedInfo: '[data-testid="oauth-connected-info"]',

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

  // ========== MCP Servers Page Methods ==========

  async navigateToServersList() {
    await this.navigate('/ui/mcp-servers/');
    await this.waitForSPAReady();
  }

  async expectServersListPage() {
    await this.page.waitForURL(/\/ui\/mcp-servers/);
    await this.waitForSPAReady();
  }

  async clickNewServer() {
    await this.page.click(this.selectors.serverNewButton);
    await this.page.waitForURL(/\/ui\/mcp-servers\/new/);
    await this.waitForSPAReady();
  }

  async expectNewServerPage() {
    await expect(this.page.locator(this.selectors.newServerPage)).toBeVisible();
  }

  async fillServerUrl(url) {
    await this.page.fill(this.selectors.serverUrlInput, url);
  }

  async fillServerName(name) {
    await this.page.fill(this.selectors.serverNameInput, name);
  }

  async fillServerDescription(description) {
    await this.page.fill(this.selectors.serverDescriptionInput, description);
  }

  async clickServerSave() {
    await this.page.click(this.selectors.serverSaveButton);
    await this.page.waitForURL(/\/ui\/mcp-servers(?!\/new)/);
    await this.waitForSPAReady();
  }

  async createMcpServer(url, name, description = '') {
    await this.navigateToServersList();
    await this.expectServersListPage();
    await this.clickNewServer();
    await this.expectNewServerPage();
    await this.fillServerName(name);
    await this.fillServerUrl(url);
    if (description) {
      await this.fillServerDescription(description);
    }
    await this.clickServerSave();
    await this.expectServersListPage();
  }

  async getServerUuidByName(name) {
    const row = this.page.locator(this.selectors.serverRowByName(name)).first();
    const testId = await row.getAttribute('data-testid');
    return testId?.replace('server-row-', '');
  }

  // ========== API Helpers (auth config creation via fetch) ==========

  async createAuthHeaderViaApi(serverId, { name, headerKey, headerValue }) {
    return await this.page.evaluate(
      async ({ baseUrl, serverId, name, headerKey, headerValue }) => {
        const resp = await fetch(`${baseUrl}/bodhi/v1/mcps/auth-configs`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          credentials: 'include',
          body: JSON.stringify({
            type: 'header',
            name: name || 'Header',
            mcp_server_id: serverId,
            header_key: headerKey,
            header_value: headerValue,
          }),
        });
        if (!resp.ok) throw new Error(`HTTP ${resp.status}: ${await resp.text()}`);
        return await resp.json();
      },
      { baseUrl: this.baseUrl, serverId, name, headerKey, headerValue }
    );
  }

  async createOAuthConfigViaApi(serverId, config) {
    return await this.page.evaluate(
      async ({ baseUrl, serverId, config }) => {
        const resp = await fetch(`${baseUrl}/bodhi/v1/mcps/auth-configs`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          credentials: 'include',
          body: JSON.stringify({ ...config, type: 'oauth', mcp_server_id: serverId }),
        });
        if (!resp.ok) throw new Error(`HTTP ${resp.status}: ${await resp.text()}`);
        return await resp.json();
      },
      { baseUrl: this.baseUrl, serverId, config }
    );
  }

  async discoverMcpEndpointsViaApi(mcpServerUrl) {
    return await this.page.evaluate(
      async ({ baseUrl, mcpServerUrl }) => {
        const resp = await fetch(`${baseUrl}/bodhi/v1/mcps/oauth/discover-mcp`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          credentials: 'include',
          body: JSON.stringify({ mcp_server_url: mcpServerUrl }),
        });
        if (!resp.ok) throw new Error(`HTTP ${resp.status}: ${await resp.text()}`);
        return await resp.json();
      },
      { baseUrl: this.baseUrl, mcpServerUrl }
    );
  }

  async dynamicRegisterViaApi({ registrationEndpoint, redirectUri, scopes }) {
    return await this.page.evaluate(
      async ({ baseUrl, registrationEndpoint, redirectUri, scopes }) => {
        const resp = await fetch(`${baseUrl}/bodhi/v1/mcps/oauth/dynamic-register`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          credentials: 'include',
          body: JSON.stringify({
            registration_endpoint: registrationEndpoint,
            redirect_uri: redirectUri,
            scopes: scopes || undefined,
          }),
        });
        if (!resp.ok) throw new Error(`HTTP ${resp.status}: ${await resp.text()}`);
        return await resp.json();
      },
      { baseUrl: this.baseUrl, registrationEndpoint, redirectUri, scopes }
    );
  }

  // ========== MCPs List Page Methods ==========

  async navigateToMcpsList() {
    await this.navigate('/ui/mcps/');
    await this.waitForSPAReady();
  }

  async expectMcpsListPage() {
    await this.page.waitForURL(/\/ui\/mcps(?:\/)?$/);
    await this.waitForSPAReady();
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

  // ========== New/Edit MCP Instance Methods ==========

  async expectNewMcpPage() {
    await this.page.waitForURL(/\/ui\/mcps\/new/);
    await this.waitForSPAReady();
  }

  async selectServerFromCombobox(serverName) {
    await this.page.click(this.selectors.serverCombobox);
    const searchInput = this.page.locator(this.selectors.serverSearch);
    await expect(searchInput).toBeVisible();
    await searchInput.fill(serverName);
    const option = this.page.locator(`[cmdk-item]`).filter({ hasText: serverName }).first();
    await expect(option).toBeVisible();
    await option.click();
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
    await this.page.waitForURL(/\/ui\/mcps(?!\/new)/);
    await this.waitForSPAReady();
  }

  async clickUpdate() {
    await this.page.click(this.selectors.updateButton);
    await this.page.waitForURL(/\/ui\/mcps(?!\/new)/);
    await this.waitForSPAReady();
  }

  async createMcpInstance(serverName, name, slug, description = '') {
    await this.navigateToMcpsList();
    await this.expectMcpsListPage();
    await this.clickNewMcp();
    await this.expectNewMcpPage();

    await this.selectServerFromCombobox(serverName);

    if (name) await this.fillName(name);
    await this.fillSlug(slug);
    if (description) await this.fillDescription(description);

    await this.clickFetchTools();
    await this.expectToolsList();
    await this.clickCreate();
  }

  async createMcpInstanceWithAllTools(serverName, name, slug, description = '') {
    await this.createMcpInstance(serverName, name, slug, description);
  }

  // ========== Auth Config Dropdown Methods ==========

  async selectAuthConfigPublic() {
    await this.page.click(this.selectors.authConfigSelect);
    await expect(this.page.locator(this.selectors.authConfigOptionPublic)).toBeVisible();
    await this.page.click(this.selectors.authConfigOptionPublic);
  }

  async selectAuthConfigById(configId) {
    await this.page.click(this.selectors.authConfigSelect);
    await expect(this.page.locator(this.selectors.authConfigOption(configId))).toBeVisible();
    await this.page.click(this.selectors.authConfigOption(configId));
  }

  async expectAuthConfigState(type) {
    const trigger = this.page.locator(this.selectors.authConfigSelect);
    await expect(trigger).toHaveAttribute('data-test-state', type);
  }

  async expectAuthConfigHeaderSummary() {
    await expect(this.page.locator(this.selectors.authConfigHeaderSummary)).toBeVisible();
  }

  async clickOAuthConnect() {
    await this.page.click(this.selectors.authConfigOAuthConnect);
  }

  async expectOAuthConnected() {
    await expect(this.page.locator(this.selectors.oauthConnectedBadge)).toBeVisible();
  }

  async expectOAuthDisconnected() {
    await expect(this.page.locator(this.selectors.oauthConnectedCard)).not.toBeVisible();
  }

  async clickDisconnect() {
    await this.page.click(this.selectors.oauthDisconnectButton);
  }

  async expectAuthConfigDropdownHasOption(configId) {
    await this.page.click(this.selectors.authConfigSelect);
    await expect(this.page.locator(this.selectors.authConfigOption(configId))).toBeVisible();
    await this.page.keyboard.press('Escape');
  }

  // ========== Composite Auth Methods ==========

  async createMcpInstanceWithHeaderAuth({
    serverName,
    name,
    slug,
    authConfigId,
    description = '',
  }) {
    await this.navigateToMcpsList();
    await this.expectMcpsListPage();
    await this.clickNewMcp();
    await this.expectNewMcpPage();

    await this.selectServerFromCombobox(serverName);

    if (name) await this.fillName(name);
    await this.fillSlug(slug);
    if (description) await this.fillDescription(description);

    await this.selectAuthConfigById(authConfigId);
    await this.expectAuthConfigHeaderSummary();

    await this.clickFetchTools();
    await this.expectToolsList();
    await this.clickCreate();
  }

  async createMcpInstanceWithOAuth({
    serverName,
    name,
    slug,
    authConfigId,
    approveSelector = '[data-testid="approve-btn"]',
  }) {
    await this.navigateToMcpsList();
    await this.expectMcpsListPage();
    await this.clickNewMcp();
    await this.expectNewMcpPage();

    await this.selectServerFromCombobox(serverName);
    await this.selectAuthConfigById(authConfigId);
    await this.clickOAuthConnect();

    await this.page.waitForURL(/\/authorize/);
    await this.page.click(approveSelector);

    await this.page.waitForURL(/\/ui\/mcps\/new/);
    await this.waitForSPAReady();
    await this.expectOAuthConnected();

    await this.fillName(name);
    await this.fillSlug(slug);
    await this.clickFetchTools();
    await this.expectToolsList();
    await this.clickCreate();
  }

  // ========== Tools Section Methods ==========

  async expectToolsSection() {
    await expect(this.page.locator(this.selectors.toolsSection)).toBeVisible();
  }

  async clickFetchTools() {
    await this.page.click(this.selectors.fetchToolsButton);
  }

  async expectToolsList() {
    await expect(this.page.locator(this.selectors.toolsList)).toBeVisible();
  }

  async expectToolItem(toolName) {
    await expect(this.page.locator(this.selectors.toolItem(toolName))).toBeVisible();
  }

  async toggleTool(toolName) {
    await this.page.click(this.selectors.toolCheckbox(toolName));
  }

  async selectAllTools() {
    await this.page.click(this.selectors.selectAllButton);
  }

  // ========== Playground Page Methods ==========

  async clickPlaygroundById(id) {
    await this.page.click(this.selectors.mcpPlaygroundButton(id));
    await this.page.waitForURL(/\/ui\/mcps\/playground\/?\?id=/);
    await this.waitForSPAReady();
  }

  async expectPlaygroundPage() {
    await expect(this.page.locator(this.selectors.playgroundPage)).toBeVisible();
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
    await expect(status).toBeVisible();
    await expect(status).toHaveAttribute('data-test-state', 'success');
  }

  async expectPlaygroundResultError() {
    const status = this.page.locator(this.selectors.playgroundResultStatus);
    await expect(status).toBeVisible();
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

  async clickPlaygroundBack() {
    await this.page.click(this.selectors.playgroundBackButton);
    await this.page.waitForURL(/\/ui\/mcps(?!\/playground)/);
    await this.waitForSPAReady();
  }
}
