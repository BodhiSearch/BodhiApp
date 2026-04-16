// External imports
import { AppInfo, SetupRequest, SetupResponse, BodhiErrorResponse } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

// Internal imports
import { UseMutationResult, useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';
import { userKeys } from '@/hooks/users/constants';

import { appInfoKeys, ENDPOINT_APP_INFO, ENDPOINT_APP_SETUP } from './constants';

export function useGetAppInfo() {
  return useQuery<AppInfo>(appInfoKeys.all, ENDPOINT_APP_INFO);
}

export function useSetupApp(options?: {
  onSuccess?: (appInfo: SetupResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<SetupResponse>, AxiosError<BodhiErrorResponse>, SetupRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<SetupResponse, SetupRequest>(ENDPOINT_APP_SETUP, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: appInfoKeys.all });
      queryClient.invalidateQueries({ queryKey: userKeys.current });
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<BodhiErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to setup app';
      options?.onError?.(message);
    },
  });
}
