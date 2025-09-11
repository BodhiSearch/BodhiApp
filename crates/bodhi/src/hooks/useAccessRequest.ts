import { BODHI_API_BASE, useMutationQuery, useQuery } from '@/hooks/useQuery';
import apiClient from '@/lib/apiClient';
import {
  ApproveUserAccessRequest,
  OpenAiApiError,
  PaginatedUserAccessResponse,
  Role,
  UserAccessRequest,
  UserAccessStatusResponse,
  UserInfo,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';
import { useMutation, UseMutationResult, useQueryClient, UseQueryResult } from 'react-query';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// Endpoint constants
export const ENDPOINT_USER_REQUEST_STATUS = `${BODHI_API_BASE}/user/request-status`;
export const ENDPOINT_USER_REQUEST_ACCESS = `${BODHI_API_BASE}/user/request-access`;
export const ENDPOINT_ACCESS_REQUESTS_PENDING = `${BODHI_API_BASE}/access-requests/pending`;
export const ENDPOINT_ACCESS_REQUESTS = `${BODHI_API_BASE}/access-requests`;

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
  users: (page?: number, pageSize?: number) => ['users', 'all', page?.toString() ?? '-1', pageSize?.toString() ?? '-1'],
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
  return useMutation<AxiosResponse<void>, AxiosError<ErrorResponse>, { id: number; role: string }>(
    async ({ id, role }) => {
      const request: ApproveUserAccessRequest = { role: role as Role };
      // We need to use the traditional mutation approach here since useMutationQuery
      // doesn't support variables in the endpoint path directly
      const response = await apiClient.post<void>(`${ENDPOINT_ACCESS_REQUESTS}/${id}/approve`, request, {
        headers: {
          'Content-Type': 'application/json',
        },
      });
      return response;
    },
    {
      onSuccess: () => {
        queryClient.invalidateQueries(['access-request']);
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to approve request';
        options?.onError?.(message);
      },
    }
  );
}

// Reject access request
export function useRejectRequest(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, number> {
  const queryClient = useQueryClient();
  return useMutation<AxiosResponse<void>, AxiosError<ErrorResponse>, number>(
    async (id: number) => {
      const response = await apiClient.post<void>(`${ENDPOINT_ACCESS_REQUESTS}/${id}/reject`, undefined, {
        headers: {
          'Content-Type': 'application/json',
        },
      });
      return response;
    },
    {
      onSuccess: () => {
        queryClient.invalidateQueries(['access-request']);
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to reject request';
        options?.onError?.(message);
      },
    }
  );
}

// List all users (admin/manager) - TODO: Replace with actual users endpoint when available
export function useAllUsers(
  page: number = 1,
  pageSize: number = 10
): UseQueryResult<{ users: UserInfo[]; total: number; page: number; page_size: number }, AxiosError<ErrorResponse>> {
  // TODO: Replace with actual users endpoint when available
  // For now, use useQuery with a mock endpoint that returns empty data
  return useQuery<{ users: UserInfo[]; total: number; page: number; page_size: number }>(
    queryKeys.users(page, pageSize),
    '/users', // Mock endpoint - will fail but maintains structure
    { page, page_size: pageSize },
    {
      retry: 0,
      enabled: false, // Disabled since endpoint doesn't exist yet
      initialData: {
        users: [],
        total: 0,
        page,
        page_size: pageSize,
      },
    }
  );
}

// Change user role (admin/manager) - TODO: Implement when user management API is available
export function useChangeUserRole(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, { userId: string; newRole: string }> {
  const queryClient = useQueryClient();

  return useMutation<AxiosResponse<void>, AxiosError<ErrorResponse>, { userId: string; newRole: string }>(
    async ({ userId, newRole }) => {
      // TODO: Implement actual API call when user management endpoint is available
      console.log(`Would change user ${userId} to role ${newRole}`);
      throw new Error('User role management not yet implemented');
    },
    {
      onSuccess: () => {
        queryClient.invalidateQueries(['users']);
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to change user role';
        options?.onError?.(message);
      },
    }
  );
}

// Remove user (admin/manager) - TODO: Implement when user management API is available
export function useRemoveUser(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, string> {
  const queryClient = useQueryClient();

  return useMutation<AxiosResponse<void>, AxiosError<ErrorResponse>, string>(
    async (userId: string) => {
      // TODO: Implement actual API call when user management endpoint is available
      console.log(`Would remove user ${userId}`);
      throw new Error('User removal not yet implemented');
    },
    {
      onSuccess: () => {
        queryClient.invalidateQueries(['users']);
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to remove user';
        options?.onError?.(message);
      },
    }
  );
}
