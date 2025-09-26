import { expect } from '@playwright/test';
import { SetupBasePage } from '@/pages/SetupBasePage.mjs';

export class SetupWelcomePage extends SetupBasePage {
  constructor(page, baseUrl) {
    super(page, baseUrl);
  }

  selectors = {
    ...this.selectors,
    welcomeTitle: 'text=Welcome to Bodhi App',
    serverNameInput: 'input[name="name"]',
    setupButton: 'button:has-text("Setup Bodhi Server")',
    benefitCards: '[data-testid="benefit-card"]',
    completePrivacyBenefit: 'text=Complete Privacy',
    alwaysFreeBenefit: 'text=Always Free',
    fullControlBenefit: 'text=Full Control',
    localPerformanceBenefit: 'text=Local Performance',
  };

  async navigateToSetup() {
    await this.navigate('/ui/setup/');
    await this.waitForSetupPage();
  }

  async expectWelcomePage() {
    await this.expectVisible(this.selectors.welcomeTitle);
    await this.expectStepIndicator(1);
  }

  async expectBenefitsDisplayed() {
    // Check for key benefits
    await expect(this.page.locator(this.selectors.completePrivacyBenefit)).toBeVisible();
    await expect(this.page.locator(this.selectors.alwaysFreeBenefit)).toBeVisible();
    await expect(this.page.locator(this.selectors.fullControlBenefit)).toBeVisible();
    await expect(this.page.locator(this.selectors.localPerformanceBenefit)).toBeVisible();
  }

  async fillServerName(name) {
    await this.expectVisible(this.selectors.serverNameInput);
    await this.page.fill(this.selectors.serverNameInput, name);
  }

  async clickSetupServer() {
    await this.expectVisible(this.selectors.setupButton);
    await this.page.click(this.selectors.setupButton);
    await this.waitForSetupPage();
  }

  async completeInitialSetup(serverName = 'My Test Bodhi Server') {
    await this.fillServerName(serverName);
    await this.clickSetupServer();
  }
}
