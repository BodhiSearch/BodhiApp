import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

export class RequestAccessPage extends BasePage {
  selectors = {
    requestAccessButton: '[data-testid="auth-card-action-0"]',
    authCard: '[data-testid="auth-card"]',
    authCardHeader: '[data-testid="auth-card-header"]',
    authCardContent: '[data-testid="auth-card-content"]',
    authCardDescription: '[data-testid="auth-card-description"]',
    authCardActions: '[data-testid="auth-card-actions"]',
    pageContainer: '[data-testid="request-access-page"]',
  };

  async expectRequestAccessPage() {
    await expect(this.page).toHaveURL(/\/ui\/request-access/);
    await this.expectVisible(this.selectors.pageContainer);
  }

  async expectAuthCardVisible() {
    await this.expectVisible(this.selectors.authCard);
    await this.expectVisible(this.selectors.authCardHeader);
    await this.expectVisible(this.selectors.authCardContent);
  }

  async expectRequestButtonVisible(shouldBeVisible = true) {
    const button = this.page.locator(this.selectors.requestAccessButton);
    if (shouldBeVisible) {
      await expect(button).toBeVisible();
      await expect(button).toBeEnabled();
    } else {
      await expect(button).not.toBeVisible();
    }
  }

  async clickRequestAccess() {
    await this.page.click(this.selectors.requestAccessButton);
  }

  async expectPendingState() {
    await this.expectVisible(this.selectors.authCardDescription);
    await expect(this.page.locator(this.selectors.authCardDescription)).toContainText(
      'pending review'
    );
  }

  async getSubmittedDate() {
    const description = await this.page.locator(this.selectors.authCardDescription).textContent();
    // Extract date in MM/DD/YYYY format from description
    const dateMatch = description.match(/(\d{1,2}\/\d{1,2}\/\d{4})/);
    return dateMatch ? dateMatch[1] : null;
  }

  async expectSubmittedDateFormat(submittedDate) {
    // Verify the date format is MM/DD/YYYY
    const datePattern = /^\d{1,2}\/\d{1,2}\/\d{4}$/;
    expect(submittedDate).toMatch(datePattern);
  }

  async navigateToRequestAccess() {
    await this.navigate('/ui/request-access');
  }

  async testProtectedPageRedirect(protectedPath) {
    await this.page.goto(`${this.baseUrl}${protectedPath}`);
    await this.waitForSPAReady();
    await this.expectRequestAccessPage();
  }
}
