import { AxiosError, AxiosResponse } from 'axios';

import { BODHI_API_BASE, useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

// ============================================================================
// Types - MCP Server (admin registry)
// ============================================================================

export interface McpServerInfo {
  id: string;
  url: string;
  name: string;
  enabled: boolean;
}

export interface McpServerResponse {
  id: string;
  url: string;
  name: string;
  description?: string;
  enabled: boolean;
  created_by: string;
  updated_by: string;
  enabled_mcp_count: number;
  disabled_mcp_count: number;
  created_at: string;
  updated_at: string;
}

export interface CreateMcpServerRequest {
  url: string;
  name: string;
  description?: string;
  enabled: boolean;
}

export interface UpdateMcpServerRequest {
  url: string;
  name: string;
  description?: string;
  enabled: boolean;
}

export interface ListMcpServersResponse {
  mcp_servers: McpServerResponse[];
}

// ============================================================================
// Types - MCP Instance (user-owned)
// ============================================================================

export interface McpTool {
  name: string;
  description?: string;
  input_schema?: unknown;
}

export interface McpResponse {
  id: string;
  mcp_server: McpServerInfo;
  slug: string;
  name: string;
  description?: string;
  enabled: boolean;
  tools_cache?: McpTool[];
  tools_filter?: string[];
  created_at: string;
  updated_at: string;
}

export interface CreateMcpRequest {
  name: string;
  slug: string;
  mcp_server_id: string;
  description?: string;
  enabled: boolean;
  tools_cache?: McpTool[];
  tools_filter?: string[];
}

export interface UpdateMcpRequest {
  name: string;
  slug: string;
  description?: string;
  enabled: boolean;
  tools_filter?: string[];
  tools_cache?: McpTool[];
}

export interface FetchMcpToolsRequest {
  mcp_server_id: string;
  auth: 'public';
}

export interface ListMcpsResponse {
  mcps: McpResponse[];
}

export interface McpToolsResponse {
  tools: McpTool[];
}

export interface McpExecuteRequest {
  params: unknown;
}

export interface McpExecuteResponse {
  result?: unknown;
  error?: string;
}

// Error type alias
interface ErrorResponse {
  error?: {
    message?: string;
  };
}

// ============================================================================
// Endpoints
// ============================================================================

export const MCPS_ENDPOINT = `${BODHI_API_BASE}/mcps`;
export const MCPS_FETCH_TOOLS_ENDPOINT = `${BODHI_API_BASE}/mcps/fetch-tools`;
export const MCP_SERVERS_ENDPOINT = `${BODHI_API_BASE}/mcp_servers`;

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
