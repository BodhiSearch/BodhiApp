'use client';

import { useUser } from '@/hooks/useQuery';
import { ROUTE_LOGIN } from '@/lib/constants';
import { UserInfo } from '@bodhiapp/ts-client';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';
import { UseQueryResult } from 'react-query';
import { AxiosError } from 'axios';

export type AuthenticatedUser = UserInfo & { auth_status: 'logged_in' };
type ErrorResponse = { error?: { message?: string } };

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
