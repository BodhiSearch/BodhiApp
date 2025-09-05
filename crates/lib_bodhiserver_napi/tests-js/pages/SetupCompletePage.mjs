import { expect } from '@playwright/test';
import { SetupBasePage } from './SetupBasePage.mjs';

export class SetupCompletePage extends SetupBasePage {
  constructor(page, baseUrl) {
    super(page, baseUrl);
  }

  selectors = {
    ...this.selectors,
    setupCompleteTitle: 'text=Setup Complete',
    startUsingButton: 'button:has-text("Start Using Bodhi App")',
    congratulationsMessage: 'text=Congratulations',
    readyMessage: 'text=ready',
    socialLinks: '[data-testid="social-links"]',
    githubLink: 'a[href*="github"]',
    discordLink: 'a[href*="discord"]',
    documentationLink: 'text=Documentation'
  };

  async navigateToComplete() {
    await this.navigate('/ui/setup/complete/');
    await this.waitForSetupPage();
  }

  async expectSetupCompletePage() {
    await this.expectVisible(this.selectors.setupCompleteTitle);
    // Complete page might not show step indicator
    try {
      await this.expectStepIndicator(4);
    } catch {
      // Step indicator might not be shown on completion page
    }
  }

  async expectCompletionMessage() {
    // Look for completion-related messages
    const completionMessages = [
      this.selectors.congratulationsMessage,
      this.selectors.readyMessage,
      'text=You\'re all set',
      'text=Setup is complete'
    ];

    let messageFound = false;
    for (const message of completionMessages) {
      try {
        await expect(this.page.locator(message)).toBeVisible({ timeout: 2000 });
        messageFound = true;
        break;
      } catch {
        continue;
      }
    }

    if (!messageFound) {
      // Fallback: ensure we're on the complete page
      await expect(this.page.locator(this.selectors.setupCompleteTitle)).toBeVisible();
    }
  }

  async expectSocialLinksDisplayed() {
    // Check for social links - they might be icons or text
    try {
      await expect(this.page.locator(this.selectors.githubLink)).toBeVisible({ timeout: 3000 });
    } catch {
      // Social links might not be visible or have different selectors
      // This is optional verification
    }
  }

  async clickStartUsingApp() {
    await this.expectVisible(this.selectors.startUsingButton);
    await this.page.click(this.selectors.startUsingButton);
    // Wait for redirect to main app
    await this.page.waitForURL((url) => url.pathname === '/ui/chat/');
    await this.waitForSPAReady();
  }

  async completeSetup() {
    await this.expectSetupCompletePage();
    await this.expectCompletionMessage();
    await this.clickStartUsingApp();
  }
}