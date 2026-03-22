import AuthCallbackPage from '@/app/auth/callback/page';
import { mockAuthCallback, mockAuthCallbackError, mockAuthCallbackStateError } from '@/test-utils/msw-v2/handlers/auth';
import { createWrapper, mockWindowLocation } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const navigateMock = vi.fn();
let mockSearch: Record<string, string | undefined> = { code: 'test-code', state: 'test-state' };
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
    useSearch: () => mockSearch,
  };
});

const server = setupServer();
beforeAll(() => {
  server.listen();
});

afterAll(() => server.close());

beforeEach(() => {
  mockWindowLocation('http://localhost:3000/ui/auth/callback');
  server.resetHandlers();
  navigateMock.mockClear();
  vi.clearAllMocks();
  mockSearch = { code: 'test-code', state: 'test-state' };
});

describe('AuthCallbackPage', () => {
  it('renders processing state initially', async () => {
    server.use(...mockAuthCallback({ location: 'http://localhost:3000/ui/chat' }));

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    expect(screen.getByTestId('oauth-callback-page')).toBeInTheDocument();
    expect(screen.getByText('Processing Login...')).toBeInTheDocument();
    expect(screen.getByText('Please wait while we complete your login...')).toBeInTheDocument();
  });

  it('handles successful OAuth callback with same-origin URL redirect', async () => {
    server.use(...mockAuthCallback({ location: 'http://localhost:3000/ui/chat' }));

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/chat' });
    });
  });

  it('handles successful OAuth callback with external URL redirect', async () => {
    mockWindowLocation('https://external.example.com/dashboard');
    server.use(...mockAuthCallback({ location: 'https://external.example.com/dashboard' }));

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(window.location.href).toBe('https://external.example.com/dashboard');
    });
  });

  it('handles same-origin URL with different path and query', async () => {
    server.use(...mockAuthCallback({ location: 'http://localhost:3000/ui/setup/download-models?step=1' }));
    render(<AuthCallbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({
        to: '/setup/download-models',
        search: { step: '1' },
      });
    });
  });

  it('handles different port as external URL', async () => {
    server.use(...mockAuthCallback({ location: 'http://localhost:8080/ui/chat' }));
    render(<AuthCallbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(window.location.href).toBe('http://localhost:8080/ui/chat');
    });
  });

  it('handles different protocol as external URL', async () => {
    server.use(...mockAuthCallback({ location: 'https://localhost:3000/ui/chat' }));

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(window.location.href).toBe('https://localhost:3000/ui/chat');
    });
  });

  it('sends all OAuth callback parameters to backend', async () => {
    mockSearch = {
      code: 'test-auth-code',
      state: 'test-state',
      scope: 'openid email profile',
      session_state: 'session-123',
    };
    server.use(
      ...mockAuthCallback(
        { location: 'http://localhost:3000/ui/chat' },
        {
          code: 'test-auth-code',
          state: 'test-state',
          scope: 'openid email profile',
          session_state: 'session-123',
        }
      )
    );

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/chat' });
    });
  });

  it('handles OAuth callback error and shows error state', async () => {
    server.use(...mockAuthCallbackStateError());

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Login Error')).toBeInTheDocument();
      expect(screen.getByText('Invalid state parameter')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Try Again' })).toBeInTheDocument();
    });
  });

  it('handles default callback response', async () => {
    server.use(...mockAuthCallback());

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/chat' });
    });
  });

  it('handles OAuth error parameters from provider', async () => {
    server.use(...mockAuthCallbackError({ message: 'OAuth provider error: access_denied - User denied access' }));

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Login Error')).toBeInTheDocument();
      expect(screen.getByText('OAuth provider error: access_denied - User denied access')).toBeInTheDocument();
    });
  });

  it('handles retry button click', async () => {
    server.use(...mockAuthCallbackError());

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Internal server error')).toBeInTheDocument();
    });
    server.use(...mockAuthCallback({ location: 'http://localhost:3000/ui/chat' }));

    const retryButton = screen.getByRole('button', { name: 'Try Again' });
    await userEvent.click(retryButton);
    await waitFor(() => {
      expect(screen.getByText('Processing Login...')).toBeInTheDocument();
    });
    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/chat' });
    });
  });

  it('disables retry button while loading', async () => {
    server.use(...mockAuthCallbackError());
    render(<AuthCallbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Internal server error')).toBeInTheDocument();
    });
    const retryButton = screen.getByRole('button', { name: 'Try Again' });
    server.use(...mockAuthCallback({ location: 'http://localhost:3000/ui/chat' }, undefined, 200));
    await userEvent.click(retryButton);
    expect(screen.getByText('Processing Login...')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Try Again' })).not.toBeInTheDocument();
  });

  it('handles empty search parameters', async () => {
    mockSearch = {};
    server.use(
      ...mockAuthCallbackError({
        code: 'missing_parameters',
        message: 'Missing required OAuth parameters',
        type: 'invalid_request_error',
      })
    );
    render(<AuthCallbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Missing required OAuth parameters')).toBeInTheDocument();
    });
  });

  it('handles custom URL in response by treating as external', async () => {
    server.use(...mockAuthCallback({ location: 'https://external.example.com/callback' }));
    render(<AuthCallbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(window.location.href).toBe('https://external.example.com/callback');
    });
  });
});
