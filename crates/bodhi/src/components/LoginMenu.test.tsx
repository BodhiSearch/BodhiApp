import { LoginMenu } from '@/components/LoginMenu';
import { ENDPOINT_APP_INFO, ENDPOINT_AUTH_INITIATE, ENDPOINT_LOGOUT, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { redirect } from 'next/navigation';

const mockPush = vi.fn();
const mockRedirect = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: mockPush,
    refresh: vi.fn(),
  }),
  usePathname: () => '/',
  redirect: vi.fn(),
}));

const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

const mockLoggedOutUser = {
  logged_in: false,
  email: null,
  roles: [],
};

const mockLoggedInUser = {
  logged_in: true,
  email: 'test@example.com',
  roles: ['user'],
};

const mockAppInfo = {
  status: 'ready',
};

const server = setupServer(
  rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
    return res(ctx.json(mockLoggedOutUser));
  }),
  rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
    return res(ctx.json(mockAppInfo));
  }),
  rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
    return res(
      ctx.status(303), // 303 redirect to OAuth URL
      ctx.set('Location', 'https://oauth.example.com/auth?client_id=test')
    );
  })
);

beforeAll(() => server.listen());
afterEach(() => {
  server.resetHandlers();
  vi.clearAllMocks();
});
afterAll(() => server.close());

describe('LoginMenu Component', () => {
  it('shows login button when logged out', async () => {
    render(<LoginMenu />, { wrapper: createWrapper() });

    await waitFor(() => {
      const loginButton = screen.getByRole('button', { name: /login/i });
      expect(loginButton).toBeInTheDocument();
      expect(loginButton).toHaveClass('border-primary');
    });
  });

  it('shows logout button and email when logged in', async () => {
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json(mockLoggedInUser));
      })
    );

    render(<LoginMenu />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /log out/i })).toBeInTheDocument();
      expect(screen.getByText(`Logged in as ${mockLoggedInUser.email}`)).toBeInTheDocument();
    });
  });

  it('handles OAuth initiation on login button click', async () => {
    render(<LoginMenu />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: /login/i });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(redirect).toHaveBeenCalledWith('https://oauth.example.com/auth?client_id=test');
    });
  });

  it('handles logout action with external redirect URL', async () => {
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json(mockLoggedInUser));
      }),
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.status(201),
          ctx.set('Location', 'http://localhost:1135/ui/login'),
          ctx.set('Content-Length', '0')
        );
      })
    );

    render(<LoginMenu />, { wrapper: createWrapper() });

    const logoutButton = await screen.findByRole('button', { name: /log out/i });
    await userEvent.click(logoutButton);

    expect(screen.getByRole('button', { name: /logging out/i })).toBeInTheDocument();

    await waitFor(() => {
      expect(redirect).toHaveBeenCalledWith('http://localhost:1135/ui/login');
    });
  });

  it('handles logout action with internal redirect URL', async () => {
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json(mockLoggedInUser));
      }),
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(ctx.delay(100), ctx.status(201), ctx.set('Location', '/ui/login'), ctx.set('Content-Length', '0'));
      })
    );

    render(<LoginMenu />, { wrapper: createWrapper() });

    const logoutButton = await screen.findByRole('button', { name: /log out/i });
    await userEvent.click(logoutButton);

    expect(screen.getByRole('button', { name: /logging out/i })).toBeInTheDocument();

    await waitFor(() => {
      expect(redirect).toHaveBeenCalledWith('/ui/login');
    });
  });

  it('handles logout error', async () => {
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json(mockLoggedInUser));
      }),
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(ctx.status(500), ctx.json({ error: { message: 'Session deletion failed' } }));
      })
    );

    render(<LoginMenu />, { wrapper: createWrapper() });

    const logoutButton = await screen.findByRole('button', { name: /log out/i });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /log out/i })).toBeInTheDocument();
    });

    expect(logoutButton).not.toBeDisabled();

    // Should redirect to login page on error
    await waitFor(() => {
      expect(redirect).toHaveBeenCalledWith('/ui/login');
    });
  });

  it('handles logout with missing Location header', async () => {
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json(mockLoggedInUser));
      }),
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(ctx.status(201), ctx.set('Content-Length', '0')); // No Location header
      })
    );

    render(<LoginMenu />, { wrapper: createWrapper() });

    const logoutButton = await screen.findByRole('button', { name: /log out/i });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(redirect).toHaveBeenCalledWith('/ui/chat');
    });
  });

  it('shows nothing during loading', async () => {
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.delay(100), ctx.json(mockLoggedInUser));
      })
    );

    const { container } = render(<LoginMenu />, { wrapper: createWrapper() });
    expect(container).toBeEmptyDOMElement();
  });

  it('shows error message when OAuth initiation fails', async () => {
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

    render(<LoginMenu />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: /login/i });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('OAuth configuration error')).toBeInTheDocument();
    });
  });

  it('shows redirecting state during OAuth initiation', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.status(303), // 303 redirect to OAuth URL
          ctx.set('Location', 'https://oauth.example.com/auth?client_id=test')
        );
      })
    );

    render(<LoginMenu />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: /login/i });
    await userEvent.click(loginButton);

    // Check for redirecting state immediately after click
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeInTheDocument();
    });
  });

  it('redirects to location when OAuth initiation returns 303', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(ctx.status(303), ctx.set('Location', 'https://example.com/redirected'));
      })
    );

    render(<LoginMenu />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: /login/i });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(redirect).toHaveBeenCalledWith('https://example.com/redirected');
    });
  });

  it('shows error when 303 response has no Location header', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(ctx.status(303)); // No Location header
      })
    );

    render(<LoginMenu />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: /login/i });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('Auth URL not found in response. Please try again.')).toBeInTheDocument();
    });
  });
});
