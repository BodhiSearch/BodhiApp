import {
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
  OpenAiApiError,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { BODHI_API_BASE, useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

export type {
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
};

type ErrorResponse = OpenAiApiError;

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
