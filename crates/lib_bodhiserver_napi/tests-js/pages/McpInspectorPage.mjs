import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

/**
 * Page object for the MCP Inspector UI (third-party).
 *
 * The Inspector is not part of BodhiApp — it is a standalone tool at
 * http://localhost:6274.  Selectors here are role-based and text-based
 * because the Inspector does not expose data-testid attributes.
 */
export class McpInspectorPage extends BasePage {
  constructor(page, baseUrl) {
    super(page, baseUrl);
  }

  // ========== Navigation ==========

  async navigateToInspector(inspectorUrl) {
    await this.page.goto(inspectorUrl);
    await this.page.waitForLoadState('networkidle');
  }

  // ========== Connection Setup ==========

  async selectTransportType(type) {
    await this.page.getByRole('combobox', { name: 'Transport Type' }).click();
    await this.page.getByRole('option', { name: type }).click();
  }

  async setServerUrl(url) {
    await this.page.getByRole('textbox', { name: 'URL' }).fill(url);
  }

  async selectConnectionType(type) {
    await this.page.getByRole('combobox', { name: 'Connection Type' }).click();
    await this.page.getByRole('option', { name: type }).click();
  }

  async setAuthToken(token) {
    await this.page.getByRole('button', { name: 'Authentication' }).click();
    await this.page.getByRole('textbox', { name: 'Header Value' }).fill(`Bearer ${token}`);

    // Ensure the header toggle switch is enabled
    const headerSwitch = this.page.getByRole('switch').first();
    const switchState = await headerSwitch.getAttribute('data-state');
    if (switchState !== 'checked') {
      await headerSwitch.click();
    }
  }

  async clickConnect() {
    await this.page.getByRole('button', { name: 'Connect' }).click();
  }

  async clickDisconnect() {
    await this.page.getByRole('button', { name: 'Disconnect' }).click();
  }

  // ========== Connection Status Assertions ==========

  async expectConnected(timeout = 30000) {
    await expect(this.page.getByText('Connected')).toBeVisible({ timeout });
  }

  async expectDisconnected(timeout = 5000) {
    await expect(this.page.getByText('Disconnected')).toBeVisible({ timeout });
  }

  async expectConnectionError(timeout = 15000) {
    // The Inspector may show an explicit error or simply never reach "Connected".
    // We assert that "Connected" does NOT appear within the timeout.
    await expect(this.page.getByText('Connected')).not.toBeVisible({ timeout });
  }

  // ========== History Assertions ==========

  async expectHistoryEntry(text) {
    await expect(this.page.getByText(text)).toBeVisible();
  }

  // ========== Tool Operations ==========

  async switchToToolsTab() {
    await this.page.getByRole('tab', { name: 'Tools' }).click();
    await this.page.waitForURL(/#tools/);
  }

  async listTools() {
    await this.page.getByRole('button', { name: 'List Tools' }).click();
  }

  async selectTool(name) {
    await this.page.getByText(name).first().click();
  }

  async fillToolTextInput(value) {
    await this.page.locator('textarea').first().fill(value);
  }

  async fillToolNumberInputs(values) {
    const numberInputs = this.page.locator('input[type="number"]');
    for (let i = 0; i < values.length; i++) {
      await numberInputs.nth(i).fill(String(values[i]));
    }
  }

  async executeSelectedTool() {
    await this.page.getByRole('button', { name: 'Run Tool' }).click();
  }

  async expectToolResult(text, timeout = 10000) {
    await expect(this.page.getByText('Success')).toBeVisible({ timeout });
    await expect(this.page.getByText(text)).toBeVisible();
  }

  async expectToolResultImage(timeout = 10000) {
    await expect(this.page.getByText('Success')).toBeVisible({ timeout });
    await expect(
      this.page.locator('img[src^="data:image"]').or(this.page.getByText('image/'))
    ).toBeVisible();
  }

  // ========== Resource Operations ==========

  async switchToResourcesTab() {
    await this.page.getByRole('tab', { name: 'Resources' }).click();
    await this.page.waitForURL(/#resources/);
  }

  async listResources() {
    await this.page.getByRole('button', { name: 'List Resources' }).click();
  }

  async listTemplates() {
    await this.page.getByRole('button', { name: 'List Templates' }).click();
  }

  async selectResource(name) {
    await this.page.getByText(name).first().click();
  }

  async readSelectedResource() {
    await this.page.getByRole('button', { name: 'Read Resource' }).click();
  }

  async expectResourceContent(text, timeout = 10000) {
    await expect(this.page.getByText(text).first()).toBeVisible({ timeout });
  }

  // ========== Prompt Operations ==========

  async switchToPromptsTab() {
    await this.page.getByRole('tab', { name: 'Prompts' }).click();
    await this.page.waitForURL(/#prompts/);
  }

  async listPrompts() {
    await this.page.getByRole('button', { name: 'List Prompts' }).click();
  }

  async selectPrompt(name) {
    await this.page.getByText(name).first().click();
  }

  async getSelectedPrompt() {
    await this.page.getByRole('button', { name: 'Get Prompt' }).click();
  }

  async expectPromptContent(text, timeout = 10000) {
    await expect(this.page.getByText(text)).toBeVisible({ timeout });
  }

  async fillPromptCombobox(placeholder, value) {
    const combobox = this.page.getByRole('combobox').filter({ hasText: placeholder });
    await combobox.click();
    await this.page.keyboard.type(value);
  }

  // ========== Ping ==========

  async switchToPingTab() {
    await this.page.getByRole('tab', { name: 'Ping' }).click();
    await this.page.waitForURL(/#ping/);
  }

  async executePing() {
    await this.page.getByRole('button', { name: 'Ping Server' }).click();
  }

  async expectPingSuccess(timeout = 5000) {
    await expect(this.page.getByText('ping').last()).toBeVisible({ timeout });
  }

  // ========== Composite Helpers ==========

  /**
   * Full setup: select transport, set URL, select Direct connection, set auth
   * token, and install the Accept-header route workaround for Playwright.
   */
  async configureDirectConnection({ inspectorUrl, serverUrl, mcpId, accessToken }) {
    // Workaround for Playwright bug #20439/#29521: extraHTTPHeaders overrides
    // the Accept header set by in-page fetch() at the CDP level.
    await this.page.route(`${serverUrl}/**/mcp`, async (route) => {
      const headers = { ...route.request().headers() };
      if (!headers['accept']?.includes('text/event-stream')) {
        headers['accept'] = 'text/event-stream, application/json';
      }
      await route.continue({ headers });
    });

    await this.navigateToInspector(inspectorUrl);
    await this.selectTransportType('Streamable HTTP');
    await this.setServerUrl(`${serverUrl}/bodhi/v1/apps/mcps/${mcpId}/mcp`);
    await this.selectConnectionType('Direct');
    await this.setAuthToken(accessToken);
  }
}
