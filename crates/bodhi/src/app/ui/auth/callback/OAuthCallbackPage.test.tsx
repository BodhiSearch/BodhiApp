import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import React from 'react';
import {
  afterAll,
  afterEach,
  beforeAll,
  describe,
  expect,
  it,
  vi
} from 'vitest';
import OAuthCallbackPage from './OAuthCallbackPage';
import { ROUTE_AUTH_CALLBACK, ROUTE_CHAT, ROUTE_LOGIN } from '@/lib/constants';

// Mock window.location with proper getter/setter support
const mockLocation: Record<string, any> = {
  href: `http://localhost:3000${ROUTE_AUTH_CALLBACK}?code=test_code&state=test_state`,
  origin: 'http://localhost:3000',
  pathname: ROUTE_AUTH_CALLBACK,
  search: '?code=test_code&state=test_state',
};

Object.defineProperty(window, 'location', {
  value: new Proxy(mockLocation, {
    set(target: Record<string, any>, prop: string | symbol, value: any) {
      target[prop as string] = value;
      return true;
    },
    get(target: Record<string, any>, prop: string | symbol) {
      return target[prop as string];
    },
  }),
  writable: true,
});

// Mock router
const mockRouter = {
  push: vi.fn(),
  replace: vi.fn(),
  back: vi.fn(),
  forward: vi.fn(),
  refresh: vi.fn(),
  prefetch: vi.fn(),
};

vi.mock('next/navigation', () => ({
  useRouter: () => mockRouter,
}));

// Setup MSW server
const server = setupServer(
  // Default success handler
  rest.post('*/bodhi/v1/auth/callback', (_req, res, ctx) => {
    return res(
      ctx.status(303),
      ctx.set('Location', ROUTE_CHAT),
      ctx.json({ success: true })
    );
  })
);

beforeAll(() => server.listen());
afterEach(() => {
  server.resetHandlers();
  vi.clearAllMocks();
  // Reset location for next test
  mockLocation.href = `http://localhost:3000${ROUTE_AUTH_CALLBACK}?code=test_code&state=test_state`;
  mockLocation.search = '?code=test_code&state=test_state';
});
afterAll(() => server.close());

