import {
  TokenDetail,
  TokenCreated,
  CreateTokenRequest,
  PaginatedTokenResponse,
  UpdateTokenRequest,
  OpenAiApiError,
} from '@bodhiapp/ts-client';
import { AxiosResponse, AxiosError } from 'axios';

import { BODHI_API_BASE, useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult } from '@/hooks/useQuery';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// Constants
export const API_TOKENS_ENDPOINT = `${BODHI_API_BASE}/tokens`;
export const ENDPOINT_TOKEN_ID = `${BODHI_API_BASE}/tokens/{id}`;

// Hooks
export function useListTokens(page: number = 1, pageSize: number = 10, options?: { enabled?: boolean }) {
  return useQuery<PaginatedTokenResponse>(
    ['tokens', page.toString(), pageSize.toString()],
    API_TOKENS_ENDPOINT,
    { page, page_size: pageSize },
    options
  );
}

export function useCreateToken(options?: {
  onSuccess?: (response: TokenCreated) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<TokenCreated>, AxiosError<ErrorResponse>, CreateTokenRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<TokenCreated, CreateTokenRequest>(API_TOKENS_ENDPOINT, 'post', {
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
interface UpdateTokenRequestWithId extends UpdateTokenRequest {
  id: string;
}

export function useUpdateToken(options?: {
  onSuccess?: (token: TokenDetail) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<TokenDetail>, AxiosError<ErrorResponse>, UpdateTokenRequestWithId> {
  const queryClient = useQueryClient();

  return useMutationQuery<TokenDetail, UpdateTokenRequestWithId>(
    ({ id }) => `${API_TOKENS_ENDPOINT}/${id}`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['tokens']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to update token';
        options?.onError?.(message);
      },
    },
    {
      transformBody: ({ id: _id, ...requestBody }) => requestBody,
    }
  );
}
