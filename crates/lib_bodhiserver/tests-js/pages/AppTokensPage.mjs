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

  /**
   * Row id for the app with `clientId`. Locating by the row's `data-test-client-id`
   * lets Playwright auto-wait for the table + row to render (no manual count/poll).
   */
  async findRowIdByClientId(clientId) {
    const row = this.page
      .getByTestId('app-tokens-table')
      .locator(`[data-test-client-id="${clientId}"]`);
    const testId = await row.getAttribute('data-testid');
    return testId.replace('app-row-', '');
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
