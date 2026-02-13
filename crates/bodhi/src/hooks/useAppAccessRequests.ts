import { OpenAiApiError } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { BODHI_API_BASE, useMutationQuery, useQuery } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// ============================================================================
// Local TypeScript interfaces (matching backend response types)
// ============================================================================

export interface ToolInstanceInfo {
  id: string;
  name: string;
  enabled: boolean;
  has_api_key: boolean;
}

export interface ToolTypeReviewInfo {
  toolset_type: string;
  name: string;
  description: string;
  instances: ToolInstanceInfo[];
}

export interface RequestedResources {
  toolset_types: { toolset_type: string }[];
}

export interface AccessRequestReviewResponse {
  id: string;
  app_client_id: string;
  app_name: string | null;
  app_description: string | null;
  flow_type: string;
  status: string;
  requested: RequestedResources;
  tools_info: ToolTypeReviewInfo[];
}

export interface ToolApprovalItem {
  toolset_type: string;
  status: string;
  instance_id?: string;
}

export interface ApproveAccessRequestBody {
  approved: {
    toolset_types: ToolApprovalItem[];
  };
}

// ============================================================================
// Endpoints
// ============================================================================

export const ENDPOINT_ACCESS_REQUESTS_REVIEW = '/bodhi/v1/access-requests/{id}/review';
export const ENDPOINT_ACCESS_REQUESTS_APPROVE = '/bodhi/v1/access-requests/{id}/approve';
export const ENDPOINT_ACCESS_REQUESTS_DENY = '/bodhi/v1/access-requests/{id}/deny';

// ============================================================================
// Query Hooks
// ============================================================================

/**
 * Fetch access request review data for the review page
 */
export function useAppAccessRequestReview(
  id: string | null,
  options?: { enabled?: boolean }
): UseQueryResult<AccessRequestReviewResponse, AxiosError<ErrorResponse>> {
  return useQuery<AccessRequestReviewResponse>(
    ['app-access-request', 'review', id ?? ''],
    `${BODHI_API_BASE}/access-requests/${id}/review`,
    undefined,
    {
      enabled: !!id,
      retry: false,
      ...options,
    }
  );
}

// ============================================================================
// Mutation Hooks
// ============================================================================

/**
 * Approve an app access request with tool instance selections
 */
export function useApproveAppAccessRequest(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, { id: string; body: ApproveAccessRequestBody }> {
  return useMutationQuery<void, { id: string; body: ApproveAccessRequestBody }>(
    ({ id }) => `${BODHI_API_BASE}/access-requests/${id}/approve`,
    'put',
    {
      onSuccess: () => {
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to approve access request';
        options?.onError?.(message);
      },
    },
    {
      transformBody: ({ body }) => body,
    }
  );
}

/**
 * Deny an app access request
 */
export function useDenyAppAccessRequest(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, { id: string }> {
  return useMutationQuery<void, { id: string }>(
    ({ id }) => `${BODHI_API_BASE}/access-requests/${id}/deny`,
    'post',
    {
      onSuccess: () => {
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to deny access request';
        options?.onError?.(message);
      },
    },
    {
      transformBody: () => undefined,
    }
  );
}
