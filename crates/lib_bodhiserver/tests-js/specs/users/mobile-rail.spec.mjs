import { AllUsersPage } from '@/pages/AllUsersPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

/**
 * Regression: at mobile width (<768px) the rail is a fixed drawer; clicking a row must open it
 * (`.shell.rail-open`). Previously the view-transition wrapping the selection fought the drawer's
 * own transform transition and the panel never opened.
 */
test.describe('Mobile detail-rail drawer', () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test('Manage Users row-click opens the rail drawer on mobile @integration', async ({
    page,
    sharedServerUrl,
  }) => {
    await page.setViewportSize({ width: 390, height: 800 });

    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const usersPage = new AllUsersPage(page, sharedServerUrl);

    await loginPage.performOAuthLogin();
    await usersPage.navigateToUsers();
    await usersPage.expectUsersPage();

    // confirm we're actually in the mobile breakpoint
    const isMobile = await page.evaluate(() => window.matchMedia('(max-width:767px)').matches);
    expect(isMobile).toBe(true);

    // before selecting, no rail drawer is open
    await expect(page.locator('.shell.rail-open')).toHaveCount(0);

    // click the current user's row → the rail drawer slides in (`.shell.rail-open`)
    await page.locator(`[data-testid="user-row-${testCredentials.username}"]`).click();
    await expect(page.locator('.shell.rail-open')).toHaveCount(1);
    await expect(
      page.locator(`[data-testid="user-detail-${testCredentials.username}"]`)
    ).toBeVisible();
  });
});
