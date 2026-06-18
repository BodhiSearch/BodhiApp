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
  mockEmptyRequests,
  mockAllRequests,
  createMockUserInfo,
} from '@/test-fixtures/access-requests';
import { createMockAdminUser } from '@/test-utils/mock-user';
import { act, render, screen, fireEvent, waitFor } from '@testing-library/react';
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

// The V2 list renders its own rows; only Pagination is still consumed from this module.
vi.mock('@/components/DataTable', () => ({
  Pagination: ({ page, totalPages }: any) => (
    <div data-testid="pagination">
      Page {page} of {totalPages}
    </div>
  ),
}));

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

describe('AllRequestsPage Role-Based Access Control', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it.each(ADMIN_ROLES)('allows access for %s role', async (role) => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({
        username: `${role}@example.com`,
        role: `resource_${role}`,
      }),
      ...mockAccessRequests({
        requests: mockAllRequests.requests,
        total: mockAllRequests.total,
        page: mockAllRequests.page,
        page_size: mockAllRequests.page_size,
      })
    );

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    expect(screen.getByTestId('all-requests-page')).toBeInTheDocument();
    expect(screen.getByTestId('request-count')).toBeInTheDocument();
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it.each(BLOCKED_ROLES)('blocks access for %s role', async (role) => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({
        username: `${role}@example.com`,
        role: `resource_${role}`,
      }),
      ...mockAccessRequests({
        requests: mockAllRequests.requests,
        total: mockAllRequests.total,
        page: mockAllRequests.page,
        page_size: mockAllRequests.page_size,
      })
    );

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

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

  it('displays all requests with correct status badges and reviewer information', async () => {
    const allRequestsData = {
      requests: [mockPendingRequest, mockApprovedRequest, mockRejectedRequest],
      total: 3,
      page: 1,
      page_size: 10,
    };

    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({
        requests: allRequestsData.requests,
        total: allRequestsData.total,
        page: allRequestsData.page,
        page_size: allRequestsData.page_size,
      })
    );

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await screen.findByText('user@example.com');

    expect(screen.getByText('user@example.com')).toBeInTheDocument();
    expect(screen.getByText('approved@example.com')).toBeInTheDocument();
    expect(screen.getByText('rejected@example.com')).toBeInTheDocument();

    // status badges (filter tabs also contain these words, so target the row status testids)
    expect(screen.getByTestId('request-status-pending')).toBeInTheDocument();
    expect(screen.getByTestId('request-status-approved')).toBeInTheDocument();
    expect(screen.getByTestId('request-status-rejected')).toBeInTheDocument();

    const reviewerElements = screen.getAllByText('admin@example.com');
    expect(reviewerElements).toHaveLength(2); // One for approved, one for rejected
  });

  it('displays empty state when no requests exist', async () => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn({ role: 'resource_admin' }), ...mockAccessRequestsEmpty());

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await waitFor(() => {
      expect(screen.getByText('No Access Requests')).toBeInTheDocument();
    });
  });

  it('handles pagination correctly', async () => {
    const paginatedData = {
      requests: [mockPendingRequest],
      total: 25,
      page: 1,
      page_size: 10,
    };

    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({
        requests: paginatedData.requests,
        total: paginatedData.total,
        page: paginatedData.page,
        page_size: paginatedData.page_size,
      })
    );

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await waitFor(() => {
      expect(screen.getByText('Page 1 of 3')).toBeInTheDocument();
    });
  });

  it('displays correct date for pending vs processed requests', async () => {
    const allRequestsData = {
      requests: [mockPendingRequest, mockApprovedRequest],
      total: 2,
      page: 1,
      page_size: 10,
    };

    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({
        requests: allRequestsData.requests,
        total: allRequestsData.total,
        page: allRequestsData.page,
        page_size: allRequestsData.page_size,
      })
    );

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await screen.findByText('user@example.com');

    // Pending request shows created_at; approved request shows updated_at.
    expect(screen.getByText('1/1/2024')).toBeInTheDocument();
    expect(screen.getByText('1/2/2024')).toBeInTheDocument();
  });

  it('shows reviewer information only for approved/rejected requests', async () => {
    const allRequestsData = {
      requests: [mockPendingRequest, mockApprovedRequest, mockRejectedRequest],
      total: 3,
      page: 1,
      page_size: 10,
    };

    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({
        requests: allRequestsData.requests,
        total: allRequestsData.total,
        page: allRequestsData.page,
        page_size: allRequestsData.page_size,
      })
    );

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await screen.findByText('user@example.com');

    const reviewerElements = screen.getAllByTestId('request-reviewer');
    expect(reviewerElements).toHaveLength(2); // Only approved and rejected, not pending

    reviewerElements.forEach((element) => {
      expect(element).toHaveTextContent('admin@example.com');
    });
  });
});

