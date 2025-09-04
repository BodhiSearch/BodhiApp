import { UseMutationResult } from 'react-query';
import { AxiosResponse, AxiosError } from 'axios';
import { useMutationQuery } from '@/hooks/useQuery';
import { useCallback } from 'react';
import { AuthCallbackRequest, RedirectResponse, OpenAiApiError } from '@bodhiapp/ts-client';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

export const ENDPOINT_AUTH_INITIATE = '/bodhi/v1/auth/initiate';
export const ENDPOINT_AUTH_CALLBACK = '/bodhi/v1/auth/callback';

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

interface UseOAuthCallbackOptions {
  onSuccess?: (response: AxiosResponse<RedirectResponse>) => void;
  onError?: (message: string) => void;
}

// OAuth callback hook
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
