import { expect } from '@playwright/test';
import { BasePage } from './BasePage.mjs';

export class SetupBasePage extends BasePage {
  constructor(page, baseUrl) {
    super(page, baseUrl);
  }

  selectors = {
    setupProgress: '[data-testid="setup-progress"]',
    stepIndicator: (step) => `text=Step ${step} of `,
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

  async expectBodhiLogo() {
    // Logo might not have data-testid, so check for common logo patterns
    const logoSelectors = ['[data-testid="bodhi-logo"]', 'img[alt*="Bodhi"]', 'svg[role="img"]'];
    let logoFound = false;

    for (const selector of logoSelectors) {
      try {
        await expect(this.page.locator(selector)).toBeVisible();
        logoFound = true;
        break;
      } catch {
        continue;
      }
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
}
