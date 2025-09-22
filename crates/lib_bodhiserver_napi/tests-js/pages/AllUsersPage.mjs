import { expect } from '@playwright/test';
import { BasePage } from './BasePage.mjs';

export class AllUsersPage extends BasePage {
  constructor(page, baseUrl) {
    super(page, baseUrl);
  }

  selectors = {
    // Page container
    usersPage: '[data-testid="users-page"]',

    // Navigation links
    pendingRequestsLink: 'a[href="/ui/users/pending"]',
    allRequestsLink: 'a[href="/ui/users/access-requests"]',
    usersLink: 'a[href="/ui/users"]',

    // Table structure
    tableRow: 'tbody tr',
    userUsername: '[data-testid="user-username"]',
    userRole: '[data-testid="user-role"]',
    userStatus: '[data-testid="user-status"]',
    userCreated: '[data-testid="user-created"]',
    userActions: '[data-testid="user-actions"]',

    // Role selection elements
    roleSelect: (username) => `[data-testid="role-select-${username}"]`,
    roleSelectTrigger: 'button[role="combobox"]',
    roleSelectContent: '[role="listbox"]',
    roleSelectItem: '[role="option"]',

    // Action buttons
    removeUserBtn: (username) => `[data-testid="remove-user-btn-${username}"]`,
    removingButton: 'button:has-text("Removing...")',

    // Role change confirmation dialog
    roleChangeDialog: '[data-testid="role-change-dialog"]',
    roleChangeTitle: '[data-testid="role-change-title"]',
    roleChangeDescription: '[data-testid="role-change-description"]',
    roleChangeCancel: '[data-testid="role-change-cancel"]',
    roleChangeConfirm: '[data-testid="role-change-confirm"]',
    changingRoleButton: 'button:has-text("Changing...")',

    // Remove user confirmation dialog
    removeUserDialog: '[data-testid="remove-user-dialog"]',
    removeUserTitle: '[data-testid="remove-user-title"]',
    removeUserDescription: '[data-testid="remove-user-description"]',
    removeUserCancel: '[data-testid="remove-user-cancel"]',
    removeUserConfirm: '[data-testid="remove-user-confirm"]',

    // Status messages
    successToast:
      '[data-state="open"]:has-text("Role Updated"), [data-state="open"]:has-text("User Removed")',
    errorToast:
      '[data-state="open"]:has-text("Update Failed"), [data-state="open"]:has-text("Removal Failed")',

    // Empty state
    noUsersMessage: 'h3:has-text("No Users")',
    noUsersDescription: 'p:has-text("User management API not yet implemented")',

    // Loading states
    skeleton: '.animate-pulse',
  };

  async navigateToUsers() {
    await this.navigate('/ui/users');
    await this.waitForSPAReady();
    await this.expectVisible(this.selectors.usersPage);
  }

  async expectUsersPage() {
    await expect(this.page).toHaveURL(/\/ui\/users/);
    await this.expectVisible(this.selectors.usersPage);
  }

  async findUserRowByUsername(username) {
    // Wait for table to be populated
    await this.page.waitForSelector(this.selectors.tableRow);

    // Find the row containing the username
    const rows = await this.page.locator(this.selectors.tableRow).all();

    for (const row of rows) {
      const usernameElement = row.locator(this.selectors.userUsername);
      const usernameText = await usernameElement.textContent();
      if (usernameText && usernameText.trim() === username) {
        return row;
      }
    }

    throw new Error(`No user found with username: ${username}`);
  }

  async expectUserExists(username) {
    const row = await this.findUserRowByUsername(username);
    await expect(row).toBeVisible();
  }

  async expectUserNotExists(username) {
    try {
      await this.findUserRowByUsername(username);
      throw new Error(`User ${username} should not exist but was found`);
    } catch (error) {
      if (error.message.includes('should not exist')) {
        throw error;
      }
      // Expected - user not found
      console.log(`Confirmed: User ${username} is not in the list`);
    }
  }

  async getUserRole(username) {
    const row = await this.findUserRowByUsername(username);
    const roleElement = row.locator(this.selectors.userRole);
    return await roleElement.textContent();
  }

