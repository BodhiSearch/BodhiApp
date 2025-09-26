import { expect } from '@playwright/test';
import { SetupBasePage } from '@/pages/SetupBasePage.mjs';

export class SetupBrowserExtensionPage extends SetupBasePage {
  constructor(page, baseUrl) {
    super(page, baseUrl);
  }

  selectors = {
    ...this.selectors,
    pageContainer: '[data-testid="browser-extension-setup-page"]',
    setupProgress: '[data-testid="setup-progress"]',
    browserSelector: '[data-testid="browser-selector"]',
    browserInfoCard: '[data-testid="browser-info-card"]',
    installExtensionLink: '[data-testid="install-extension-link"]',
    refreshButton: '[data-testid="refresh-button"]',
    skipButton: '[data-testid="skip-button"]',
    nextButton: '[data-testid="next-button"]',
    continueButton: '[data-testid="continue-button"]',
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
      this.selectors.extensionNotFound +
        ', ' +
        this.selectors.extensionFound +
        ', ' +
        this.selectors.extensionDetecting,
      { timeout: 5000 }
    );
  }

  async expectUnsupportedBrowserUI() {
    // Should show continue button for unsupported browsers
    await this.expectVisible(this.selectors.continueButton);
    await expect(this.page.locator(this.selectors.comingSoonMessage)).toBeVisible();

    // Should NOT show extension detection UI
    await expect(this.page.locator(this.selectors.refreshButton)).not.toBeVisible();
    await expect(this.page.locator(this.selectors.skipButton)).not.toBeVisible();
  }

  async expectExtensionDetecting() {
    await this.expectVisible(this.selectors.extensionDetecting);
  }

  async expectExtensionNotFound() {
    await this.expectVisible(this.selectors.extensionNotFound);
    await this.expectVisible(this.selectors.refreshButton);
    await this.expectVisible(this.selectors.skipButton);

    // Should show install guidance
    await expect(this.page.locator('text=Install the extension to continue')).toBeVisible();
  }

  async expectExtensionFound() {
    await this.expectVisible(this.selectors.extensionFound);
    await this.expectVisible(this.selectors.nextButton);

    await expect(
      this.page.locator('text=Perfect! The Bodhi Browser extension is installed')
    ).toBeVisible();
  }

  async clickRefresh() {
    await this.expectVisible(this.selectors.refreshButton);
    await this.page.click(this.selectors.refreshButton);

    // Should trigger page reload
    await this.page.waitForLoadState('networkidle');
  }

  async clickSkip() {
    await this.expectVisible(this.selectors.skipButton);
    await this.page.click(this.selectors.skipButton);
  }

  async clickNext() {
    await this.expectVisible(this.selectors.nextButton);
    await this.page.click(this.selectors.nextButton);
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

    if (browser === 'firefox' || browser === 'safari') {
      // Unsupported browser - just continue
      await this.clickContinue();
      return;
    }

    // Supported browser (Chrome/Edge)
    if (skipExtension) {
      await this.clickSkip();
    } else if (extensionInstalled) {
      await this.clickNext();
    } else {
      // Extension not installed - skip for now
      await this.clickSkip();
    }
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
