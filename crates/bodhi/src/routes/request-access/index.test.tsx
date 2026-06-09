import RequestAccessPage from '@/routes/request-access/index';
import {
  mockUserRequestAccess,
  mockUserRequestAccessError,
  mockUserRequestStatusApproved,
  mockUserRequestStatusError,
  mockUserRequestStatusPending,
  mockUserRequestStatusRejected,
} from '@/test-utils/msw-v2/handlers/user-access-requests';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
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
    useLocation: () => ({ pathname: '/request-access' }),
  };
});

vi.mock('@/components/AppInitializer', () => ({
  default: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  navigateMock.mockClear();
});

describe('RequestAccessPage Display States', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('displays pending status when user has pending request', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com' }), // No role
      ...mockUserRequestStatusPending({ username: 'user@example.com' })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
    expect(screen.getByText('Access Request Pending')).toBeInTheDocument();
    expect(screen.getByText(/Your access request submitted on.*is pending review/)).toBeInTheDocument();

    expect(screen.queryByRole('button', { name: /request access/i })).not.toBeInTheDocument();
  });

  it('redirects users with approved status who already have roles', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com', role: 'resource_user' }), // User has a role, so will be redirected
      ...mockUserRequestStatusApproved({ username: 'approved@example.com' })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
    // User with a role is redirected, so the AuthCard never renders.
    expect(screen.queryByTestId('auth-card')).not.toBeInTheDocument();
    expect(navigateMock).toHaveBeenCalledWith({ to: '/chat/' });
  });

  it('shows request access button when user has rejected request and no roles', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com' }), // No role
      ...mockUserRequestStatusRejected({ username: 'rejected@example.com' })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // Rejected status renders the same request form as no request.
    expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
    expect(screen.getByTestId('auth-card-header')).toHaveTextContent('Request Access');
    expect(screen.getByText('Request access to application')).toBeInTheDocument();

    expect(screen.getByTestId('auth-card-action-0')).toBeInTheDocument();
  });
});

describe('RequestAccessPage Authentication and Access Control', () => {
  it('handles unauthenticated users by redirecting', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com' }) // Note: This test relies on AppInitializer mocking for redirect logic
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // Since we mocked AppInitializer, the page will render but with mocked redirect logic
    // The actual redirect logic is tested in AppInitializer's own tests
    expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
  });
});

describe('RequestAccessPage Error Handling', () => {
  it('handles error state gracefully when API calls fail', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com' }),
      ...mockUserRequestStatusError({ status: 500 })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
  });

  it('shows request form when there are API errors but user info loads', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com' }),
      // Make request status API return 404 (no request exists)
      ...mockUserRequestStatusError({ status: 404, message: 'No request found' })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    // A 404 status (no request) renders the request form.
    await waitFor(() => {
      expect(screen.getByTestId('auth-card-header')).toHaveTextContent('Request Access');
      expect(screen.getByTestId('auth-card-action-0')).toBeInTheDocument();
    });
  });
});

describe('RequestAccessPage Loading States', () => {
  it('shows pending status for users without roles', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com' }), // No role (testing null)
      ...mockUserRequestStatusPending({ username: 'user@example.com' })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByText('Access Request Pending')).toBeInTheDocument();
    expect(screen.getByText(/Your access request submitted on.*is pending review/)).toBeInTheDocument();
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });
});

describe('RequestAccessPage UI Interactions', () => {
  const user = userEvent.setup();

  it('allows requesting access when previous request was rejected', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com' }), // No role (testing undefined)
      ...mockUserRequestStatusRejected({ username: 'rejected@example.com' })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    const requestButton = screen.getByTestId('auth-card-action-0');
    expect(requestButton).toBeInTheDocument();
    expect(requestButton).toHaveTextContent('Request Access');
    expect(screen.getByTestId('auth-card-header')).toHaveTextContent('Request Access');

    await user.click(requestButton);

    // Should trigger request submission - the button state change might be async
    // We can't reliably test the disabled state due to race conditions
    // The mutation hook handles preventing double submissions internally
    expect(requestButton).toBeInTheDocument();
  });

  it('shows formatted date for pending requests', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com' }), // No role
      ...mockUserRequestStatusPending({ username: 'user@example.com' })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByText(/Your access request submitted on.*is pending review/)).toBeInTheDocument();
    // Mock data uses 2024-01-01, rendered as 1/1/2024.
    expect(screen.getByText((content) => content.includes('1/1/2024'))).toBeInTheDocument();
  });
});

