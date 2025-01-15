import { useQuery, useMutationQuery } from './useQuery';
import { useQueryClient } from 'react-query';

export const API_TOKENS_ENDPOINT = '/api/tokens';

// Types
export interface CreateTokenRequest {
  name?: string;
}

export interface TokenResponse {
  offline_token: string;
  name?: string;
  status: string;
  created_at: string;
  updated_at: string;
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

// Hooks
export function useListTokens(page: number = 1, pageSize: number = 10) {
  return useQuery<ListTokensResponse>(
    ['tokens', page.toString(), pageSize.toString()],
    API_TOKENS_ENDPOINT,
    { page, page_size: pageSize }
  );
}

export function useCreateToken() {
  const queryClient = useQueryClient();

  return useMutationQuery<TokenResponse, CreateTokenRequest>(
    API_TOKENS_ENDPOINT,
    'post',
    {
      onSuccess: () => {
        // Invalidate all token list queries
        queryClient.invalidateQueries(['tokens']);
      },
    }
  );
}
