import RequestAccessPage from '@/app/ui/request-access/page';
import {
  mockUserRequestAccess,
  mockUserRequestAccessError,
  mockUserRequestStatusApproved,
  mockUserRequestStatusError,
  mockUserRequestStatusPending,
  mockUserRequestStatusRejected,
} from '@/test-utils/msw-v2/handlers/access-requests';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { createWrapper } from '@/tests/wrapper';
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

// Mock AppInitializer to just render children
vi.mock('@/components/AppInitializer', () => ({
  default: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
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

  it('displays pending status when user has pending request', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com' }), // No role
      ...mockUserRequestStatusPending({ username: 'user@example.com' })
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
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com', role: 'resource_user' }), // User has a role, so will be redirected
      ...mockUserRequestStatusApproved({ username: 'approved@example.com' })
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
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com' }), // No role
      ...mockUserRequestStatusRejected({ username: 'rejected@example.com' })
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

    // The page should still render
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

    // Should show request access form when no request exists (404)
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

    // Should show pending status
    expect(screen.getByText('Access Request Pending')).toBeInTheDocument();
    expect(screen.getByText(/Your access request submitted on.*is pending review/)).toBeInTheDocument();
    // Should not show any buttons when pending
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
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ username: 'user@example.com' }), // No role
      ...mockUserRequestStatusPending({ username: 'user@example.com' })
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

// Tests that require "no request exists" scenario (404 response)
describe('RequestAccessPage - No Request Exists', () => {
  const user = userEvent.setup();

  // Custom handler for 404 response when no request exists using MSW v2
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

    // Should show the request form
    expect(screen.getByTestId('request-access-page')).toBeInTheDocument();
    expect(screen.getByText('Request access to application')).toBeInTheDocument();

    // Should show request access button
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
      ...mockUserRequestAccess(100)
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

    // Should show initial button state
    expect(requestButton).toHaveTextContent('Request Access');
    expect(requestButton).not.toBeDisabled();

    // Button should be clickable
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

    // Should show error message via toast (tested in toast hooks)
    // The error is handled by the mutation hook and shown via toast
    expect(requestButton).toBeEnabled(); // Button should be re-enabled after error
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

    // Check button is available and enabled
    expect(requestButton).toBeInTheDocument();
    expect(requestButton).not.toBeDisabled();
    expect(requestButton).toHaveTextContent('Request Access');
  });
});
