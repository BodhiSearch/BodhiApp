import { UsersManagementPage } from '@/pages/UsersManagementPage.mjs';
import { expect } from '@playwright/test';

export class AllAccessRequestsPage extends UsersManagementPage {
  allRequestsSelectors = {
    pageContainer: '[data-testid="all-requests-page"]',
    // Published to the shell header.
    pendingPill: '[data-testid="pending-pill"]',

    filterTab: (id) => `[data-testid="requests-filter-${id}"]`,

    // V2 list rows; no <table>.
    requestsTable: '[data-testid="requests-table"]',

    requestRow: (username) => `[data-testid="request-row-${username}"]`,

    usernameCell: '[data-testid="request-username"]',
    dateCell: '[data-testid="request-date"]',
    statusBadge: (status) => `[data-testid="request-status-${status}"]`,
    reviewerCell: '[data-testid="request-reviewer"]',

    roleSelect: (username) => `[data-testid="role-select-${username}"]`,
    approveBtn: (username) => `[data-testid="approve-btn-${username}"]`,
    rejectBtn: (username) => `[data-testid="reject-btn-${username}"]`,

    // Mirrors the row's role/approve/reject; opens on row select.
    detailRail: '[data-testid="request-detail-rail"]',
    detailRoleSelect: '[data-testid="request-detail-role-select"]',
    detailApprove: '[data-testid="request-detail-approve"]',
    detailReject: '[data-testid="request-detail-reject"]',
    detailClose: '[data-testid="request-detail-close"]',

    emptyState: '[data-testid="no-requests"]',
    loadingSkeleton: '[data-testid="loading-skeleton"]',
    pagination: '[data-testid="pagination"]',
  };

  async navigateToAllRequests() {
    await this.navigate('/ui/users/access-requests/');
    await this.waitForSPAReady();
    await this.page.waitForSelector('[data-testid="all-requests-page"][data-pagestatus="ready"]');
  }

  async navigateToAllRequestsViaShell() {
    await this.navViaShell('users', 'access-requests');
    await this.page.waitForSelector('[data-testid="all-requests-page"][data-pagestatus="ready"]');
  }

  async expectAllRequestsPage() {
    await expect(this.page).toHaveURL(/\/ui\/users\/access-requests\/?$/);
    await this.expectVisible(this.allRequestsSelectors.pageContainer);
  }

  // V2 rows are divs, not <table> rows — overrides the parent's table-based lookup
  // so the inherited approve/reject/role-select helpers target the right row.
  async findRequestRowByUsername(username) {
    const row = this.page.locator(this.allRequestsSelectors.requestRow(username));
    await row.waitFor({ state: 'visible' });
    return row;
  }

  async findRequestByUsername(username) {
    return await this.findRequestRowByUsername(username);
  }

  // status: 'pending' | 'approved' | 'rejected' | 'all'
  async filterBy(status) {
    await this.page.locator(this.allRequestsSelectors.filterTab(status)).click();
  }

  async navigateToPending() {
    await this.navigateToAllRequests();
    await this.filterBy('pending');
  }

  async expectRequestVisible(username) {
    await expect(this.page.locator(this.allRequestsSelectors.requestRow(username))).toBeVisible();
  }

  async expectRequestNotVisible(username) {
    await expect(this.page.locator(this.allRequestsSelectors.requestRow(username))).toHaveCount(0);
  }

  async expectEmpty() {
    await this.expectVisible(this.allRequestsSelectors.emptyState);
  }

  // The page binds every row's select to a single shared `selectedRole`, so this
  // must be set immediately before approving that row.
  async selectRole(username, roleValue) {
    await this.page.locator(this.allRequestsSelectors.roleSelect(username)).selectOption(roleValue);
  }

  async getAvailableRoleLabels(username) {
    const labels = await this.page
      .locator(`${this.allRequestsSelectors.roleSelect(username)} option`)
      .allTextContents();
    return labels.map((l) => l.trim());
  }

  async expectRoleNotAvailable(username, roleLabel) {
    const labels = await this.getAvailableRoleLabels(username);
    expect(labels).not.toContain(roleLabel);
  }

