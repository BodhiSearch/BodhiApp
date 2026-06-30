import { BasePage } from '@/pages/BasePage.mjs';

export class AccessRequestReviewPage extends BasePage {
  selectors = {
    reviewPage: '[data-testid="review-access-page"]',
    approveButton: '[data-testid="review-approve-button"]',
    denyButton: '[data-testid="review-deny-button"]',
    approvedRoleSelect: '[data-testid="review-approved-role-select"]',
  };

  // MCP selectors
  mcpToggle(url) {
    return `[data-testid="review-mcp-toggle-${url}"]`;
  }

  mcpSelectTrigger(url) {
    return `[data-testid="review-mcp-select-trigger-${url}"]`;
  }

  mcpInstanceOption(instanceId) {
    return `[data-testid="review-mcp-instance-option-${instanceId}"]`;
  }

  async waitForReviewPage() {
    await this.expectVisible(this.selectors.reviewPage);
  }

  async selectMcpInstance(url, instanceId) {
    await this.page.click(this.mcpSelectTrigger(url));
    await this.page.locator(this.mcpInstanceOption(instanceId)).click();
  }

  async clickApprove() {
    await this.page.click(this.selectors.approveButton);
  }

  async approve() {
    await this.waitForReviewPage();
    await this.clickApprove();
  }

  // --- Model/MCP grant controls (shown when the app requests the matching flags) ---

  async toggleListModels() {
    await this.page.click('[data-testid="review-list-models-toggle"]');
  }

  async toggleListMcps() {
    await this.page.click('[data-testid="review-list-mcps-toggle"]');
  }

  /** Switch the model access picker to Specific. With no items selected this grants
   *  no models — a deterministic "no model access" grant. Switching to Specific
   *  auto-opens the slide-in picker panel, so close it before continuing. */
  async setModelAccessSpecific() {
    await this.page.click('[data-testid="review-model-access-mode-specific"]');
    const done = this.page.locator('[data-testid="review-model-access-panel-done"]');
    await done.click();
    // Wait for the Sheet overlay to detach so it no longer intercepts clicks.
    await this.page.locator('[data-testid="review-model-access-panel"]').waitFor({ state: 'hidden' });
  }

  /**
   * Approve after configuring the model/MCP grant controls.
   * @param {Object} opts
   * @param {boolean} [opts.listModels] toggle "list all models" on
   * @param {boolean} [opts.modelsSpecific] switch model access to Specific (empty = no models)
   * @param {boolean} [opts.listMcps] toggle "list all MCPs" on
   */
  async approveWithGrants({ listModels = false, modelsSpecific = false, listMcps = false } = {}) {
    await this.waitForReviewPage();
    if (listModels) await this.toggleListModels();
    if (modelsSpecific) await this.setModelAccessSpecific();
    if (listMcps) await this.toggleListMcps();
    await this.clickApprove();
  }

  async clickDeny() {
    await this.page.click(this.selectors.denyButton);
  }

  /**
   * Approve with specific MCP server selections.
   * @param {Array<{url: string, instanceId: string}>} selections
   */
  async approveWithMcps(selections) {
    await this.waitForReviewPage();

    for (const { url, instanceId } of selections) {
      await this.selectMcpInstance(url, instanceId);
    }

    await this.clickApprove();
  }

  /**
   * Approve with MCP selections.
   * @param {Object} params
   * @param {Array<{url: string, instanceId: string}>} [params.mcps]
   */
  async approveWithResources({ mcps = [] }) {
    await this.waitForReviewPage();

    for (const { url, instanceId } of mcps) {
      await this.selectMcpInstance(url, instanceId);
    }

    await this.clickApprove();
  }

  /**
   * Select the approved role from the role dropdown.
   * @param {string} role - The role value (e.g. 'scope_user_user', 'scope_user_power_user')
   */
  async selectApprovedRole(role) {
    await this.page.click(this.selectors.approvedRoleSelect);
    await this.page.locator(`[data-testid="review-approved-role-option-${role}"]`).click();
  }

  /**
   * Approve with a specific role and optional resource selections.
   * @param {string} role - The approved_role value to select
   * @param {Object} [resources] - Optional resource selections
   * @param {Array<{url: string, instanceId: string}>} [resources.mcps]
   */
  async approveWithRole(role, { mcps = [] } = {}) {
    await this.waitForReviewPage();
    await this.selectApprovedRole(role);

    for (const { url, instanceId } of mcps) {
      await this.selectMcpInstance(url, instanceId);
    }

    await this.clickApprove();
  }
}
