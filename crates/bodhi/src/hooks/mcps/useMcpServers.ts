import {
  McpServerResponse,
  McpServerRequest,
  McpServerInfo,
  ListMcpServersResponse,
  BodhiApiError,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

import { mcpServerKeys, MCP_SERVERS_ENDPOINT } from './constants';

type ErrorResponse = BodhiApiError;

// ============================================================================
// Query Hooks - MCP Servers
// ============================================================================

export function useListMcpServers(
  params?: {
    enabled?: boolean;
  },
  options?: { enabled?: boolean }
): UseQueryResult<ListMcpServersResponse, AxiosError<ErrorResponse>> {
  const queryParams = params?.enabled !== undefined ? { enabled: String(params.enabled) } : undefined;
  return useQuery<ListMcpServersResponse>(mcpServerKeys.list(params), MCP_SERVERS_ENDPOINT, queryParams, options);
}

export function useGetMcpServer(
  id: string,
  options?: { enabled?: boolean }
): UseQueryResult<McpServerResponse, AxiosError<ErrorResponse>> {
  return useQuery<McpServerResponse>(mcpServerKeys.detail(id), `${MCP_SERVERS_ENDPOINT}/${id}`, undefined, options);
}

// ============================================================================
// Mutation Hooks - MCP Server admin
// ============================================================================

export function useCreateMcpServer(options?: {
  onSuccess?: (server: McpServerResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<McpServerResponse>, AxiosError<ErrorResponse>, McpServerRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<McpServerResponse, McpServerRequest>(() => MCP_SERVERS_ENDPOINT, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: mcpServerKeys.all });
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
}): UseMutationResult<AxiosResponse<McpServerResponse>, AxiosError<ErrorResponse>, McpServerRequest & { id: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<McpServerResponse, McpServerRequest & { id: string }>(
    ({ id }) => `${MCP_SERVERS_ENDPOINT}/${id}`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries({ queryKey: mcpServerKeys.all });
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

// Re-export types for consumers
export type { McpServerResponse, McpServerRequest, McpServerInfo, ListMcpServersResponse };
