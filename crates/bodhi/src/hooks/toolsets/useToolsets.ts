import {
  ToolsetResponse,
  ToolsetRequest,
  ApiKeyUpdate,
  ListToolsetsResponse,
  ToolsetDefinition,
  ToolDefinition,
  AppToolsetConfig,
  OpenAiApiError,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

import { toolsetKeys, TOOLSETS_ENDPOINT } from './constants';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// Re-export types for consumers
export type { ToolsetResponse, ToolsetRequest, ApiKeyUpdate, ToolsetDefinition, ToolDefinition, AppToolsetConfig };

// ============================================================================
// Query Hooks - Toolset CRUD
// ============================================================================

/**
 * List all toolsets for the authenticated user
 */
export function useListToolsets(options?: {
  enabled?: boolean;
}): UseQueryResult<ListToolsetsResponse, AxiosError<ErrorResponse>> {
  return useQuery<ListToolsetsResponse>(toolsetKeys.all, TOOLSETS_ENDPOINT, undefined, options);
}

/**
 * Get a single toolset by UUID
 */
export function useGetToolset(
  id: string,
  options?: { enabled?: boolean }
): UseQueryResult<ToolsetResponse, AxiosError<ErrorResponse>> {
  return useQuery<ToolsetResponse>(toolsetKeys.detail(id), `${TOOLSETS_ENDPOINT}/${id}`, undefined, options);
}

// ============================================================================
// Mutation Hooks - Toolset CRUD
// ============================================================================

/**
 * Create a new toolset
 */
export function useCreateToolset(options?: {
  onSuccess?: (toolset: ToolsetResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<ToolsetResponse>, AxiosError<ErrorResponse>, ToolsetRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<ToolsetResponse, ToolsetRequest>(() => TOOLSETS_ENDPOINT, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: toolsetKeys.all });
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to create toolset';
      options?.onError?.(message);
    },
  });
}

/**
 * Update an existing toolset
 */
export function useUpdateToolset(options?: {
  onSuccess?: (toolset: ToolsetResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<ToolsetResponse>, AxiosError<ErrorResponse>, ToolsetRequest & { id: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<ToolsetResponse, ToolsetRequest & { id: string }>(
    ({ id }) => `${TOOLSETS_ENDPOINT}/${id}`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries({ queryKey: toolsetKeys.all });
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to update toolset';
        options?.onError?.(message);
      },
    },
    { transformBody: ({ id: _id, ...body }) => body }
  );
}

/**
 * Delete a toolset
 */
export function useDeleteToolset(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, { id: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<void, { id: string }>(
    ({ id }) => `${TOOLSETS_ENDPOINT}/${id}`,
    'delete',
    {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: toolsetKeys.all });
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to delete toolset';
        options?.onError?.(message);
      },
    },
    { noBody: true }
  );
}
