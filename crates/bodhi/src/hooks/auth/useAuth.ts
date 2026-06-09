import { useCallback } from 'react';

import { AuthCallbackRequest, AuthInitiateRequest, RedirectResponse, BodhiErrorResponse } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { UseMutationResult } from '@/hooks/useQuery';

import { useMutationQuery, useQueryClient } from '@/hooks/useQuery';

import {
  ENDPOINT_UI_LOGIN,
  ENDPOINT_AUTH_INITIATE,
  ENDPOINT_AUTH_CALLBACK,
  ENDPOINT_DASHBOARD_AUTH_INITIATE,
  ENDPOINT_DASHBOARD_AUTH_CALLBACK,
  ENDPOINT_LOGOUT,
} from './constants';

interface UseOAuthInitiateOptions {
  onSuccess?: (response: AxiosResponse<RedirectResponse>) => void;
  onError?: (message: string) => void;
}

export function useOAuthInitiate(
  options?: UseOAuthInitiateOptions
): UseMutationResult<AxiosResponse<RedirectResponse>, AxiosError<BodhiErrorResponse>, AuthInitiateRequest> {
  const handleSuccess = useCallback(
    (response: AxiosResponse<RedirectResponse>) => {
      options?.onSuccess?.(response);
    },
    [options]
  );

  const handleError = useCallback(
    (error: AxiosError<BodhiErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to initiate OAuth authentication';
      options?.onError?.(message);
    },
    [options]
  );

  return useMutationQuery<RedirectResponse, AuthInitiateRequest>(
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
    }
  );
}

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

interface UseOAuthCallbackOptions {
  onSuccess?: (response: AxiosResponse<RedirectResponse>) => void;
  onError?: (message: string) => void;
}

export function useOAuthCallback(
  options?: UseOAuthCallbackOptions
): UseMutationResult<AxiosResponse<RedirectResponse>, AxiosError<BodhiErrorResponse>, AuthCallbackRequest> {
  const handleSuccess = useCallback(
    (response: AxiosResponse<RedirectResponse>) => {
      options?.onSuccess?.(response);
    },
    [options]
  );

  const handleError = useCallback(
    (error: AxiosError<BodhiErrorResponse>) => {
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
    }
  );
}

interface UseLogoutOptions {
  onSuccess?: (response: AxiosResponse<RedirectResponse>) => void;
  onError?: (error: AxiosError<BodhiErrorResponse>) => void;
}

export function useLogout(
  options?: UseLogoutOptions
): UseMutationResult<AxiosResponse<RedirectResponse>, AxiosError<BodhiErrorResponse>, void> {
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

interface UseDashboardOAuthInitiateOptions {
  onSuccess?: (response: AxiosResponse<RedirectResponse>) => void;
  onError?: (message: string) => void;
}

export function useDashboardOAuthInitiate(
  options?: UseDashboardOAuthInitiateOptions
): UseMutationResult<AxiosResponse<RedirectResponse>, AxiosError<BodhiErrorResponse>, void> {
  const handleSuccess = useCallback(
    (response: AxiosResponse<RedirectResponse>) => {
      options?.onSuccess?.(response);
    },
    [options]
  );

  const handleError = useCallback(
    (error: AxiosError<BodhiErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to initiate dashboard authentication';
      options?.onError?.(message);
    },
    [options]
  );

  return useMutationQuery<RedirectResponse, void>(
    ENDPOINT_DASHBOARD_AUTH_INITIATE,
    'post',
    {
      onSuccess: handleSuccess,
      onError: handleError,
    },
    {
      headers: {
        'Cache-Control': 'no-cache, no-store, must-revalidate',
      },
    }
  );
}

interface UseDashboardOAuthCallbackOptions {
  onSuccess?: (response: AxiosResponse<RedirectResponse>) => void;
  onError?: (message: string) => void;
}

export function useDashboardOAuthCallback(
  options?: UseDashboardOAuthCallbackOptions
): UseMutationResult<AxiosResponse<RedirectResponse>, AxiosError<BodhiErrorResponse>, AuthCallbackRequest> {
  const handleSuccess = useCallback(
    (response: AxiosResponse<RedirectResponse>) => {
      options?.onSuccess?.(response);
    },
    [options]
  );

  const handleError = useCallback(
    (error: AxiosError<BodhiErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to complete dashboard authentication';
      options?.onError?.(message);
    },
    [options]
  );

  return useMutationQuery<RedirectResponse, AuthCallbackRequest>(
    ENDPOINT_DASHBOARD_AUTH_CALLBACK,
    'post',
    {
      onSuccess: handleSuccess,
      onError: handleError,
    },
    {
      headers: {
        'Cache-Control': 'no-cache, no-store, must-revalidate',
      },
    }
  );
}

interface UseLogoutHandlerOptions {
  onSuccess?: (response: AxiosResponse<RedirectResponse>) => void;
  onError?: (message: string) => void;
}

export function useLogoutHandler(options?: UseLogoutHandlerOptions) {
  const { mutate: logout, isPending: isLoading } = useLogout({
    onSuccess: (response: AxiosResponse<RedirectResponse>) => {
      options?.onSuccess?.(response);
    },
    onError: (error: AxiosError<BodhiErrorResponse>) => {
      console.error('Logout failed:', error);
      const errorMessage =
        error.response?.data?.error?.message || error.message || 'An unexpected error occurred. Please try again.';
      options?.onError?.(errorMessage);
    },
  });

  return { logout, isLoading };
}
