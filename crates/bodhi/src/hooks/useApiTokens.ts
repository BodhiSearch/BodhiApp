import { API_TOKENS_ENDPOINT, useMutationQuery, useQuery } from '@/hooks/useQuery';
import { useQueryClient } from 'react-query';
import { AxiosResponse, AxiosError } from 'axios';
import { UseMutationResult } from 'react-query';
import { OpenAiApiError } from '@bodhiapp/ts-client';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

export interface CreateTokenRequest {
  name?: string;
}

export interface TokenResponse {
  offline_token: string;
}

export interface ApiToken {
  id: string;
  name: string;
  status: 'active' | 'inactive';
  created_at: string;
  updated_at: string;
}

export interface ListTokensResponse {
  data: ApiToken[];
  total: number;
  page: number;
  page_size: number;
}

export interface UpdateTokenRequest {
  id: string;
  name?: string;
  status: 'active' | 'inactive';
}

// Hooks
export function useListTokens(page: number = 1, pageSize: number = 10, options?: { enabled?: boolean }) {
  return useQuery<ListTokensResponse>(
    ['tokens', page.toString(), pageSize.toString()],
    API_TOKENS_ENDPOINT,
    { page, page_size: pageSize },
    options
  );
}

export function useCreateToken(options?: {
  onSuccess?: (response: TokenResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<TokenResponse>, AxiosError<ErrorResponse>, CreateTokenRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<TokenResponse, CreateTokenRequest>(API_TOKENS_ENDPOINT, 'post', {
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

export function useUpdateToken(options?: {
  onSuccess?: (token: ApiToken) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<ApiToken>, AxiosError<ErrorResponse>, UpdateTokenRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<ApiToken, UpdateTokenRequest>((params) => `${API_TOKENS_ENDPOINT}/${params.id}`, 'put', {
    onSuccess: (response) => {
      queryClient.invalidateQueries(['tokens']);
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to update token';
      options?.onError?.(message);
    },
  });
}
