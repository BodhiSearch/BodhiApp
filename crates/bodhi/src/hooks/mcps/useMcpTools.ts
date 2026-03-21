import {
  McpTool,
  FetchMcpToolsRequest,
  McpToolsResponse,
  McpExecuteRequest,
  McpExecuteResponse,
  OpenAiApiError,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { useMutationQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult } from '@/hooks/useQuery';

import { mcpKeys, MCPS_ENDPOINT, MCPS_FETCH_TOOLS_ENDPOINT } from './constants';

type ErrorResponse = OpenAiApiError;

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
        queryClient.invalidateQueries({ queryKey: mcpKeys.all });
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

// Re-export types for consumers
export type { McpTool, FetchMcpToolsRequest, McpToolsResponse, McpExecuteRequest, McpExecuteResponse };
