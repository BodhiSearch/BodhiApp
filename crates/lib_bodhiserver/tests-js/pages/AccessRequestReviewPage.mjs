import { BasePage } from '@/pages/BasePage.mjs';

export class AccessRequestReviewPage extends BasePage {
  selectors = {
    reviewPage: '[data-testid="review-access-page"]',
    approveButton: '[data-testid="review-approve-button"]',
    denyButton: '[data-testid="review-deny-button"]',
    approvedRoleSelect: '[data-testid="review-approved-role-select"]',
  };

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

  async toggleListModels() {
    await this.page.click('[data-testid="review-list-models-toggle"]');
  }

  async toggleListMcps() {
    await this.page.click('[data-testid="review-list-mcps-toggle"]');
  }

  // Pre-populated in exchange mode.
  async isListModelsChecked() {
    return (
      (await this.page
        .locator('[data-testid="review-list-models-toggle"]')
        .getAttribute('aria-checked')) === 'true'
    );
  }

  async isListMcpsChecked() {
    return (
      (await this.page
        .locator('[data-testid="review-list-mcps-toggle"]')
        .getAttribute('aria-checked')) === 'true'
    );
  }

  // Empty `ids` leaves the grant empty — a deterministic "no access" grant.
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

  // The model picker defaults to Specific, so the mode is already set.
  async grantSpecificModels(ids) {
    await this.page.click('[data-testid="review-model-access-mode-specific"]');
    await this.page.click('[data-testid="review-model-access-add"]');
    await this.pickFromOpenPanel('review-model-access', ids);
  }

  async grantAllModels() {
    await this.page.click('[data-testid="review-model-access-mode-all"]');
  }

  // That picker defaults to Specific, so the mode is already set.
  async grantSpecificMcps(ids) {
    await this.page.click('[data-testid="review-mcp-access-add"]');
    await this.pickFromOpenPanel('review-mcp-access', ids);
  }

  async grantAllMcps() {
    await this.page.click('[data-testid="review-mcp-access-mode-all"]');
  }

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

  async approveWithMcps(selections) {
    await this.waitForReviewPage();

    for (const { url, instanceId } of selections) {
      await this.selectMcpInstance(url, instanceId);
    }

    await this.clickApprove();
  }

  async selectApprovedRole(role) {
    await this.page.click(this.selectors.approvedRoleSelect);
    await this.page.locator(`[data-testid="review-approved-role-option-${role}"]`).click();
  }

  async approveWithRole(role, { mcps = [] } = {}) {
    await this.waitForReviewPage();
    await this.selectApprovedRole(role);

    for (const { url, instanceId } of mcps) {
      await this.selectMcpInstance(url, instanceId);
    }

    await this.clickApprove();
  }
}
