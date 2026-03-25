import { SetupBasePage } from '@/pages/SetupBasePage.mjs';
import { expect } from '@playwright/test';

export class SetupBrowserExtensionPage extends SetupBasePage {
  selectors = {
    ...this.selectors,
    pageContainer: '[data-testid="browser-extension-setup-page"]',
    setupProgress: '[data-testid="setup-progress"]',
    browserSelector: '[data-testid="browser-selector"]',
    browserInfoCard: '[data-testid="browser-info-card"]',
    installExtensionLink: '[data-testid="install-extension-link"]',
    refreshButton: '[data-testid="refresh-button"]',
    continueButton: '[data-testid="browser-extension-continue"]',
    welcomeTitle: 'text=Browser Extension Setup',
    extensionDetecting: '[data-testid="extension-detecting"]',
    extensionFound: '[data-testid="extension-found"]',
    extensionNotFound: '[data-testid="extension-not-found"]',
    comingSoonMessage: 'text=coming soon',
    helpSection: 'text=Need help?',
  };

  async navigateToBrowserExtensionSetup() {
    await this.navigate('/ui/setup/browser-extension/');
    await this.waitForSetupPage();
  }

  async expectBrowserExtensionPage() {
    await this.expectVisible(this.selectors.pageContainer);
    await this.expectVisible(this.selectors.welcomeTitle);
    await this.expectStepIndicator(5);
    await this.expectStepStatus(5, 'current');

    // Verify step label is visible
    await this.expectVisible(this.selectors.stepLabel(5));
  }

  async expectBrowserSelectorPresent() {
    await this.expectVisible(this.selectors.browserSelector);
  }

  async expectBrowserDetected(browserName) {
    const browserSelector = this.page.locator(this.selectors.browserSelector);
    await expect(browserSelector).toContainText(browserName);
    await expect(browserSelector).toContainText('(detected)');
  }

  async selectBrowser(browserType) {
    await this.page.click(this.selectors.browserSelector);

    // Wait for dropdown options to appear
    await this.page.waitForSelector(`text=${browserType}`, { state: 'visible' });
    await this.page.click(`text=${browserType}`);
  }

  async expectBrowserInfoCard(statusMessage) {
    await this.expectVisible(this.selectors.browserInfoCard);

    const infoCard = this.page.locator(this.selectors.browserInfoCard);
    await expect(infoCard).toContainText(statusMessage);
  }

  async expectSupportedBrowserUI() {
    // Should show extension detection UI for supported browsers
    await this.page.waitForSelector(
      `${this.selectors.extensionNotFound}, ${this.selectors.extensionFound}, ${this.selectors.extensionDetecting}`,
      { timeout: 5000 }
    );
  }

  async expectUnsupportedBrowserUI() {
    // Should show continue button for unsupported browsers
    await this.expectVisible(this.selectors.continueButton);
    await expect(this.page.locator(this.selectors.comingSoonMessage)).toBeVisible();

    // Should NOT show extension detection UI
    await expect(this.page.locator(this.selectors.refreshButton)).not.toBeVisible();
  }

  async expectExtensionDetecting() {
    await this.expectVisible(this.selectors.extensionDetecting);
  }

  async expectExtensionNotFound() {
    await this.expectVisible(this.selectors.extensionNotFound);
    await this.expectVisible(this.selectors.refreshButton);
    await this.expectVisible(this.selectors.continueButton);

    // Should show install guidance
    await expect(
      this.page.locator('text=Install the extension and click below to verify')
    ).toBeVisible();

    // Verify button shows "Skip for Now"
    const continueButton = this.page.locator(this.selectors.continueButton);
    await expect(continueButton).toContainText('Skip for Now');
  }

  async expectExtensionFound() {
    await this.expectVisible(this.selectors.extensionFound);
    await this.expectVisible(this.selectors.continueButton);

    await expect(
      this.page.locator('text=The Bodhi Browser extension is installed and ready to use')
    ).toBeVisible();

    // Verify button shows "Continue"
    const continueButton = this.page.locator(this.selectors.continueButton);
    await expect(continueButton).toContainText('Continue');
  }

  async clickRefresh() {
    await this.expectVisible(this.selectors.refreshButton);
    await this.page.click(this.selectors.refreshButton);

    // Should trigger page reload
    await this.page.waitForLoadState('networkidle');
  }

  async clickContinue() {
    await this.expectVisible(this.selectors.continueButton);
    await this.page.click(this.selectors.continueButton);
  }

  async expectNavigationToComplete() {
    await this.page.waitForURL('**/ui/setup/complete/', { timeout: 10000 });
    expect(this.page.url()).toContain('/ui/setup/complete/');
  }

  async expectHelpSection() {
    await this.expectVisible(this.selectors.helpSection);
    await expect(this.page.locator('text=install the extension later')).toBeVisible();
  }

  async completeBrowserExtensionSetup(options = {}) {
    const { browser = 'chrome', extensionInstalled = false, skipExtension = false } = options;

    // All browsers now use the unified continue button
    await this.clickContinue();
  }

  // Browser-specific test helpers
  async simulateChromeWithExtension(extensionId = 'test-extension-id') {
    // This would be used in tests to simulate extension detection
    // In real tests, we'd mock the browser detection and extension detection hooks
  }

  async simulateFirefoxBrowser() {
    // This would simulate Firefox browser detection
    // In real tests, we'd mock the browser detection hook
  }

  async simulateExtensionInstallation() {
    // This would simulate the extension installation process
    // In real tests, we'd mock the extension detection state changes
  }
}
