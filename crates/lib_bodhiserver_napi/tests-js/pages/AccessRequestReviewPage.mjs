import { BasePage } from '@/pages/BasePage.mjs';

export class AccessRequestReviewPage extends BasePage {
  selectors = {
    reviewPage: '[data-testid="review-access-page"]',
    approveButton: '[data-testid="review-approve-button"]',
    denyButton: '[data-testid="review-deny-button"]',
  };

  toolCheckbox(toolsetType) {
    return `[data-testid="review-tool-checkbox-${toolsetType}"]`;
  }

  instanceSelectTrigger(toolsetType) {
    return `[data-testid="review-instance-select-${toolsetType}"]`;
  }

  instanceOption(instanceId) {
    return `[data-testid="review-instance-option-${instanceId}"]`;
  }

  async waitForReviewPage() {
    await this.expectVisible(this.selectors.reviewPage);
  }

  async selectInstance(toolsetType, instanceId) {
    // Click the select trigger to open the dropdown
    await this.page.click(this.instanceSelectTrigger(toolsetType));
    // Radix Select renders options in a portal, so use page-level locator
    await this.page.locator(this.instanceOption(instanceId)).click();
  }

  async clickApprove() {
    await this.page.click(this.selectors.approveButton);
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
}
