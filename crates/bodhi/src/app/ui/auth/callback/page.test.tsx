import AuthCallbackPage from '@/app/ui/auth/callback/page';
import { ENDPOINT_AUTH_CALLBACK } from '@/hooks/useOAuth';
import { createWrapper, mockWindowLocation } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
let mockSearchParams = new URLSearchParams('code=test-code&state=test-state');
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: vi.fn((key: string) => mockSearchParams.get(key)),
    forEach: vi.fn((callback: (value: string, key: string) => void) => {
      mockSearchParams.forEach(callback);
    }),
  }),
}));
vi.mock('next/image', () => ({
  default: () => <img alt="mocked image" />,
}));

const server = setupServer();
beforeAll(() => {
  server.listen();
});

afterAll(() => server.close());

beforeEach(() => {
  mockWindowLocation('http://localhost:3000/ui/auth/callback');
  server.resetHandlers();
  pushMock.mockClear();
  vi.clearAllMocks();
  mockSearchParams = new URLSearchParams('code=test-code&state=test-state');
});

describe('AuthCallbackPage', () => {
  it('renders processing state initially', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(
          ctx.delay(100), // Delay to keep processing state
          ctx.status(200),
          ctx.json({ location: 'http://localhost:3000/ui/chat' })
        );
      })
    );

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    expect(screen.getByTestId('oauth-callback-page')).toBeInTheDocument();
    expect(screen.getByText('Processing Login...')).toBeInTheDocument();
    expect(screen.getByText('Please wait while we complete your login...')).toBeInTheDocument();
  });

  it('handles successful OAuth callback with same-origin URL redirect', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (req, res, ctx) => {
        return res(ctx.status(200), ctx.json({ location: 'http://localhost:3000/ui/chat' }));
      })
    );

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/chat');
    });
  });

  it('handles successful OAuth callback with external URL redirect', async () => {
    mockWindowLocation('https://external.example.com/dashboard');
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ location: 'https://external.example.com/dashboard' }));
      })
    );

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(window.location.href).toBe('https://external.example.com/dashboard');
    });
  });

  it('handles same-origin URL with different path, query, and hash', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(
          ctx.status(200),
          ctx.json({ location: 'http://localhost:3000/ui/setup/download-models?step=1#section' })
        );
      })
    );
    render(<AuthCallbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/download-models?step=1#section');
    });
  });

  it('handles different port as external URL', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ location: 'http://localhost:8080/ui/chat' }));
      })
    );
    render(<AuthCallbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(window.location.href).toBe('http://localhost:8080/ui/chat');
    });
  });

  it('handles different protocol as external URL', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ location: 'https://localhost:3000/ui/chat' }));
      })
    );

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(window.location.href).toBe('https://localhost:3000/ui/chat');
    });
  });

  it('sends all OAuth callback parameters to backend', async () => {
    mockSearchParams = new URLSearchParams(
      'code=test-auth-code&state=test-state&scope=openid email profile&session_state=session-123'
    );

    let receivedParams: any = null;

    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, async (req, res, ctx) => {
        receivedParams = await req.json();
        return res(ctx.status(200), ctx.json({ location: 'http://localhost:3000/ui/chat' }));
      })
    );

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(receivedParams).toEqual({
        code: 'test-auth-code',
        state: 'test-state',
        scope: 'openid email profile',
        session_state: 'session-123',
      });
    });
  });

  it('handles OAuth callback error and shows error state', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(
          ctx.status(400),
          ctx.json({
            error: {
              message: 'Invalid state parameter',
              type: 'invalid_request',
              code: 'invalid_state',
            },
          })
        );
      })
    );

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Login Error')).toBeInTheDocument();
      expect(screen.getByText('Invalid state parameter')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Try Again' })).toBeInTheDocument();
    });
  });

  it('handles missing location in successful response', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({}));
      })
    );

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Login Error')).toBeInTheDocument();
      expect(screen.getByText('Redirect URL not found in response. Please try again.')).toBeInTheDocument();
    });
  });

  it('handles OAuth error parameters from provider', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(
          ctx.status(400),
          ctx.json({
            error: {
              message: 'OAuth provider error: access_denied - User denied access',
              type: 'oauth_error',
              code: 'access_denied',
            },
          })
        );
      })
    );

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Login Error')).toBeInTheDocument();
      expect(screen.getByText('OAuth provider error: access_denied - User denied access')).toBeInTheDocument();
    });
  });

  it('handles retry button click', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: {
              message: 'Internal server error',
              type: 'internal_server_error',
              code: 'server_error',
            },
          })
        );
      })
    );

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Internal server error')).toBeInTheDocument();
    });
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ location: 'http://localhost:3000/ui/chat' }));
      })
    );

    const retryButton = screen.getByRole('button', { name: 'Try Again' });
    await userEvent.click(retryButton);
    await waitFor(() => {
      expect(screen.getByText('Processing Login...')).toBeInTheDocument();
    });
    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/chat');
    });
  });

  it('disables retry button while loading', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: {
              message: 'Server error',
              type: 'internal_server_error',
              code: 'server_error',
            },
          })
        );
      })
    );
    render(<AuthCallbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Server error')).toBeInTheDocument();
    });
    const retryButton = screen.getByRole('button', { name: 'Try Again' });
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(
          ctx.delay(200), // Longer delay to test disabled state
          ctx.status(200),
          ctx.json({ location: 'http://localhost:3000/ui/chat' })
        );
      })
    );
    await userEvent.click(retryButton);
    expect(screen.getByText('Processing Login...')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Try Again' })).not.toBeInTheDocument();
  });

  it('handles empty search parameters', async () => {
    mockSearchParams = new URLSearchParams('');
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, async (req, res, ctx) => {
        const params = await req.json();
        expect(params).toEqual({});
        return res(
          ctx.status(400),
          ctx.json({
            error: {
              message: 'Missing required OAuth parameters',
              type: 'invalid_request',
              code: 'missing_parameters',
            },
          })
        );
      })
    );
    render(<AuthCallbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Missing required OAuth parameters')).toBeInTheDocument();
    });
  });

  it('handles invalid URL in response by treating as external', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ location: 'invalid-url-format' }));
      })
    );
    render(<AuthCallbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(window.location.href).toBe('invalid-url-format');
    });
  });
});
