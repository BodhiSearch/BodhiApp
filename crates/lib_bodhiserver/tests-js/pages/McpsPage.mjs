import { McpFixtures } from '@/fixtures/mcpFixtures.mjs';
import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

/**
 * Page object for MCP management on /ui/mcps and /ui/mcps/servers pages
 */
export class McpsPage extends BasePage {
  static MCP_CONNECTION_TIMEOUT = McpFixtures.MCP_CONNECTION_TIMEOUT;

  selectors = {
    // V2 My MCPs (server-centric list + detail rail). The old My-MCPs/My-MCP-Servers tab pages and
    // their flat instance/server rows were removed; servers list here, instances live in the rail.
    myMcpsContent: '[data-testid="my-mcps-content"]',
    myMcpsList: '[data-testid="my-mcps-list"]',
    serverRowV2: (id) => `[data-testid="my-mcps-row-${id}"]`,
    railConfigureServer: '[data-testid="my-mcps-configure-server"]',
    railInstance: (id) => `[data-testid="my-mcps-instance-${id}"]`,
    railInstancePlay: (id) => `[data-testid="my-mcps-instance-play-${id}"]`,
    railInstanceEdit: (id) => `[data-testid="my-mcps-instance-edit-${id}"]`,
    railInstanceDelete: (id) => `[data-testid="my-mcps-instance-delete-${id}"]`,
    railDeleteDialog: '[data-testid="my-mcps-delete-dialog"]',

    // MCP Server new/edit page (forms unchanged)

    // Server view page - auth config inline form
    addAuthConfigButton: '[data-testid="add-auth-config-button"]',
    authConfigForm: '[data-testid="auth-config-form"]',
    authConfigTypeSelect: '[data-testid="auth-config-type-select"]',
    authConfigNameInput: '[data-testid="auth-config-name-input"]',
    oauthRegistrationTypeSelect: '[data-testid="oauth-registration-type-select"]',
    authConfigAuthEndpointInput: '[data-testid="auth-config-auth-endpoint-input"]',
    authConfigTokenEndpointInput: '[data-testid="auth-config-token-endpoint-input"]',
    authConfigRegistrationEndpointInput: '[data-testid="auth-config-registration-endpoint-input"]',
    authConfigScopesInput: '[data-testid="auth-config-scopes-input"]',
    authConfigClientIdInput: '[data-testid="auth-config-client-id-input"]',
    authConfigClientSecretInput: '[data-testid="auth-config-client-secret-input"]',
    authConfigSaveButton: '[data-testid="auth-config-save-button"]',
    authConfigCancelButton: '[data-testid="auth-config-cancel-button"]',
    authConfigDiscoverStatus: '[data-testid="auth-config-discover-status"]',

    // MCP Server new/edit page
    newServerPage: '[data-testid="new-mcp-server-page"]',
    editServerPage: '[data-testid="edit-mcp-server-page"]',
    serverUrlInput: '[data-testid="mcp-server-url-input"]',
    serverNameInput: '[data-testid="mcp-server-name-input"]',
    serverDescriptionInput: '[data-testid="mcp-server-description-input"]',
    serverEnabledSwitch: '[data-testid="mcp-server-enabled-switch"]',
    serverSaveButton: '[data-testid="mcp-server-save-button"]',

    // V2 My MCPs list page (server-centric)
    pageContainer: '[data-testid="my-mcps-content"]',

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
    authConfigHeaderCredentials: '[data-testid="auth-config-header-credentials"]',
    credentialField: (paramKey) => `[data-testid="credential-field-${paramKey}"]`,
    credentialInput: (paramKey) => `[data-testid="credential-input-${paramKey}"]`,
    credentialToggle: (paramKey) => `[data-testid="credential-toggle-${paramKey}"]`,
    authConfigOAuthConnect: '[data-testid="auth-config-oauth-connect"]',
    authConfigNewRedirect: '[data-testid="auth-config-new-redirect"]',

    // OAuth connected state (unchanged)
    oauthConnectedCard: '[data-testid="oauth-connected-card"]',
    oauthConnectedBadge: '[data-testid="oauth-connected-badge"]',
    oauthDisconnectButton: '[data-testid="oauth-disconnect-button"]',
    oauthConnectedInfo: '[data-testid="oauth-connected-info"]',

    // Playground page
    playgroundPage: '[data-testid="mcp-playground-page"]',
    playgroundLoading: '[data-testid="mcp-playground-loading"]',
    playgroundToolSidebar: '[data-testid="mcp-playground-tool-sidebar"]',
    playgroundToolList: '[data-testid="mcp-playground-tool-list"]',
    playgroundTool: (name) => `[data-testid="mcp-playground-tool-${name}"]`,
    playgroundRefreshButton: '[data-testid="mcp-playground-refresh-button"]',
    playgroundToolName: '[data-testid="mcp-playground-tool-name"]',
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
    // V2: the playground header bar was replaced by the shell breadcrumb; "back" is the MCP crumb.
    playgroundBackButton: 'a.shell-bc-seg[href="/ui/mcps/"]',
  };

