import { BODHI_API_BASE, useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { OpenAiApiError, PaginatedUserAccessResponse, Role, UserAccessStatusResponse } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// Endpoint constants
export const ENDPOINT_USER_REQUEST_STATUS = `${BODHI_API_BASE}/user/request-status`;
export const ENDPOINT_USER_REQUEST_ACCESS = `${BODHI_API_BASE}/user/request-access`;
export const ENDPOINT_ACCESS_REQUESTS_PENDING = `${BODHI_API_BASE}/access-requests/pending`;
export const ENDPOINT_ACCESS_REQUESTS = `${BODHI_API_BASE}/access-requests`;
export const ENDPOINT_ACCESS_REQUEST_APPROVE = '/bodhi/v1/access-requests/{id}/approve';
export const ENDPOINT_ACCESS_REQUEST_REJECT = '/bodhi/v1/access-requests/{id}/reject';

const queryKeys = {
  requestStatus: ['access-request', 'status'],
  pendingRequests: (page?: number, pageSize?: number) => [
    'access-request',
    'pending',
    page?.toString() ?? '-1',
    pageSize?.toString() ?? '-1',
  ],
  allRequests: (page?: number, pageSize?: number) => [
    'access-request',
    'all',
    page?.toString() ?? '-1',
    pageSize?.toString() ?? '-1',
  ],
};

// User request status
export function useRequestStatus(): UseQueryResult<UserAccessStatusResponse, AxiosError<ErrorResponse>> {
  return useQuery<UserAccessStatusResponse>(queryKeys.requestStatus, ENDPOINT_USER_REQUEST_STATUS, undefined, {
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
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, void> {
  const queryClient = useQueryClient();
  return useMutationQuery<void, void>(ENDPOINT_USER_REQUEST_ACCESS, 'post', {
    onSuccess: () => {
      queryClient.invalidateQueries(queryKeys.requestStatus);
      options?.onSuccess?.();
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to submit access request';
      options?.onError?.(message);
    },
  });
}

// List pending requests (admin/manager)
export function usePendingRequests(
  page: number = 1,
  pageSize: number = 10
): UseQueryResult<PaginatedUserAccessResponse, AxiosError<ErrorResponse>> {
  return useQuery<PaginatedUserAccessResponse>(
    queryKeys.pendingRequests(page, pageSize),
    ENDPOINT_ACCESS_REQUESTS_PENDING,
    { page, page_size: pageSize },
    {
      retry: 1,
    }
  );
}

// List all requests (admin/manager)
export function useAllRequests(
  page: number = 1,
  pageSize: number = 10
): UseQueryResult<PaginatedUserAccessResponse, AxiosError<ErrorResponse>> {
  return useQuery<PaginatedUserAccessResponse>(
    queryKeys.allRequests(page, pageSize),
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
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, { id: number; role: string }> {
  const queryClient = useQueryClient();
  // Transform from: {id: number; role: string} → endpoint: /access-requests/${id}/approve, body: {role: role as Role}
  return useMutationQuery<void, { id: number; role: string }>(
    ({ id }) => `${ENDPOINT_ACCESS_REQUESTS}/${id}/approve`,
    'post',
    {
      onSuccess: () => {
        queryClient.invalidateQueries(['access-request']);
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to approve request';
        options?.onError?.(message);
      },
    },
    {
      transformBody: ({ role }) => ({ role: role as Role }),
    }
  );
}

// Reject access request
export function useRejectRequest(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, number> {
  const queryClient = useQueryClient();
  // POST with path variables and no meaningful body
  return useMutationQuery<void, number>(
    (id: number) => `${ENDPOINT_ACCESS_REQUESTS}/${id}/reject`,
    'post',
    {
      onSuccess: () => {
        queryClient.invalidateQueries(['access-request']);
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to reject request';
        options?.onError?.(message);
      },
    },
    {
      transformBody: () => undefined, // POST with empty body
    }
  );
}
