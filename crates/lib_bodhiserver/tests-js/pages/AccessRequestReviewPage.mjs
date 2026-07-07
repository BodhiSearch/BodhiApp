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

  /** True when the "list all models" toggle is on (pre-populated in exchange mode). */
  async isListModelsChecked() {
    return (
      (await this.page
        .locator('[data-testid="review-list-models-toggle"]')
        .getAttribute('aria-checked')) === 'true'
    );
  }

  /** True when the "list all MCPs" toggle is on. */
  async isListMcpsChecked() {
    return (
      (await this.page
        .locator('[data-testid="review-list-mcps-toggle"]')
        .getAttribute('aria-checked')) === 'true'
    );
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

  /** Grant exactly `ids` models. The model picker defaults to Specific, so the mode is
   *  already set — open the panel via its Add button. Empty ids = no model access. */
  async grantSpecificModels(ids) {
    await this.page.click('[data-testid="review-model-access-mode-specific"]');
    await this.page.click('[data-testid="review-model-access-add"]');
    await this.pickFromOpenPanel('review-model-access', ids);
  }

  /** Grant ALL models at consent (the picker defaults to Specific/none now). */
  async grantAllModels() {
    await this.page.click('[data-testid="review-model-access-mode-all"]');
  }

  /** Grant exactly `ids` owner-extra MCPs. That picker defaults to Specific, so the
   *  mode is already set — open the panel via its Add button. */
  async grantSpecificMcps(ids) {
    await this.page.click('[data-testid="review-mcp-access-add"]');
    await this.pickFromOpenPanel('review-mcp-access', ids);
  }

  /** Grant ALL owner-extra MCPs at consent. */
  async grantAllMcps() {
    await this.page.click('[data-testid="review-mcp-access-mode-all"]');
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
    allModels = false,
    modelIds = null,
    listMcps = false,
    allMcps = false,
    mcpIds = null,
    role = null,
  } = {}) {
    await this.waitForReviewPage();
    if (listModels) await this.toggleListModels();
    if (allModels) await this.grantAllModels();
    if (modelIds) await this.grantSpecificModels(modelIds);
    if (listMcps) await this.toggleListMcps();
    if (allMcps) await this.grantAllMcps();
    if (mcpIds) await this.grantSpecificMcps(mcpIds);
    if (role) await this.selectApprovedRole(role);
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
