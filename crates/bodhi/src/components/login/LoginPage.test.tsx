import LoginPage, { LoginContent } from '@/components/login/LoginPage';
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import {
  afterAll,
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';
import { ENDPOINT_AUTH_INITIATE } from '@/hooks/useQuery';

// Mock window.location
const mockLocation = {
  href: '',
  origin: 'http://localhost:3000',
};
Object.defineProperty(window, 'location', {
  value: mockLocation,
  writable: true,
});

// Mock the hooks
const server = setupServer(
  // Default mock for app info endpoint
  rest.get('*/bodhi/v1/info', (req, res, ctx) => {
    return res(
      ctx.status(200),
      ctx.json({
        status: 'ready',
      })
    );
  })
);

const pushMock = vi.fn();
vi.mock('@/lib/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

beforeAll(() => server.listen());
beforeEach(() => {
  vi.clearAllMocks();
  pushMock.mockClear();
  mockLocation.href = '';
});
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('LoginContent', () => {
  it('renders sign in button', () => {
    render(<LoginContent />, { wrapper: createWrapper() });

    expect(screen.getByText('Welcome to Bodhi')).toBeInTheDocument();
    expect(
      screen.getByText('Sign in to access your AI assistant')
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Sign In' })).toBeInTheDocument();
  });

  it('renders sign in button with correct styling', () => {
    render(<LoginContent />, { wrapper: createWrapper() });

    const signInButton = screen.getByRole('button', { name: 'Sign In' });
    expect(signInButton).toHaveClass('w-full');
    expect(signInButton).not.toBeDisabled();
  });

  it('calls OAuth initiate when sign in button is clicked', async () => {
    const authUrl = 'https://auth.example.com/oauth/authorize?client_id=test';

    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (req, res, ctx) => {
        return res(
          ctx.status(401),
          ctx.set('WWW-Authenticate', 'Bearer realm="OAuth"'),
          ctx.json({
            auth_url: authUrl,
          })
        );
      })
    );

    render(<LoginContent />, { wrapper: createWrapper() });

    const signInButton = screen.getByRole('button', { name: 'Sign In' });
    await userEvent.click(signInButton);

    // Should redirect to auth URL
    await waitFor(() => {
      expect(mockLocation.href).toBe(authUrl);
    });
  });

  it('disables button when OAuth is loading', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (req, res, ctx) => {
        return res(
          ctx.delay(200), // Increased delay to ensure we can catch the loading state
          ctx.status(401),
          ctx.set('WWW-Authenticate', 'Bearer realm="OAuth"'),
          ctx.json({
            auth_url:
              'https://auth.example.com/oauth/authorize?client_id=test',
          })
        );
      })
    );

    render(<LoginContent />, { wrapper: createWrapper() });

    const signInButton = screen.getByRole('button', { name: 'Sign In' });

    // Click the button to start the OAuth flow
    await userEvent.click(signInButton);

    // Wait for the loading state to appear (AuthCard shows loading animation)
    await waitFor(
      () => {
        expect(screen.getByTestId('auth-card-loading')).toBeInTheDocument();
      },
      { timeout: 150 }
    );
  });

  it('displays error state when OAuth initiation fails', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (req, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: {
              message: 'OAuth service unavailable',
              type: 'server_error',
            },
          })
        );
      })
    );

    render(<LoginContent />, { wrapper: createWrapper() });

    const signInButton = screen.getByRole('button', { name: 'Sign In' });
    await userEvent.click(signInButton);

    await waitFor(() => {
      expect(screen.getByText('Authentication Error')).toBeInTheDocument();
    });

    expect(screen.getByText('OAuth service unavailable')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Try Again' })).toBeInTheDocument();
  });

  it('allows retry after error', async () => {
    const authUrl = 'https://auth.example.com/oauth/authorize';
    let callCount = 0;

    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (req, res, ctx) => {
        callCount++;
        if (callCount === 1) {
          return res(
            ctx.status(500),
            ctx.json({
              error: {
                message: 'Temporary error',
                type: 'server_error',
              },
            })
          );
        }
        return res(
          ctx.status(401),
          ctx.json({ auth_url: authUrl })
        );
      })
    );

    render(<LoginContent />, { wrapper: createWrapper() });

    // First attempt - should fail
    const signInButton = screen.getByRole('button', { name: 'Sign In' });
    await userEvent.click(signInButton);

    await waitFor(() => {
      expect(screen.getByText('Authentication Error')).toBeInTheDocument();
    });

    // Retry - should succeed
    const tryAgainButton = screen.getByRole('button', { name: 'Try Again' });
    await userEvent.click(tryAgainButton);

    await waitFor(() => {
      expect(mockLocation.href).toBe(authUrl);
    });
  });

  it('clears error state when retrying', async () => {
    const user = userEvent.setup();

    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (req, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: { message: 'Server error' },
          })
        );
      })
    );

    render(<LoginContent />, { wrapper: createWrapper() });

    const signInButton = screen.getByRole('button', { name: 'Sign In' });
    await user.click(signInButton);

    // Wait for error state
    await waitFor(() => {
      expect(screen.getByText('Authentication Error')).toBeInTheDocument();
    });

    // Now mock a successful response for retry
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (req, res, ctx) => {
        return res(
          ctx.status(401),
          ctx.json({
            auth_url: 'https://oauth-server.com/auth',
          })
        );
      })
    );

    const tryAgainButton = screen.getByRole('button', { name: 'Try Again' });
    await user.click(tryAgainButton);

    // Should briefly clear error state
    await waitFor(() => {
      expect(screen.queryByText('Authentication Error')).not.toBeInTheDocument();
    });
  });
});

describe('LoginPage', () => {
  it('renders the login content within app initializer', async () => {
    render(<LoginPage />, { wrapper: createWrapper() });

    // Wait for the app initializer to finish loading
    await waitFor(() => {
      expect(screen.getByTestId('login-page')).toBeInTheDocument();
    });

    expect(screen.getByText('Welcome to Bodhi')).toBeInTheDocument();
  });
});