  async approveRequest(username, roleValue) {
    await this.selectRole(username, roleValue);
    await this.page.locator(this.allRequestsSelectors.approveBtn(username)).click();
    await this.waitForToast(/Request Approved/);
  }

  async rejectRequest(username) {
    await this.page.locator(this.allRequestsSelectors.rejectBtn(username)).click();
    await this.waitForToast(/Request Rejected/);
  }

  async getRequestData(username) {
    const row = await this.findRequestByUsername(username);

    const data = {
      username: await row.locator(this.allRequestsSelectors.usernameCell).textContent(),
      date: await row.locator(this.allRequestsSelectors.dateCell).textContent(),
      status: await this.getRequestStatus(row),
      reviewer: await this.getRequestReviewer(row),
      hasActions: await this.hasActions(row),
    };

    return data;
  }

  async getRequestStatus(row) {
    for (const status of ['pending', 'approved', 'rejected']) {
      const badge = row.locator(this.allRequestsSelectors.statusBadge(status));
      if (await badge.isVisible()) {
        return status;
      }
    }
    return null;
  }

  async getRequestReviewer(row) {
    const reviewerCell = row.locator(this.allRequestsSelectors.reviewerCell);
    if (await reviewerCell.isVisible()) {
      const text = await reviewerCell.textContent();
      return text?.trim() || null;
    }
    return null;
  }

  async openDetailRail(username) {
    await (await this.findRequestRowByUsername(username)).click();
    await this.page.waitForSelector(this.allRequestsSelectors.detailRail);
  }

  async approveFromRail() {
    await this.page.locator(this.allRequestsSelectors.detailApprove).click();
  }

  async rejectFromRail() {
    await this.page.locator(this.allRequestsSelectors.detailReject).click();
  }

  async hasActions(row) {
    const approveBtn = row.locator('button:has-text("Approve")');
    return await approveBtn.isVisible();
  }

  async verifyRequestStatus(username, expectedStatus, expectedReviewer = null) {
    const data = await this.getRequestData(username);

    expect(data.status).toBe(expectedStatus);

    if (expectedStatus !== 'pending' && expectedReviewer) {
      expect(data.reviewer).toBe(expectedReviewer);
    } else if (expectedStatus === 'pending') {
      expect(data.reviewer).toBeNull();
      expect(data.hasActions).toBe(true);
    }
  }

  async verifyAllRequests(expectedRequests) {
    // expectedRequests: Array of { username, status, reviewer? }
    for (const expected of expectedRequests) {
      await this.verifyRequestStatus(expected.username, expected.status, expected.reviewer);
    }
  }

  async getTotalRequestCount() {
    await this.page.waitForSelector(this.allRequestsSelectors.requestsTable);
    if (
      await this.page
        .locator(this.allRequestsSelectors.emptyState)
        .isVisible()
        .catch(() => false)
    ) {
      return 0;
    }
    // V2 list rows (no <table>)
    return await this.page.locator('[data-testid^="request-row-"]').count();
  }

  async verifyRequestCount(expectedCount) {
    const actualCount = await this.getTotalRequestCount();
    expect(actualCount).toBe(expectedCount);
  }

  async verifyDateDisplay(username, isPending) {
    const data = await this.getRequestData(username);
    // pending shows created_at, others show updated_at, so only presence is checked
    expect(data.date).toBeTruthy();
  }

  async verifyEmptyState() {
    await this.expectVisible(this.allRequestsSelectors.emptyState);
  }

  async verifyPageTitle() {
    // V2 identifies the page by its container + breadcrumb (no page-title cell).
    await this.expectVisible(this.allRequestsSelectors.pageContainer);
  }

  async verifyPendingPill(expectedPending) {
    const pill = this.page.locator(this.allRequestsSelectors.pendingPill);
    if (expectedPending > 0) {
      await expect(pill).toHaveText(`${expectedPending} pending review`);
    } else {
      await expect(pill).toHaveCount(0);
    }
  }
}
