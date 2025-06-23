import LoginPage, { LoginContent } from '@/app/ui/login/page';
import { ENDPOINT_APP_INFO, ENDPOINT_AUTH_INITIATE, ENDPOINT_LOGOUT, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { createWrapper, mockWindowLocation } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { redirect } from 'next/navigation';

// Mock the hooks
const server = setupServer();
const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  redirect: vi.fn(),
}));

beforeAll(() => {
  server.listen();
});

afterAll(() => server.close());

beforeEach(() => {
  mockWindowLocation('http://localhost:3000/ui/login');
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
        return res(ctx.delay(100), ctx.status(200), ctx.json({ logged_in: false }));
      }),
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.delay(100), ctx.status(200), ctx.json({ status: 'ready' }));
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
        return res(ctx.status(200), ctx.json({ location: 'http://localhost:1135/ui/login' }));
      })
    );
  });

  it('renders login button when user is not logged in', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const loginButton = screen.getByRole('button', { name: 'Login' });
    expect(loginButton).toBeDefined();
    expect(screen.getByText('Login to use the Bodhi App')).toBeInTheDocument();
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
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.status(201), // 201 Created for new OAuth session
          ctx.json({ location: 'https://oauth.example.com/auth?client_id=test' })
        );
      })
    );

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    // Should show "Redirecting..." after successful response and remain disabled
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeDisabled();
    });

    await waitFor(() => {
      expect(window.location.href).toBe('https://oauth.example.com/auth?client_id=test');
    });
  });

  it('shows initiating and redirecting states during OAuth initiation', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.status(201), // 201 Created for new OAuth session
          ctx.json({ location: 'https://oauth.example.com/auth?client_id=test' })
        );
      })
    );

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    // Check for initiating state during request
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /initiating/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /initiating/i })).toBeDisabled();
    });

    // Check for redirecting state after successful response
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeDisabled();
    });
  });

  it('displays error message when OAuth initiation fails and re-enables button', async () => {
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

    // Verify login button is re-enabled after error
    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Login' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Login' })).not.toBeDisabled();
    });
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

  it('handles already authenticated user with external redirect URL', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.status(200), // 200 OK for already authenticated user
          ctx.json({ location: 'https://external.example.com/dashboard' })
        );
      })
    );

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(window.location.href).toBe('https://external.example.com/dashboard');
    });
  });

  it('shows error when response has no location field and re-enables button', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(ctx.status(201), ctx.json({})); // No location field
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

    // Verify button is re-enabled after error
    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Login' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Login' })).not.toBeDisabled();
    });
  });

  it('handles invalid URL in response by treating as external and keeping button disabled', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(ctx.status(201), ctx.json({ location: 'invalid-url-format' }));
      })
    );

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    // Should show "Redirecting..." and remain disabled even for invalid URLs
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeDisabled();
    });

    await waitFor(() => {
      expect(window.location.href).toBe('invalid-url-format');
    });
  });

  it('handles already authenticated user with same-origin redirect URL', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ location: 'http://localhost:3000/ui/chat' }));
      })
    );

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });

    await act(async () => {
      await userEvent.click(loginButton);
    });

    // Should show "Redirecting..." after successful response
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeDisabled();
    });

    // Should use router.push for same-origin URLs
    await waitFor(
      () => {
        expect(pushMock).toHaveBeenCalledWith('/ui/chat');
      },
      { timeout: 3000 }
    );
  });
});

describe('LoginContent with user Logged In', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ logged_in: true, email: 'test@example.com' }));
      }),
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ status: 'ready' }));
      }),
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ location: 'http://localhost:1135/ui/login' }));
      })
    );
  });

  it('renders welcome message and logout button when user is logged in', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    expect(screen.getByText('You are logged in as test@example.com')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Go to Home' })).toBeInTheDocument();
  });

  it('calls logout function when logout button is clicked and redirects to external location', async () => {
    server.use(
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ location: 'http://localhost:1135/ui/test/login' }));
      })
    );
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(redirect).toHaveBeenCalledWith('http://localhost:1135/ui/test/login');
    });
  });

  it('calls logout function when logout button is clicked and redirects to internal location', async () => {
    server.use(
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ location: '/ui/login' }));
      })
    );
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });

  it('disables logout button and shows loading text when logging out', async () => {
    server.use(
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(ctx.delay(100), ctx.status(200), ctx.json({ location: 'http://localhost:1135/ui/test/login' }));
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
