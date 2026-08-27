import { BasePage } from '@/pages/BasePage.mjs';

// Consent page at /ui/apps/auth/ — the app navigates here top-level with the raw
// OAuth params; the owner reviews and approves/denies before Keycloak authorize.
export class AppsAuthPage extends BasePage {
  selectors = {
    consentPage: '[data-testid="consent-page"]',
    approveButton: '[data-testid="consent-approve-button"]',
    denyButton: '[data-testid="consent-deny-button"]',
    approvedRoleSelect: '[data-testid="consent-approved-role-select"]',
    modelsSection: '[data-testid="consent-models-section"]',
    mcpsSection: '[data-testid="consent-mcps-section"]',
    roleOnlySummary: '[data-testid="consent-role-only-summary"]',
    reauthBanner: '[data-testid="consent-reauth-banner"]',
    error: '[data-testid="consent-error"]',
    listModelsToggle: '[data-testid="consent-list-models-toggle"]',
    listMcpsToggle: '[data-testid="consent-list-mcps-toggle"]',
  };

  async waitForConsentPage() {
    // Wait for the grantable model/MCP lists to settle — the access pickers
    // re-render when they load and clicking mid-load drops the event.
    await this.page.waitForSelector(`${this.selectors.consentPage}[data-test-state="ready"]`);
  }

  async waitForError() {
    await this.page.waitForSelector(this.selectors.error);
  }

  async clickApprove() {
    await this.page.click(this.selectors.approveButton);
  }

  async approve() {
    await this.waitForConsentPage();
    await this.clickApprove();
  }

  async clickDeny() {
    await this.page.click(this.selectors.denyButton);
  }

  async toggleListModels() {
    await this.page.click(this.selectors.listModelsToggle);
  }

  async toggleListMcps() {
    await this.page.click(this.selectors.listMcpsToggle);
  }

  // Pre-populated when reauthorizing an explicit prior grant.
  async isListModelsChecked() {
    return (
      (await this.page.locator(this.selectors.listModelsToggle).getAttribute('aria-checked')) ===
      'true'
    );
  }

  async isListMcpsChecked() {
    return (
      (await this.page.locator(this.selectors.listMcpsToggle).getAttribute('aria-checked')) ===
      'true'
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

  // Mode is clicked explicitly: a reauth prefill can restore "All", hiding the Add button.
  async grantSpecificModels(ids) {
    await this.page.click('[data-testid="consent-model-access-mode-specific"]');
    await this.page.click('[data-testid="consent-model-access-add"]');
    await this.pickFromOpenPanel('consent-model-access', ids);
  }

  async grantAllModels() {
    await this.page.click('[data-testid="consent-model-access-mode-all"]');
  }

  async grantSpecificMcps(ids) {
    await this.page.click('[data-testid="consent-mcp-access-mode-specific"]');
    await this.page.click('[data-testid="consent-mcp-access-add"]');
    await this.pickFromOpenPanel('consent-mcp-access', ids);
  }

  async grantAllMcps() {
    await this.page.click('[data-testid="consent-mcp-access-mode-all"]');
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
    await this.waitForConsentPage();
    if (listModels) await this.toggleListModels();
    if (allModels) await this.grantAllModels();
    if (modelIds) await this.grantSpecificModels(modelIds);
    if (listMcps) await this.toggleListMcps();
    if (allMcps) await this.grantAllMcps();
    if (mcpIds) await this.grantSpecificMcps(mcpIds);
    if (role) await this.selectApprovedRole(role);
    await this.clickApprove();
  }

  // Grants the given MCP instance ids via the consent MCP picker, then approves.
  async approveWithMcps(mcpIds) {
    await this.waitForConsentPage();
    await this.grantSpecificMcps(mcpIds);
    await this.clickApprove();
  }

  async selectApprovedRole(role) {
    await this.page.click(this.selectors.approvedRoleSelect);
    await this.page.locator(`[data-testid="consent-approved-role-option-${role}"]`).click();
  }

  async approveWithRole(role, { mcpIds = [] } = {}) {
    await this.waitForConsentPage();
    await this.selectApprovedRole(role);
    if (mcpIds.length > 0) await this.grantSpecificMcps(mcpIds);
    await this.clickApprove();
  }
}
