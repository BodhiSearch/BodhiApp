import { useMutation, UseMutationResult } from 'react-query';
import { AxiosResponse, AxiosError } from 'axios';
import apiClient from '@/lib/apiClient';

export const ENDPOINT_AUTH_INITIATE = '/bodhi/v1/auth/initiate';
export const ENDPOINT_AUTH_CALLBACK = '/bodhi/v1/auth/callback';

interface AuthInitiateResponse {
  auth_url?: string;
}

interface ErrorResponse {
  error?: {
    message?: string;
  };
}

interface UseOAuthInitiateOptions {
  onSuccess?: (response: AxiosResponse<AuthInitiateResponse>) => void;
  onError?: (message: string) => void;
}

export function useOAuthInitiate(
  options?: UseOAuthInitiateOptions
): UseMutationResult<AxiosResponse<AuthInitiateResponse>, AxiosError<ErrorResponse>, void> {
  return useMutation<AxiosResponse<AuthInitiateResponse>, AxiosError<ErrorResponse>, void>(
    () =>
      apiClient.post(
        ENDPOINT_AUTH_INITIATE,
        {},
        {
          maxRedirects: 0, // Don't follow redirects automatically
          validateStatus: (status) => status === 303 || status === 200, // Accept 303 redirects and 200 success
        }
      ),
    {
      onSuccess: (response) => {
        options?.onSuccess?.(response);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to initiate OAuth authentication';
        options?.onError?.(message);
      },
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
  code: string;
  state: string;
  [key: string]: string;
}

interface OAuthCallbackResponse {
  success?: boolean;
}

interface UseOAuthCallbackOptions {
  onSuccess?: (response: AxiosResponse<OAuthCallbackResponse>) => void;
  onError?: (message: string) => void;
}

// OAuth callback hook
export function useOAuthCallback(
  options?: UseOAuthCallbackOptions
): UseMutationResult<AxiosResponse<OAuthCallbackResponse>, AxiosError<ErrorResponse>, OAuthCallbackRequest> {
  return useMutation<AxiosResponse<OAuthCallbackResponse>, AxiosError<ErrorResponse>, OAuthCallbackRequest>(
    (callbackData: OAuthCallbackRequest) => apiClient.post(ENDPOINT_AUTH_CALLBACK, callbackData),
    {
      onSuccess: (response) => {
        options?.onSuccess?.(response);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to complete OAuth authentication';
        options?.onError?.(message);
      },
    }
  );
}
