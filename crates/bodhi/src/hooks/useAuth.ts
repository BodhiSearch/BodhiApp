// External imports
import { useCallback } from 'react';

// Type imports
import { AuthCallbackRequest, RedirectResponse, OpenAiApiError } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { UseMutationResult } from '@/hooks/useQuery';

// Internal imports
import { useMutationQuery, useQueryClient } from './useQuery';

// Constants
export const ENDPOINT_UI_LOGIN = '/ui/login';
export const ENDPOINT_AUTH_INITIATE = '/bodhi/v1/auth/initiate';
export const ENDPOINT_AUTH_CALLBACK = '/bodhi/v1/auth/callback';
export const ENDPOINT_LOGOUT = '/bodhi/v1/logout';

// Type alias
type ErrorResponse = OpenAiApiError;

// OAuth Initiate Hook
interface UseOAuthInitiateOptions {
  onSuccess?: (response: AxiosResponse<RedirectResponse>) => void;
  onError?: (message: string) => void;
}

export function useOAuthInitiate(
  options?: UseOAuthInitiateOptions
): UseMutationResult<AxiosResponse<RedirectResponse>, AxiosError<ErrorResponse>, void> {
  const handleSuccess = useCallback(
    (response: AxiosResponse<RedirectResponse>) => {
      options?.onSuccess?.(response);
    },
    [options]
  );

  const handleError = useCallback(
    (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to initiate OAuth authentication';
      options?.onError?.(message);
    },
    [options]
  );

  return useMutationQuery<RedirectResponse, void>(
    ENDPOINT_AUTH_INITIATE,
    'post',
    {
      onSuccess: handleSuccess,
      onError: handleError,
    },
    {
      headers: {
        'Cache-Control': 'no-cache, no-store, must-revalidate',
      },
      skipCacheInvalidation: true,
    }
  );
}

// Extract OAuth parameters from URL
export function extractOAuthParams(url: string): AuthCallbackRequest {
  try {
    const urlObj = new URL(url);
    const params: AuthCallbackRequest = {};

    urlObj.searchParams.forEach((value, key) => {
      // All parameters are flattened in the generated type
      params[key] = value;
    });

    return params;
  } catch {
    return {};
  }
}

// OAuth Callback Hook
interface UseOAuthCallbackOptions {
  onSuccess?: (response: AxiosResponse<RedirectResponse>) => void;
  onError?: (message: string) => void;
}

export function useOAuthCallback(
  options?: UseOAuthCallbackOptions
): UseMutationResult<AxiosResponse<RedirectResponse>, AxiosError<ErrorResponse>, AuthCallbackRequest> {
  const handleSuccess = useCallback(
    (response: AxiosResponse<RedirectResponse>) => {
      options?.onSuccess?.(response);
    },
    [options]
  );

  const handleError = useCallback(
    (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to complete OAuth authentication';
      options?.onError?.(message);
    },
    [options]
  );

  return useMutationQuery<RedirectResponse, AuthCallbackRequest>(
    ENDPOINT_AUTH_CALLBACK,
    'post',
    {
      onSuccess: handleSuccess,
      onError: handleError,
    },
    {
      headers: {
        'Cache-Control': 'no-cache, no-store, must-revalidate',
      },
      skipCacheInvalidation: true,
    }
  );
}

// Logout Hook (from useQuery.ts)
interface UseLogoutOptions {
  onSuccess?: (response: AxiosResponse<RedirectResponse>) => void;
  onError?: (error: AxiosError<ErrorResponse>) => void;
}

export function useLogout(
  options?: UseLogoutOptions
): UseMutationResult<AxiosResponse<RedirectResponse>, AxiosError<ErrorResponse>, void> {
  const queryClient = useQueryClient();
  return useMutationQuery<RedirectResponse, void>(ENDPOINT_LOGOUT, 'post', {
    ...options,
    onSuccess: (data, _variables, _context) => {
      queryClient.invalidateQueries();
      if (options?.onSuccess) {
        options.onSuccess(data);
      }
    },
  });
}

// Logout Handler Hook (from useLogoutHandler.ts)
interface UseLogoutHandlerOptions {
  onSuccess?: (response: AxiosResponse<RedirectResponse>) => void;
  onError?: (message: string) => void;
}

export function useLogoutHandler(options?: UseLogoutHandlerOptions) {
  const { mutate: logout, isLoading } = useLogout({
    onSuccess: (response: AxiosResponse<RedirectResponse>) => {
      options?.onSuccess?.(response);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      console.error('Logout failed:', error);
      const errorMessage =
        error.response?.data?.error?.message || error.message || 'An unexpected error occurred. Please try again.';
      options?.onError?.(errorMessage);
    },
  });

  return { logout, isLoading };
}
