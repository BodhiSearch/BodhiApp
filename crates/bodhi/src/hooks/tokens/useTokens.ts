import {
  TokenDetail,
  TokenCreated,
  CreateTokenRequest,
  PaginatedTokenResponse,
  UpdateTokenRequest,
  BodhiErrorResponse,
} from '@bodhiapp/ts-client';
import { AxiosResponse, AxiosError } from 'axios';

import { useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult } from '@/hooks/useQuery';

import { tokenKeys, API_TOKENS_ENDPOINT } from './constants';

// Hooks
export function useListTokens(page: number = 1, pageSize: number = 10, options?: { enabled?: boolean }) {
  return useQuery<PaginatedTokenResponse>(
    tokenKeys.list(page, pageSize),
    API_TOKENS_ENDPOINT,
    { page, page_size: pageSize },
    options
  );
}

export function useCreateToken(options?: {
  onSuccess?: (response: TokenCreated) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<TokenCreated>, AxiosError<BodhiErrorResponse>, CreateTokenRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<TokenCreated, CreateTokenRequest>(API_TOKENS_ENDPOINT, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: tokenKeys.all });
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<BodhiErrorResponse>) => {
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
}): UseMutationResult<AxiosResponse<TokenDetail>, AxiosError<BodhiErrorResponse>, UpdateTokenRequestWithId> {
  const queryClient = useQueryClient();

  return useMutationQuery<TokenDetail, UpdateTokenRequestWithId>(
    ({ id }) => `${API_TOKENS_ENDPOINT}/${id}`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries({ queryKey: tokenKeys.all });
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to update token';
        options?.onError?.(message);
      },
    },
    {
      transformBody: ({ id: _id, ...requestBody }) => requestBody,
    }
  );
}
