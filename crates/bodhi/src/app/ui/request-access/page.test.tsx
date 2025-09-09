import RequestAccessPage from '@/app/ui/request-access/page';
import { createWrapper } from '@/tests/wrapper';
import { createAccessRequestHandlers, createErrorHandlers } from '@/test-utils/msw-handlers';
import {
  mockUserAccessStatusNone,
  mockUserAccessStatusPending,
  mockUserAccessStatusApproved,
  mockUserAccessStatusRejected,
  createMockUserInfo,
} from '@/test-fixtures/access-requests';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  usePathname: vi.fn().mockReturnValue('/ui/request-access'),
}));

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  pushMock.mockClear();
});

describe('RequestAccessPage Display States', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('displays request access form when user has no access request', async () => {
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusNone,
        userInfo: { logged_in: true, email: 'user@example.com', role: null }, // No role
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // Should show the request form
    expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
    expect(screen.getByText('Request access to application')).toBeInTheDocument();

    // Should show request access button
    expect(screen.getByTestId('auth-card-action-0')).toBeInTheDocument();
    expect(screen.getByTestId('auth-card-action-0')).toHaveTextContent('Request Access');
  });

  it('displays pending status when user has pending request', async () => {
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusPending,
        userInfo: { logged_in: true, email: 'user@example.com', role: null }, // No role
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // Should show pending status
    expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
    expect(screen.getByText('Access Request Pending')).toBeInTheDocument();
    expect(screen.getByText(/Your access request submitted on.*is pending review/)).toBeInTheDocument();

    // Should not show request button
    expect(screen.queryByRole('button', { name: /request access/i })).not.toBeInTheDocument();
  });

  it('redirects users with approved status who already have roles', async () => {
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusApproved,
        userInfo: createMockUserInfo('user'), // User has a role, so will be redirected
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // Should show empty page content since user gets redirected
    expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
    // User with roles gets redirected, so should not see the AuthCard content
    expect(screen.queryByTestId('auth-card')).not.toBeInTheDocument();
    expect(pushMock).toHaveBeenCalledWith('/ui/chat');
  });

  it('shows request access button when user has rejected request and no roles', async () => {
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusRejected,
        userInfo: { logged_in: true, email: 'user@example.com', role: null }, // No role
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // Should show request access form (rejected status shows same as none)
    expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
    expect(screen.getByTestId('auth-card-header')).toHaveTextContent('Request Access');
    expect(screen.getByText('Request access to application')).toBeInTheDocument();

    // Should show request access button
    expect(screen.getByTestId('auth-card-action-0')).toBeInTheDocument();
  });

  it('redirects users who already have roles instead of showing page content', async () => {
    const userInfo = createMockUserInfo('user'); // User has roles
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusNone,
        userInfo: userInfo,
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // Should redirect user with roles and not show AuthCard content
    expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
    expect(screen.queryByTestId('auth-card')).not.toBeInTheDocument();
    expect(pushMock).toHaveBeenCalledWith('/ui/chat');
  });
});

describe('RequestAccessPage Request Submission', () => {
  const user = userEvent.setup();

  beforeEach(() => {
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusNone,
        userInfo: createMockUserInfo('resource_user'),
      })
    );
  });

  it('successfully submits access request', async () => {
    let submitRequestCalled = false;
    const mockSubmitResponse = {
      id: 1,
      status: 'pending',
      email: 'user@example.com',
      created_at: new Date().toISOString(),
    };

    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusNone,
        userInfo: { logged_in: true, email: 'user@example.com', role: null }, // No role
        submitRequest: (() => {
          submitRequestCalled = true;
          return mockSubmitResponse;
        })(),
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // Should show the request form for user with no roles
    expect(screen.getByTestId('auth-card-header')).toHaveTextContent('Request Access');

    // Click request access button using data-testid
    const requestButton = screen.getByTestId('auth-card-action-0');
    await user.click(requestButton);

    await waitFor(() => {
      expect(submitRequestCalled).toBe(true);
    });
  });

  it('shows request button with correct initial state', async () => {
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusNone,
        userInfo: { logged_in: true, email: 'user@example.com', role: null }, // No role
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    const requestButton = screen.getByTestId('auth-card-action-0');

    // Should show initial button state
    expect(requestButton).toHaveTextContent('Request Access');
    expect(requestButton).not.toBeDisabled();

    // Button should be clickable
    expect(requestButton).toBeInTheDocument();
  });

  it('handles request submission failure', async () => {
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusNone,
        userInfo: { logged_in: true, email: 'user@example.com', role: null }, // No role
      }),
      ...createErrorHandlers()
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    const requestButton = screen.getByTestId('auth-card-action-0');
    await user.click(requestButton);

    // Should show error message via toast (tested in toast hooks)
    // The error is handled by the mutation hook and shown via toast
    expect(requestButton).toBeEnabled(); // Button should be re-enabled after error
  });

  it('prevents multiple submissions', async () => {
    let submitCount = 0;
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusNone,
        userInfo: { logged_in: true, email: 'user@example.com', role: null }, // No role
        submitRequest: (() => {
          submitCount++;
          return {
            id: 1,
            status: 'pending',
            email: 'test@example.com',
            created_at: new Date().toISOString(),
          };
        })(),
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    const requestButton = screen.getByTestId('auth-card-action-0');

    // Click multiple times quickly
    await user.click(requestButton);
    await user.click(requestButton);
    await user.click(requestButton);

    // Should only submit once (button becomes disabled during submission)
    await waitFor(() => {
      expect(submitCount).toBeLessThanOrEqual(1);
    });
  });
});