describe('OAuthCallbackPage', () => {
  it('displays loading state initially', () => {
    render(<OAuthCallbackPage />, { wrapper: createWrapper() });

    // Check for loading state elements
    expect(screen.getByTestId('oauth-callback-loading')).toBeInTheDocument();
    expect(screen.getByText('Completing Authentication')).toBeInTheDocument();
    expect(screen.getByTestId('auth-card-loading')).toBeInTheDocument();
  });

  it('handles successful OAuth callback with redirect', async () => {
    render(<OAuthCallbackPage />, { wrapper: createWrapper() });

    // Wait for successful processing and redirect
    await waitFor(() => {
      expect(mockLocation.href).toBe(ROUTE_CHAT);
    });
  });

  it('displays error state for validation errors', async () => {
    server.use(
      rest.post('*/bodhi/v1/auth/callback', (_req, res, ctx) => {
        return res(
          ctx.status(422),
          ctx.json({
            error: { message: 'Invalid authorization code' },
          })
        );
      })
    );

    render(<OAuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-callback-error')).toBeInTheDocument();
    });

    expect(screen.getByText('Authentication Failed')).toBeInTheDocument();
    expect(screen.getByText('Invalid authorization code')).toBeInTheDocument();
  });

  it('displays error state for server errors', async () => {
    server.use(
      rest.post('*/bodhi/v1/auth/callback', (_req, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: { message: 'Internal server error' },
          })
        );
      })
    );

    render(<OAuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-callback-error')).toBeInTheDocument();
    });

    expect(screen.getByText('Authentication Failed')).toBeInTheDocument();
    expect(screen.getByText('Internal server error')).toBeInTheDocument();
  });

  it('handles "Try Login Again" button click', async () => {
    server.use(
      rest.post('*/bodhi/v1/auth/callback', (_req, res, ctx) => {
        return res(
          ctx.status(422),
          ctx.json({
            error: { message: 'Invalid code' },
          })
        );
      })
    );

    render(<OAuthCallbackPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-callback-error')).toBeInTheDocument();
    });

    const tryAgainButton = screen.getByRole('button', { name: 'Try Login Again' });
    await userEvent.click(tryAgainButton);

    expect(mockRouter.push).toHaveBeenCalledWith(ROUTE_LOGIN);
  });

  it('handles URL parsing errors gracefully', () => {
    // Mock URL constructor to throw an error
    const originalURL = global.URL;
    const mockURL = vi.fn().mockImplementation(() => {
      throw new Error('Invalid URL');
    });
    // Copy static methods from original URL
    Object.setPrototypeOf(mockURL, originalURL);
    Object.assign(mockURL, originalURL);
    global.URL = mockURL as any;

    render(<OAuthCallbackPage />, { wrapper: createWrapper() });

    // Should show error state
    expect(screen.getByTestId('oauth-callback-error')).toBeInTheDocument();
    expect(screen.getByText('Authentication Failed')).toBeInTheDocument();
    expect(screen.getByText('Invalid callback URL')).toBeInTheDocument();

    // Restore original URL constructor
    global.URL = originalURL;
  });

  it('does not process callback for non-callback URLs', () => {
    // Change location to a non-callback URL
    mockLocation.href = `http://localhost:3000${ROUTE_LOGIN}`;
    mockLocation.pathname = ROUTE_LOGIN;
    mockLocation.search = '';

    render(<OAuthCallbackPage />, { wrapper: createWrapper() });

    // Should show error state immediately since it's not a callback URL
    expect(screen.getByTestId('oauth-callback-error')).toBeInTheDocument();
    expect(screen.getByText('Authentication Failed')).toBeInTheDocument();
  });

  it('prevents multiple processing of the same callback', async () => {
    let callCount = 0;
    server.use(
      rest.post('*/bodhi/v1/auth/callback', (_req, res, ctx) => {
        callCount++;
        return res(
          ctx.status(303),
          ctx.set('Location', ROUTE_CHAT),
          ctx.json({ success: true })
        );
      })
    );

    // Use React.StrictMode to trigger double rendering
    const StrictWrapper = ({ children }: { children: React.ReactNode }) => (
      <React.StrictMode>
        {createWrapper()({ children })}
      </React.StrictMode>
    );

    render(<OAuthCallbackPage />, { wrapper: StrictWrapper });

    await waitFor(() => {
      expect(mockLocation.href).toBe(ROUTE_CHAT);
    });

    // Should only call the API once despite strict mode double rendering
    expect(callCount).toBe(1);
  });

  it('extracts OAuth parameters correctly from URL', () => {
    // Test with different URL parameters
    mockLocation.href = `http://localhost:3000${ROUTE_AUTH_CALLBACK}?code=auth_code_123&state=state_456`;
    mockLocation.search = '?code=auth_code_123&state=state_456';

    render(<OAuthCallbackPage />, { wrapper: createWrapper() });

    // Component should process successfully
    expect(screen.getByTestId('oauth-callback-loading')).toBeInTheDocument();
  });

  it('extracts additional OAuth parameters from URL', () => {
    // Test with additional parameters
    mockLocation.href = `http://localhost:3000${ROUTE_AUTH_CALLBACK}?code=auth_code_123&state=state_456&session_state=sess123&iss=https://auth.example.com`;
    mockLocation.search = '?code=auth_code_123&state=state_456&session_state=sess123&iss=https://auth.example.com';

    render(<OAuthCallbackPage />, { wrapper: createWrapper() });

    // Component should process successfully and extract all parameters
    expect(screen.getByTestId('oauth-callback-loading')).toBeInTheDocument();
  });

  it('handles successful callback with fallback redirect when no location header', async () => {
    server.use(
      rest.post('*/bodhi/v1/auth/callback', (_req, res, ctx) => {
        return res(
          ctx.status(303),
          // No Location header
          ctx.json({ success: true })
        );
      })
    );

    render(<OAuthCallbackPage />, { wrapper: createWrapper() });

    // Wait for successful processing and fallback redirect
    await waitFor(() => {
      expect(mockLocation.href).toBe(ROUTE_CHAT);
    });
  });
});