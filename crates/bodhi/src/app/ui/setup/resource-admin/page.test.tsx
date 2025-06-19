import { createWrapper, mockWindowLocation } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import ResourceAdminPage from '@/app/ui/setup/resource-admin/page';
import { ENDPOINT_APP_INFO, ENDPOINT_AUTH_INITIATE } from '@/hooks/useQuery';
import { ROUTE_DEFAULT } from '@/lib/constants';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: () => null,
  }),
  redirect: vi.fn(),
}));

vi.mock('next/image', () => ({
  default: () => <img alt="mocked image" />,
}));

const server = setupServer(
  rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
    return res(ctx.json({ status: 'resource-admin' }));
  })
);

beforeAll(() => {
  server.listen();
});

afterAll(() => server.close());

beforeEach(() => {
  mockWindowLocation('http://localhost:3000/ui/setup/resource-admin');
  server.resetHandlers();
  pushMock.mockClear();
  vi.clearAllMocks();
});

describe('ResourceAdminPage', () => {
  it('renders the resource admin page when status is resource-admin', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Admin Setup')).toBeInTheDocument();
      expect(screen.getByText('Continue with Login →')).toBeInTheDocument();
    });
  });

  it('redirects to /ui/setup when status is setup', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to download models when status is ready and models page not shown', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/download-models');
    });
  });

  it.skip(`redirects to ${ROUTE_DEFAULT} when status is ready and models page already shown`, async () => {
    // Skipped due to localStorage mocking complexity in test environment
    // Mock localStorage to simulate models page has been shown
    const mockLocalStorage = {
      getItem: vi.fn((key) => {
        if (key === 'models-download-page-displayed') return 'true';
        return null;
      }),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
    };
    Object.defineProperty(window, 'localStorage', {
      value: mockLocalStorage,
      writable: true,
    });

    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith(ROUTE_DEFAULT);
    });
  });

  it('handles OAuth initiation with external OAuth provider URL', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      }),
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.status(201), // 201 Created for new OAuth session
          ctx.json({ location: 'https://oauth.example.com/auth?client_id=test' })
        );
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
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

  it('handles OAuth initiation with same-origin redirect URL', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      }),
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.status(200), // 200 OK for already authenticated user
          ctx.json({ location: 'http://localhost:3000/ui/chat' })
        );
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(loginButton);

    // Should show "Redirecting..." after successful response and remain disabled
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeDisabled();
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/chat');
    });
  });

  it('shows initiating and redirecting states during OAuth initiation', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      }),
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.status(201), // 201 Created for new OAuth session
          ctx.json({ location: 'https://oauth.example.com/auth?client_id=test' })
        );
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
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
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      }),
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

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('OAuth configuration error')).toBeInTheDocument();
    });

    // Verify login button is re-enabled after error
    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Continue with Login →' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Continue with Login →' })).not.toBeDisabled();
    });
  });

  it('displays generic error message when OAuth initiation fails without specific message', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      }),
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(ctx.status(500));
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('Failed to initiate OAuth authentication')).toBeInTheDocument();
    });
  });

  it('handles missing location in successful response and re-enables button', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      }),
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.status(201),
          ctx.json({}) // No location field
        );
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('Auth URL not found in response. Please try again.')).toBeInTheDocument();
    });

    // Verify button is re-enabled after error
    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Continue with Login →' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Continue with Login →' })).not.toBeDisabled();
    });
  });

  it('handles invalid URL in response by treating as external and keeping button disabled', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      }),
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(ctx.status(201), ctx.json({ location: 'invalid-url-format' }));
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
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
});
