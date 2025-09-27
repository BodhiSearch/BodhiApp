import AuthCallbackPage from '@/app/ui/auth/callback/page';
import { ENDPOINT_AUTH_CALLBACK } from '@/hooks/useOAuth';
import { createWrapper, mockWindowLocation } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { mockAuthCallback, mockAuthCallbackError } from '@/test-utils/msw-v2/handlers/auth';
import { http, HttpResponse } from 'msw';

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
    server.use(...mockAuthCallback({ delay: 100, location: 'http://localhost:3000/ui/chat' }));

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    expect(screen.getByTestId('oauth-callback-page')).toBeInTheDocument();
    expect(screen.getByText('Processing Login...')).toBeInTheDocument();
    expect(screen.getByText('Please wait while we complete your login...')).toBeInTheDocument();
  });

  it('handles successful OAuth callback with same-origin URL redirect', async () => {
    server.use(...mockAuthCallback({ location: 'http://localhost:3000/ui/chat' }));

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/chat');
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

  it('handles same-origin URL with different path, query, and hash', async () => {
    server.use(...mockAuthCallback({ location: 'http://localhost:3000/ui/setup/download-models?step=1#section' }));
    render(<AuthCallbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/download-models?step=1#section');
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
    mockSearchParams = new URLSearchParams(
      'code=test-auth-code&state=test-state&scope=openid email profile&session_state=session-123'
    );

    let receivedParams: any = null;

    server.use(
      http.post(ENDPOINT_AUTH_CALLBACK, async ({ request }) => {
        receivedParams = await request.json();
        return HttpResponse.json({ location: 'http://localhost:3000/ui/chat' }, { status: 200 });
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
      ...mockAuthCallbackError({
        status: 400,
        code: 'invalid_state',
        message: 'Invalid state parameter',
        type: 'invalid_request',
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
    server.use(...mockAuthCallback({ noLocation: true }));

    render(<AuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Login Error')).toBeInTheDocument();
      expect(screen.getByText('Redirect URL not found in response. Please try again.')).toBeInTheDocument();
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
    server.use(...mockAuthCallbackError({ status: 500, message: 'Internal server error' }));

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
      expect(pushMock).toHaveBeenCalledWith('/ui/chat');
    });
  });

  it('disables retry button while loading', async () => {
    server.use(...mockAuthCallbackError({ status: 500, message: 'Server error' }));
    render(<AuthCallbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Server error')).toBeInTheDocument();
    });
    const retryButton = screen.getByRole('button', { name: 'Try Again' });
    server.use(
      ...mockAuthCallback({ delay: 200, location: 'http://localhost:3000/ui/chat' }) // Longer delay to test disabled state
    );
    await userEvent.click(retryButton);
    expect(screen.getByText('Processing Login...')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Try Again' })).not.toBeInTheDocument();
  });

  it('handles empty search parameters', async () => {
    mockSearchParams = new URLSearchParams('');
    server.use(
      ...mockAuthCallbackError({
        status: 400,
        code: 'missing_parameters',
        message: 'Missing required OAuth parameters',
        type: 'invalid_request',
      })
    );
    render(<AuthCallbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Missing required OAuth parameters')).toBeInTheDocument();
    });
  });

  it('handles invalid URL in response by treating as external', async () => {
    server.use(...mockAuthCallback({ invalidUrl: true }));
    render(<AuthCallbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(window.location.href).toBe('invalid-url-format');
    });
  });
});
