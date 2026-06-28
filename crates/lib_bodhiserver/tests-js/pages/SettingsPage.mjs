import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

/**
 * App Settings (V2) page object. The V2 screen is an AppShell list: setting rows
 * (`setting-row-{key}`) open a right detail rail on click. Only the backend-editable settings
 * (BODHI_EXEC_VARIANT, BODHI_KEEP_ALIVE_SECS) render the rail editor; every other row shows a
 * read-only note.
 */
export class SettingsPage extends BasePage {
  selectors = {
    page: '[data-testid="settings-page"]',
    groupNav: '[data-testid="settings-group-nav"]',
    row: (key) => `[data-testid="setting-row-${key}"]`,
    source: (key) => `[data-testid="setting-source-${key}"]`,
    value: (key) => `[data-testid="setting-value-${key}"]`,
    rail: (key) => `[data-testid="setting-detail-${key}"]`,
    railClose: '[data-testid="setting-detail-close"]',
    newValue: '[data-testid="setting-new-value"]',
    save: '[data-testid="setting-save"]',
    cancel: '[data-testid="setting-cancel"]',
    reset: '[data-testid="setting-reset"]',
    readonlyNote: '[data-testid="setting-readonly-note"]',
    filterModified: '[data-testid="settings-filter-modified"]',
    filterAll: '[data-testid="settings-filter-all"]',
  };

  async navigateToSettings() {
    // Skip the rail view-transition so the close button doesn't detach mid-animation.
    await this.page.emulateMedia({ reducedMotion: 'reduce' });
    await this.navigate('/ui/settings');
    await this.waitForSPAReady();
    await this.expectVisible(this.selectors.page);
    await expect(this.page.locator(this.selectors.page)).toHaveAttribute(
      'data-pagestatus',
      'ready'
    );
  }

  async expectSettingsPage() {
    await expect(this.page).toHaveURL(/\/ui\/settings/);
    await this.expectVisible(this.selectors.page);
    await this.expectVisible(this.selectors.groupNav);
  }

  async expectSettingVisible(key) {
    await expect(this.page.locator(this.selectors.row(key))).toBeVisible();
  }

  async openSetting(key) {
    await this.page.locator(this.selectors.row(key)).click();
    await this.expectVisible(this.selectors.rail(key));
  }

  /** A read-only setting opens a rail with a note and NO editor. */
  async expectReadOnly(key) {
    await this.openSetting(key);
    const rail = this.page.locator(this.selectors.rail(key));
    await expect(rail.locator(this.selectors.readonlyNote)).toBeVisible();
    await expect(rail.locator(this.selectors.newValue)).toHaveCount(0);
    await expect(rail.locator(this.selectors.save)).toHaveCount(0);
    await this.page.locator(this.selectors.railClose).click();
  }

  /** Edit an editable setting via the rail and Save. */
  async editSetting(key, newValue) {
    await this.openSetting(key);
    const rail = this.page.locator(this.selectors.rail(key));
    const input = rail.locator(this.selectors.newValue);
    await input.fill(String(newValue));
    const save = rail.locator(this.selectors.save);
    await expect(save).toBeEnabled();
    await save.click();
    // Wait for the mutation to actually land before moving on: the list row reflects the new value
    // and the Save button drops back to its non-dirty "Saved" (disabled) state. Without this, a
    // following navigate()/refetch can race the in-flight PUT and read the stale value under load.
    await expect(this.page.locator(this.selectors.value(key))).toHaveText(String(newValue));
    await expect(save).toBeDisabled();
    await this.waitForSPAReady();
  }

  async expectSettingValue(key, expectedValue) {
    await expect(this.page.locator(this.selectors.value(key))).toHaveText(String(expectedValue));
  }

  /** Switch the list between the "Modified" and "All" filter tabs. */
  async filterBy(which) {
    const selector =
      which === 'modified' ? this.selectors.filterModified : this.selectors.filterAll;
    await this.page.locator(selector).click();
    await this.waitForSPAReady();
  }

  /** Count the currently-visible setting rows. */
  async visibleSettingCount() {
    return this.page.locator('[data-testid^="setting-row-"]').count();
  }
}
