import { UseMutationResult } from 'react-query';
import { AxiosResponse, AxiosError } from 'axios';
import { useMutationQuery } from '@/hooks/useQuery';
import { ErrorResponse } from '@/types/models';
import { useCallback } from 'react';
import { RedirectResponse } from '@/types/api';

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
    [options?.onSuccess]
  );

  const handleError = useCallback(
    (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to initiate OAuth authentication';
      options?.onError?.(message);
    },
    [options?.onError]
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
export function extractOAuthParams(url: string): Record<string, string> {
  try {
    const urlObj = new URL(url);
    const params: Record<string, string> = {};

    urlObj.searchParams.forEach((value, key) => {
      params[key] = value;
    });

    return params;
  } catch {
    return {};
  }
}

// OAuth callback interfaces
interface OAuthCallbackRequest {
  code?: string;
  state?: string;
  error?: string;
  error_description?: string;
  [key: string]: string | undefined;
}

interface OAuthCallbackResponse {
  location: string;
}

interface UseOAuthCallbackOptions {
  onSuccess?: (response: AxiosResponse<OAuthCallbackResponse>) => void;
  onError?: (message: string) => void;
}

// OAuth callback hook
export function useOAuthCallback(
  options?: UseOAuthCallbackOptions
): UseMutationResult<AxiosResponse<OAuthCallbackResponse>, AxiosError<ErrorResponse>, OAuthCallbackRequest> {
  const handleSuccess = useCallback(
    (response: AxiosResponse<OAuthCallbackResponse>) => {
      options?.onSuccess?.(response);
    },
    [options?.onSuccess]
  );

  const handleError = useCallback(
    (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to complete OAuth authentication';
      options?.onError?.(message);
    },
    [options?.onError]
  );

  return useMutationQuery<OAuthCallbackResponse, OAuthCallbackRequest>(
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