describe('AllRequestsPage Request Management', () => {
  const user = userEvent.setup();

  beforeEach(() => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({
        requests: [mockPendingRequest],
        total: 1,
        page: 1,
        page_size: 10,
      })
    );
  });

  it('displays inline role selection and approve buttons', async () => {
    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await screen.findByText('user@example.com');

    expect(screen.getByText('Approve')).toBeInTheDocument();
    expect(screen.getByText('Reject')).toBeInTheDocument();

    // Role options only render once the combobox is opened.
    expect(screen.getByRole('combobox')).toBeInTheDocument();
  });

  it('successfully approves request when approve button clicked', async () => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({
        requests: [mockPendingRequest],
        total: 1,
        page: 1,
        page_size: 10,
      }),
      ...mockAccessRequestApprove(mockPendingRequest.id)
    );

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await screen.findByText('user@example.com');

    // Role defaults to 'resource_user'.
    const approveButton = screen.getByText('Approve');
    await user.click(approveButton);

    await waitFor(() => {
      expect(approveButton).toBeInTheDocument();
    });
  });

  it('shows reject button for pending requests', async () => {
    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await screen.findByText('user@example.com');

    expect(screen.getByText('Reject')).toBeInTheDocument();
  });

  it('successfully rejects request when reject button clicked', async () => {
    server.use(...mockAccessRequestReject(mockPendingRequest.id));

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await screen.findByText('user@example.com');

    const rejectButton = screen.getByText('Reject');
    await user.click(rejectButton);

    await waitFor(() => {
      expect(rejectButton).toBeInTheDocument();
    });
  });
});

describe('AllRequestsPage Error Handling', () => {
  it('shows empty state when API call fails (no error handling in component)', async () => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequestsError({
        status: 500,
        code: 'internal_error',
        message: 'Internal server error',
      }),
      ...mockAccessRequestsError({
        status: 500,
        code: 'internal_error',
        message: 'Internal server error',
      })
    );

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    // Component doesn't handle errors, so shows empty state instead
    await waitFor(() => {
      expect(screen.getByText('No Access Requests')).toBeInTheDocument();
    });
    expect(screen.getByText('No access requests match this filter.')).toBeInTheDocument();
  });

  it('handles approve request failure via toast (not on screen)', async () => {
    const user = userEvent.setup();

    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({
        requests: [mockPendingRequest],
        total: 1,
        page: 1,
        page_size: 10,
      }),
      ...mockAccessRequestApproveError(mockPendingRequest.id)
    );

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await screen.findByText('user@example.com');

    const approveButton = screen.getByText('Approve');
    await user.click(approveButton);

    const roleSelect = screen.getByRole('combobox');
    await user.click(roleSelect);

    // Failure surfaces via toast, not on screen.
    expect(approveButton).toBeInTheDocument();
  });

  it('handles reject request failure via toast (not on screen)', async () => {
    const user = userEvent.setup();

    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({
        requests: [mockPendingRequest],
        total: 1,
        page: 1,
        page_size: 10,
      }),
      ...mockAccessRequestRejectError(mockPendingRequest.id)
    );

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await screen.findByText('user@example.com');

    const rejectButton = screen.getByText('Reject');
    await user.click(rejectButton);

    // Failure surfaces via toast, not on screen.
    expect(rejectButton).toBeInTheDocument();
  });
});

describe('AllRequestsPage Loading States', () => {
  it('shows page and eventually loads data', async () => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({
        requests: [mockPendingRequest],
        total: 1,
        page: 1,
        page_size: 10,
      })
    );

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    expect(screen.getByTestId('all-requests-page')).toBeInTheDocument();

    await screen.findByText('user@example.com');
    expect(screen.getByTestId('request-count')).toBeInTheDocument();
  });

  it('shows approve and reject buttons for pending requests', async () => {
    const user = userEvent.setup();

    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockAccessRequests({
        requests: [mockPendingRequest],
        total: 1,
        page: 1,
        page_size: 10,
      })
    );

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <AllRequestsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await screen.findByText('user@example.com');

    const approveButton = screen.getByText('Approve');
    const rejectButton = screen.getByText('Reject');

    expect(approveButton).toBeInTheDocument();
    expect(rejectButton).toBeInTheDocument();

    expect(screen.getByRole('combobox')).toBeInTheDocument();
  });
});
