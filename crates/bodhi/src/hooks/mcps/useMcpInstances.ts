import { Mcp, McpRequest, ListMcpsResponse, BodhiErrorResponse } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

import { mcpKeys, MCPS_ENDPOINT } from './constants';

// ============================================================================
// Query Hooks - MCP Instance CRUD
// ============================================================================

export function useListMcps(options?: {
  enabled?: boolean;
}): UseQueryResult<ListMcpsResponse, AxiosError<BodhiErrorResponse>> {
  return useQuery<ListMcpsResponse>(mcpKeys.all, MCPS_ENDPOINT, undefined, options);
}

export function useGetMcp(id: string, options?: { enabled?: boolean }): UseQueryResult<Mcp, AxiosError<BodhiErrorResponse>> {
  return useQuery<Mcp>(mcpKeys.detail(id), `${MCPS_ENDPOINT}/${id}`, undefined, options);
}

// ============================================================================
// Mutation Hooks - MCP Instance CRUD
// ============================================================================

export function useCreateMcp(options?: {
  onSuccess?: (mcp: Mcp) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<Mcp>, AxiosError<BodhiErrorResponse>, McpRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<Mcp, McpRequest>(() => MCPS_ENDPOINT, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: mcpKeys.all });
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<BodhiErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to create MCP';
      options?.onError?.(message);
    },
  });
}

export function useUpdateMcp(options?: {
  onSuccess?: (mcp: Mcp) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<Mcp>, AxiosError<BodhiErrorResponse>, McpRequest & { id: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<Mcp, McpRequest & { id: string }>(
    ({ id }) => `${MCPS_ENDPOINT}/${id}`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries({ queryKey: mcpKeys.all });
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
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
}): UseMutationResult<AxiosResponse<void>, AxiosError<BodhiErrorResponse>, { id: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<void, { id: string }>(
    ({ id }) => `${MCPS_ENDPOINT}/${id}`,
    'delete',
    {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: mcpKeys.all });
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to delete MCP';
        options?.onError?.(message);
      },
    },
    { noBody: true }
  );
}

// Re-export types for consumers
export type { Mcp, McpRequest, ListMcpsResponse };
