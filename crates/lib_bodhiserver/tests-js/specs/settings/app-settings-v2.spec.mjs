import { LoginPage } from '@/pages/LoginPage.mjs';
import { SettingsPage } from '@/pages/SettingsPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

test.describe('App Settings V2', () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test('lists settings, gates the editor, and edits an editable setting @integration', async ({
    page,
    sharedServerUrl,
  }) => {
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const settingsPage = new SettingsPage(page, sharedServerUrl);

    await test.step('Login and open V2 App Settings', async () => {
      await loginPage.performOAuthLogin();
      await settingsPage.navigateToSettings();
      await settingsPage.expectSettingsPage();
    });

    await test.step('Read-only settings open a rail with no editor', async () => {
      await settingsPage.expectSettingVisible('BODHI_HOME');
      await settingsPage.expectReadOnly('BODHI_HOME');
    });

    await test.step('Edit BODHI_KEEP_ALIVE_SECS via the rail editor', async () => {
      await settingsPage.editSetting('BODHI_KEEP_ALIVE_SECS', 700);
      await settingsPage.navigateToSettings();
      await settingsPage.expectSettingValue('BODHI_KEEP_ALIVE_SECS', 700);
    });

    await test.step('Filter tabs: Modified narrows the list, All restores it', async () => {
      // BODHI_KEEP_ALIVE_SECS was just edited → it is "modified" and stays visible under Modified.
      await settingsPage.filterBy('all');
      const allCount = await settingsPage.visibleSettingCount();
      await settingsPage.filterBy('modified');
      await settingsPage.expectSettingVisible('BODHI_KEEP_ALIVE_SECS');
      const modifiedCount = await settingsPage.visibleSettingCount();
      expect(modifiedCount).toBeLessThanOrEqual(allCount);
      // Restore the All tab so the reset step below sees every row.
      await settingsPage.filterBy('all');
    });

    await test.step('Reset BODHI_KEEP_ALIVE_SECS back to its default', async () => {
      await settingsPage.openSetting('BODHI_KEEP_ALIVE_SECS');
      const reset = page
        .locator(settingsPage.selectors.rail('BODHI_KEEP_ALIVE_SECS'))
        .locator(settingsPage.selectors.reset);
      if (await reset.isVisible().catch(() => false)) {
        await reset.click();
        await settingsPage.waitForSPAReady();
      }
    });
  });
});
