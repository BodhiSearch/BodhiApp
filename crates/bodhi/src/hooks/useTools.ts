import { OpenAiApiError } from '@bodhiapp/ts-client';
import { AxiosResponse, AxiosError } from 'axios';

import { BODHI_API_BASE, useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// ============================================================================
// Types (matching backend DTOs)
// ============================================================================

export interface FunctionDefinition {
  name: string;
  description: string;
  parameters: Record<string, unknown>;
}

export interface ToolDefinition {
  type: string;
  function: FunctionDefinition;
}

export interface UserToolConfigSummary {
  enabled: boolean;
  has_api_key: boolean;
}

export interface ToolListItem {
  type: string;
  function: FunctionDefinition;
  app_enabled: boolean;
  user_config?: UserToolConfigSummary;
}

export interface ListToolsResponse {
  tools: ToolListItem[];
}

export interface UserToolConfig {
  tool_id: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface EnhancedToolConfigResponse {
  tool_id: string;
  app_enabled: boolean;
  config: UserToolConfig;
}

export interface UpdateToolConfigRequest {
  enabled: boolean;
  api_key?: string;
}

export interface AppToolConfig {
  tool_id: string;
  enabled: boolean;
  updated_by: string;
  created_at: string;
  updated_at: string;
}

export interface AppToolConfigResponse {
  tool_id: string;
  enabled: boolean;
  updated_by: string;
  created_at: string;
  updated_at: string;
}

// ============================================================================
// Endpoints
// ============================================================================

export const TOOLS_ENDPOINT = `${BODHI_API_BASE}/tools`;
export const TOOL_CONFIG_ENDPOINT = `${BODHI_API_BASE}/tools/{tool_id}/config`;
export const TOOL_APP_CONFIG_ENDPOINT = `${BODHI_API_BASE}/tools/{tool_id}/app-config`;

// ============================================================================
// Query Hooks
// ============================================================================

/**
 * Fetch all available tools with their status
 */
export function useAvailableTools(options?: {
  enabled?: boolean;
}): UseQueryResult<ListToolsResponse, AxiosError<ErrorResponse>> {
  return useQuery<ListToolsResponse>(['tools', 'available'], TOOLS_ENDPOINT, undefined, options);
}

/**
 * Fetch tool configuration for a specific tool
 * Note: 404 is a valid response meaning "no config exists yet" - we don't retry on 404
 */
export function useToolConfig(
  toolId: string,
  options?: { enabled?: boolean }
): UseQueryResult<EnhancedToolConfigResponse, AxiosError<ErrorResponse>> {
  return useQuery<EnhancedToolConfigResponse>(
    ['tools', 'config', toolId],
    `${BODHI_API_BASE}/tools/${toolId}/config`,
    undefined,
    {
      ...options,
      enabled: options?.enabled !== false && !!toolId,
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

interface UpdateToolConfigRequestWithId extends UpdateToolConfigRequest {
  toolId: string;
}

/**
 * Update tool configuration (enable/disable, API key)
 */
export function useUpdateToolConfig(options?: {
  onSuccess?: (response: EnhancedToolConfigResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<EnhancedToolConfigResponse>,
  AxiosError<ErrorResponse>,
  UpdateToolConfigRequestWithId
> {
  const queryClient = useQueryClient();

  return useMutationQuery<EnhancedToolConfigResponse, UpdateToolConfigRequestWithId>(
    ({ toolId }) => `${BODHI_API_BASE}/tools/${toolId}/config`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['tools']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to update tool configuration';
        options?.onError?.(message);
      },
    },
    {
      transformBody: ({ toolId: _toolId, ...requestBody }) => requestBody,
    }
  );
}

interface DeleteToolConfigRequest {
  toolId: string;
}

/**
 * Delete tool configuration (clears API key)
 */
export function useDeleteToolConfig(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, DeleteToolConfigRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<void, DeleteToolConfigRequest>(
    ({ toolId }) => `${BODHI_API_BASE}/tools/${toolId}/config`,
    'delete',
    {
      onSuccess: () => {
        queryClient.invalidateQueries(['tools']);
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to delete tool configuration';
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

interface SetAppToolEnabledRequest {
  toolId: string;
}

/**
 * Enable a tool at app level (admin only)
 */
export function useSetAppToolEnabled(options?: {
  onSuccess?: (response: AppToolConfigResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AppToolConfigResponse>, AxiosError<ErrorResponse>, SetAppToolEnabledRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<AppToolConfigResponse, SetAppToolEnabledRequest>(
    ({ toolId }) => `${BODHI_API_BASE}/tools/${toolId}/app-config`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['tools']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to enable tool for app';
        options?.onError?.(message);
      },
    },
    {
      noBody: true,
    }
  );
}

/**
 * Disable a tool at app level (admin only)
 */
export function useSetAppToolDisabled(options?: {
  onSuccess?: (response: AppToolConfigResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AppToolConfigResponse>, AxiosError<ErrorResponse>, SetAppToolEnabledRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<AppToolConfigResponse, SetAppToolEnabledRequest>(
    ({ toolId }) => `${BODHI_API_BASE}/tools/${toolId}/app-config`,
    'delete',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['tools']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to disable tool for app';
        options?.onError?.(message);
      },
    },
    {
      noBody: true,
    }
  );
}
