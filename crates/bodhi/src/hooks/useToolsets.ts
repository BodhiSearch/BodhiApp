import {
  AppToolsetConfigResponse,
  ToolsetResponse,
  CreateToolsetRequest,
  UpdateToolsetRequest,
  ApiKeyUpdateDto,
  ListToolsetsResponse,
  ToolsetTypeResponse,
  ListToolsetTypesResponse,
  ToolDefinition,
  OpenAiApiError,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { BODHI_API_BASE, useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// Re-export types for consumers
export type {
  ToolsetResponse,
  CreateToolsetRequest,
  UpdateToolsetRequest,
  ApiKeyUpdateDto,
  ToolsetTypeResponse,
  ToolDefinition,
  AppToolsetConfigResponse,
};

// ============================================================================
// Endpoints
// ============================================================================

export const TOOLSETS_ENDPOINT = `${BODHI_API_BASE}/toolsets`;
export const TOOLSET_TYPES_ENDPOINT = `${BODHI_API_BASE}/toolset_types`;

// ============================================================================
// Query Hooks - Toolset CRUD
// ============================================================================

/**
 * List all toolsets for the authenticated user
 */
export function useToolsets(options?: {
  enabled?: boolean;
}): UseQueryResult<ListToolsetsResponse, AxiosError<ErrorResponse>> {
  return useQuery<ListToolsetsResponse>(['toolsets'], TOOLSETS_ENDPOINT, undefined, options);
}

/**
 * Get a single toolset by UUID
 */
export function useToolset(
  id: string,
  options?: { enabled?: boolean }
): UseQueryResult<ToolsetResponse, AxiosError<ErrorResponse>> {
  return useQuery<ToolsetResponse>(['toolsets', id], `${TOOLSETS_ENDPOINT}/${id}`, undefined, options);
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
}): UseMutationResult<AxiosResponse<ToolsetResponse>, AxiosError<ErrorResponse>, CreateToolsetRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<ToolsetResponse, CreateToolsetRequest>(() => TOOLSETS_ENDPOINT, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries(['toolsets']);
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
}): UseMutationResult<
  AxiosResponse<ToolsetResponse>,
  AxiosError<ErrorResponse>,
  UpdateToolsetRequest & { id: string }
> {
  const queryClient = useQueryClient();

  return useMutationQuery<ToolsetResponse, UpdateToolsetRequest & { id: string }>(
    ({ id }) => `${TOOLSETS_ENDPOINT}/${id}`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['toolsets']);
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
        queryClient.invalidateQueries(['toolsets']);
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

// ============================================================================
// Query Hooks - Toolset Types
// ============================================================================

/**
 * List all available toolset types (for admin and create form)
 */
export function useToolsetTypes(options?: {
  enabled?: boolean;
}): UseQueryResult<ListToolsetTypesResponse, AxiosError<ErrorResponse>> {
  return useQuery<ListToolsetTypesResponse>(['toolsets', 'types'], TOOLSET_TYPES_ENDPOINT, undefined, options);
}

// ============================================================================
// Admin Mutation Hooks (App-level configuration)
// ============================================================================

/**
 * Enable a toolset type at app level (admin only)
 */
export function useEnableToolsetType(options?: {
  onSuccess?: (response: AppToolsetConfigResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AppToolsetConfigResponse>, AxiosError<ErrorResponse>, { scope: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<AppToolsetConfigResponse, { scope: string }>(
    ({ scope }) => `${TOOLSET_TYPES_ENDPOINT}/${scope}/app-config`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['toolsets']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to enable toolset type';
        options?.onError?.(message);
      },
    },
    { noBody: true }
  );
}

/**
 * Disable a toolset type at app level (admin only)
 */
export function useDisableToolsetType(options?: {
  onSuccess?: (response: AppToolsetConfigResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AppToolsetConfigResponse>, AxiosError<ErrorResponse>, { scope: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<AppToolsetConfigResponse, { scope: string }>(
    ({ scope }) => `${TOOLSET_TYPES_ENDPOINT}/${scope}/app-config`,
    'delete',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['toolsets']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to disable toolset type';
        options?.onError?.(message);
      },
    },
    { noBody: true }
  );
}