  // ========== MCP Servers Methods (V2: registered via the form; surfaced in the My MCPs list) ==========

  async clickNewServer() {
    await this.navigate('/ui/mcps/servers/new/');
    await this.page.waitForURL(/\/ui\/mcps\/servers\/new/);
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
    // V2: after save the server form returns to the My MCPs list (not a separate servers list).
    await this.page.click(this.selectors.serverSaveButton);
    await this.page.waitForURL(/\/ui\/mcps\/?(\?|$)/);
    await this.waitForSPAReady();
  }

  async createMcpServer(url, name, description = '') {
    await this.clickNewServer();
    await this.expectNewServerPage();
    await this.fillServerName(name);
    await this.fillServerUrl(url);
    if (description) {
      await this.fillServerDescription(description);
    }
    await this.clickServerSave();
    await this.expectMcpsListPage();
  }

  async getServerUuidByName(name) {
    const row = this.page.locator(`[data-test-server-name="${name}"]`).first();
    await expect(row).toBeVisible();
    return await row.getAttribute('data-test-uuid');
  }

  // Open a server's detail rail by name (My Instances + Connect with live here in V2).
  async openServerRail(name) {
    await this.page.locator(`[data-test-server-name="${name}"]`).first().locator('.cat-name').first().click();
    await this.page.waitForSelector('[data-testid^="my-mcps-detail-"]');
  }

  // ========== Server View / Configure Page (admin auth-config management) ==========

  async openConfigureServer(name) {
    await this.openServerRail(name);
    await this.page.click(this.selectors.railConfigureServer);
    await this.page.waitForURL(/\/ui\/mcps\/servers\/view/);
    await this.waitForSPAReady();
  }

  async clickViewServerById(id) {
    await this.navigate(`/ui/mcps/servers/view/?id=${id}`);
    await this.page.waitForURL(/\/ui\/mcps\/servers\/view/);
    await this.waitForSPAReady();
  }

  async clickAddAuthConfig() {
    await this.page.click(this.selectors.addAuthConfigButton);
    await expect(this.page.locator(this.selectors.authConfigForm)).toBeVisible();
  }

  async selectInlineAuthConfigType(type) {
    await this.page.click(this.selectors.authConfigTypeSelect);
    const optionText = type === 'oauth' ? 'OAuth' : 'Header / Query Params';
    await this.page.getByRole('option', { name: optionText }).click();
  }

  async waitForDiscoveryComplete() {
    await expect(this.page.locator(this.selectors.authConfigDiscoverStatus)).not.toBeVisible();
  }

  async expectRegistrationType(type) {
    const selector = this.selectors.oauthRegistrationTypeSelect;
    const expectedText =
      type === 'dynamic_registration' ? 'Dynamic Registration' : 'Pre-Registered';
    await expect(this.page.locator(selector)).toContainText(expectedText);
  }

  async clickInlineAuthConfigSave() {
    await this.page.click(this.selectors.authConfigSaveButton);
  }

  async expectAuthConfigRow(configName) {
    await expect(this.page.locator(`text=${configName}`)).toBeVisible();
  }

  // ========== API Helpers (auth config creation via fetch) ==========