  async expectUserRole(username, expectedRole) {
    const actualRole = await this.getUserRole(username);
    expect(actualRole).toContain(expectedRole);
  }

  async getUserStatus(username) {
    const row = await this.findUserRowByUsername(username);
    const statusElement = row.locator(this.selectors.userStatus);
    return await statusElement.textContent();
  }

  async expectUserStatus(username, expectedStatus) {
    const actualStatus = await this.getUserStatus(username);
    expect(actualStatus).toContain(expectedStatus);
  }

  async selectRoleForUser(username, roleDisplayName) {
    const row = await this.findUserRowByUsername(username);

    // Click the role select trigger in this row
    const roleSelectTrigger = row.locator(this.selectors.roleSelectTrigger);
    await roleSelectTrigger.click();

    // Wait for the select content to be visible
    await this.page.waitForSelector(this.selectors.roleSelectContent, {
      state: 'visible',
    });

    // Find role option by exact display name
    const roleOption = this.page
      .locator(this.selectors.roleSelectItem)
      .filter({ hasText: new RegExp(`^${roleDisplayName}$`) });

    // Wait for the option to be visible and click it
    await roleOption.waitFor({ state: 'visible' });
    await roleOption.click();

    // Wait for dropdown to close
    await this.page.waitForSelector(this.selectors.roleSelectContent, {
      state: 'hidden',
    });

    console.log(`Selected role: ${roleDisplayName} for user: ${username}`);
  }

  async changeUserRole(username, newRoleDisplayName) {
    // Select the new role
    await this.selectRoleForUser(username, newRoleDisplayName);

    // Wait for role change dialog to appear
    await this.expectVisible(this.selectors.roleChangeDialog);
    await this.expectVisible(this.selectors.roleChangeTitle);

    // Click confirm button
    const confirmButton = this.page.locator(this.selectors.roleChangeConfirm);
    await expect(confirmButton).toBeEnabled();
    await confirmButton.click();

    // Wait for any success toast to disappear
    await this.waitForToastToHide();

    // Wait for dialog to close
    await this.page.waitForSelector(this.selectors.roleChangeDialog, {
      state: 'hidden',
    });
  }

  async removeUser(username) {
    const row = await this.findUserRowByUsername(username);

    // Click remove button
    const removeButton = row.locator(this.selectors.removeUserBtn(username));
    await expect(removeButton).toBeEnabled();
    await removeButton.click();

    // Wait for remove dialog to appear
    await this.expectVisible(this.selectors.removeUserDialog);
    await this.expectVisible(this.selectors.removeUserTitle);

    // Click confirm button
    const confirmButton = this.page.locator(this.selectors.removeUserConfirm);
    await expect(confirmButton).toBeEnabled();
    await confirmButton.click();

    // Wait for dialog to close
    await this.page.waitForSelector(this.selectors.removeUserDialog, {
      state: 'hidden',
    });
  }

  async expectRoleChangeDialog() {
    await this.expectVisible(this.selectors.roleChangeDialog);
    await this.expectVisible(this.selectors.roleChangeTitle);
    await this.expectVisible(this.selectors.roleChangeDescription);
  }

  async expectRemoveUserDialog() {
    await this.expectVisible(this.selectors.removeUserDialog);
    await this.expectVisible(this.selectors.removeUserTitle);
    await this.expectVisible(this.selectors.removeUserDescription);
  }

  async cancelRoleChange() {
    const cancelButton = this.page.locator(this.selectors.roleChangeCancel);
    await expect(cancelButton).toBeEnabled();
    await cancelButton.click();

    // Wait for dialog to close
    await this.page.waitForSelector(this.selectors.roleChangeDialog, {
      state: 'hidden',
    });
  }

  async cancelRemoveUser() {
    const cancelButton = this.page.locator(this.selectors.removeUserCancel);
    await expect(cancelButton).toBeEnabled();
    await cancelButton.click();

    // Wait for dialog to close
    await this.page.waitForSelector(this.selectors.removeUserDialog, {
      state: 'hidden',
    });
  }