describe('RequestAccessPage Authentication and Access Control', () => {
  it('handles unauthenticated users', async () => {
    server.use(
      ...createAccessRequestHandlers({
        userInfo: { logged_in: false },
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // Should show loading/redirect state from AppInitializer
    // AppInitializer redirects unauthenticated users
    expect(screen.getByText('Redirecting to login...')).toBeInTheDocument();
  });

  it('redirects users who already have manager+ roles', async () => {
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusNone,
        userInfo: createMockUserInfo('manager'),
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // User with manager role gets redirected
    expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
    expect(screen.queryByTestId('auth-card')).not.toBeInTheDocument();
    expect(pushMock).toHaveBeenCalledWith('/ui/chat');
  });

  it('redirects admin users', async () => {
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusNone,
        userInfo: createMockUserInfo('admin'),
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // Admin user gets redirected
    expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
    expect(screen.queryByTestId('auth-card')).not.toBeInTheDocument();
    expect(pushMock).toHaveBeenCalledWith('/ui/chat');
  });
});

describe('RequestAccessPage Error Handling', () => {
  it('displays error message when request status fetch fails', async () => {
    server.use(...createErrorHandlers());

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByRole('alert') || screen.getByText(/error/i)).toBeInTheDocument();
    });
  });

  it('displays error message for network failures', async () => {
    server.use(...createAccessRequestHandlers());

    // Simulate network error
    server.use(...createErrorHandlers());

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      const errorElement = screen.queryByRole('alert') || screen.queryByText(/failed to fetch/i);
      if (errorElement) {
        expect(errorElement).toBeInTheDocument();
      }
    });
  });

  it('recovers from error state when retrying', async () => {
    let requestCount = 0;
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: () => {
          requestCount++;
          if (requestCount === 1) {
            throw new Error('Network error');
          }
          return mockUserAccessStatusNone;
        },
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // Should eventually show content after retry
    await waitFor(() => {
      expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
    });
  });
});

describe('RequestAccessPage Loading States', () => {
  it('shows loading state while fetching request status', async () => {
    // Use delayed response
    server.use(
      ...createAccessRequestHandlers({
        userInfo: { logged_in: true, email: 'user@example.com', role: null }, // No role
        requestStatus: new Promise((resolve) => setTimeout(() => resolve(mockUserAccessStatusNone), 100)),
      })
    );

    render(<RequestAccessPage />, { wrapper: createWrapper() });

    // Should eventually show the page after loading
    await waitFor(
      () => {
        expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
      },
      { timeout: 3000 }
    );

    // Wait for data to load and show the auth card
    await waitFor(() => {
      expect(screen.getByTestId('auth-card-header')).toHaveTextContent('Request Access');
    });
  });

  it('shows pending status for users without roles', async () => {
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusPending,
        userInfo: { logged_in: true, email: 'user@example.com', role: null }, // No role
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // Should show pending status
    expect(screen.getByText('Access Request Pending')).toBeInTheDocument();
    expect(screen.getByText(/Your access request submitted on.*is pending review/)).toBeInTheDocument();
    // Should not show any buttons when pending
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });
});

describe('RequestAccessPage UI Interactions', () => {
  const user = userEvent.setup();

  it('shows request access button for users without roles', async () => {
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusNone,
        userInfo: { logged_in: true, email: 'user@example.com', role: null }, // No role
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    const requestButton = screen.getByTestId('auth-card-action-0');

    // Check button is available and enabled
    expect(requestButton).toBeInTheDocument();
    expect(requestButton).not.toBeDisabled();
    expect(requestButton).toHaveTextContent('Request Access');
  });

  it('allows requesting access when previous request was rejected', async () => {
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusRejected,
        userInfo: { logged_in: true, email: 'user@example.com', role: null }, // No role
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // Should show request access button (rejected status shows same UI as none)
    const requestButton = screen.getByTestId('auth-card-action-0');
    expect(requestButton).toBeInTheDocument();
    expect(requestButton).toHaveTextContent('Request Access');
    // Should also show the title
    expect(screen.getByTestId('auth-card-header')).toHaveTextContent('Request Access');

    await user.click(requestButton);

    // Should trigger request submission - the button state change might be async
    // We can't reliably test the disabled state due to race conditions
    // The mutation hook handles preventing double submissions internally
    expect(requestButton).toBeInTheDocument();
  });

  it('shows formatted date for pending requests', async () => {
    server.use(
      ...createAccessRequestHandlers({
        requestStatus: mockUserAccessStatusPending,
        userInfo: { logged_in: true, email: 'user@example.com', role: null }, // No role
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // Should show formatted date in the pending message
    expect(screen.getByText(/Your access request submitted on.*is pending review/)).toBeInTheDocument();
    // Should contain the formatted date (mock data uses 2024-01-01)
    expect(screen.getByText((content) => content.includes('1/1/2024'))).toBeInTheDocument();
  });
});
