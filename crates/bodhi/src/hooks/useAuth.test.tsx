import { renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import React from 'react';
import { createWrapper } from '@/tests/wrapper';
import { extractOAuthParams, useOAuthCallback, useOAuthInitiate, useLogoutHandler } from './useAuth';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import {
  mockAuthInitiate,
  mockAuthInitiateError,
  mockAuthInitiateConfigError,
  mockAuthCallback,
  mockAuthCallbackError,
  mockAuthCallbackInvalidCode,
  mockLogout,
  mockLogoutSessionError,
} from '@/test-utils/msw-v2/handlers/auth';
import { Button } from '@/components/ui/button';

setupMswV2();

// OAuth Parameter Extraction Tests
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

// OAuth Initiate Hook Tests
describe('useOAuthInitiate', () => {
  it('handles successful OAuth initiation for unauthenticated user with 201 created', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      ...mockAuthInitiate({
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
        status: 201,
        data: { location: 'http://localhost:3000/ui/chat' },
      })
    );
    expect(mockOnError).not.toHaveBeenCalled();
  });

  it('handles OAuth initiation error with specific message', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(...mockAuthInitiateConfigError());

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

    server.use(...mockAuthInitiateError());

    const { result } = renderHook(() => useOAuthInitiate({ onSuccess: mockOnSuccess, onError: mockOnError }), {
      wrapper: createWrapper(),
    });

    result.current.mutate();

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(mockOnError).toHaveBeenCalledWith('Internal server error');
    expect(mockOnSuccess).not.toHaveBeenCalled();
  });

  it('handles response with default location', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(...mockAuthInitiate());

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

  it('handles network errors', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(...mockAuthInitiateError());

    const { result } = renderHook(() => useOAuthInitiate({ onSuccess: mockOnSuccess, onError: mockOnError }), {
      wrapper: createWrapper(),
    });

    result.current.mutate();

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(mockOnError).toHaveBeenCalledWith('Internal server error');
    expect(mockOnSuccess).not.toHaveBeenCalled();
  });
});

// OAuth Callback Hook Tests
describe('useOAuthCallback', () => {
  it('handles successful OAuth callback', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      ...mockAuthCallback({
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

    server.use(...mockAuthCallbackInvalidCode());

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

    server.use(...mockAuthCallback());

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

// Logout Handler Hook Tests
// Simple component that uses the useLogoutHandler hook
const LogoutButton: React.FC<{ onSuccess?: (response: any) => void; onError?: (message: string) => void }> = ({
  onSuccess,
  onError,
}) => {
  const { logout, isLoading: isLoggingOut } = useLogoutHandler({ onSuccess, onError });
  return (
    <Button onClick={() => logout()} disabled={isLoggingOut}>
      {isLoggingOut ? 'Logging out...' : 'Log Out'}
    </Button>
  );
};

describe('useLogoutHandler', () => {
  it('calls onSuccess callback when logout succeeds', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(...mockLogout({ location: 'http://localhost:1135/ui/login' }));

    render(<LogoutButton onSuccess={mockOnSuccess} onError={mockOnError} />, { wrapper: createWrapper() });

    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    expect(logoutButton).toBeInTheDocument();

    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(mockOnSuccess).toHaveBeenCalledWith(
        expect.objectContaining({
          status: 200,
          data: expect.objectContaining({ location: 'http://localhost:1135/ui/login' }),
        })
      );
      expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    });

    expect(logoutButton).not.toBeDisabled();
    expect(mockOnError).not.toHaveBeenCalled();
  });

  it('calls onError callback when logout fails', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(...mockLogoutSessionError());

    render(<LogoutButton onSuccess={mockOnSuccess} onError={mockOnError} />, { wrapper: createWrapper() });

    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    });

    expect(logoutButton).not.toBeDisabled();
    expect(mockOnSuccess).not.toHaveBeenCalled();
    expect(mockOnError).toHaveBeenCalledWith('Session deletion failed');
  });

  it('handles logout without callbacks', async () => {
    server.use(...mockLogout({ location: 'http://localhost:1135/ui/login' }));

    render(<LogoutButton />, { wrapper: createWrapper() });

    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    });

    expect(logoutButton).not.toBeDisabled();
  });
});