  async expectRoleChangeInProgress() {
    // Check if changing button is visible
    await this.expectVisible(this.selectors.changingRoleButton);
  }

  async expectRemovalInProgress(username) {
    const row = await this.findUserRowByUsername(username);
    const removingButton = row.locator(this.selectors.removingButton);
    await expect(removingButton).toBeVisible();
  }

  async waitForRoleChangeSuccess() {
    await this.waitForToast(/Role Updated/);
  }

  async waitForUserRemovalSuccess() {
    await this.waitForToast(/User Removed/);
  }

  async waitForRoleChangeError() {
    await this.waitForToast(/Update Failed/);
  }

  async waitForRemovalError() {
    await this.waitForToast(/Removal Failed/);
  }

  async getUserCount() {
    // Wait for at least one table row to ensure table is loaded
    await this.page.waitForSelector(this.selectors.tableRow, { timeout: 5000 });

    // Now count all rows
    const rows = await this.page.locator(this.selectors.tableRow).count();
    return rows;
  }

  async expectNoUsers() {
    await this.expectVisible(this.selectors.noUsersMessage);
    await this.expectVisible(this.selectors.noUsersDescription);
  }

  async getAllUsernames() {
    const rows = await this.page.locator(this.selectors.tableRow).all();
    const usernames = [];

    for (const row of rows) {
      const usernameElement = row.locator(this.selectors.userUsername);
      const usernameText = await usernameElement.textContent();
      if (usernameText) {
        usernames.push(usernameText.trim());
      }
    }

    return usernames;
  }

  async getAvailableRolesForUser(username) {
    const row = await this.findUserRowByUsername(username);

    // Click the role select trigger to open dropdown
    const roleSelectTrigger = row.locator(this.selectors.roleSelectTrigger);
    await roleSelectTrigger.click();

    // Wait for the dropdown to open
    await this.page.waitForSelector(this.selectors.roleSelectContent, {
      state: 'visible',
    });

    // Get all available role options
    const roleOptions = await this.page.locator(this.selectors.roleSelectItem).allTextContents();

    // Close dropdown by pressing Escape key
    await this.page.keyboard.press('Escape');

    // Wait for dropdown to close
    await this.page.waitForSelector(this.selectors.roleSelectContent, {
      state: 'hidden',
    });

    return roleOptions.map((role) => role.trim());
  }

  async expectRoleNotAvailable(username, roleName) {
    const availableRoles = await this.getAvailableRolesForUser(username);

    if (availableRoles.includes(roleName)) {
      throw new Error(
        `Role "${roleName}" should not be available for user ${username}, but found in: ${availableRoles.join(', ')}`
      );
    }

    console.log(
      `Confirmed: Role "${roleName}" not available for ${username}. Available roles: ${availableRoles.join(', ')}`
    );
  }

  async expectRoleAvailable(username, roleName) {
    const availableRoles = await this.getAvailableRolesForUser(username);

    if (!availableRoles.includes(roleName)) {
      throw new Error(
        `Role "${roleName}" should be available for user ${username}, but not found in: ${availableRoles.join(', ')}`
      );
    }

    console.log(
      `Confirmed: Role "${roleName}" available for ${username}. Available roles: ${availableRoles.join(', ')}`
    );
  }

  async verifyUsersWithRoles(expectedUsers) {
    // Verify the total count matches
    const actualCount = await this.getUserCount();
    expect(actualCount).toBe(expectedUsers.length);

    // Verify each user exists with correct role
    for (const expectedUser of expectedUsers) {
      await this.expectUserExists(expectedUser.username);
      if (expectedUser.role) {
        await this.expectUserRole(expectedUser.username, expectedUser.role);
      }
      if (expectedUser.status) {
        await this.expectUserStatus(expectedUser.username, expectedUser.status);
      }
    }
  }

  async expectUsersPageLoading() {
    await this.expectVisible(this.selectors.skeleton);
  }

  async waitForUsersPageLoaded() {
    // Wait for loading to finish
    await this.page.waitForSelector(this.selectors.skeleton, {
      state: 'hidden',
    });
  }

  // Additional methods for UI restriction testing

