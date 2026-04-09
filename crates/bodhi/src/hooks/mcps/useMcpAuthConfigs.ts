import {
  CreateAuthConfig,
  CreateMcpAuthConfigRequest,
  McpAuthConfigResponse,
  McpAuthConfigsListResponse,
  McpAuthType,
  OAuthTokenResponse,
  BodhiApiError,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

import {
  authConfigKeys,
  oauthTokenKeys,
  mcpServerKeys,
  MCPS_AUTH_CONFIGS_ENDPOINT,
  MCPS_OAUTH_TOKENS_ENDPOINT,
} from './constants';

type ErrorResponse = BodhiApiError;

// ============================================================================
// Query Hooks - Unified Auth Configs
// ============================================================================

export function useListAuthConfigs(
  serverId: string,
  options?: { enabled?: boolean }
): UseQueryResult<McpAuthConfigsListResponse, AxiosError<ErrorResponse>> {
  return useQuery<McpAuthConfigsListResponse>(
    authConfigKeys.list(serverId),
    MCPS_AUTH_CONFIGS_ENDPOINT,
    { mcp_server_id: serverId },
    { ...options, enabled: !!serverId && options?.enabled !== false }
  );
}

export function useGetAuthConfig(
  configId: string,
  options?: { enabled?: boolean }
): UseQueryResult<McpAuthConfigResponse, AxiosError<ErrorResponse>> {
  return useQuery<McpAuthConfigResponse>(
    authConfigKeys.detail(configId),
    `${MCPS_AUTH_CONFIGS_ENDPOINT}/${configId}`,
    undefined,
    { ...options, enabled: !!configId && options?.enabled !== false }
  );
}

// ============================================================================
// Query Hooks - OAuth Tokens
// ============================================================================

export function useGetOAuthToken(
  tokenId: string,
  options?: { enabled?: boolean }
): UseQueryResult<OAuthTokenResponse, AxiosError<ErrorResponse>> {
  return useQuery<OAuthTokenResponse>(
    oauthTokenKeys.detail(tokenId),
    `${MCPS_OAUTH_TOKENS_ENDPOINT}/${tokenId}`,
    undefined,
    { ...options, enabled: !!tokenId && options?.enabled !== false }
  );
}

// ============================================================================
// Mutation Hooks - Unified Auth Config CRUD
// ============================================================================

export function useCreateAuthConfig(options?: {
  onSuccess?: (config: McpAuthConfigResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<McpAuthConfigResponse>, AxiosError<ErrorResponse>, CreateAuthConfig> {
  const queryClient = useQueryClient();

  return useMutationQuery<McpAuthConfigResponse, CreateAuthConfig>(() => MCPS_AUTH_CONFIGS_ENDPOINT, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: authConfigKeys.all });
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to create auth config';
      options?.onError?.(message);
    },
  });
}

export function useDeleteAuthConfig(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, { configId: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<void, { configId: string }>(
    ({ configId }) => `${MCPS_AUTH_CONFIGS_ENDPOINT}/${configId}`,
    'delete',
    {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: authConfigKeys.all });
        queryClient.invalidateQueries({ queryKey: mcpServerKeys.all });
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to delete auth config';
        options?.onError?.(message);
      },
    },
    { noBody: true }
  );
}

// ============================================================================
// Mutation Hooks - OAuth Token operations
// ============================================================================

export function useDeleteOAuthToken(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, { tokenId: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<void, { tokenId: string }>(
    ({ tokenId }) => `${MCPS_OAUTH_TOKENS_ENDPOINT}/${tokenId}`,
    'delete',
    {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: oauthTokenKeys.all });
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to delete OAuth token';
        options?.onError?.(message);
      },
    },
    { noBody: true }
  );
}

// Re-export types for consumers
export type {
  CreateAuthConfig,
  CreateMcpAuthConfigRequest,
  McpAuthConfigResponse,
  McpAuthConfigsListResponse,
  McpAuthType,
  OAuthTokenResponse,
};
