import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

/**
 * Manage Users (V2) page object. The V2 screen is an AppShell list: rows are `div`s
 * (`user-row-{username}`), and role-change / remove happen in the right detail rail opened
 * by clicking a row.
 */
export class AllUsersPage extends BasePage {
  selectors = {
    usersPage: '[data-testid="users-page"]',
    row: (username) => `[data-testid="user-row-${username}"]`,
    rowUsername: '[data-testid="user-username"]',
    roleCell: (username) => `[data-testid="user-role-${username}"]`,
    role: '[data-testid="user-role"]',

    // rail (opens on row select)
    rail: (username) => `[data-testid="user-detail-${username}"]`,
    railClose: '[data-testid="user-detail-close"]',
    roleSelect: (username) => `[data-testid="role-select-${username}"]`,
    saveRole: (username) => `[data-testid="save-role-${username}"]`,
    removeBtn: (username) => `[data-testid="remove-user-btn-${username}"]`,
    noActions: (username) => `[data-testid="no-actions-${username}"]`,
    currentUserIndicator: '[data-testid="current-user-indicator"]',
    restrictedUserIndicator: '[data-testid="restricted-user-indicator"]',

    // filters / empty
    filterAll: '[data-testid="users-filter-all"]',
    noUsers: '[data-testid="no-users"]',
  };

  async navigateToUsers() {
    // Skip the rail view-transition so the close button doesn't detach mid-animation (the hook
    // honors prefers-reduced-motion and applies the update synchronously).
    await this.page.emulateMedia({ reducedMotion: 'reduce' });
    await this.navigate('/ui/users');
    await this.waitForSPAReady();
    await this.expectVisible(this.selectors.usersPage);
    await expect(this.page.locator(this.selectors.usersPage)).toHaveAttribute(
      'data-pagestatus',
      'ready'
    );
  }

  async expectUsersPage() {
    await expect(this.page).toHaveURL(/\/ui\/users/);
    await this.expectVisible(this.selectors.usersPage);
  }

  async expectUserExists(username) {
    await expect(this.page.locator(this.selectors.row(username))).toBeVisible();
  }

  async expectUserNotExists(username) {
    await expect(this.page.locator(this.selectors.row(username))).toHaveCount(0);
  }

  async expectUserRole(username, expectedRole) {
    const roleEl = this.page
      .locator(this.selectors.roleCell(username))
      .locator(this.selectors.role);
    await expect(roleEl).toHaveText(expectedRole);
  }

  /** Open a user's detail rail by clicking the row (no rail must already be open). */
  async openUser(username) {
    await this.closeRail();
    const row = this.page.locator(this.selectors.row(username));
    await expect(row).toBeVisible();
    await row.click();
    await this.expectVisible(this.selectors.rail(username));
  }

  /** Close any open detail rail and wait for the view-transition to fully remove it. */
  async closeRail() {
    const close = this.page.locator(this.selectors.railClose);
    // The rail open/close is a view transition; the close button detaches mid-animation.
    // Click only if present, then wait for it to be gone (toHaveCount(0) auto-retries).
    if ((await close.count()) > 0) {
      await close.click({ force: true }).catch(() => {});
    }
    await expect(close).toHaveCount(0);
  }

  /** Change a user's role via the rail's native select + Save. */
  async changeUserRole(username, newRoleValue, expectedLabel) {
    await this.closeRail();
    await this.openUser(username);
    const select = this.page.locator(this.selectors.roleSelect(username));
    await select.selectOption(newRoleValue);
    await expect(select).toHaveValue(newRoleValue);
    const save = this.page.locator(this.selectors.saveRole(username));
    await expect(save).toBeEnabled();
    await save.click();
    // The mutation invalidates + refetches the list; the rail re-renders with the new role,
    // which flips Save back to its disabled "Saved" state. Wait for that to confirm the PUT settled.
    await expect(save).toBeDisabled();
    if (expectedLabel) {
      await expect(
        this.page.locator(this.selectors.roleCell(username)).locator(this.selectors.role)
      ).toHaveText(expectedLabel);
    }
    await this.closeRail();
    await this.waitForSPAReady();
  }

  /** Remove a user via the rail's two-click confirm. */
  async removeUser(username) {
    await this.openUser(username);
    const removeBtn = this.page.locator(this.selectors.removeBtn(username));
    await expect(removeBtn).toHaveText(/Remove user/);
    await removeBtn.click();
    await expect(removeBtn).toHaveText(/Confirm remove/);
    await removeBtn.click();
    await this.waitForSPAReady();
  }

  async getUserCount() {
    await this.page.waitForSelector('[data-testid^="user-row-"]');
    return this.page.locator('[data-testid^="user-row-"]').count();
  }

  async getAllUsernames() {
    const cells = await this.page
      .locator('[data-testid^="user-row-"] [data-testid="user-username"]')
      .allTextContents();
    return cells.map((t) => t.trim());
  }

  async verifyUsersInHierarchicalOrder(expectedUsernames) {
    const actual = await this.getAllUsernames();
    expect(actual).toEqual(expectedUsernames);
  }

  /** A user the actor can't modify (self or higher-ranked): rail shows a read-only note, no controls. */
  async expectNoActionsForUser(username) {
    await this.openUser(username);
    await expect(this.page.locator(this.selectors.noActions(username))).toBeVisible();
    await expect(this.page.locator(this.selectors.roleSelect(username))).toHaveCount(0);
    await expect(this.page.locator(this.selectors.removeBtn(username))).toHaveCount(0);
    await this.closeRail();
  }

  /** A modifiable user: rail shows the role select + remove button. */
  async expectActionsForUser(username) {
    await this.openUser(username);
    await expect(this.page.locator(this.selectors.roleSelect(username))).toBeVisible();
    await expect(this.page.locator(this.selectors.removeBtn(username))).toBeVisible();
    await this.closeRail();
  }

  async expectCurrentUserIndicator(username) {
    await this.openUser(username);
    const indicator = this.page
      .locator(this.selectors.rail(username))
      .locator(this.selectors.currentUserIndicator);
    await expect(indicator).toBeVisible();
    await this.closeRail();
  }

  async expectRestrictedUserIndicator(username) {
    await this.openUser(username);
    const indicator = this.page
      .locator(this.selectors.rail(username))
      .locator(this.selectors.restrictedUserIndicator);
    await expect(indicator).toBeVisible();
    await this.closeRail();
  }

  /** Roles the actor may assign to a user, read from the rail's native <option>s. */
  async getAvailableRolesForUser(username) {
    await this.openUser(username);
    const options = await this.page
      .locator(this.selectors.roleSelect(username))
      .locator('option')
      .allTextContents();
    await this.closeRail();
    return options.map((o) => o.trim());
  }

  async expectRoleAvailable(username, roleLabel) {
    const available = await this.getAvailableRolesForUser(username);
    expect(available).toContain(roleLabel);
  }

  async expectRoleNotAvailable(username, roleLabel) {
    const available = await this.getAvailableRolesForUser(username);
    expect(available).not.toContain(roleLabel);
  }

  async changeUserRoleAndVerify(username, newRoleValue, expectedLabel) {
    await this.changeUserRole(username, newRoleValue);
    await this.expectUserRole(username, expectedLabel);
  }

  async removeUserAndVerify(username) {
    const countBefore = await this.getUserCount();
    await this.removeUser(username);
    await expect(this.page.locator(this.selectors.row(username))).toHaveCount(0);
    const countAfter = await this.getUserCount();
    expect(countAfter).toBe(countBefore - 1);
  }
}