  async navigateToPendingRequests() {
    await this.navigate('/ui/users/pending');
    await this.waitForSPAReady();
  }

  async navigateToAllRequests() {
    await this.navigate('/ui/users/access-requests');
    await this.waitForSPAReady();
  }

  async expectNoActionsForUser(username) {
    // Wait for the user row to be fully loaded
    await this.page.waitForSelector(
      `tbody tr:has([data-testid="user-username"]:has-text("${username}"))`,
      { timeout: 10000 }
    );

    const row = await this.findUserRowByUsername(username);
    const actionsCell = row.locator(`[data-testid="user-actions-${username}"]`);

    // Check for "no actions" indicators (either "You" or "Restricted")
    const noActionsIndicator = actionsCell.locator(`[data-testid="no-actions-${username}"]`);
    await expect(noActionsIndicator).toBeVisible();

    // Ensure role select and remove buttons are not present
    const roleSelect = actionsCell.locator(`[data-testid="role-select-${username}"]`);
    const removeButton = actionsCell.locator(`[data-testid="remove-user-btn-${username}"]`);

    await expect(roleSelect).not.toBeVisible();
    await expect(removeButton).not.toBeVisible();

    console.log(
      `Confirmed: User ${username} has no action buttons (self-modification/hierarchy restriction)`
    );
  }

  async expectActionsForUser(username) {
    // Wait for the user row to be fully loaded
    await this.page.waitForSelector(
      `tbody tr:has([data-testid="user-username"]:has-text("${username}"))`,
      { timeout: 10000 }
    );

    const row = await this.findUserRowByUsername(username);
    const actionsCell = row.locator(`[data-testid="user-actions-${username}"]`);

    // Check for actions container
    const actionsContainer = actionsCell.locator(
      `[data-testid="user-actions-container-${username}"]`
    );
    await expect(actionsContainer).toBeVisible();

    // Ensure role select and remove buttons are present
    const roleSelect = actionsCell.locator(`[data-testid="role-select-${username}"]`);
    const removeButton = actionsCell.locator(`[data-testid="remove-user-btn-${username}"]`);

    await expect(roleSelect).toBeVisible();
    await expect(removeButton).toBeVisible();

    console.log(`Confirmed: User ${username} has visible action buttons`);
  }

  async expectCurrentUserIndicator(username) {
    const row = await this.findUserRowByUsername(username);
    const actionsCell = row.locator(`[data-testid="user-actions-${username}"]`);
    const currentUserIndicator = actionsCell.locator('[data-testid="current-user-indicator"]');

    await expect(currentUserIndicator).toBeVisible();
    const text = await currentUserIndicator.textContent();
    expect(text.trim()).toBe('You');

    console.log(`Confirmed: User ${username} shows "You" indicator`);
  }

  async expectRestrictedUserIndicator(username) {
    const row = await this.findUserRowByUsername(username);
    const actionsCell = row.locator(`[data-testid="user-actions-${username}"]`);
    const restrictedIndicator = actionsCell.locator('[data-testid="restricted-user-indicator"]');

    await expect(restrictedIndicator).toBeVisible();
    const text = await restrictedIndicator.textContent();
    expect(text.trim()).toBe('Restricted');

    console.log(`Confirmed: User ${username} shows "Restricted" indicator`);
  }

  async getUserActionsVisibility(username) {
    try {
      await this.expectActionsForUser(username);
      return 'visible';
    } catch {
      try {
        await this.expectNoActionsForUser(username);
        const row = await this.findUserRowByUsername(username);
        const actionsCell = row.locator(`[data-testid="user-actions-${username}"]`);
        const currentUserIndicator = actionsCell.locator('[data-testid="current-user-indicator"]');

        if (await currentUserIndicator.isVisible()) {
          return 'self';
        } else {
          return 'restricted';
        }
      } catch {
        return 'unknown';
      }
    }
  }

  async verifyUsersInHierarchicalOrder(expectedUsernames) {
    const actualUsernames = await this.getAllUsernames();
    expect(actualUsernames).toEqual(expectedUsernames);
    console.log(`Confirmed: Users displayed in hierarchical order: ${actualUsernames.join(' → ')}`);
  }
}
