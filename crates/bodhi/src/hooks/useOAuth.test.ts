import { renderHook } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { createWrapper } from '@/tests/wrapper';
import { useOAuthInitiate, useOAuthCallback, oauthUtils } from '@/hooks/useOAuth';

// Mock window.location with proper setter support
const mockLocation = {
  href: '',
  origin: 'http://localhost:3000',
  search: '?code=test_code',
};

// Create a proper setter that captures assignments
Object.defineProperty(window, 'location', {
  value: new Proxy(mockLocation, {
    set(target, prop, value) {
      target[prop] = value;
      return true;
    },
    get(target, prop) {
      return target[prop];
    },
  }),
  writable: true,
});

// Setup MSW server with all required endpoints
const server = setupServer(
  // Auth initiate endpoint - returns 401 with auth_url for unauthenticated users
  rest.post(`*/bodhi/v1/auth/initiate`, (req, res, ctx) => {
    return res(
      ctx.status(401),
      ctx.json({
        auth_url: 'https://oauth-server.com/auth?client_id=test&redirect_uri=http://localhost:3000/ui/auth/callback',
      })
    );
  }),

  // Auth callback endpoint - handles POST requests
  rest.post(`*/bodhi/v1/auth/callback`, (req, res, ctx) => {
    // Default success with 303 redirect
    return res(ctx.status(303), ctx.set('Location', '/ui/chat'), ctx.json({ success: true }));
  })
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('useOAuthInitiate', () => {
  it('should call onSuccess callback with auth URL', async () => {
    const onSuccess = vi.fn();
    const { result } = renderHook(() => useOAuthInitiate({ onSuccess }), {
      wrapper: createWrapper(),
    });

    result.current.mutate();

    await vi.waitFor(() => {
      expect(onSuccess).toHaveBeenCalledWith({
        auth_url: 'https://oauth-server.com/auth?client_id=test&redirect_uri=http://localhost:3000/ui/auth/callback',
      });
    });
  });

  it('should call onError callback for server errors', async () => {
    server.use(
      rest.post(`*/bodhi/v1/auth/initiate`, (req, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: { message: 'Internal server error' },
          })
        );
      })
    );

    const onError = vi.fn();
    const { result } = renderHook(() => useOAuthInitiate({ onError }), {
      wrapper: createWrapper(),
    });

    result.current.mutate();

    await vi.waitFor(() => {
      expect(onError).toHaveBeenCalledWith('Internal server error');
    });
  });
});

describe('useOAuthCallback', () => {
  it('should handle successful callback with redirect location', async () => {
    const onSuccess = vi.fn();
    const { result } = renderHook(() => useOAuthCallback({ onSuccess }), {
      wrapper: createWrapper(),
    });

    // Trigger the callback with OAuth parameters
    result.current.mutate({
      code: 'test_code',
      state: 'test_state',
      additional_params: {},
    });

    await vi.waitFor(() => {
      expect(onSuccess).toHaveBeenCalledWith('/ui/chat');
    });
  });

  it('should call onError callback for validation errors', async () => {
    server.use(
      rest.post(`*/bodhi/v1/auth/callback`, (req, res, ctx) => {
        return res(
          ctx.status(422),
          ctx.json({
            error: { message: 'Invalid authorization code' },
          })
        );
      })
    );

    const onError = vi.fn();
    const { result } = renderHook(() => useOAuthCallback({ onError }), {
      wrapper: createWrapper(),
    });

    result.current.mutate({
      code: 'invalid_code',
      additional_params: {},
    });

    await vi.waitFor(() => {
      expect(onError).toHaveBeenCalledWith('Invalid authorization code');
    });
  });

  it('should call onSuccess with no location when no location header', async () => {
    server.use(
      rest.post(`*/bodhi/v1/auth/callback`, (req, res, ctx) => {
        // Success but no Location header
        return res(ctx.status(303), ctx.json({ success: true }));
      })
    );

    const onSuccess = vi.fn();
    const { result } = renderHook(() => useOAuthCallback({ onSuccess }), {
      wrapper: createWrapper(),
    });

    result.current.mutate({
      code: 'test_code',
      additional_params: {},
    });

    await vi.waitFor(() => {
      expect(onSuccess).toHaveBeenCalledWith(undefined);
    });
  });
});

describe('oauthUtils', () => {
  it('should extract OAuth parameters from URL', () => {
    const url = 'http://localhost:3000/ui/auth/callback?code=test_code&state=test_state';
    const params = oauthUtils.extractOAuthParams(url);

    expect(params).toEqual({
      code: 'test_code',
      state: 'test_state',
      error: undefined,
      error_description: undefined,
      additional_params: {},
    });
  });

  it('should extract OAuth error parameters from URL', () => {
    const url = 'http://localhost:3000/ui/auth/callback?error=access_denied&error_description=User%20denied%20access';
    const params = oauthUtils.extractOAuthParams(url);

    expect(params).toEqual({
      code: undefined,
      state: undefined,
      error: 'access_denied',
      error_description: 'User denied access',
      additional_params: {},
    });
  });

  it('should extract additional dynamic parameters', () => {
    const url =
      'http://localhost:3000/ui/auth/callback?code=test_code&session_state=session123&iss=https://auth.example.com&custom_param=value123';
    const params = oauthUtils.extractOAuthParams(url);

    expect(params).toEqual({
      code: 'test_code',
      state: undefined,
      error: undefined,
      error_description: undefined,
      additional_params: {
        session_state: 'session123',
        iss: 'https://auth.example.com',
        custom_param: 'value123',
      },
    });
  });

  it('should handle missing parameters', () => {
    const url = 'http://localhost:3000/ui/auth/callback';
    const params = oauthUtils.extractOAuthParams(url);

    expect(params).toEqual({
      code: undefined,
      state: undefined,
      error: undefined,
      error_description: undefined,
      additional_params: {},
    });
  });
});
