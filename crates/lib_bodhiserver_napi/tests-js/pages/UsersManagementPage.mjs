import { expect } from '@playwright/test';
import { BasePage } from './BasePage.mjs';

export class UsersManagementPage extends BasePage {
  constructor(page, baseUrl) {
    super(page, baseUrl);
  }

  selectors = {
    // Navigation
    pendingRequestsLink: 'a[href="/ui/access-requests/pending"]',
    allRequestsLink: 'a[href="/ui/access-requests"]',
    usersLink: 'a[href="/ui/users"]',

    // Page containers
    pendingRequestsPage: '[data-testid="pending-requests-page"]',
    usersPage: '[data-testid="users-page"]',

    // Table structure
    tableRow: 'tbody tr',
    usernameCell: 'td:first-child',
    statusCell: 'td:has(span:has-text("Pending"))',
    actionsCell: 'td:last-child',

    // Shadcn UI Select component selectors (Radix primitives)
    roleSelectTrigger: 'button[role="combobox"]', // SelectTrigger
    roleSelectContent: '[role="listbox"]', // SelectContent (Radix Portal)
    roleSelectItem: '[role="option"]', // SelectItem
    roleSelectViewport: '[data-radix-select-viewport]', // SelectViewport

    // Action buttons
    approveButton: 'button:has-text("Approve")',
    rejectButton: 'button:has-text("Reject")',

    // Status messages
    successToast: '[data-state="open"]:has-text("Request Approved")',
    errorToast: '[data-state="open"]:has-text("Failed")',
    noRequestsMessage: 'h3:has-text("No Pending Requests")',

    // Loading states
    approvingButton: 'button:has-text("Approving...")',
    rejectingButton: 'button:has-text("Rejecting...")',
  };

  async navigateToPendingRequests() {
    await this.navigate('/ui/access-requests/pending');
    await this.waitForSPAReady();
    await this.expectVisible(this.selectors.pendingRequestsPage);
  }

  async navigateToAllRequests() {
    await this.navigate('/ui/access-requests');
    await this.waitForSPAReady();
  }

  async navigateToUsers() {
    await this.navigate('/ui/users');
    await this.waitForSPAReady();
    await this.expectVisible(this.selectors.usersPage);
  }

  async expectPendingRequestsPage() {
    await expect(this.page).toHaveURL(/\/ui\/access-requests\/pending/);
    await this.expectVisible(this.selectors.pendingRequestsPage);
  }

  async findRequestRowByUsername(username) {
    // Wait for table to be populated
    await this.page.waitForSelector(this.selectors.tableRow);

    // Find the row containing the username
    const rows = await this.page.locator(this.selectors.tableRow).all();

    for (const row of rows) {
      const usernameText = await row.locator(this.selectors.usernameCell).textContent();
      if (usernameText && usernameText.trim() === username) {
        return row;
      }
    }

    throw new Error(`No pending request found for username: ${username}`);
  }

  async expectRequestExists(username) {
    const row = await this.findRequestRowByUsername(username);
    await expect(row).toBeVisible();
  }

  async expectNoRequests() {
    await this.expectVisible(this.selectors.noRequestsMessage);
  }

  async selectRoleForRequest(username, roleDisplayName) {
    const row = await this.findRequestRowByUsername(username);

    // Click the role select trigger (button with combobox role) in this row
    const roleSelectTrigger = row.locator(this.selectors.roleSelectTrigger);
    await roleSelectTrigger.click();

    // Wait for the Radix Select portal content to be visible
    // The SelectContent is rendered in a Portal outside the table row
    await this.page.waitForSelector(this.selectors.roleSelectContent, {
      state: 'visible',
    });

    // Find role option by exact display name (e.g., "User" for resource_user)
    // Use exact text matching to avoid matching "Power User" when looking for "User"
    const roleOption = this.page
      .locator(this.selectors.roleSelectItem)
      .filter({ hasText: new RegExp(`^${roleDisplayName}$`) });

    // Wait for the option to be visible
    await roleOption.waitFor({ state: 'visible' });

    // Click the role option
    await roleOption.click();

    // Wait for dropdown to close (content becomes hidden)
    await this.page.waitForSelector(this.selectors.roleSelectContent, {
      state: 'hidden',
    });

    console.log(`Selected role: ${roleDisplayName} for user: ${username}`);
  }

