import { expect } from '@playwright/test';
import { UsersManagementPage } from './UsersManagementPage.mjs';

export class AllAccessRequestsPage extends UsersManagementPage {
  constructor(page, baseUrl) {
    super(page, baseUrl);
  }

  // Extended selectors for all requests page
  allRequestsSelectors = {
    // Page structure
    pageContainer: '[data-testid="all-requests-page"]',
    pageTitle: '[data-testid="page-title"]',
    requestCount: '[data-testid="request-count"]',
    
    // Navigation tabs
    navLinkPending: '[data-testid="nav-link-pending"]',
    navLinkAll: '[data-testid="nav-link-all"]',
    navLinkUsers: '[data-testid="nav-link-users"]',
    
    // Table structure
    requestsTable: '[data-testid="requests-table"]',
    tableBody: 'tbody',
    
    // Row-level selectors (will use dynamic testid)
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
    await this.navigate('/ui/access-requests');
    await this.waitForSPAReady();
    await this.expectVisible(this.allRequestsSelectors.pageContainer);
  }

  async expectAllRequestsPage() {
    await expect(this.page).toHaveURL(/\/ui\/access-requests\/?$/);
    await this.expectVisible(this.allRequestsSelectors.pageContainer);
  }

  // Request verification methods
  async findRequestByUsername(username) {
    // Use the existing findRequestRowByUsername from parent class for table rows
    // This uses the standard table structure selectors
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
      await this.verifyRequestStatus(
        expected.username, 
        expected.status, 
        expected.reviewer
      );
    }
  }

  async getTotalRequestCount() {
    // Wait for table to load or show empty state
    try {
      // Check if we have the requests table
      await this.page.waitForSelector(this.allRequestsSelectors.requestsTable, { timeout: 5000 });
      
      // Check if empty state is showing
      if (await this.page.locator(this.allRequestsSelectors.emptyState).isVisible()) {
        return 0;
      }
      
      // Count table rows
      const rows = await this.page.locator('tbody tr').all();
      return rows.length;
    } catch (error) {
      // If table doesn't exist, might be empty state
      if (await this.page.locator(this.allRequestsSelectors.emptyState).isVisible()) {
        return 0;
      }
      return 0;
    }
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
    await this.expectVisible(this.allRequestsSelectors.pageTitle);
    const titleText = await this.page.locator(this.allRequestsSelectors.pageTitle).textContent();
    expect(titleText).toContain('All Access Requests');
  }

  async verifyRequestCountDisplay(expectedTotal) {
    const countText = await this.page.locator(this.allRequestsSelectors.requestCount).textContent();
    const expectedText = expectedTotal === 1 ? '1 total request' : `${expectedTotal} total requests`;
    expect(countText).toBe(expectedText);
  }
}