import { renderHook, waitFor } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, beforeAll, describe, expect, it, vi } from 'vitest';
import { createWrapper } from '@/tests/wrapper';
import {
  extractOAuthParams,
  useOAuthCallback,
  useOAuthInitiate,
} from './useOAuth';
import { ENDPOINT_AUTH_CALLBACK, ENDPOINT_AUTH_INITIATE } from './useQuery';

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());

describe('extractOAuthParams', () => {
  it('extracts all query parameters without filtering', () => {
    const url = 'https://example.com/callback?code=abc123&state=xyz789&error=access_denied&error_description=User%20denied%20access';
    const params = extractOAuthParams(url);

    expect(params).toEqual({
      code: 'abc123',
      state: 'xyz789',
      error: 'access_denied',
      error_description: 'User denied access',
    });
  });

  it('extracts all parameters including custom ones', () => {
    const url = 'https://example.com/callback?code=abc123&state=xyz789&session_state=sess123&iss=https://issuer.com&custom_param=value';
    const params = extractOAuthParams(url);

    expect(params).toEqual({
      code: 'abc123',
      state: 'xyz789',
      session_state: 'sess123',
      iss: 'https://issuer.com',
      custom_param: 'value',
    });
  });

  it('handles no parameters', () => {
    const url = 'https://example.com/callback';
    const params = extractOAuthParams(url);

    expect(params).toEqual({});
  });
});

describe('useOAuthInitiate', () => {
  it('handles successful OAuth initiation with 303 redirect', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.status(303), // 303 redirect to OAuth URL
          ctx.set('Location', 'https://oauth.example.com/auth?client_id=test'),
        );
      })
    );

    const { result } = renderHook(
      () => useOAuthInitiate({ onSuccess: mockOnSuccess, onError: mockOnError }),
      { wrapper: createWrapper() }
    );

    result.current.mutate();

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(mockOnSuccess).toHaveBeenCalledWith(
      expect.objectContaining({
        status: 303,
        headers: expect.objectContaining({ location: 'https://oauth.example.com/auth?client_id=test' }),
      })
    );
    expect(mockOnError).not.toHaveBeenCalled();
  });

  it('handles OAuth initiation error with specific message', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

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

    const { result } = renderHook(
      () => useOAuthInitiate({ onSuccess: mockOnSuccess, onError: mockOnError }),
      { wrapper: createWrapper() }
    );

    result.current.mutate();

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(mockOnError).toHaveBeenCalledWith('OAuth configuration error');
    expect(mockOnSuccess).not.toHaveBeenCalled();
  });

  it('handles generic error when no specific message provided', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(ctx.status(500));
      })
    );

    const { result } = renderHook(
      () => useOAuthInitiate({ onSuccess: mockOnSuccess, onError: mockOnError }),
      { wrapper: createWrapper() }
    );

    result.current.mutate();

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(mockOnError).toHaveBeenCalledWith('Failed to initiate OAuth authentication');
    expect(mockOnSuccess).not.toHaveBeenCalled();
  });

  it('handles 303 response without Location header', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(ctx.status(303)); // No Location header
      })
    );

    const { result } = renderHook(
      () => useOAuthInitiate({ onSuccess: mockOnSuccess, onError: mockOnError }),
      { wrapper: createWrapper() }
    );

    result.current.mutate();

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(mockOnSuccess).toHaveBeenCalledWith(
      expect.objectContaining({
        status: 303,
        headers: expect.any(Object),
      })
    );
    expect(mockOnError).not.toHaveBeenCalled();
  });

  it('handles network errors', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res) => {
        return res.networkError('Network connection failed');
      })
    );

    const { result } = renderHook(
      () => useOAuthInitiate({ onSuccess: mockOnSuccess, onError: mockOnError }),
      { wrapper: createWrapper() }
    );

    result.current.mutate();

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(mockOnError).toHaveBeenCalledWith('Failed to initiate OAuth authentication');
    expect(mockOnSuccess).not.toHaveBeenCalled();
  });
});

describe('useOAuthCallback', () => {
  it('handles successful OAuth callback', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(
          ctx.status(200),
          ctx.json({ success: true })
        );
      })
    );

    const { result } = renderHook(
      () => useOAuthCallback({ onSuccess: mockOnSuccess, onError: mockOnError }),
      { wrapper: createWrapper() }
    );

    const callbackRequest = {
      code: 'auth_code_123',
      state: 'state_xyz',
    };

    result.current.mutate(callbackRequest);

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(mockOnSuccess).toHaveBeenCalledWith(
      expect.objectContaining({
        status: 200,
        data: { success: true },
      })
    );
    expect(mockOnError).not.toHaveBeenCalled();
  });

  it('handles OAuth callback error with specific message', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(
          ctx.status(422),
          ctx.json({
            error: {
              message: 'Invalid authorization code',
              type: 'invalid_request_error',
              code: 'invalid_auth_code',
            },
          })
        );
      })
    );

    const { result } = renderHook(
      () => useOAuthCallback({ onSuccess: mockOnSuccess, onError: mockOnError }),
      { wrapper: createWrapper() }
    );

    const callbackRequest = {
      code: 'invalid_code',
      state: 'state_xyz',
    };

    result.current.mutate(callbackRequest);

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(mockOnError).toHaveBeenCalledWith('Invalid authorization code');
    expect(mockOnSuccess).not.toHaveBeenCalled();
  });

  it('handles OAuth callback error without specific message', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
        return res(ctx.status(500));
      })
    );

    const { result } = renderHook(
      () => useOAuthCallback({ onSuccess: mockOnSuccess, onError: mockOnError }),
      { wrapper: createWrapper() }
    );

    const callbackRequest = {
      code: 'auth_code_123',
      state: 'state_xyz',
    };

    result.current.mutate(callbackRequest);

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(mockOnError).toHaveBeenCalledWith('Failed to complete OAuth authentication');
    expect(mockOnSuccess).not.toHaveBeenCalled();
  });

  it('handles callback with additional parameters', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (req, res, ctx) => {
        return res(
          ctx.status(200),
          ctx.json({ success: true })
        );
      })
    );

    const { result } = renderHook(
      () => useOAuthCallback({ onSuccess: mockOnSuccess, onError: mockOnError }),
      { wrapper: createWrapper() }
    );

    const callbackRequest = {
      code: 'auth_code_123',
      state: 'state_xyz',
      session_state: 'session_123',
      iss: 'https://issuer.com',
    };

    result.current.mutate(callbackRequest);

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(mockOnSuccess).toHaveBeenCalled();
    expect(mockOnError).not.toHaveBeenCalled();
  });
});
