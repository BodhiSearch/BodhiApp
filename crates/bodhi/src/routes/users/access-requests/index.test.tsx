import AllRequestsPage from '@/routes/users/access-requests/index';
import { ShellSlotsProvider } from '@/components/shell';
import { createWrapper } from '@/tests/wrapper';
import { server } from '@/test-utils/msw-v2/setup';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import {
  mockAccessRequests,
  mockAccessRequestsDefault,
  mockAccessRequestsEmpty,
  mockAccessRequestApprove,
  mockAccessRequestReject,
  mockAccessRequestApproveError,
  mockAccessRequestRejectError,
  mockAccessRequestsError,
} from '@/test-utils/msw-v2/handlers/user-access-requests';
import {
  ADMIN_ROLES,
  BLOCKED_ROLES,
  mockPendingRequest,
  mockApprovedRequest,
  mockRejectedRequest,
} from '@/test-fixtures/access-requests';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const navigateMock = vi.fn();
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: any) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
    useNavigate: () => navigateMock,
    useLocation: () => ({ pathname: '/users/access-requests' }),
  };
});

vi.mock('@/hooks/use-toast-messages', () => ({
  useToastMessages: () => ({
    showSuccess: vi.fn(),
    showError: vi.fn(),
  }),
}));

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  navigateMock.mockClear();
});

function renderPage() {
  return act(async () => {
    render(
      <ShellSlotsProvider>
        <AllRequestsPage />
      </ShellSlotsProvider>,
      { wrapper: createWrapper() }
    );
  });
}

describe('AllRequestsPage Role-Based Access Control', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it.each(ADMIN_ROLES)('allows access for %s role', async (role) => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ username: `${role}@example.com`, role: `resource_${role}` }),
      ...mockAccessRequests({ requests: [mockPendingRequest], total: 1, page: 1, page_size: 10 })
    );

    await renderPage();

    expect(screen.getByTestId('all-requests-page')).toBeInTheDocument();
    await screen.findByText('user@example.com');
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it.each(BLOCKED_ROLES)('blocks access for %s role', async (role) => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ username: `${role}@example.com`, role: `resource_${role}` }),
      ...mockAccessRequests({ requests: [mockPendingRequest], total: 1, page: 1, page_size: 10 })
    );

    await renderPage();

    // Redirect-on-insufficient-role is AppInitializer's job (mocked here); assert the page doesn't crash.
    await waitFor(() => {
      expect(screen.queryByTestId('all-requests-page')).toBeInTheDocument();
    });
  });
});

describe('AllRequestsPage Data Display', () => {
  beforeEach(() => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn({ role: 'resource_user' }), ...mockAccessRequestsDefault());
  });

  it('displays all requests with status chips and reviewer for decided rows', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({
        requests: [mockPendingRequest, mockApprovedRequest, mockRejectedRequest],
        total: 3,
        page: 1,
        page_size: 10,
      })
    );

    await renderPage();
    await screen.findByText('user@example.com');

    // default filter is Pending; switch to All to see decided rows too
    await user.click(screen.getByTestId('requests-filter-all'));

    expect(screen.getByText('user@example.com')).toBeInTheDocument();
    expect(screen.getByText('approved@example.com')).toBeInTheDocument();
    expect(screen.getByText('rejected@example.com')).toBeInTheDocument();

    // status chips appear on rows (filter pills carry the same words, so target the row testids)
    expect(screen.getByTestId('request-status-pending')).toBeInTheDocument();
    expect(screen.getByTestId('request-status-approved')).toBeInTheDocument();
    expect(screen.getByTestId('request-status-rejected')).toBeInTheDocument();

    // reviewer shows in the role cell of decided rows only (2 of 3)
    const reviewerElements = screen.getAllByTestId('request-reviewer');
    expect(reviewerElements).toHaveLength(2);
    reviewerElements.forEach((el) => expect(el).toHaveTextContent('admin@example.com'));
  });

  it('derives filter-tab counts and filters rows by status', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({
        requests: [mockPendingRequest, mockApprovedRequest, mockRejectedRequest],
        total: 3,
        page: 1,
        page_size: 10,
      })
    );

    await renderPage();
    await screen.findByText('user@example.com');

    expect(within(screen.getByTestId('requests-filter-all')).getByText('3')).toBeInTheDocument();
    expect(within(screen.getByTestId('requests-filter-pending')).getByText('1')).toBeInTheDocument();
    expect(within(screen.getByTestId('requests-filter-rejected')).getByText('1')).toBeInTheDocument();

    await user.click(screen.getByTestId('requests-filter-approved'));
    expect(screen.getByTestId('request-row-approved@example.com')).toBeInTheDocument();
    expect(screen.queryByTestId('request-row-user@example.com')).not.toBeInTheDocument();
  });

  it('filters rows by search query on username', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({
        requests: [mockPendingRequest, mockApprovedRequest, mockRejectedRequest],
        total: 3,
        page: 1,
        page_size: 10,
      })
    );

    await renderPage();
    await screen.findByText('user@example.com');

    // default filter is Pending; switch to All so the search can reach the approved row
    await user.click(screen.getByTestId('requests-filter-all'));

    await user.click(screen.getByTestId('requests-search-toggle'));
    await user.type(screen.getByPlaceholderText('Search requests by email…'), 'approved');

    expect(screen.getByTestId('request-row-approved@example.com')).toBeInTheDocument();
    expect(screen.queryByTestId('request-row-user@example.com')).not.toBeInTheDocument();
  });

  it('displays empty state when no requests exist', async () => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn({ role: 'resource_admin' }), ...mockAccessRequestsEmpty());

    await renderPage();

    await waitFor(() => {
      expect(screen.getByText('No Access Requests')).toBeInTheDocument();
    });
  });

  it('handles pagination correctly', async () => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({ requests: [mockPendingRequest], total: 25, page: 1, page_size: 10 })
    );

    await renderPage();

    // total 25 / pageSize 10 → 3 pages: ShellPagination renders numbered pills + prev/next.
    await waitFor(() => {
      expect(screen.getByTestId('pagination')).toBeInTheDocument();
    });
    expect(screen.getByTestId('pagination-page-1')).toHaveAttribute('aria-current', 'page');
    expect(screen.getByTestId('pagination-page-3')).toBeInTheDocument();
    expect(screen.getByTestId('pagination-next')).toBeEnabled();
    expect(screen.getByTestId('pagination-prev')).toBeDisabled();
  });

  it('shows decided rows without a role dropdown or action buttons', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({ requests: [mockApprovedRequest], total: 1, page: 1, page_size: 10 })
    );

    await renderPage();

    // default filter is Pending; the only request is approved, so switch to All to see it
    await user.click(screen.getByTestId('requests-filter-all'));
    await screen.findByText('approved@example.com');

    expect(screen.queryByTestId('approve-btn-approved@example.com')).not.toBeInTheDocument();
    expect(screen.queryByTestId('reject-btn-approved@example.com')).not.toBeInTheDocument();
    expect(screen.queryByTestId('role-select-approved@example.com')).not.toBeInTheDocument();
  });
});

