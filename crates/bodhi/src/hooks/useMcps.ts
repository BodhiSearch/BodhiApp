import {
  AuthHeaderResponse,
  CreateAuthHeaderRequest,
  CreateOAuthConfigRequest,
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
  OAuthConfigResponse,
  OAuthConfigsListResponse,
  OAuthDiscoverRequest,
  OAuthDiscoverResponse,
  OAuthLoginRequest,
  OAuthLoginResponse,
  OAuthTokenExchangeRequest,
  OAuthTokenResponse,
  OpenAiApiError,
  UpdateAuthHeaderRequest,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { BODHI_API_BASE, useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

export type {
  AuthHeaderResponse,
  CreateAuthHeaderRequest,
  CreateOAuthConfigRequest,
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
  OAuthConfigResponse,
  OAuthConfigsListResponse,
  OAuthDiscoverRequest,
  OAuthDiscoverResponse,
  OAuthLoginRequest,
  OAuthLoginResponse,
  OAuthTokenExchangeRequest,
  OAuthTokenResponse,
  UpdateAuthHeaderRequest,
};

type ErrorResponse = OpenAiApiError;

// ============================================================================
// Endpoints
// ============================================================================

export const MCPS_ENDPOINT = `${BODHI_API_BASE}/mcps`;
export const MCPS_FETCH_TOOLS_ENDPOINT = `${BODHI_API_BASE}/mcps/fetch-tools`;
export const MCPS_AUTH_HEADERS_ENDPOINT = `${BODHI_API_BASE}/mcps/auth-headers`;
export const MCPS_OAUTH_DISCOVER_ENDPOINT = `${BODHI_API_BASE}/mcps/oauth/discover`;
export const MCPS_OAUTH_TOKENS_ENDPOINT = `${BODHI_API_BASE}/mcps/oauth-tokens`;
export const MCP_SERVERS_ENDPOINT = `${BODHI_API_BASE}/mcp_servers`;

export const MCP_SERVERS_OAUTH_CONFIGS_ENDPOINT = (serverId: string) =>
  `${BODHI_API_BASE}/mcp-servers/${serverId}/oauth-configs`;

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
// Query Hooks - Auth Headers
// ============================================================================

export function useAuthHeader(
  id: string,
  options?: { enabled?: boolean }
): UseQueryResult<AuthHeaderResponse, AxiosError<ErrorResponse>> {
  return useQuery<AuthHeaderResponse>(['auth-headers', id], `${MCPS_AUTH_HEADERS_ENDPOINT}/${id}`, undefined, options);
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
// Query Hooks - OAuth Configs (nested under server)
// ============================================================================

export function useListOAuthConfigs(
  serverId: string,
  options?: { enabled?: boolean }
): UseQueryResult<OAuthConfigsListResponse, AxiosError<ErrorResponse>> {
  return useQuery<OAuthConfigsListResponse>(
    ['oauth-configs', 'list', serverId],
    `${BODHI_API_BASE}/mcp-servers/${serverId}/oauth-configs`,
    undefined,
    { ...options, enabled: !!serverId && options?.enabled !== false }
  );
}

export function useOAuthConfig(
  serverId: string,
  id: string,
  options?: { enabled?: boolean }
): UseQueryResult<OAuthConfigResponse, AxiosError<ErrorResponse>> {
  return useQuery<OAuthConfigResponse>(
    ['oauth-configs', id],
    `${BODHI_API_BASE}/mcp-servers/${serverId}/oauth-configs/${id}`,
    undefined,
    options
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
// Mutation Hooks - Auth Headers CRUD
// ============================================================================

export function useCreateAuthHeader(options?: {
  onSuccess?: (header: AuthHeaderResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AuthHeaderResponse>, AxiosError<ErrorResponse>, CreateAuthHeaderRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<AuthHeaderResponse, CreateAuthHeaderRequest>(() => MCPS_AUTH_HEADERS_ENDPOINT, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries(['auth-headers']);
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to create auth header';
      options?.onError?.(message);
    },
  });
}

export function useUpdateAuthHeader(options?: {
  onSuccess?: (header: AuthHeaderResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<AuthHeaderResponse>,
  AxiosError<ErrorResponse>,
  UpdateAuthHeaderRequest & { id: string }
> {
  const queryClient = useQueryClient();

  return useMutationQuery<AuthHeaderResponse, UpdateAuthHeaderRequest & { id: string }>(
    ({ id }) => `${MCPS_AUTH_HEADERS_ENDPOINT}/${id}`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['auth-headers']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to update auth header';
        options?.onError?.(message);
      },
    },
    { transformBody: ({ id: _id, ...body }) => body }
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
// Mutation Hooks - OAuth Config CRUD (nested under server)
// ============================================================================

export function useCreateOAuthConfig(options?: {
  onSuccess?: (config: OAuthConfigResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<OAuthConfigResponse>,
  AxiosError<ErrorResponse>,
  CreateOAuthConfigRequest & { serverId: string }
> {
  const queryClient = useQueryClient();

  return useMutationQuery<OAuthConfigResponse, CreateOAuthConfigRequest & { serverId: string }>(
    ({ serverId }) => `${BODHI_API_BASE}/mcp-servers/${serverId}/oauth-configs`,
    'post',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['oauth-configs']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to create OAuth config';
        options?.onError?.(message);
      },
    },
    { transformBody: ({ serverId: _sid, ...body }) => body }
  );
}

// ============================================================================
// Mutation Hooks - OAuth Discovery & Login & Token
// ============================================================================

export function useOAuthDiscover(options?: {
  onSuccess?: (response: OAuthDiscoverResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<OAuthDiscoverResponse>, AxiosError<ErrorResponse>, OAuthDiscoverRequest> {
  return useMutationQuery<OAuthDiscoverResponse, OAuthDiscoverRequest>(() => MCPS_OAUTH_DISCOVER_ENDPOINT, 'post', {
    onSuccess: (response) => options?.onSuccess?.(response.data),
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to discover OAuth endpoints';
      options?.onError?.(message);
    },
  });
}

export function useOAuthLogin(options?: {
  onSuccess?: (response: OAuthLoginResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<OAuthLoginResponse>,
  AxiosError<ErrorResponse>,
  OAuthLoginRequest & { id: string; serverId: string }
> {
  return useMutationQuery<OAuthLoginResponse, OAuthLoginRequest & { id: string; serverId: string }>(
    ({ serverId, id }) => `${BODHI_API_BASE}/mcp-servers/${serverId}/oauth-configs/${id}/login`,
    'post',
    {
      onSuccess: (response) => options?.onSuccess?.(response.data),
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to initiate OAuth login';
        options?.onError?.(message);
      },
    },
    { transformBody: ({ id: _id, serverId: _sid, ...body }) => body }
  );
}

export function useOAuthTokenExchange(options?: {
  onSuccess?: (response: OAuthTokenResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<OAuthTokenResponse>,
  AxiosError<ErrorResponse>,
  OAuthTokenExchangeRequest & { id: string; serverId: string }
> {
  return useMutationQuery<OAuthTokenResponse, OAuthTokenExchangeRequest & { id: string; serverId: string }>(
    ({ serverId, id }) => `${BODHI_API_BASE}/mcp-servers/${serverId}/oauth-configs/${id}/token`,
    'post',
    {
      onSuccess: (response) => options?.onSuccess?.(response.data),
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to exchange OAuth token';
        options?.onError?.(message);
      },
    },
    { transformBody: ({ id: _id, serverId: _sid, ...body }) => body }
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
