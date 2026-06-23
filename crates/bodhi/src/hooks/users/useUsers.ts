import { useEffect } from 'react';

import { UserInfo, UserInfoEnvelope, UserListResponse, BodhiErrorResponse } from '@bodhiapp/ts-client';
import { useNavigate } from '@tanstack/react-router';
import { AxiosError, AxiosResponse } from 'axios';

import { UseQueryResult, UseMutationResult, useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';
import { ROUTE_LOGIN } from '@/lib/constants';
import { extractErrorMessage } from '@/lib/errorUtils';

import { userKeys, ENDPOINT_USER_INFO, ENDPOINT_USERS } from './constants';

export type AuthenticatedUser = UserInfo & { auth_status: 'logged_in' };

export function useGetUser(options?: { enabled?: boolean }) {
  return useQuery<UserInfoEnvelope | null>(userKeys.current, ENDPOINT_USER_INFO, undefined, {
    retry: false,
    enabled: options?.enabled ?? true,
  });
}

// Redirects to login when the user is not authenticated.
export function useGetAuthenticatedUser(): UseQueryResult<AuthenticatedUser, AxiosError<BodhiErrorResponse>> {
  const navigate = useNavigate();
  const { data: userInfo, isLoading, error, ...queryResult } = useGetUser();

  useEffect(() => {
    if (!isLoading && userInfo?.auth_status !== 'logged_in') {
      navigate({ to: ROUTE_LOGIN });
    }
  }, [userInfo, isLoading, navigate]);

  return {
    ...queryResult,
    data: userInfo?.auth_status === 'logged_in' ? userInfo : undefined,
    isLoading,
    error,
  } as UseQueryResult<AuthenticatedUser, AxiosError<BodhiErrorResponse>>;
}

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

export function useChangeUserRole(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<BodhiErrorResponse>, { userId: string; newRole: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<void, { userId: string; newRole: string }>(
    ({ userId }) => `${ENDPOINT_USERS}/${userId}/role`,
    'put',
    {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: userKeys.all });
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        options?.onError?.(extractErrorMessage(error, 'Failed to change user role'));
      },
    },
    {
      transformBody: ({ newRole }) => ({ role: newRole }),
    }
  );
}

export function useRemoveUser(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<BodhiErrorResponse>, string> {
  const queryClient = useQueryClient();

  return useMutationQuery<void, string>(
    (userId: string) => `${ENDPOINT_USERS}/${userId}`,
    'delete',
    {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: userKeys.all });
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        options?.onError?.(extractErrorMessage(error, 'Failed to remove user'));
      },
    },
    {
      noBody: true,
    }
  );
}
