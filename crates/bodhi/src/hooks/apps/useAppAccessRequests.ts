import {
  AccessRequestActionResponse,
  AccessRequestReviewResponse,
  AppAccessSummary,
  ApproveAccessRequest,
  ListAppAccessResponse,
  McpApproval,
  McpServerReviewInfo,
  BodhiErrorResponse,
  RequestedResources,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

import { appAccessRequestKeys, ENDPOINT_ACCESS_REQUESTS, ENDPOINT_ACCESS_REQUESTS_APPS } from './constants';

export type {
  AccessRequestActionResponse,
  AccessRequestReviewResponse,
  AppAccessSummary,
  ApproveAccessRequest,
  ListAppAccessResponse,
  McpApproval,
  McpServerReviewInfo,
  RequestedResources,
};

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

export function useListAppAccess(options?: {
  enabled?: boolean;
}): UseQueryResult<ListAppAccessResponse, AxiosError<BodhiErrorResponse>> {
  return useQuery<ListAppAccessResponse>(appAccessRequestKeys.list(), ENDPOINT_ACCESS_REQUESTS_APPS, undefined, {
    retry: false,
    ...options,
  });
}

export function useRevokeAppAccess(options?: {
  onSuccess?: (data: AppAccessSummary) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AppAccessSummary>, AxiosError<BodhiErrorResponse>, { id: string }> {
  const queryClient = useQueryClient();
  return useMutationQuery<AppAccessSummary, { id: string }>(
    ({ id }) => `${ENDPOINT_ACCESS_REQUESTS}/${id}/revoke`,
    'post',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries({ queryKey: appAccessRequestKeys.all });
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to revoke app access';
        options?.onError?.(message);
      },
    },
    {
      transformBody: () => undefined,
    }
  );
}

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