  async approveRequest(username, roleDisplayName = 'User') {
    const row = await this.findRequestRowByUsername(username);

    // First select the role using display name
    await this.selectRoleForRequest(username, roleDisplayName);

    // Then click approve button in the same row
    const approveButton = row.locator(this.selectors.approveButton);
    await expect(approveButton).toBeEnabled();
    await approveButton.click();
  }

  async rejectRequest(username) {
    const row = await this.findRequestRowByUsername(username);

    // Click reject button in the row
    const rejectButton = row.locator(this.selectors.rejectButton);
    await expect(rejectButton).toBeEnabled();
    await rejectButton.click();
  }

  async waitForApprovalSuccess() {
    await this.waitForToast(/Request Approved/);
  }

  async waitForRejectionSuccess() {
    await this.waitForToast(/Request Rejected/);
  }

  async waitForApprovalError() {
    await this.waitForToast(/Failed/);
  }

  async expectApprovalInProgress(username) {
    const row = await this.findRequestRowByUsername(username);
    const approvingButton = row.locator(this.selectors.approvingButton);
    await expect(approvingButton).toBeVisible();
  }

  async expectRejectionInProgress(username) {
    const row = await this.findRequestRowByUsername(username);
    const rejectingButton = row.locator(this.selectors.rejectingButton);
    await expect(rejectingButton).toBeVisible();
  }

  async getRequestCount() {
    try {
      const rows = await this.page.locator(this.selectors.tableRow).all();
      return rows.length;
    } catch {
      return 0;
    }
  }

  async expectRequestNotInList(username) {
    // Wait a moment for the page to update
    await this.page.waitForTimeout(1000);

    // Check if the "No Pending Requests" message is shown (empty list case)
    const noRequestsVisible = await this.page.locator(this.selectors.noRequestsMessage).isVisible();
    if (noRequestsVisible) {
      console.log('No pending requests remaining - list is empty');
      return;
    }

    // If not empty, verify the specific request is not in the list
    try {
      await this.findRequestRowByUsername(username);
      throw new Error(`Request for ${username} should not exist but was found`);
    } catch (error) {
      if (error.message.includes('should not exist')) {
        throw error;
      }
      // Expected - request not found in non-empty list
      console.log(`Confirmed: Request for ${username} is not in the list`);
    }
  }

  async getAllPendingUsernames() {
    const rows = await this.page.locator(this.selectors.tableRow).all();
    const usernames = [];

    for (const row of rows) {
      const usernameText = await row.locator(this.selectors.usernameCell).textContent();
      if (usernameText) {
        usernames.push(usernameText.trim());
      }
    }

    return usernames;
  }

  async getAvailableRolesForRequest(username) {
    const row = await this.findRequestRowByUsername(username);

    // Click the role select trigger to open dropdown
    const roleSelectTrigger = row.locator(this.selectors.roleSelectTrigger);
    await roleSelectTrigger.click();

    // Wait for the dropdown to open
    await this.page.waitForSelector(this.selectors.roleSelectContent, {
      state: 'visible',
    });

    // Get all available role options
    const roleOptions = await this.page.locator(this.selectors.roleSelectItem).allTextContents();

    // Close dropdown by pressing Escape key (more reliable than clicking)
    await this.page.keyboard.press('Escape');

    // Wait for dropdown to close
    await this.page.waitForSelector(this.selectors.roleSelectContent, {
      state: 'hidden',
    });

    return roleOptions.map((role) => role.trim());
  }

  async expectRoleNotAvailable(username, roleName) {
    const availableRoles = await this.getAvailableRolesForRequest(username);

    if (availableRoles.includes(roleName)) {
      throw new Error(
        `Role "${roleName}" should not be available for user ${username}, but found in: ${availableRoles.join(', ')}`
      );
    }

    console.log(
      `Confirmed: Role "${roleName}" not available for ${username}. Available roles: ${availableRoles.join(', ')}`
    );
  }
}
