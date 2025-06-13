import { ErrorResponse } from '@/types/models';
import { AxiosError, AxiosResponse } from 'axios';
import { UseMutationResult } from 'react-query';
import { ENDPOINT_AUTH_CALLBACK, ENDPOINT_AUTH_INITIATE, useMutationQuery } from './useQuery';

// OAuth Types
export interface AuthInitiateResponse {
  auth_url: string;
}

export interface AuthCallbackRequest {
  [key: string]: string; // Accept any query parameters as key-value pairs
}

// OAuth Utility Functions - Extract all query parameters without filtering
export function extractOAuthParams(url: string): AuthCallbackRequest {
  const urlObj = new URL(url);
  const params = new URLSearchParams(urlObj.search);

  const request: AuthCallbackRequest = {};

  // Extract ALL parameters - let backend decide what's valid
  params.forEach((value, key) => {
    request[key] = value;
  });

  return request;
}



// OAuth Hooks - "Dumb" implementation that relies on backend for all decisions
export function useOAuthInitiate(options?: {
  onSuccess?: (response: AxiosResponse<AuthInitiateResponse>) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<AuthInitiateResponse>,
  AxiosError<ErrorResponse>,
  void
> {
  return useMutationQuery<AuthInitiateResponse, void>(
    ENDPOINT_AUTH_INITIATE,
    'post',
    {
      onSuccess: (response) => {
        // Let the page handle the response - don't make decisions here
        options?.onSuccess?.(response);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message =
          error?.response?.data?.error?.message || 'Failed to initiate OAuth flow';
        options?.onError?.(message);
      },
    },
    {
      // Accept both success (200-399) and auth required (401) as valid responses
      validateStatus: (status) => (status >= 200 && status < 400) || status === 401,
    }
  );
}

export function useOAuthCallback(options?: {
  onSuccess?: (response: AxiosResponse<void>) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<void>,
  AxiosError<ErrorResponse>,
  AuthCallbackRequest
> {
  return useMutationQuery<void, AuthCallbackRequest>(
    ENDPOINT_AUTH_CALLBACK,
    'post',
    {
      onSuccess: (response) => {
        // Let the page handle the response - don't make decisions here
        options?.onSuccess?.(response);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message =
          error?.response?.data?.error?.message || 'OAuth authentication failed';
        options?.onError?.(message);
      },
    }
  );
}
