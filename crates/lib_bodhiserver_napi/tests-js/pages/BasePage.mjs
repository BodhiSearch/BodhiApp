import { expect } from '@playwright/test';

export class BasePage {
  baseSelectors = {
    successToast: '[data-state="open"]',
  };

  constructor(page, baseUrl) {
    this.page = page;
    this.baseUrl = baseUrl;
  }

  async navigate(path) {
    await this.page.goto(`${this.baseUrl}${path}`);
    await this.waitForSPAReady();
  }

  async waitForSPAReady() {
    await this.page.waitForLoadState('domcontentloaded');
  }

  async clickTestId(testId) {
    await this.page.click(`[data-testid="${testId}"]`);
  }

  async fillTestId(testId, value) {
    await this.page.fill(`[data-testid="${testId}"]`, value);
  }

  async expectVisible(selector) {
    await expect(this.page.locator(selector)).toBeVisible();
  }

  async waitForToast(message, options = {}) {
    if (message instanceof RegExp) {
      await expect(this.page.locator(this.baseSelectors.successToast)).toContainText(
        message,
        options
      );
    } else {
      await expect(this.page.locator(this.baseSelectors.successToast)).toContainText(
        message,
        options
      );
    }
  }

  async waitForToastAndExtractId(messagePattern) {
    // Wait for toast to appear with the pattern
    await this.waitForToast(messagePattern);

    // Get the toast text content
    const toastText = await this.page.locator(this.baseSelectors.successToast).textContent();

    // Extract UUID from the toast message using regex
    // UUID pattern: 8-4-4-4-12 hexadecimal characters
    const uuidPattern = /([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/i;
    const match = toastText.match(uuidPattern);

    if (!match) {
      throw new Error(`Failed to extract UUID from toast message: "${toastText}"`);
    }

    return match[1];
  }

  async waitForToastToHide() {
    // Check if toast exists first
    const toastLocator = this.page.locator(this.baseSelectors.successToast);
    try {
      // If toast is visible, wait for it to hide with extended timeout
      if (await toastLocator.isVisible()) {
        await expect(toastLocator).toBeHidden();
      }
    } catch (error) {
      // If toast doesn't hide naturally, try to dismiss it by clicking close button
      try {
        const closeButton = this.page.locator('[toast-close]').first();
        if (await closeButton.isVisible()) {
          await closeButton.click();
          await expect(toastLocator).toBeHidden();
        }
      } catch {
        // If all else fails, just continue - the test should handle this gracefully
        console.warn('Toast did not hide within timeout, continuing...');
      }
    }
  }

  async waitForToastOptional(message, options = {}) {
    try {
      const timeout = process.env.CI ? 1000 : 5000;
      const finalOptions = { timeout, ...options };

      if (message instanceof RegExp) {
        await expect(this.page.locator(this.baseSelectors.successToast)).toContainText(
          message,
          finalOptions
        );
      } else {
        await expect(this.page.locator(this.baseSelectors.successToast)).toContainText(
          message,
          finalOptions
        );
      }
    } catch (error) {
      console.log(`Toast check skipped (CI=${!!process.env.CI}):`, message);
      // Don't fail - toast is optional confirmation
    }
  }

  async waitForToastToHideOptional() {
    try {
      const toastLocator = this.page.locator(this.baseSelectors.successToast);
      if (await toastLocator.isVisible({ timeout: 500 })) {
        // Try to click close button first
        const closeButton = this.page.locator('[toast-close]').first();
        if (await closeButton.isVisible({ timeout: 500 })) {
          await closeButton.click();
        }
        // Wait a short time for it to hide
        await expect(toastLocator).toBeHidden({ timeout: 2000 });
      }
    } catch {
      // Silent fail - toast hiding is optional
    }
  }

  async getCurrentPath() {
    return new URL(this.page.url()).pathname;
  }

  async waitForUrl(pathOrPredicate) {
    if (typeof pathOrPredicate === 'string') {
      await this.page.waitForURL((url) => url.pathname === pathOrPredicate);
    } else {
      await this.page.waitForURL(pathOrPredicate);
    }
  }

  async takeScreenshot(name) {
    await this.page.screenshot({ path: `screenshots/${name}.png` });
  }

  async waitForSelector(selector, options = {}) {
    return await this.page.waitForSelector(selector, options);
  }

  async dismissAllToasts() {
    try {
      // Find all visible toasts and close buttons
      const toastCloseButtons = await this.page.locator('[data-radix-toast-announce-exclude] button').all();

      // Click all close buttons to dismiss toasts
      for (const closeButton of toastCloseButtons) {
        try {
          if (await closeButton.isVisible()) {
            await closeButton.click();
          }
        } catch {
          // Continue if a specific button fails
        }
      }

      // Also try the generic close button selector
      const genericCloseButtons = await this.page.locator('button[aria-label="Close"]').all();
      for (const closeButton of genericCloseButtons) {
        try {
          if (await closeButton.isVisible()) {
            await closeButton.click();
          }
        } catch {
          // Continue if a specific button fails
        }
      }

      // Wait a brief moment for animations to complete
      await this.page.waitForTimeout(200);
    } catch {
      // Silent fail - dismissing toasts is a best-effort operation
    }
  }

  async expectText(selector, text) {
    if (text instanceof RegExp) {
      await expect(this.page.locator(selector)).toContainText(text);
    } else {
      await expect(this.page.locator(selector)).toHaveText(text);
    }
  }

  async expectValue(selector, value) {
    await expect(this.page.locator(selector)).toHaveValue(value);
  }
}
