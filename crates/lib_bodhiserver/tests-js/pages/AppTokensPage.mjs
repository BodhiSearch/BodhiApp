import { BasePage } from '@/pages/BasePage.mjs';

/** Page object for the App Tokens management screen (`/ui/tokens/apps/`). */
export class AppTokensPage extends BasePage {
  selectors = {
    page: '[data-testid="app-tokens-page"]',
    empty: '[data-testid="app-tokens-empty"]',
    detailRail: '[data-testid="app-detail-rail"]',
    revoke: '[data-testid="app-revoke"]',
    revokeConfirm: '[data-testid="app-revoke-confirm"]',
  };

  row(id) {
    return `[data-testid="app-row-${id}"]`;
  }

  async navigateToAppTokens() {
    await this.navigate('/ui/tokens/apps/');
    await this.waitForSPAReady();
    await this.expectVisible(this.selectors.page);
  }

  /** Reach the screen through the shell nav — proves the "App Tokens" entry is wired. */
  async navigateViaShell() {
    await this.navViaShell('api-keys', 'app-tokens');
    await this.expectVisible(this.selectors.page);
  }

  /** Find the row id of the first app whose visible text contains `clientId`. */
  async findRowIdByClientId(clientId) {
    const rows = this.page.locator('[data-testid^="app-row-"]');
    const count = await rows.count();
    for (let i = 0; i < count; i++) {
      const row = rows.nth(i);
      if ((await row.textContent())?.includes(clientId)) {
        const testId = await row.getAttribute('data-testid');
        return testId.replace('app-row-', '');
      }
    }
    return null;
  }

  async openRail(id) {
    await this.page.click(this.row(id));
    await this.expectVisible(this.selectors.detailRail);
  }

  async revokeAccess() {
    await this.page.click(this.selectors.revoke);
    await this.page.click(this.selectors.revokeConfirm);
  }
}
