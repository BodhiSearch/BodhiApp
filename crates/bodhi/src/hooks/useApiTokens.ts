import { BODHI_API_BASE, useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { AxiosResponse, AxiosError } from 'axios';
import { UseMutationResult } from '@/hooks/useQuery';
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

// Constants
export const API_TOKENS_ENDPOINT = `${BODHI_API_BASE}/tokens`;
export const ENDPOINT_TOKEN_ID = `${BODHI_API_BASE}/tokens/{id}`;

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

  return useMutationQuery<ApiToken, UpdateTokenRequestWithId>(
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
      transformBody: ({ id, ...requestBody }) => requestBody,
    }
  );
}