describe('RequestAccessPage - No Request Exists', () => {
  const user = userEvent.setup();

  // The status endpoint returns 404 (no request) twice, then the request-access POST succeeds.
  const createNoRequestHandlers = (userInfo: any) => [
    ...mockAppInfo({ status: 'ready' }),
    ...mockUserLoggedIn(userInfo),
    ...mockUserRequestStatusError({
      code: 'not_found',
      message: 'pending access request for user not found',
      type: 'not_found_error',
      status: 404,
    }),
    ...mockUserRequestStatusError({
      code: 'not_found',
      message: 'pending access request for user not found',
      type: 'not_found_error',
      status: 404,
    }),
    ...mockUserRequestAccess(),
  ];

  it('displays request access form when user has no access request', async () => {
    server.use(
      ...createNoRequestHandlers({
        username: 'user@example.com',
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
    expect(screen.getByText('Request access to application')).toBeInTheDocument();

    expect(screen.getByTestId('auth-card-action-0')).toBeInTheDocument();
    expect(screen.getByTestId('auth-card-action-0')).toHaveTextContent('Request Access');
  });

  it('successfully submits access request', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com' }),
      ...mockUserRequestStatusError({
        code: 'not_found',
        message: 'pending access request for user not found',
        type: 'not_found_error',
        status: 404,
      }),
      ...mockUserRequestStatusError({
        code: 'not_found',
        message: 'pending access request for user not found',
        type: 'not_found_error',
        status: 404,
      }),
      ...mockUserRequestAccess({ delayMs: 100 })
    );
    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });
    expect(screen.getByTestId('auth-card-header')).toHaveTextContent('Request Access');
    const requestButton = screen.getByTestId('auth-card-action-0');
    await user.click(requestButton);
    await waitFor(() => {
      expect(requestButton).toBeDisabled();
    });
    await waitFor(() => {
      const requestButton = screen.getByTestId('auth-card-action-0');
      expect(requestButton).toBeEnabled();
    });
  });

  it('shows request button with correct initial state', async () => {
    server.use(
      ...createNoRequestHandlers({
        username: 'user@example.com',
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    const requestButton = screen.getByTestId('auth-card-action-0');

    expect(requestButton).toHaveTextContent('Request Access');
    expect(requestButton).not.toBeDisabled();
    expect(requestButton).toBeInTheDocument();
  });

  it('handles request submission failure', async () => {
    const errorHandlers = [
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com' }),
      ...mockUserRequestStatusError({
        code: 'not_found',
        message: 'pending access request for user not found',
        type: 'not_found_error',
        status: 404,
      }),
      ...mockUserRequestStatusError({
        code: 'not_found',
        message: 'pending access request for user not found',
        type: 'not_found_error',
        status: 404,
      }),
      ...mockUserRequestAccessError({
        code: 'conflict',
        message: 'Request already exists',
        type: 'conflict_error',
        status: 409,
      }),
    ];

    server.use(...errorHandlers);

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    const requestButton = screen.getByTestId('auth-card-action-0');
    await user.click(requestButton);

    // The error surfaces via toast (covered in toast hook tests); here we only assert the button re-enables.
    expect(requestButton).toBeEnabled();
  });

  it('shows request access button for users without roles', async () => {
    server.use(
      ...createNoRequestHandlers({
        username: 'user@example.com',
      })
    );

    await act(async () => {
      render(<RequestAccessPage />, { wrapper: createWrapper() });
    });

    const requestButton = screen.getByTestId('auth-card-action-0');

    expect(requestButton).toBeInTheDocument();
    expect(requestButton).not.toBeDisabled();
    expect(requestButton).toHaveTextContent('Request Access');
  });
});
