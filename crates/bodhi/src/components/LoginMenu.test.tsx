import { LoginMenu } from '@/components/LoginMenu';
import { ENDPOINT_APP_INFO, ENDPOINT_AUTH_INITIATE, ENDPOINT_LOGOUT, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

const mockPush = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: mockPush,
    refresh: vi.fn(),
  }),
  usePathname: () => '/',
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
      ctx.status(401), // 401 when login required
      ctx.json({ auth_url: 'https://oauth.example.com/auth?client_id=test' })
    );
  })
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
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
    // Mock window.location.href
    const originalLocation = window.location;
    delete (window as any).location;
    window.location = { ...originalLocation, href: '' };

    render(<LoginMenu />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: /login/i });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(window.location.href).toBe('https://oauth.example.com/auth?client_id=test');
    });

    // Restore window.location
    window.location = originalLocation;
  });

  it('handles logout action', async () => {
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json(mockLoggedInUser));
      }),
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(ctx.delay(100), ctx.status(200));
      })
    );

    render(<LoginMenu />, { wrapper: createWrapper() });

    const logoutButton = await screen.findByRole('button', { name: /log out/i });
    await userEvent.click(logoutButton);

    expect(screen.getByRole('button', { name: /logging out/i })).toBeInTheDocument();
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
          ctx.status(401), // 401 when login required
          ctx.json({ auth_url: 'https://oauth.example.com/auth?client_id=test' })
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
    // Mock window.location.href
    const originalLocation = window.location;
    delete (window as any).location;
    window.location = { ...originalLocation, href: '' };

    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(ctx.status(303), ctx.set('Location', 'https://example.com/redirected'), ctx.json({}));
      })
    );

    render(<LoginMenu />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: /login/i });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(window.location.href).toBe('https://example.com/redirected');
    });

    // Restore window.location
    window.location = originalLocation;
  });
});
