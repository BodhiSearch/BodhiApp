import { renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { createWrapper } from '@/tests/wrapper';
import { extractOAuthParams, useOAuthCallback, useOAuthInitiate } from './useOAuth';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import {
  mockAuthInitiate,
  mockAuthInitiateError,
  mockAuthCallback,
  mockAuthCallbackError,
} from '@/test-utils/msw-v2/handlers/auth';

setupMswV2();

describe('extractOAuthParams', () => {
  it('extracts all query parameters without filtering', () => {
    const url =
      'https://example.com/callback?code=abc123&state=xyz789&error=access_denied&error_description=User%20denied%20access';
    const params = extractOAuthParams(url);

    expect(params).toEqual({
      code: 'abc123',
      state: 'xyz789',
      error: 'access_denied',
      error_description: 'User denied access',
    });
  });

  it('extracts all parameters including custom ones', () => {
    const url =
      'https://example.com/callback?code=abc123&state=xyz789&session_state=sess123&iss=https://issuer.com&custom_param=value';
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
  it('handles successful OAuth initiation for unauthenticated user with 201 created', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      ...mockAuthInitiate({
        status: 201,
        location: 'https://oauth.example.com/auth?client_id=test',
      })
    );

    const { result } = renderHook(() => useOAuthInitiate({ onSuccess: mockOnSuccess, onError: mockOnError }), {
      wrapper: createWrapper(),
    });

    result.current.mutate();

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(mockOnSuccess).toHaveBeenCalledWith(
      expect.objectContaining({
        status: 201,
        data: { location: 'https://oauth.example.com/auth?client_id=test' },
      })
    );
    expect(mockOnError).not.toHaveBeenCalled();
  });

  it('handles already authenticated user with 200 response', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      ...mockAuthInitiate({
        status: 200,
        location: 'http://localhost:3000/ui/chat',
      })
    );

    const { result } = renderHook(() => useOAuthInitiate({ onSuccess: mockOnSuccess, onError: mockOnError }), {
      wrapper: createWrapper(),
    });

    result.current.mutate();

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(mockOnSuccess).toHaveBeenCalledWith(
      expect.objectContaining({
        status: 200,
        data: { location: 'http://localhost:3000/ui/chat' },
      })
    );
    expect(mockOnError).not.toHaveBeenCalled();
  });

  it('handles OAuth initiation error with specific message', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      ...mockAuthInitiateError({
        status: 500,
        message: 'OAuth configuration error',
        code: 'oauth_config_error',
      })
    );

    const { result } = renderHook(() => useOAuthInitiate({ onSuccess: mockOnSuccess, onError: mockOnError }), {
      wrapper: createWrapper(),
    });

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

    server.use(...mockAuthInitiateError({ status: 500, empty: true }));

    const { result } = renderHook(() => useOAuthInitiate({ onSuccess: mockOnSuccess, onError: mockOnError }), {
      wrapper: createWrapper(),
    });

    result.current.mutate();

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(mockOnError).toHaveBeenCalledWith('Failed to initiate OAuth authentication');
    expect(mockOnSuccess).not.toHaveBeenCalled();
  });

  it('handles response without location in JSON', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(...mockAuthInitiate({ status: 201, noLocation: true }));

    const { result } = renderHook(() => useOAuthInitiate({ onSuccess: mockOnSuccess, onError: mockOnError }), {
      wrapper: createWrapper(),
    });

    result.current.mutate();

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(mockOnSuccess).toHaveBeenCalledWith(
      expect.objectContaining({
        status: 201,
        data: {},
      })
    );
    expect(mockOnError).not.toHaveBeenCalled();
  });

  it('handles network errors', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(...mockAuthInitiateError({ status: 500, empty: true }));

    const { result } = renderHook(() => useOAuthInitiate({ onSuccess: mockOnSuccess, onError: mockOnError }), {
      wrapper: createWrapper(),
    });

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
      ...mockAuthCallback({
        status: 200,
        location: 'http://localhost:3000/ui/chat',
      })
    );

    const { result } = renderHook(() => useOAuthCallback({ onSuccess: mockOnSuccess, onError: mockOnError }), {
      wrapper: createWrapper(),
    });

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
        data: { location: 'http://localhost:3000/ui/chat' },
      })
    );
    expect(mockOnError).not.toHaveBeenCalled();
  });

  it('handles OAuth callback error with specific message', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      ...mockAuthCallbackError({
        status: 422,
        message: 'Invalid authorization code',
        code: 'invalid_auth_code',
      })
    );

    const { result } = renderHook(() => useOAuthCallback({ onSuccess: mockOnSuccess, onError: mockOnError }), {
      wrapper: createWrapper(),
    });

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
      ...mockAuthCallbackError({
        status: 500,
        message: 'Failed to complete OAuth authentication',
      })
    );

    const { result } = renderHook(() => useOAuthCallback({ onSuccess: mockOnSuccess, onError: mockOnError }), {
      wrapper: createWrapper(),
    });

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

    server.use(...mockAuthCallback({ status: 200, noLocation: true }));

    const { result } = renderHook(() => useOAuthCallback({ onSuccess: mockOnSuccess, onError: mockOnError }), {
      wrapper: createWrapper(),
    });

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
