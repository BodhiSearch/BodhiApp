// External imports
import { useQueryClient, UseQueryResult, UseMutationResult, useMutation } from 'react-query';
import { AxiosError, AxiosResponse } from 'axios';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';

// Type imports
import { UserInfo, UserResponse, UserListResponse, OpenAiApiError } from '@bodhiapp/ts-client';

// Internal imports
import { useQuery, useMutationQuery } from '@/hooks/useQuery';
import apiClient from '@/lib/apiClient';
import { ROUTE_LOGIN } from '@/lib/constants';

// Constants at top
export const BODHI_API_BASE = '/bodhi/v1';
export const ENDPOINT_USER_INFO = `${BODHI_API_BASE}/user`;
export const ENDPOINT_USERS = `${BODHI_API_BASE}/users`;
export const ENDPOINT_USER_ROLE = `${BODHI_API_BASE}/users/{user_id}/role`;
export const ENDPOINT_USER_ID = `${BODHI_API_BASE}/users/{user_id}`;

// Types
export type AuthenticatedUser = UserInfo & { auth_status: 'logged_in' };
type ErrorResponse = OpenAiApiError;

// Basic user info hook
export function useUser(options?: { enabled?: boolean }) {
  return useQuery<UserResponse | null>('user', ENDPOINT_USER_INFO, undefined, {
    retry: false,
    enabled: options?.enabled ?? true,
  });
}

// Authenticated user hook with redirect functionality
export function useAuthenticatedUser(): UseQueryResult<AuthenticatedUser, AxiosError<ErrorResponse>> {
  const router = useRouter();
  const { data: userInfo, isLoading, error, ...queryResult } = useUser();

  useEffect(() => {
    if (!isLoading && userInfo?.auth_status !== 'logged_in') {
      router.push(ROUTE_LOGIN);
    }
  }, [userInfo, isLoading, router]);

  // Return the query result but with narrowed type for the data
  return {
    ...queryResult,
    data: userInfo?.auth_status === 'logged_in' ? userInfo : undefined,
    isLoading,
    error,
  } as UseQueryResult<AuthenticatedUser, AxiosError<ErrorResponse>>;
}

// User management hooks - List all users (admin/manager)
export function useAllUsers(
  page: number = 1,
  pageSize: number = 10
): UseQueryResult<UserListResponse, AxiosError<ErrorResponse>> {
  return useQuery<UserListResponse>(
    ['users', 'all', page.toString(), pageSize.toString()],
    ENDPOINT_USERS,
    { page, page_size: pageSize },
    {
      retry: 1,
      refetchOnWindowFocus: false,
    }
  );
}

// Change user role (admin/manager)
export function useChangeUserRole(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, { userId: string; newRole: string }> {
  const queryClient = useQueryClient();

  return useMutation<AxiosResponse<void>, AxiosError<ErrorResponse>, { userId: string; newRole: string }>(
    async ({ userId, newRole }) => {
      // We need to use the traditional mutation approach here since useMutationQuery
      // doesn't support variables in the endpoint path directly and body transformation
      return await apiClient.put(
        `${ENDPOINT_USERS}/${userId}/role`,
        { role: newRole },
        {
          headers: {
            'Content-Type': 'application/json',
          },
        }
      );
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

// Remove user (admin only)
export function useRemoveUser(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, string> {
  const queryClient = useQueryClient();

  return useMutation<AxiosResponse<void>, AxiosError<ErrorResponse>, string>(
    async (userId: string) => {
      // We need to use the traditional mutation approach here since useMutationQuery
      // doesn't support variables in the endpoint path directly
      return await apiClient.delete(`${ENDPOINT_USERS}/${userId}`, {
        headers: {
          'Content-Type': 'application/json',
        },
      });
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
