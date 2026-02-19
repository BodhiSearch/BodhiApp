import {
  AccessRequestReviewResponse,
  McpServerReviewInfo,
  OpenAiApiError,
  RequestedResources,
  Toolset,
  ToolTypeReviewInfo,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { BODHI_API_BASE, useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// Re-export types for consumers
export type { AccessRequestReviewResponse, McpServerReviewInfo, RequestedResources, ToolTypeReviewInfo, Toolset };

export interface ToolApprovalItem {
  toolset_type: string;
  status: string;
  instance?: { id: string };
}

export interface McpApprovalItem {
  url: string;
  status: string;
  instance?: { id: string };
}

export interface ApproveAccessRequestBody {
  approved: {
    toolsets: ToolApprovalItem[];
    mcps: McpApprovalItem[];
  };
}

export interface AccessRequestActionResponse {
  status: string;
  flow_type: string;
  redirect_url?: string;
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
  onSuccess?: (data: AccessRequestActionResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<AccessRequestActionResponse>,
  AxiosError<ErrorResponse>,
  { id: string; body: ApproveAccessRequestBody }
> {
  const queryClient = useQueryClient();
  return useMutationQuery<AccessRequestActionResponse, { id: string; body: ApproveAccessRequestBody }>(
    ({ id }) => `${BODHI_API_BASE}/access-requests/${id}/approve`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['app-access-request']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to approve access request';
        options?.onError?.(message);
      },
    },
    {
      transformBody: ({ body }) => body,
      skipCacheInvalidation: true,
    }
  );
}

/**
 * Deny an app access request
 */
export function useDenyAppAccessRequest(options?: {
  onSuccess?: (data: AccessRequestActionResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AccessRequestActionResponse>, AxiosError<ErrorResponse>, { id: string }> {
  const queryClient = useQueryClient();
  return useMutationQuery<AccessRequestActionResponse, { id: string }>(
    ({ id }) => `${BODHI_API_BASE}/access-requests/${id}/deny`,
    'post',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['app-access-request']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to deny access request';
        options?.onError?.(message);
      },
    },
    {
      transformBody: () => undefined,
      skipCacheInvalidation: true,
    }
  );
}
