import { useMutationQuery } from './useQuery';

export const CREATE_TOKEN_ENDPOINT = '/api/tokens';

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

export function useCreateToken() {
  return useMutationQuery<TokenResponse, CreateTokenRequest>(
    CREATE_TOKEN_ENDPOINT,
    'post'
  );
}
