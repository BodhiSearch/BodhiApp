import { expect } from '@playwright/test';
import { SetupBasePage } from '@/pages/SetupBasePage.mjs';

export class SetupWelcomePage extends SetupBasePage {
  constructor(page, baseUrl) {
    super(page, baseUrl);
  }

  selectors = {
    ...this.selectors,
    pageContainer: '[data-testid="setup-welcome-page"]',
    welcomeCard: '[data-testid="welcome-card"]',
    benefitsGrid: '[data-testid="benefits-grid"]',
    benefitCard: (title) => `[data-testid="benefit-card-${title}"]`,
    browserAIBenefit: '[data-testid="benefit-card-browser-ai-revolution"]',
    multiUserBenefit: '[data-testid="benefit-card-multi-user-ready"]',
    serverNameInput: '[data-testid="server-name-input"]',
    descriptionInput: '[data-testid="description-input"]',
    welcomeTitle: 'text=Welcome to Bodhi App',
    setupButton: 'button:has-text("Setup Bodhi Server")',
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
    // Check for updated benefits
    await expect(this.page.locator('text=Complete Privacy')).toBeVisible();
    await expect(this.page.locator('text=Cost Freedom')).toBeVisible();
    await expect(this.page.locator('text=Browser AI Revolution')).toBeVisible();
    await expect(this.page.locator('text=Multi-User Ready')).toBeVisible();
    await expect(this.page.locator('text=Hybrid Flexibility')).toBeVisible();
    await expect(this.page.locator('text=Open Ecosystem')).toBeVisible();
  }

  async expectNewFeatureBadges() {
    await this.expectVisible(this.selectors.browserAIBenefit);
    await this.expectVisible(this.selectors.multiUserBenefit);

    const browserCard = this.page.locator(this.selectors.browserAIBenefit);
    await expect(browserCard.locator('text=NEW')).toBeVisible();

    const multiUserCard = this.page.locator(this.selectors.multiUserBenefit);
    await expect(multiUserCard.locator('text=NEW')).toBeVisible();
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
