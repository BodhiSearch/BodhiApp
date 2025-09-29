// External imports
import { UseMutationResult } from '@/hooks/useQuery';
import { AxiosError, AxiosResponse } from 'axios';

// Type imports
import { AppInfo, SetupRequest, SetupResponse, OpenAiApiError } from '@bodhiapp/ts-client';

// Internal imports
import { useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';

// Constants
export const ENDPOINT_APP_INFO = '/bodhi/v1/info';
export const ENDPOINT_APP_SETUP = '/bodhi/v1/setup';

// Type alias
type ErrorResponse = OpenAiApiError;

export function useAppInfo() {
  return useQuery<AppInfo>('appInfo', ENDPOINT_APP_INFO);
}

export function useSetupApp(options?: {
  onSuccess?: (appInfo: SetupResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<SetupResponse>, AxiosError<ErrorResponse>, SetupRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<SetupResponse, SetupRequest>(ENDPOINT_APP_SETUP, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries('appInfo');
      queryClient.invalidateQueries('user');
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to setup app';
      options?.onError?.(message);
    },
  });
}
