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
    // Wait for the grantable model/MCP lists to settle — the access pickers
    // re-render when they load and clicking mid-load drops the event.
    await this.page.waitForSelector(`${this.selectors.reviewPage}[data-test-state="ready"]`);
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

  /** Wait for an open Specific picker panel (prefix `review-model-access` |
   *  `review-mcp-access`), click each grant id, then close it. Empty `ids` leaves
   *  the grant empty — a deterministic "no access" grant. */
  async pickFromOpenPanel(prefix, ids) {
    await this.page.waitForSelector(`[data-testid="${prefix}-panel"]`);
    for (const id of ids) {
      const item = this.page.locator(`[data-testid="${prefix}-panel-item-${id}"]`);
      await item.waitFor({ state: 'visible' });
      await item.click();
    }
    await this.page.click(`[data-testid="${prefix}-panel-done"]`);
    // Wait for the Sheet overlay to detach so it no longer intercepts clicks.
    await this.page.locator(`[data-testid="${prefix}-panel"]`).waitFor({ state: 'hidden' });
  }

  /** Grant exactly `ids` models. The model picker defaults to All, so clicking
   *  Specific switches mode and auto-opens the panel. Empty ids = no model access. */
  async grantSpecificModels(ids) {
    await this.page.click('[data-testid="review-model-access-mode-specific"]');
    await this.pickFromOpenPanel('review-model-access', ids);
  }

  /** Grant exactly `ids` owner-extra MCPs. That picker defaults to Specific, so the
   *  mode is already set — open the panel via its Add button. */
  async grantSpecificMcps(ids) {
    await this.page.click('[data-testid="review-mcp-access-add"]');
    await this.pickFromOpenPanel('review-mcp-access', ids);
  }

  /**
   * Approve after configuring the model/MCP grant controls.
   * @param {Object} opts
   * @param {boolean}  [opts.listModels] toggle "list all models" on
   * @param {string[]} [opts.modelIds] switch model access to Specific and grant these ids ([] = none)
   * @param {boolean}  [opts.listMcps] toggle "list all MCPs" on
   * @param {string[]} [opts.mcpIds] grant these owner-extra MCP ids
   */
  async approveWithGrants({
    listModels = false,
    modelIds = null,
    listMcps = false,
    mcpIds = null,
  } = {}) {
    await this.waitForReviewPage();
    if (listModels) await this.toggleListModels();
    if (modelIds) await this.grantSpecificModels(modelIds);
    if (listMcps) await this.toggleListMcps();
    if (mcpIds) await this.grantSpecificMcps(mcpIds);
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
