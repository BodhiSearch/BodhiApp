import { expect } from '@playwright/test';

export class BasePage {
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
    // Give SPA time to initialize
    await this.page.waitForTimeout(500);
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

  async waitForToast(message) {
    if (message instanceof RegExp) {
      await expect(this.page.locator('[data-state="open"]')).toContainText(message);
    } else {
      await expect(this.page.locator('[data-state="open"]')).toContainText(message);
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
