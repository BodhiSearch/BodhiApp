import {
  AccessRequestActionResponse,
  AccessRequestReviewResponse,
  ApproveAccessRequest,
  McpApproval,
  McpServerReviewInfo,
  BodhiErrorResponse,
  RequestedResources,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

import { appAccessRequestKeys, ENDPOINT_ACCESS_REQUESTS } from './constants';

// Re-export types for consumers
export type {
  AccessRequestActionResponse,
  AccessRequestReviewResponse,
  ApproveAccessRequest,
  McpApproval,
  McpServerReviewInfo,
  RequestedResources,
};

// ============================================================================
// Query Hooks
// ============================================================================

/**
 * Fetch access request review data for the review page
 */
export function useGetAppAccessRequestReview(
  id: string | null,
  options?: { enabled?: boolean }
): UseQueryResult<AccessRequestReviewResponse, AxiosError<BodhiErrorResponse>> {
  return useQuery<AccessRequestReviewResponse>(
    appAccessRequestKeys.detail(id ?? ''),
    `${ENDPOINT_ACCESS_REQUESTS}/${id}/review`,
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
  AxiosError<BodhiErrorResponse>,
  { id: string; body: ApproveAccessRequest }
> {
  const queryClient = useQueryClient();
  return useMutationQuery<AccessRequestActionResponse, { id: string; body: ApproveAccessRequest }>(
    ({ id }) => `${ENDPOINT_ACCESS_REQUESTS}/${id}/approve`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries({ queryKey: appAccessRequestKeys.all });
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
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
  onSuccess?: (data: AccessRequestActionResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AccessRequestActionResponse>, AxiosError<BodhiErrorResponse>, { id: string }> {
  const queryClient = useQueryClient();
  return useMutationQuery<AccessRequestActionResponse, { id: string }>(
    ({ id }) => `${ENDPOINT_ACCESS_REQUESTS}/${id}/deny`,
    'post',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries({ queryKey: appAccessRequestKeys.all });
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to deny access request';
        options?.onError?.(message);
      },
    },
    {
      transformBody: () => undefined,
    }
  );
}
