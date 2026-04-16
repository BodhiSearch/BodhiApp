import {
  BodhiErrorResponse,
  PaginatedUserAccessResponse,
  ResourceRole,
  UserAccessStatusResponse,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

import {
  accessRequestKeys,
  ENDPOINT_USER_REQUEST_STATUS,
  ENDPOINT_USER_REQUEST_ACCESS,
  ENDPOINT_ACCESS_REQUESTS_PENDING,
  ENDPOINT_ACCESS_REQUESTS,
} from './constants';

// User request status
export function useGetRequestStatus(): UseQueryResult<UserAccessStatusResponse, AxiosError<BodhiErrorResponse>> {
  return useQuery<UserAccessStatusResponse>(accessRequestKeys.status, ENDPOINT_USER_REQUEST_STATUS, undefined, {
    retry: (failureCount, error) => {
      // Don't retry on 404 (no request exists) - this is expected
      if (error?.response?.status === 404) {
        return false;
      }
      return failureCount < 1;
    },
  });
}

// Submit access request
export function useSubmitAccessRequest(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<BodhiErrorResponse>, void> {
  const queryClient = useQueryClient();
  return useMutationQuery<void, void>(ENDPOINT_USER_REQUEST_ACCESS, 'post', {
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: accessRequestKeys.status });
      options?.onSuccess?.();
    },
    onError: (error: AxiosError<BodhiErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to submit access request';
      options?.onError?.(message);
    },
  });
}

// List pending requests (admin/manager)
export function useListPendingRequests(
  page: number = 1,
  pageSize: number = 10
): UseQueryResult<PaginatedUserAccessResponse, AxiosError<BodhiErrorResponse>> {
  return useQuery<PaginatedUserAccessResponse>(
    accessRequestKeys.pending(page, pageSize),
    ENDPOINT_ACCESS_REQUESTS_PENDING,
    { page, page_size: pageSize },
    {
      retry: 1,
    }
  );
}

// List all requests (admin/manager)
export function useListAllRequests(
  page: number = 1,
  pageSize: number = 10
): UseQueryResult<PaginatedUserAccessResponse, AxiosError<BodhiErrorResponse>> {
  return useQuery<PaginatedUserAccessResponse>(
    accessRequestKeys.list(page, pageSize),
    ENDPOINT_ACCESS_REQUESTS,
    { page, page_size: pageSize },
    {
      retry: 1,
    }
  );
}

// Approve access request
export function useApproveRequest(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<BodhiErrorResponse>, { id: string; role: string }> {
  const queryClient = useQueryClient();
  // Transform from: {id: string; role: string} → endpoint: /access-requests/${id}/approve, body: {role: role as Role}
  return useMutationQuery<void, { id: string; role: string }>(
    ({ id }) => `${ENDPOINT_ACCESS_REQUESTS}/${id}/approve`,
    'post',
    {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: accessRequestKeys.all });
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to approve request';
        options?.onError?.(message);
      },
    },
    {
      transformBody: ({ role }) => ({ role: role as ResourceRole }),
    }
  );
}

// Reject access request
export function useRejectRequest(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<BodhiErrorResponse>, string> {
  const queryClient = useQueryClient();
  // POST with path variables and no meaningful body
  return useMutationQuery<void, string>(
    (id: string) => `${ENDPOINT_ACCESS_REQUESTS}/${id}/reject`,
    'post',
    {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: accessRequestKeys.all });
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to reject request';
        options?.onError?.(message);
      },
    },
    {
      transformBody: () => undefined, // POST with empty body
    }
  );
}
