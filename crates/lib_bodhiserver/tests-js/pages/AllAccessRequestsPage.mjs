import { UsersManagementPage } from '@/pages/UsersManagementPage.mjs';
import { expect } from '@playwright/test';

export class AllAccessRequestsPage extends UsersManagementPage {
  // Extended selectors for all requests page
  allRequestsSelectors = {
    // Page structure
    pageContainer: '[data-testid="all-requests-page"]',
    requestCount: '[data-testid="request-count"]',

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
    await this.navViaShell('api-keys', 'access-requests');
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
      // Extract reviewer name from "by {reviewer}" format
      return text?.replace('by ', '').trim() || null;
    }
    return null;
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

  async verifyRequestCountDisplay(expectedTotal) {
    const countText = await this.page.locator(this.allRequestsSelectors.requestCount).textContent();
    const expectedText =
      expectedTotal === 1 ? '1 total request' : `${expectedTotal} total requests`;
    expect(countText).toBe(expectedText);
  }
}
