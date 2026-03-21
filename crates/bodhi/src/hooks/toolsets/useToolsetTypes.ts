import { AppToolsetConfig, ListToolsetTypesResponse, OpenAiApiError } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

import { toolsetKeys, toolsetTypeKeys, TOOLSET_TYPES_ENDPOINT } from './constants';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// ============================================================================
// Query Hooks - Toolset Types
// ============================================================================

/**
 * List all available toolset types (for admin and create form)
 */
export function useListToolsetTypes(options?: {
  enabled?: boolean;
}): UseQueryResult<ListToolsetTypesResponse, AxiosError<ErrorResponse>> {
  return useQuery<ListToolsetTypesResponse>(toolsetTypeKeys.all, TOOLSET_TYPES_ENDPOINT, undefined, options);
}

// ============================================================================
// Admin Mutation Hooks (App-level configuration)
// ============================================================================

/**
 * Enable a toolset type at app level (admin only)
 */
export function useEnableToolsetType(options?: {
  onSuccess?: (response: AppToolsetConfig) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AppToolsetConfig>, AxiosError<ErrorResponse>, { toolset_type: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<AppToolsetConfig, { toolset_type: string }>(
    ({ toolset_type }) => `${TOOLSET_TYPES_ENDPOINT}/${toolset_type}/app-config`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries({ queryKey: toolsetKeys.all });
        queryClient.invalidateQueries({ queryKey: toolsetTypeKeys.all });
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
  onSuccess?: (response: AppToolsetConfig) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AppToolsetConfig>, AxiosError<ErrorResponse>, { toolset_type: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<AppToolsetConfig, { toolset_type: string }>(
    ({ toolset_type }) => `${TOOLSET_TYPES_ENDPOINT}/${toolset_type}/app-config`,
    'delete',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries({ queryKey: toolsetKeys.all });
        queryClient.invalidateQueries({ queryKey: toolsetTypeKeys.all });
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
