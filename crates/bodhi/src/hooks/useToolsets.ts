import {
  AppToolsetConfigResponse,
  EnhancedToolsetConfigResponse,
  ListToolsetsResponse,
  OpenAiApiError,
  UpdateToolsetConfigRequest,
  UserToolsetConfig,
  UserToolsetConfigSummary,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { BODHI_API_BASE, useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// Re-export types for consumers
export type {
  AppToolsetConfigResponse,
  EnhancedToolsetConfigResponse,
  ListToolsetsResponse,
  UpdateToolsetConfigRequest,
  UserToolsetConfig,
  UserToolsetConfigSummary,
};

// ============================================================================
// Endpoints
// ============================================================================

export const TOOLSETS_ENDPOINT = `${BODHI_API_BASE}/toolsets`;
export const TOOLSET_CONFIG_ENDPOINT = `${BODHI_API_BASE}/toolsets/{toolset_id}/config`;
export const TOOLSET_APP_CONFIG_ENDPOINT = `${BODHI_API_BASE}/toolsets/{toolset_id}/app-config`;

// ============================================================================
// Query Hooks
// ============================================================================

/**
 * Fetch all available toolsets with their status
 */
export function useAvailableToolsets(options?: {
  enabled?: boolean;
}): UseQueryResult<ListToolsetsResponse, AxiosError<ErrorResponse>> {
  return useQuery<ListToolsetsResponse>(['toolsets', 'available'], TOOLSETS_ENDPOINT, undefined, options);
}

/**
 * Fetch toolset configuration for a specific toolset
 * Note: 404 is a valid response meaning "no config exists yet" - we don't retry on 404
 */
export function useToolsetConfig(
  toolsetId: string,
  options?: { enabled?: boolean }
): UseQueryResult<EnhancedToolsetConfigResponse, AxiosError<ErrorResponse>> {
  return useQuery<EnhancedToolsetConfigResponse>(
    ['toolsets', 'config', toolsetId],
    `${BODHI_API_BASE}/toolsets/${toolsetId}/config`,
    undefined,
    {
      ...options,
      enabled: options?.enabled !== false && !!toolsetId,
      // Don't retry on 404 - it means no config exists, which is a valid state
      retry: (failureCount, error) => {
        if (error?.response?.status === 404) return false;
        return failureCount < 3;
      },
    }
  );
}

// ============================================================================
// Mutation Hooks
// ============================================================================

interface UpdateToolsetConfigRequestWithId extends UpdateToolsetConfigRequest {
  toolsetId: string;
}

/**
 * Update toolset configuration (enable/disable, API key)
 */
export function useUpdateToolsetConfig(options?: {
  onSuccess?: (response: EnhancedToolsetConfigResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<EnhancedToolsetConfigResponse>,
  AxiosError<ErrorResponse>,
  UpdateToolsetConfigRequestWithId
> {
  const queryClient = useQueryClient();

  return useMutationQuery<EnhancedToolsetConfigResponse, UpdateToolsetConfigRequestWithId>(
    ({ toolsetId }) => `${BODHI_API_BASE}/toolsets/${toolsetId}/config`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['toolsets']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to update toolset configuration';
        options?.onError?.(message);
      },
    },
    {
      transformBody: ({ toolsetId: _toolsetId, ...requestBody }) => requestBody,
    }
  );
}

interface DeleteToolsetConfigRequest {
  toolsetId: string;
}

/**
 * Delete toolset configuration (clears API key)
 */
export function useDeleteToolsetConfig(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, DeleteToolsetConfigRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<void, DeleteToolsetConfigRequest>(
    ({ toolsetId }) => `${BODHI_API_BASE}/toolsets/${toolsetId}/config`,
    'delete',
    {
      onSuccess: () => {
        queryClient.invalidateQueries(['toolsets']);
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to delete toolset configuration';
        options?.onError?.(message);
      },
    },
    {
      noBody: true,
    }
  );
}

// ============================================================================
// Admin Mutation Hooks (App-level configuration)
// ============================================================================

interface SetAppToolsetEnabledRequest {
  toolsetId: string;
}

/**
 * Enable a toolset at app level (admin only)
 */
export function useSetAppToolsetEnabled(options?: {
  onSuccess?: (response: AppToolsetConfigResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AppToolsetConfigResponse>, AxiosError<ErrorResponse>, SetAppToolsetEnabledRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<AppToolsetConfigResponse, SetAppToolsetEnabledRequest>(
    ({ toolsetId }) => `${BODHI_API_BASE}/toolsets/${toolsetId}/app-config`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['toolsets']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to enable toolset for app';
        options?.onError?.(message);
      },
    },
    {
      noBody: true,
    }
  );
}

/**
 * Disable a toolset at app level (admin only)
 */
export function useSetAppToolsetDisabled(options?: {
  onSuccess?: (response: AppToolsetConfigResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AppToolsetConfigResponse>, AxiosError<ErrorResponse>, SetAppToolsetEnabledRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<AppToolsetConfigResponse, SetAppToolsetEnabledRequest>(
    ({ toolsetId }) => `${BODHI_API_BASE}/toolsets/${toolsetId}/app-config`,
    'delete',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['toolsets']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to disable toolset for app';
        options?.onError?.(message);
      },
    },
    {
      noBody: true,
    }
  );
}