  async createAuthConfigViaApi(serverId, { name, entries }) {
    // entries is array of { param_type: 'header' | 'query', param_key: string }
    return await this.page.evaluate(
      async ({ baseUrl, serverId, name, entries }) => {
        const resp = await fetch(`${baseUrl}/bodhi/v1/mcps/auth-configs`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          credentials: 'include',
          body: JSON.stringify({
            type: 'header',
            name: name || 'Header / Query Params',
            mcp_server_id: serverId,
            entries: entries,
          }),
        });
        if (!resp.ok) throw new Error(`HTTP ${resp.status}: ${await resp.text()}`);
        return await resp.json();
      },
      { baseUrl: this.baseUrl, serverId, name, entries }
    );
  }

  async createAuthHeaderViaApi(serverId, { name, headerKey, headerValue }) {
    return await this.createAuthConfigViaApi(serverId, {
      name,
      entries: [{ param_type: 'header', param_key: headerKey }],
    });
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
    // V2: there is no per-list "New" button; New Instance is the shell nav sub-page.
    await this.navigate('/ui/mcps/new/');
    await this.page.waitForURL(/\/ui\/mcps\/new/);
    await this.waitForSPAReady();
  }

  // V2: instances live in a server's detail rail, not a flat list. `ensureInstanceVisible` opens the
  // owning server's rail (scanning server rows if no rail is open) so name-keyed lookups keep working
  // for specs that don't track which server an instance belongs to.
  async ensureInstanceVisible(instanceName) {
    const inst = this.page.locator(`[data-test-instance-name="${instanceName}"]`).first();
    if (await inst.isVisible().catch(() => false)) return inst;
    const serverRows = this.page.locator('[data-test-server-name]');
    await expect(serverRows.first()).toBeVisible();
    const count = await serverRows.count();
    for (let i = 0; i < count; i++) {
      // Click the SERVER name cell (not the leftmost LinkRow target) so the row's onSelect fires.
      await serverRows.nth(i).locator('.cat-name').first().click();
      await this.page.waitForSelector('[data-testid^="my-mcps-detail-"]');
      if (await inst.isVisible({ timeout: 2000 }).catch(() => false)) return inst;
    }
    await expect(inst).toBeVisible();
    return inst;
  }

  async getMcpRowByName(name) {
    return await this.ensureInstanceVisible(name);
  }

  async getMcpUuidByName(name) {
    const row = await this.ensureInstanceVisible(name);
    return await row.getAttribute('data-test-uuid');
  }

  // Open whichever server rail contains the instance row for `id` (no-op if already visible).
  async ensureInstanceRowVisible(id) {
    const row = this.page.locator(this.selectors.railInstance(id));
    if (await row.isVisible().catch(() => false)) return;
    const serverRows = this.page.locator('[data-test-server-name]');
    await expect(serverRows.first()).toBeVisible();
    const count = await serverRows.count();
    for (let i = 0; i < count; i++) {
      await serverRows.nth(i).locator('.cat-name').first().click();
      await this.page.waitForSelector('[data-testid^="my-mcps-detail-"]');
      if (await row.isVisible({ timeout: 2000 }).catch(() => false)) return;
    }
    await expect(row).toBeVisible();
  }

  async clickEditById(id) {
    await this.ensureInstanceRowVisible(id);
    await this.page.click(this.selectors.railInstanceEdit(id));
    await this.page.waitForURL(/\/ui\/mcps\/new\/?\?id=/);
    await this.waitForSPAReady();
  }

  async clickDeleteById(id) {
    await this.ensureInstanceRowVisible(id);
    await this.page.click(this.selectors.railInstanceDelete(id));
  }

  async confirmDelete() {
    await expect(this.page.locator(this.selectors.railDeleteDialog)).toBeVisible();
    await this.page.locator(this.selectors.railDeleteDialog).getByRole('button', { name: /^Delete$/ }).click();
    await expect(this.page.locator(this.selectors.railDeleteDialog)).not.toBeVisible();
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

  async toggleEnabled() {
    await this.page.locator(this.selectors.enabledSwitch).click();
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

    await this.clickCreate();
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

  async expectAuthConfigHeaderCredentials() {
    await expect(this.page.locator(this.selectors.authConfigHeaderCredentials)).toBeVisible();
  }

  async expectCredentialField(paramKey) {
    await expect(this.page.locator(this.selectors.credentialField(paramKey))).toBeVisible();
  }

  async fillCredentialValue(paramKey, value) {
    await this.page.fill(this.selectors.credentialInput(paramKey), value);
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
    credentials = [],
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
    await this.expectAuthConfigHeaderCredentials();

    for (const cred of credentials) {
      await this.fillCredentialValue(cred.param_key, cred.value);
    }

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
    await this.clickCreate();
  }

  // ========== Playground Connection Status Methods ==========

  async expectPlaygroundConnected(timeout = McpsPage.MCP_CONNECTION_TIMEOUT) {
    const status = this.page.locator('[data-testid="mcp-playground-connection-status"]');
    await expect(status).toHaveText('connected', { timeout });
  }

  async expectPlaygroundConnectionError(timeout = McpsPage.MCP_CONNECTION_TIMEOUT) {
    const status = this.page.locator('[data-testid="mcp-playground-connection-status"]');
    await expect(status).toHaveText('error', { timeout });
  }

  async expectPlaygroundConnecting(timeout = 5000) {
    const status = this.page.locator('[data-testid="mcp-playground-connection-status"]');
    await expect(status).toHaveText('connecting', { timeout });
  }

  // ========== Playground Page Methods ==========

  async clickPlaygroundById(id) {
    // V2: the play action is on the instance row inside the server rail; open it if needed.
    await this.ensureInstanceRowVisible(id);
    await this.page.click(this.selectors.railInstancePlay(id));
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
