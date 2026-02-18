import {
  CreateMcpRequest,
  EnableMcpServerRequest,
  ListMcpServersResponse,
  ListMcpsResponse,
  McpExecuteRequest,
  McpExecuteResponse,
  McpResponse,
  McpServer,
  McpTool,
  McpToolsResponse,
  OpenAiApiError,
  UpdateMcpRequest,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { BODHI_API_BASE, useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// Re-export types for consumers
export type {
  CreateMcpRequest,
  EnableMcpServerRequest,
  ListMcpServersResponse,
  ListMcpsResponse,
  McpExecuteRequest,
  McpExecuteResponse,
  McpResponse,
  McpServer,
  McpTool,
  McpToolsResponse,
  UpdateMcpRequest,
};

// ============================================================================
// Endpoints
// ============================================================================

export const MCPS_ENDPOINT = `${BODHI_API_BASE}/mcps`;
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
// Query Hooks - MCP Servers (admin allowlist)
// ============================================================================

export function useMcpServers(options?: {
  enabled?: boolean;
}): UseQueryResult<ListMcpServersResponse, AxiosError<ErrorResponse>> {
  return useQuery<ListMcpServersResponse>(['mcp_servers'], MCP_SERVERS_ENDPOINT, undefined, options);
}

export function useMcpServerCheck(
  url: string,
  options?: { enabled?: boolean }
): UseQueryResult<ListMcpServersResponse, AxiosError<ErrorResponse>> {
  return useQuery<ListMcpServersResponse>(
    ['mcp_servers', 'check', url],
    MCP_SERVERS_ENDPOINT,
    { url },
    { ...options, enabled: !!url && options?.enabled !== false }
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

export function useEnableMcpServer(options?: {
  onSuccess?: (server: McpServer) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<McpServer>, AxiosError<ErrorResponse>, EnableMcpServerRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<McpServer, EnableMcpServerRequest>(() => MCP_SERVERS_ENDPOINT, 'put', {
    onSuccess: (response) => {
      queryClient.invalidateQueries(['mcp_servers']);
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to enable MCP server';
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
