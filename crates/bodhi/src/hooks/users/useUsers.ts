// External imports
import { useEffect } from 'react';

import { UserInfo, UserInfoEnvelope, UserResponse, UserListResponse, BodhiErrorResponse } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';
import { useNavigate } from '@tanstack/react-router';

// Internal imports
import { UseQueryResult, UseMutationResult, useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';
import { ROUTE_LOGIN } from '@/lib/constants';

import { userKeys, ENDPOINT_USER_INFO, ENDPOINT_USERS } from './constants';

// Types
export type AuthenticatedUser = UserInfo & { auth_status: 'logged_in' };
// Basic user info hook
export function useGetUser(options?: { enabled?: boolean }) {
  return useQuery<UserInfoEnvelope | null>(userKeys.current, ENDPOINT_USER_INFO, undefined, {
    retry: false,
    enabled: options?.enabled ?? true,
  });
}

// Authenticated user hook with redirect functionality
export function useGetAuthenticatedUser(): UseQueryResult<AuthenticatedUser, AxiosError<BodhiErrorResponse>> {
  const navigate = useNavigate();
  const { data: userInfo, isLoading, error, ...queryResult } = useGetUser();

  useEffect(() => {
    if (!isLoading && userInfo?.auth_status !== 'logged_in') {
      navigate({ to: ROUTE_LOGIN });
    }
  }, [userInfo, isLoading, navigate]);

  // Return the query result but with narrowed type for the data
  return {
    ...queryResult,
    data: userInfo?.auth_status === 'logged_in' ? userInfo : undefined,
    isLoading,
    error,
  } as UseQueryResult<AuthenticatedUser, AxiosError<BodhiErrorResponse>>;
}

// User management hooks - List all users (admin/manager)
export function useListUsers(
  page: number = 1,
  pageSize: number = 10
): UseQueryResult<UserListResponse, AxiosError<BodhiErrorResponse>> {
  return useQuery<UserListResponse>(
    userKeys.list(page, pageSize),
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
}): UseMutationResult<AxiosResponse<void>, AxiosError<BodhiErrorResponse>, { userId: string; newRole: string }> {
  const queryClient = useQueryClient();

  // Transform from: {userId: string; newRole: string} → endpoint: /users/${userId}/role, body: {role: newRole}
  return useMutationQuery<void, { userId: string; newRole: string }>(
    ({ userId }) => `${ENDPOINT_USERS}/${userId}/role`,
    'put',
    {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: userKeys.all });
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to change user role';
        options?.onError?.(message);
      },
    },
    {
      transformBody: ({ newRole }) => ({ role: newRole }),
    }
  );
}

// Remove user (admin only)
export function useRemoveUser(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<BodhiErrorResponse>, string> {
  const queryClient = useQueryClient();

  // DELETE with path variables and no body
  return useMutationQuery<void, string>(
    (userId: string) => `${ENDPOINT_USERS}/${userId}`,
    'delete',
    {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: userKeys.all });
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to remove user';
        options?.onError?.(message);
      },
    },
    {
      noBody: true,
    }
  );
}
