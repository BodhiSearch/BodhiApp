import {
  CreateAuthConfigBody,
  CreateMcpAuthConfigRequest,
  McpAuthConfigResponse,
  McpAuthConfigsListResponse,
  McpServerInfo,
  McpServerResponse,
  CreateMcpServerRequest,
  UpdateMcpServerRequest,
  ListMcpServersResponse,
  McpTool,
  McpResponse,
  McpAuth,
  CreateMcpRequest,
  UpdateMcpRequest,
  FetchMcpToolsRequest,
  ListMcpsResponse,
  McpToolsResponse,
  McpExecuteRequest,
  McpExecuteResponse,
  OAuthDiscoverMcpRequest,
  OAuthDiscoverMcpResponse,
  DynamicRegisterRequest,
  DynamicRegisterResponse,
  OAuthLoginRequest,
  OAuthLoginResponse,
  OAuthTokenExchangeRequest,
  OAuthTokenResponse,
  OpenAiApiError,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { BODHI_API_BASE, useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

export type {
  CreateAuthConfigBody,
  CreateMcpAuthConfigRequest,
  McpAuthConfigResponse,
  McpAuthConfigsListResponse,
  McpServerInfo,
  McpServerResponse,
  CreateMcpServerRequest,
  UpdateMcpServerRequest,
  ListMcpServersResponse,
  McpTool,
  McpResponse,
  McpAuth,
  CreateMcpRequest,
  UpdateMcpRequest,
  FetchMcpToolsRequest,
  ListMcpsResponse,
  McpToolsResponse,
  McpExecuteRequest,
  McpExecuteResponse,
  OAuthDiscoverMcpRequest,
  OAuthDiscoverMcpResponse,
  DynamicRegisterRequest,
  DynamicRegisterResponse,
  OAuthLoginRequest,
  OAuthLoginResponse,
  OAuthTokenExchangeRequest,
  OAuthTokenResponse,
};

type ErrorResponse = OpenAiApiError;

// ============================================================================
// Endpoints
// ============================================================================

export const MCPS_ENDPOINT = `${BODHI_API_BASE}/mcps`;
export const MCPS_FETCH_TOOLS_ENDPOINT = `${BODHI_API_BASE}/mcps/fetch-tools`;
export const MCPS_OAUTH_DISCOVER_MCP_ENDPOINT = `${BODHI_API_BASE}/mcps/oauth/discover-mcp`;
export const MCPS_OAUTH_TOKENS_ENDPOINT = `${BODHI_API_BASE}/mcps/oauth-tokens`;
export const MCP_SERVERS_ENDPOINT = `${BODHI_API_BASE}/mcps/servers`;
export const MCPS_OAUTH_DYNAMIC_REGISTER_STANDALONE_ENDPOINT = `${BODHI_API_BASE}/mcps/oauth/dynamic-register`;
export const MCPS_AUTH_CONFIGS_ENDPOINT = `${BODHI_API_BASE}/mcps/auth-configs`;

// ============================================================================
// Query Hooks - MCP Instance CRUD
// ============================================================================

export function useMcps(options?: { enabled?: boolean }): UseQueryResult<ListMcpsResponse, AxiosError<ErrorResponse>> {
  return useQuery<ListMcpsResponse>(['mcps'], MCPS_ENDPOINT, undefined, options);
}

export function useMcp(
  id: string,
  options?: { enabled?: boolean }
): UseQueryResult<McpResponse, AxiosError<ErrorResponse>> {
  return useQuery<McpResponse>(['mcps', id], `${MCPS_ENDPOINT}/${id}`, undefined, options);
}

// ============================================================================
// Query Hooks - MCP Servers
// ============================================================================

export function useMcpServers(
  params?: {
    enabled?: boolean;
  },
  options?: { enabled?: boolean }
): UseQueryResult<ListMcpServersResponse, AxiosError<ErrorResponse>> {
  const queryParams = params?.enabled !== undefined ? { enabled: String(params.enabled) } : undefined;
  const key = params?.enabled !== undefined ? ['mcp_servers', String(params.enabled)] : ['mcp_servers'];
  return useQuery<ListMcpServersResponse>(key, MCP_SERVERS_ENDPOINT, queryParams, options);
}

export function useMcpServer(
  id: string,
  options?: { enabled?: boolean }
): UseQueryResult<McpServerResponse, AxiosError<ErrorResponse>> {
  return useQuery<McpServerResponse>(['mcp_servers', id], `${MCP_SERVERS_ENDPOINT}/${id}`, undefined, options);
}

// ============================================================================
// Query Hooks - Unified Auth Configs
// ============================================================================

export function useListAuthConfigs(
  serverId: string,
  options?: { enabled?: boolean }
): UseQueryResult<McpAuthConfigsListResponse, AxiosError<ErrorResponse>> {
  return useQuery<McpAuthConfigsListResponse>(
    ['auth-configs', 'unified', serverId],
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
    ['auth-configs', 'unified', configId],
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
    ['oauth-tokens', tokenId],
    `${MCPS_OAUTH_TOKENS_ENDPOINT}/${tokenId}`,
    undefined,
    { ...options, enabled: !!tokenId && options?.enabled !== false }
  );
}

// ============================================================================
// Mutation Hooks - MCP Instance CRUD
// ============================================================================

export function useCreateMcp(options?: {
  onSuccess?: (mcp: McpResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<McpResponse>, AxiosError<ErrorResponse>, CreateMcpRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<McpResponse, CreateMcpRequest>(() => MCPS_ENDPOINT, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries(['mcps']);
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to create MCP';
      options?.onError?.(message);
    },
  });
}

export function useUpdateMcp(options?: {
  onSuccess?: (mcp: McpResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<McpResponse>, AxiosError<ErrorResponse>, UpdateMcpRequest & { id: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<McpResponse, UpdateMcpRequest & { id: string }>(
    ({ id }) => `${MCPS_ENDPOINT}/${id}`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['mcps']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to update MCP';
        options?.onError?.(message);
      },
    },
    { transformBody: ({ id: _id, ...body }) => body }
  );
}

export function useDeleteMcp(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, { id: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<void, { id: string }>(
    ({ id }) => `${MCPS_ENDPOINT}/${id}`,
    'delete',
    {
      onSuccess: () => {
        queryClient.invalidateQueries(['mcps']);
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to delete MCP';
        options?.onError?.(message);
      },
    },
    { noBody: true }
  );
}

// ============================================================================
// Mutation Hooks - MCP Server admin
// ============================================================================

export function useCreateMcpServer(options?: {
  onSuccess?: (server: McpServerResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<McpServerResponse>, AxiosError<ErrorResponse>, CreateMcpServerRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<McpServerResponse, CreateMcpServerRequest>(() => MCP_SERVERS_ENDPOINT, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries(['mcp_servers']);
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to create MCP server';
      options?.onError?.(message);
    },
  });
}

export function useUpdateMcpServer(options?: {
  onSuccess?: (server: McpServerResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<McpServerResponse>,
  AxiosError<ErrorResponse>,
  UpdateMcpServerRequest & { id: string }
> {
  const queryClient = useQueryClient();

  return useMutationQuery<McpServerResponse, UpdateMcpServerRequest & { id: string }>(
    ({ id }) => `${MCP_SERVERS_ENDPOINT}/${id}`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['mcp_servers']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to update MCP server';
        options?.onError?.(message);
      },
    },
    { transformBody: ({ id: _id, ...body }) => body }
  );
}

// ============================================================================
// Mutation Hooks - Tool discovery
// ============================================================================

export function useFetchMcpTools(options?: {
  onSuccess?: (tools: McpToolsResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<McpToolsResponse>, AxiosError<ErrorResponse>, FetchMcpToolsRequest> {
  return useMutationQuery<McpToolsResponse, FetchMcpToolsRequest>(() => MCPS_FETCH_TOOLS_ENDPOINT, 'post', {
    onSuccess: (response) => {
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to fetch tools';
      options?.onError?.(message);
    },
  });
}

// ============================================================================
// Mutation Hooks - Tool operations
// ============================================================================

export function useRefreshMcpTools(options?: {
  onSuccess?: (tools: McpToolsResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<McpToolsResponse>, AxiosError<ErrorResponse>, { id: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<McpToolsResponse, { id: string }>(
    ({ id }) => `${MCPS_ENDPOINT}/${id}/tools/refresh`,
    'post',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['mcps']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to fetch tools';
        options?.onError?.(message);
      },
    },
    { noBody: true }
  );
}

export function useExecuteMcpTool(options?: {
  onSuccess?: (response: McpExecuteResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<McpExecuteResponse>,
  AxiosError<ErrorResponse>,
  { id: string; toolName: string; params: unknown }
> {
  return useMutationQuery<McpExecuteResponse, { id: string; toolName: string; params: unknown }>(
    ({ id, toolName }) => `${MCPS_ENDPOINT}/${id}/tools/${toolName}/execute`,
    'post',
    {
      onSuccess: (response) => options?.onSuccess?.(response.data),
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to execute tool';
        options?.onError?.(message);
      },
    },
    { transformBody: ({ id: _id, toolName: _tn, ...body }) => body }
  );
}

// ============================================================================
// Mutation Hooks - OAuth Discovery & Login & Token
// ============================================================================

export function useDiscoverMcp(options?: {
  onSuccess?: (response: OAuthDiscoverMcpResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<OAuthDiscoverMcpResponse>, AxiosError<ErrorResponse>, OAuthDiscoverMcpRequest> {
  return useMutationQuery<OAuthDiscoverMcpResponse, OAuthDiscoverMcpRequest>(
    () => MCPS_OAUTH_DISCOVER_MCP_ENDPOINT,
    'post',
    {
      onSuccess: (response) => options?.onSuccess?.(response.data),
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to discover MCP OAuth endpoints';
        options?.onError?.(message);
      },
    }
  );
}

export function useStandaloneDynamicRegister(options?: {
  onSuccess?: (response: DynamicRegisterResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<DynamicRegisterResponse>, AxiosError<ErrorResponse>, DynamicRegisterRequest> {
  return useMutationQuery<DynamicRegisterResponse, DynamicRegisterRequest>(
    () => MCPS_OAUTH_DYNAMIC_REGISTER_STANDALONE_ENDPOINT,
    'post',
    {
      onSuccess: (response) => options?.onSuccess?.(response.data),
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to register dynamic client';
        options?.onError?.(message);
      },
    }
  );
}

export function useOAuthLogin(options?: {
  onSuccess?: (response: OAuthLoginResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<OAuthLoginResponse>,
  AxiosError<ErrorResponse>,
  OAuthLoginRequest & { id: string }
> {
  return useMutationQuery<OAuthLoginResponse, OAuthLoginRequest & { id: string }>(
    ({ id }) => `${MCPS_AUTH_CONFIGS_ENDPOINT}/${id}/login`,
    'post',
    {
      onSuccess: (response) => options?.onSuccess?.(response.data),
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to initiate OAuth login';
        options?.onError?.(message);
      },
    },
    { transformBody: ({ id: _id, ...body }) => body }
  );
}

export function useOAuthTokenExchange(options?: {
  onSuccess?: (response: OAuthTokenResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<OAuthTokenResponse>,
  AxiosError<ErrorResponse>,
  OAuthTokenExchangeRequest & { id: string }
> {
  return useMutationQuery<OAuthTokenResponse, OAuthTokenExchangeRequest & { id: string }>(
    ({ id }) => `${MCPS_AUTH_CONFIGS_ENDPOINT}/${id}/token`,
    'post',
    {
      onSuccess: (response) => options?.onSuccess?.(response.data),
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to exchange OAuth token';
        options?.onError?.(message);
      },
    },
    { transformBody: ({ id: _id, ...body }) => body }
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
        queryClient.invalidateQueries(['oauth-tokens']);
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

// ============================================================================
// Mutation Hooks - Unified Auth Config CRUD
// ============================================================================

export function useCreateAuthConfig(options?: {
  onSuccess?: (config: McpAuthConfigResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<McpAuthConfigResponse>, AxiosError<ErrorResponse>, CreateAuthConfigBody> {
  const queryClient = useQueryClient();

  return useMutationQuery<McpAuthConfigResponse, CreateAuthConfigBody>(() => MCPS_AUTH_CONFIGS_ENDPOINT, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries(['auth-configs']);
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
        queryClient.invalidateQueries(['auth-configs']);
        queryClient.invalidateQueries(['mcp_servers']);
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