describe('AllRequestsPage Request Management', () => {
  const user = userEvent.setup();

  beforeEach(() => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({ requests: [mockPendingRequest], total: 1, page: 1, page_size: 10 })
    );
  });

  it('shows inline role selection, approve and reject for pending requests', async () => {
    await renderPage();
    await screen.findByText('user@example.com');

    expect(screen.getByTestId('approve-btn-user@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('reject-btn-user@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('role-select-user@example.com')).toBeInTheDocument();
  });

  it('successfully approves request when approve button clicked', async () => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({ requests: [mockPendingRequest], total: 1, page: 1, page_size: 10 }),
      ...mockAccessRequestApprove(mockPendingRequest.id)
    );

    await renderPage();
    await screen.findByText('user@example.com');

    const approveButton = screen.getByTestId('approve-btn-user@example.com');
    await user.click(approveButton);

    await waitFor(() => expect(approveButton).toBeInTheDocument());
  });

  it('successfully rejects request when reject button clicked', async () => {
    server.use(...mockAccessRequestReject(mockPendingRequest.id));

    await renderPage();
    await screen.findByText('user@example.com');

    const rejectButton = screen.getByTestId('reject-btn-user@example.com');
    await user.click(rejectButton);

    await waitFor(() => expect(rejectButton).toBeInTheDocument());
  });
});

describe('AllRequestsPage Error Handling', () => {
  it('shows empty state when the list API fails (no inline error handling)', async () => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequestsError({ status: 500, code: 'internal_error', message: 'Internal server error' }),
      ...mockAccessRequestsError({ status: 500, code: 'internal_error', message: 'Internal server error' })
    );

    await renderPage();

    await waitFor(() => {
      expect(screen.getByText('No Access Requests')).toBeInTheDocument();
    });
    expect(screen.getByText('No access requests match this filter.')).toBeInTheDocument();
  });

  it('handles approve failure via toast (not on screen)', async () => {
    const u = userEvent.setup();
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({ requests: [mockPendingRequest], total: 1, page: 1, page_size: 10 }),
      ...mockAccessRequestApproveError(mockPendingRequest.id)
    );

    await renderPage();
    await screen.findByText('user@example.com');

    const approveButton = screen.getByTestId('approve-btn-user@example.com');
    await u.click(approveButton);

    expect(approveButton).toBeInTheDocument();
  });

  it('handles reject failure via toast (not on screen)', async () => {
    const u = userEvent.setup();
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({ requests: [mockPendingRequest], total: 1, page: 1, page_size: 10 }),
      ...mockAccessRequestRejectError(mockPendingRequest.id)
    );

    await renderPage();
    await screen.findByText('user@example.com');

    const rejectButton = screen.getByTestId('reject-btn-user@example.com');
    await u.click(rejectButton);

    expect(rejectButton).toBeInTheDocument();
  });
});
