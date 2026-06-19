import { UsersManagementPage } from '@/pages/UsersManagementPage.mjs';
import { expect } from '@playwright/test';

export class AllAccessRequestsPage extends UsersManagementPage {
  // Extended selectors for all requests page
  allRequestsSelectors = {
    // Page structure
    pageContainer: '[data-testid="all-requests-page"]',
    // Pending-count pill published to the shell header (replaced the old request-count subtitle)
    pendingPill: '[data-testid="pending-pill"]',

    // V2 filter tabs (replaced the old nav-link tabs)
    filterTab: (id) => `[data-testid="requests-filter-${id}"]`,

    // List structure (V2 list rows; no <table>)
    requestsTable: '[data-testid="requests-table"]',

    // Row-level selectors (dynamic testid)
    requestRow: (username) => `[data-testid="request-row-${username}"]`,

    // Cell-level selectors
    usernameCell: '[data-testid="request-username"]',
    dateCell: '[data-testid="request-date"]',
    statusBadge: (status) => `[data-testid="request-status-${status}"]`,
    reviewerCell: '[data-testid="request-reviewer"]',

    // Action selectors for pending requests
    roleSelect: (username) => `[data-testid="role-select-${username}"]`,
    approveBtn: (username) => `[data-testid="approve-btn-${username}"]`,
    rejectBtn: (username) => `[data-testid="reject-btn-${username}"]`,

    // Detail rail (opens on row select; mirrors the row's role/approve/reject)
    detailRail: '[data-testid="request-detail-rail"]',
    detailRoleSelect: '[data-testid="request-detail-role-select"]',
    detailApprove: '[data-testid="request-detail-approve"]',
    detailReject: '[data-testid="request-detail-reject"]',
    detailClose: '[data-testid="request-detail-close"]',

    // State indicators
    emptyState: '[data-testid="no-requests"]',
    loadingSkeleton: '[data-testid="loading-skeleton"]',
    pagination: '[data-testid="pagination"]',
  };

  // Navigation methods
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

  /**
   * V2 list rows are `request-row-${username}` divs (no <table>). Override the
   * parent's table-based lookup so the inherited approve/reject/role-select
   * helpers operate on the correct row.
   */
  async findRequestRowByUsername(username) {
    const row = this.page.locator(this.allRequestsSelectors.requestRow(username));
    await row.waitFor({ state: 'visible' });
    return row;
  }

  async findRequestByUsername(username) {
    return await this.findRequestRowByUsername(username);
  }

  // Filter tabs (V2 replaces the old /ui/users/pending route with a `pending` filter) ──
  /** Click a status filter tab (`pending` | `approved` | `rejected` | `all`). */
  async filterBy(status) {
    await this.page.locator(this.allRequestsSelectors.filterTab(status)).click();
  }

  /** Navigate to the requests page on its default `pending` filter (replaces navigateToPendingRequests). */
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

  // In-row approval (V2 lists approve/reject + a native role <select> per pending row) ──
  /**
   * Set a pending row's role <select> to the given role value
   * (e.g. 'resource_manager', 'resource_power_user'). The page binds every row's
   * select to a single shared `selectedRole`, so this must be set immediately
   * before approving that row.
   */
  async selectRole(username, roleValue) {
    await this.page.locator(this.allRequestsSelectors.roleSelect(username)).selectOption(roleValue);
  }

  /** Labels of the role options offered for a pending request (mirrors getAvailableRoles). */
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

  /** Approve a pending request with a role value, waiting for the success toast. */
  async approveRequest(username, roleValue) {
    await this.selectRole(username, roleValue);
    await this.page.locator(this.allRequestsSelectors.approveBtn(username)).click();
    await this.waitForToast(/Request Approved/);
  }

  /** Reject a pending request, waiting for the success toast. */
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
    // Check which status badge is visible
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

  // Detail-rail helpers (the rail opens when a row is selected) ──────────────
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
    // Check if approve/reject buttons are present (only for pending)
    const approveBtn = row.locator('button:has-text("Approve")');
    return await approveBtn.isVisible();
  }

  // Verification methods
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
    // Date validation logic - pending shows created_at, others show updated_at
    // The actual date will differ, but we verify it exists
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
