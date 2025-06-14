import LoginPage, { LoginContent } from '@/app/ui/login/page';
import { ENDPOINT_APP_INFO, ENDPOINT_AUTH_INITIATE, ENDPOINT_LOGOUT, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import {
  act,
  render,
  screen,
  waitFor
} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import {
  afterAll,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi
} from 'vitest';

// Mock the hooks
const server = setupServer();
const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

beforeAll(() => server.listen());
afterAll(() => server.close());
beforeEach(() => {
  server.resetHandlers();
  pushMock.mockClear();
  vi.clearAllMocks();
});

describe('LoginContent loading states', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (req, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.status(200),
          ctx.json({ logged_in: false })
        );
      }),
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.status(200),
          ctx.json({ status: 'ready' })
        );
      })
    );
  });

  it('shows loading indicator while fetching data', async () => {
    render(<LoginContent />, { wrapper: createWrapper() });
    expect(screen.getByText('Loading...')).toBeInTheDocument();
    await waitFor(() => expect(screen.getByRole('button', { name: 'Login' })).toBeInTheDocument());
  });
});

describe('LoginContent with user not Logged In', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (req, res, ctx) => {
        return res(ctx.status(200), ctx.json({ logged_in: false }));
      }),
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ status: 'ready' }));
      }),
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(ctx.status(500), ctx.json({ error: { message: 'OAuth configuration error' } }));
      }),
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({}));
      })
    );
  });

  it('renders login button when user is not logged in', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const loginButton = screen.getByRole('button', { name: 'Login' });
    expect(loginButton).toBeDefined();
    expect(
      screen.getByText('Login to use the Bodhi App')
    ).toBeInTheDocument();
  });

  it('renders login button with correct styling', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const loginButton = screen.getByRole('button', { name: 'Login' });
    expect(loginButton).toHaveClass('w-full');
    expect(loginButton).not.toBeDisabled();
  });

  it('handles OAuth initiation when login required and redirects to auth URL', async () => {
    // Mock window.location.href
    const mockLocation = { href: '' };
    Object.defineProperty(window, 'location', {
      value: mockLocation,
      writable: true,
    });

    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.status(303), // 303 redirect to OAuth URL
          ctx.set('Location', 'https://oauth.example.com/auth?client_id=test'),
        );
      })
    );

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(window.location.href).toBe('https://oauth.example.com/auth?client_id=test');
    });
  });

  it('shows redirecting state during OAuth initiation', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.status(303), // 303 redirect to OAuth URL
          ctx.set('Location', 'https://oauth.example.com/auth?client_id=test'),
        );
      })
    );

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    // Check for redirecting state immediately after click
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeInTheDocument();
    });
  });

  it('displays error message when OAuth initiation fails', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: {
              message: 'OAuth configuration error',
              type: 'internal_server_error',
              code: 'oauth_config_error',
            },
          })
        );
      })
    );

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('OAuth configuration error')).toBeInTheDocument();
    });

    // Verify login button is still available after error
    expect(screen.getByRole('button', { name: 'Login' })).toBeInTheDocument();
  });

  it('displays generic error message when OAuth initiation fails without specific message', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(ctx.status(500));
      })
    );

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('Failed to initiate OAuth authentication')).toBeInTheDocument();
    });
  });

  it('handles already authenticated user with 303 redirect', async () => {
    // Mock window.location.href
    const mockLocation = { href: '' };
    Object.defineProperty(window, 'location', {
      value: mockLocation,
      writable: true,
    });

    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.status(303), // 303 when already authenticated
          ctx.set('Location', 'http://localhost:3000/ui/chat'),
        );
      })
    );

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    // Should redirect to the location header value
    await waitFor(() => {
      expect(window.location.href).toBe('http://localhost:3000/ui/chat');
    });
  });

  it('shows error when 303 response has no Location header', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(ctx.status(303)); // No Location header
      })
    );

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('Auth URL not found in response. Please try again.')).toBeInTheDocument();
    });
  });
});

describe('LoginContent with user Logged In', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(
          ctx.status(200),
          ctx.json({ logged_in: true, email: 'test@example.com' })
        );
      }),
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ status: 'ready' }));
      }),
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({}));
      })
    );
  });

  it('renders welcome message and logout button when user is logged in', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    expect(
      screen.getByText('You are logged in as test@example.com')
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Go to Home' })).toBeInTheDocument();
  });

  it('calls logout function when logout button is clicked and pushes the route in location', async () => {
    server.use(
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(
          ctx.status(200),
          ctx.set('Location', 'http://localhost:1135/ui/test/login'),
          ctx.json({})
        );
      })
    );
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);
    expect(pushMock).toHaveBeenCalledWith(
      'http://localhost:1135/ui/test/login'
    );
  });

  it('disables logout button and shows loading text when logging out', async () => {
    server.use(
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.status(200),
          ctx.set('Location', 'http://localhost:1135/ui/test/login'),
          ctx.json({})
        );
      })
    );

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);
    const loggingOut = screen.getByRole('button', { name: 'Logging out...' });
    expect(loggingOut).toBeInTheDocument();
    expect(loggingOut).toHaveAttribute('disabled');
  });

  it('renders buttons with correct styling', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    expect(logoutButton).toHaveClass('w-full');
  });
});

describe('LoginContent access control', () => {
  it('redirects to setup when app is not setup', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ status: 'setup' }));
      })
    );
    await act(async () => {
      render(<LoginPage />, { wrapper: createWrapper() });
    });
    await waitFor(() => expect(pushMock).toHaveBeenCalledWith('/ui/setup'));
  });
});
