import { useMutation, UseMutationResult } from 'react-query';
import { AxiosResponse, AxiosError } from 'axios';
import apiClient from '@/lib/apiClient';

export const ENDPOINT_AUTH_INITIATE = '/bodhi/v1/auth/initiate';

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
    () => apiClient.get(ENDPOINT_AUTH_INITIATE),
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
