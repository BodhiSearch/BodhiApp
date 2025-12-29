import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

export class SetupBasePage extends BasePage {
  selectors = {
    setupProgress: '[data-testid="setup-progress"]',
    stepIndicator: (step) => `[data-testid="step-indicator-${step}"]`,
    stepLabel: (step) => `[data-testid="step-label-${step}"]`,
    stepCounter: '[data-testid="step-counter"]',
    bodhiLogo: '[data-testid="bodhi-logo"]',
    continueButton: 'button:has-text("Continue")',
    backButton: 'button:has-text("Back")',
    skipButton: 'button:has-text("Skip")',
  };

  async waitForSetupPage() {
    await this.waitForSPAReady();
  }

  async expectStepIndicator(step) {
    await this.expectVisible(this.selectors.stepIndicator(step));
  }

  async expectStepStatus(stepNumber, expectedStatus) {
    const stepIndicator = this.page.locator(this.selectors.stepIndicator(stepNumber));
    await expect(stepIndicator).toHaveAttribute('data-status', expectedStatus);
  }

  async expectBodhiLogo() {
    // Logo might not have data-testid, so check for common logo patterns
    const logoSelectors = ['[data-testid="bodhi-logo"]', 'img[alt*="Bodhi"]', 'svg[role="img"]'];
    let logoFound = false;

    for (const selector of logoSelectors) {
      try {
        await expect(this.page.locator(selector)).toBeVisible();
        logoFound = true;
        break;
      } catch {}
    }

    if (!logoFound) {
      // Fallback to checking for any logo-like element
      await expect(this.page.locator('text=Bodhi').first()).toBeVisible();
    }
  }

  async clickContinue() {
    await this.page.click(this.selectors.continueButton);
    await this.waitForSPAReady();
  }

  async clickBack() {
    await this.page.click(this.selectors.backButton);
    await this.waitForSPAReady();
  }

  async clickSkip() {
    await this.page.click(this.selectors.skipButton);
    await this.waitForSPAReady();
  }

  // Setup-specific navigation helpers
  async expectSetupStep(stepNumber, pathname) {
    await this.expectToBeOnPage(pathname);
    await this.expectStepIndicator(stepNumber);
    await this.expectStepStatus(stepNumber, 'current');
  }

  async navigateToSetupStep(path, stepNumber) {
    await this.navigateAndWaitForPage(path);
    await this.expectSetupStep(stepNumber, path);
  }

  async expectNavigationToSetupStep(pathname, stepNumber) {
    await this.page.waitForURL((url) => url.pathname === pathname);
    await this.expectCurrentPath(pathname);
    await this.expectStepIndicator(stepNumber);
  }

  // Common setup navigation patterns
  async expectNavigationToWelcome() {
    await this.expectNavigationToSetupStep('/ui/setup/', 1);
  }

  async expectNavigationToResourceAdmin() {
    await this.expectNavigationToSetupStep('/ui/setup/resource-admin/', 2);
  }

  async expectNavigationToDownloadModels() {
    await this.expectNavigationToSetupStep('/ui/setup/download-models/', 3);
  }

  async expectNavigationToApiModels() {
    await this.expectNavigationToSetupStep('/ui/setup/api-models/', 4);
  }

  async expectNavigationToBrowserExtension() {
    await this.expectNavigationToSetupStep('/ui/setup/browser-extension/', 5);
  }

  async expectNavigationToComplete() {
    await this.expectNavigationToSetupStep('/ui/setup/complete/', 6);
  }
}
