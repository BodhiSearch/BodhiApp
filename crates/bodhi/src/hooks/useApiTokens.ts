import { API_TOKENS_ENDPOINT, useMutationQuery, useQuery } from '@/hooks/useQuery';
import { useQueryClient, useMutation } from 'react-query';
import { AxiosResponse, AxiosError } from 'axios';
import { UseMutationResult } from 'react-query';
import apiClient from '@/lib/apiClient';
import {
  ApiToken,
  ApiTokenResponse,
  CreateApiTokenRequest,
  PaginatedApiTokenResponse,
  UpdateApiTokenRequest,
  OpenAiApiError,
} from '@bodhiapp/ts-client';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// Hooks
export function useListTokens(page: number = 1, pageSize: number = 10, options?: { enabled?: boolean }) {
  return useQuery<PaginatedApiTokenResponse>(
    ['tokens', page.toString(), pageSize.toString()],
    API_TOKENS_ENDPOINT,
    { page, page_size: pageSize },
    options
  );
}

export function useCreateToken(options?: {
  onSuccess?: (response: ApiTokenResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<ApiTokenResponse>, AxiosError<ErrorResponse>, CreateApiTokenRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<ApiTokenResponse, CreateApiTokenRequest>(API_TOKENS_ENDPOINT, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries(['tokens']);
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to generate token';
      options?.onError?.(message);
    },
  });
}

// Interface for update token request that includes the ID for URL construction
interface UpdateTokenRequestWithId extends UpdateApiTokenRequest {
  id: string;
}

export function useUpdateToken(options?: {
  onSuccess?: (token: ApiToken) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<ApiToken>, AxiosError<ErrorResponse>, UpdateTokenRequestWithId> {
  const queryClient = useQueryClient();

  return useMutation<AxiosResponse<ApiToken>, AxiosError<ErrorResponse>, UpdateTokenRequestWithId>(
    async (variables) => {
      // Extract id from variables and create request body without id
      const { id, ...requestBody } = variables;
      const response = await apiClient.put<ApiToken>(`${API_TOKENS_ENDPOINT}/${id}`, requestBody, {
        headers: {
          'Content-Type': 'application/json',
        },
      });
      return response;
    },
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['tokens']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to update token';
        options?.onError?.(message);
      },
    }
  );
}
