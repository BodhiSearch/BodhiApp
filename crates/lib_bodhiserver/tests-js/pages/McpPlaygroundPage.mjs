import { expect } from '@playwright/test';

import { BasePage } from '@/pages/BasePage.mjs';

/**
 * Page object for the Screen-V2 MCP Playground (`/ui/mcps/playground/?id=...`).
 *
 * Capability layout:
 *   sidebar (left)  -- instance picker + capability nav (overview/tools/prompts/resources/templates)
 *   rail    (right) -- list of items for the active non-overview capability
 *   center          -- overview dashboard OR detail view for the selected item
 */
export class McpPlaygroundPage extends BasePage {
  static MCP_CONNECTION_TIMEOUT = 30000;

  selectors = {
    page: '[data-testid="mcp-playground-page"]',
    loading: '[data-testid="mcp-playground-loading"]',

    sidebar: '[data-testid="mcp-playground-sidebar"]',
    instancePicker: '[data-testid="mcp-playground-instance-picker"]',
    instanceTrigger: '[data-testid="mcp-playground-instance-trigger"]',
    instanceOption: (id) => `[data-testid="mcp-playground-instance-option-${id}"]`,

    capability: (feature) => `[data-testid="mcp-playground-capability-${feature}"]`,
    capabilityCount: (feature) => `[data-testid="mcp-playground-capability-count-${feature}"]`,

    rail: '[data-testid="mcp-playground-rail"]',
    railList: '[data-testid="mcp-playground-rail-list"]',
    railItem: (id) => `[data-testid="mcp-playground-rail-item-${id}"]`,

    connectionStatus: '[data-testid="mcp-playground-connection-status"]',
    refreshButton: '[data-testid="mcp-playground-refresh-button"]',

    overview: '[data-testid="mcp-playground-overview"]',
    overviewCard: (feature) => `[data-testid="mcp-playground-overview-card-${feature}"]`,

    toolDetail: '[data-testid="mcp-playground-tool-detail"]',
    toolName: '[data-testid="mcp-playground-tool-name"]',
    param: (name) => `[data-testid="mcp-playground-param-${name}"]`,
    runButton: '[data-testid="mcp-playground-run-button"]',
    resetButton: '[data-testid="mcp-playground-reset-button"]',

    promptDetail: '[data-testid="mcp-playground-prompt-detail"]',
    promptName: '[data-testid="mcp-playground-prompt-name"]',
    promptPreviewButton: '[data-testid="mcp-playground-prompt-preview-button"]',
    promptMessage: (i) => `[data-testid="mcp-playground-prompt-msg-${i}"]`,

    resultStatus: '[data-testid="mcp-playground-result-status"]',
    resultTab: (tab) => `[data-testid="mcp-playground-result-tab-${tab}"]`,
    resultRaw: '[data-testid="mcp-playground-result-raw"]',
    resultRequest: '[data-testid="mcp-playground-result-request"]',
    resultError: '[data-testid="mcp-playground-result-error"]',
    copyButton: '[data-testid="mcp-playground-copy-button"]',

    backCrumb: 'a.shell-bc-seg[href="/ui/mcps/"]',
  };

  async waitForLoaded() {
    await expect(this.page.locator(this.selectors.page)).toBeVisible();
    await this.waitForSPAReady();
  }

  async expectConnected(timeout = McpPlaygroundPage.MCP_CONNECTION_TIMEOUT) {
    const status = this.page.locator(this.selectors.connectionStatus);
    await expect(status).toHaveAttribute('data-test-state', 'connected', { timeout });
  }

  async expectConnectionError(timeout = McpPlaygroundPage.MCP_CONNECTION_TIMEOUT) {
    const status = this.page.locator(this.selectors.connectionStatus);
    await expect(status).toHaveAttribute('data-test-state', 'error', { timeout });
  }

  async expectConnecting(timeout = 5000) {
    const status = this.page.locator(this.selectors.connectionStatus);
    await expect(status).toHaveAttribute('data-test-state', 'connecting', { timeout });
  }

  async clickRefresh() {
    await this.page.click(this.selectors.refreshButton);
  }

  async selectCapability(feature) {
    await this.page.click(this.selectors.capability(feature));
  }

  async expectCapabilityCount(feature, count) {
    const text = String(count);
    await expect(this.page.locator(this.selectors.capabilityCount(feature))).toHaveText(text);
  }

  async openOverviewCard(feature) {
    await this.page.click(this.selectors.overviewCard(feature));
  }

  async selectTool(name) {
    // Make sure the rail is mounted on the tools feature.
    const railItem = this.page.locator(this.selectors.railItem(name));
    if (!(await railItem.isVisible().catch(() => false))) {
      await this.selectCapability('tools');
    }
    await this.page.click(this.selectors.railItem(name));
  }

  async expectToolSelected(name) {
    await expect(this.page.locator(this.selectors.toolName)).toContainText(name);
  }

  async fillParam(name, value) {
    const paramContainer = this.page.locator(this.selectors.param(name));
    const input = paramContainer.locator('input, textarea').first();
    await input.fill(value);
  }

  async clickRun() {
    await this.page.click(this.selectors.runButton);
  }

  async clickReset() {
    await this.page.click(this.selectors.resetButton);
  }

  async expectResultSuccess() {
    const status = this.page.locator(this.selectors.resultStatus);
    await expect(status).toBeVisible();
    await expect(status).toHaveAttribute('data-test-state', 'success');
  }

  async expectResultError() {
    const status = this.page.locator(this.selectors.resultStatus);
    await expect(status).toBeVisible();
    await expect(status).toHaveAttribute('data-test-state', 'error');
  }

  async clickResultTab(tab) {
    await this.page.click(this.selectors.resultTab(tab));
  }

  async getResultRaw() {
    return await this.page.locator(this.selectors.resultRaw).textContent();
  }

  async getResultRequest() {
    return await this.page.locator(this.selectors.resultRequest).textContent();
  }

  async expectCopyButtonVisible() {
    await expect(this.page.locator(this.selectors.copyButton)).toBeVisible();
  }

  async selectPrompt(name) {
    const railItem = this.page.locator(this.selectors.railItem(name));
    if (!(await railItem.isVisible().catch(() => false))) {
      await this.selectCapability('prompts');
    }
    await this.page.click(this.selectors.railItem(name));
  }

  async expectPromptSelected(name) {
    await expect(this.page.locator(this.selectors.promptName)).toContainText(name);
  }

  async clickPromptPreview() {
    await this.page.click(this.selectors.promptPreviewButton);
  }

  async expectPromptMessageVisible(index) {
    await expect(this.page.locator(this.selectors.promptMessage(index))).toBeVisible();
  }

  async goBackToList() {
    await this.page.click(this.selectors.backCrumb);
    await this.page.waitForURL(/\/ui\/mcps(?!\/playground)/);
    await this.waitForSPAReady();
  }
}
