import { useMutationQuery } from '@/hooks/useQuery';
import { ENDPOINT_AUTH_INITIATE, ENDPOINT_AUTH_CALLBACK } from '@/hooks/useQuery';
import { ErrorResponse } from '@/types/models';
import { AxiosError, AxiosResponse } from 'axios';
import { useMutation, UseMutationResult } from 'react-query';
import apiClient from '@/lib/apiClient';

// OAuth request/response types matching backend
export interface AuthCallbackRequest {
  code?: string;
  state?: string;
  error?: string;
  error_description?: string;
  additional_params: Record<string, string>;
}

export interface AuthStatusResponse {
  authenticated: boolean;
  user?: {
    logged_in: boolean;
    email?: string;
    roles: string[];
  };
}

export interface AuthInitiateResponse {
  auth_url: string;
}

/**
 * Hook for initiating OAuth flow
 * Note: Redirect logic is handled by the page component for clarity
 */
export const useOAuthInitiate = (options?: {
  onSuccess?: (response: AuthInitiateResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AuthInitiateResponse>, AxiosError<ErrorResponse>, void> => {
  return useMutation<AxiosResponse<AuthInitiateResponse>, AxiosError<ErrorResponse>, void>(
    async () => {
      const response = await apiClient.post<AuthInitiateResponse>(
        ENDPOINT_AUTH_INITIATE,
        {},
        {
          headers: {
            'Content-Type': 'application/json',
          },
        }
      );
      return response;
    },
    {
      onSuccess: (response) => {
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        // Backend returns 401 with auth_url for unauthenticated users
        if (error.response?.status === 401) {
          const responseData = error.response.data as any;
          if (responseData?.auth_url) {
            options?.onSuccess?.(responseData);
            return;
          }
        }

        const message = error?.response?.data?.error?.message || 'Failed to initiate OAuth flow';
        console.error('Failed to initiate OAuth flow:', error);
        options?.onError?.(message);
      },
    }
  );
};

/**
 * Hook for completing OAuth callback
 * Note: Redirect logic is handled by the page component for clarity
 */
export const useOAuthCallback = (options?: {
  onSuccess?: (location?: string) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, AuthCallbackRequest> => {
  return useMutationQuery<void, AuthCallbackRequest>(ENDPOINT_AUTH_CALLBACK, 'post', {
    onSuccess: (response) => {
      // Check if this is a 303 redirect response with Location header
      if (response.status === 303) {
        const location = response.headers.location;
        options?.onSuccess?.(location);
        return;
      }

      // For other successful responses, pass undefined as location
      options?.onSuccess?.(undefined);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      // Backend returns 303 redirect on success, but axios treats it as an error due to maxRedirects: 0
      if (error.response?.status === 303) {
        const location = error.response.headers.location;
        options?.onSuccess?.(location);
        return;
      }

      // Handle validation errors (422) and other errors
      const message = error?.response?.data?.error?.message || 'Failed to complete OAuth callback';
      console.error('Failed to complete OAuth callback:', error);
      options?.onError?.(message);
    },
  });
};

/**
 * OAuth utility functions
 */
export const oauthUtils = {
  /**
   * Extract all OAuth parameters from URL including known and additional dynamic parameters
   */
  extractOAuthParams: (url: string): AuthCallbackRequest => {
    const urlObj = new URL(url);
    const searchParams = new URLSearchParams(urlObj.search);

    // Extract known OAuth 2.1 parameters
    const code = searchParams.get('code') || undefined;
    const state = searchParams.get('state') || undefined;
    const error = searchParams.get('error') || undefined;
    const errorDescription = searchParams.get('error_description') || undefined;

    // Extract all additional parameters
    const additional_params: Record<string, string> = {};
    const knownParams = new Set(['code', 'state', 'error', 'error_description']);

    for (const [key, value] of searchParams.entries()) {
      if (!knownParams.has(key)) {
        additional_params[key] = value;
      }
    }

    return {
      code,
      state,
      error,
      error_description: errorDescription,
      additional_params,
    };
  },
};
