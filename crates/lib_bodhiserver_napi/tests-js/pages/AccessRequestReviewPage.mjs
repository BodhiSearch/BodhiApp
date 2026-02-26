import { BasePage } from '@/pages/BasePage.mjs';

export class AccessRequestReviewPage extends BasePage {
  selectors = {
    reviewPage: '[data-testid="review-access-page"]',
    approveButton: '[data-testid="review-approve-button"]',
    denyButton: '[data-testid="review-deny-button"]',
    approvedRoleSelect: '[data-testid="review-approved-role-select"]',
  };

  // Toolset selectors
  toolCheckbox(toolsetType) {
    return `[data-testid="review-tool-checkbox-${toolsetType}"]`;
  }

  instanceSelectTrigger(toolsetType) {
    return `[data-testid="review-instance-select-${toolsetType}"]`;
  }

  instanceOption(instanceId) {
    return `[data-testid="review-instance-option-${instanceId}"]`;
  }

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

  async selectInstance(toolsetType, instanceId) {
    await this.page.click(this.instanceSelectTrigger(toolsetType));
    await this.page.locator(this.instanceOption(instanceId)).click();
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

  async clickDeny() {
    await this.page.click(this.selectors.denyButton);
  }

  /**
   * Approve with specific toolset selections.
   * @param {Array<{toolsetType: string, instanceId: string}>} selections
   */
  async approveWithToolsets(selections) {
    await this.waitForReviewPage();

    for (const { toolsetType, instanceId } of selections) {
      await this.selectInstance(toolsetType, instanceId);
    }

    await this.clickApprove();
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
   * Approve with both toolset and MCP selections.
   * @param {Object} params
   * @param {Array<{toolsetType: string, instanceId: string}>} [params.toolsets]
   * @param {Array<{url: string, instanceId: string}>} [params.mcps]
   */
  async approveWithResources({ toolsets = [], mcps = [] }) {
    await this.waitForReviewPage();

    for (const { toolsetType, instanceId } of toolsets) {
      await this.selectInstance(toolsetType, instanceId);
    }

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
   * @param {Array<{toolsetType: string, instanceId: string}>} [resources.toolsets]
   * @param {Array<{url: string, instanceId: string}>} [resources.mcps]
   */
  async approveWithRole(role, { toolsets = [], mcps = [] } = {}) {
    await this.waitForReviewPage();
    await this.selectApprovedRole(role);

    for (const { toolsetType, instanceId } of toolsets) {
      await this.selectInstance(toolsetType, instanceId);
    }

    for (const { url, instanceId } of mcps) {
      await this.selectMcpInstance(url, instanceId);
    }

    await this.clickApprove();
  }
}
