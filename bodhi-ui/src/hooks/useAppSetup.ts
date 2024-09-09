import { useMutation, useQuery, useQueryClient } from 'react-query';
import axios, { AxiosError } from 'axios';

export interface AppInfo {
  status: 'setup' | 'ready' | 'resource-admin' | string;
}

const bodhi_url = process.env.NEXT_PUBLIC_BODHI_URL || '';

export function useAppSetup() {
  const queryClient = useQueryClient();

  const getAppInfo = async (): Promise<AppInfo> => {
    const response = await axios.get(`${bodhi_url}/app/info`);
    return response.data;
  };

  const appInfoQuery = useQuery<AppInfo, Error>('appInfo', getAppInfo, {
    retry: false,
    refetchOnWindowFocus: false,
  });

  const setupMutation = useMutation<AppInfo, Error, boolean>(
    async (authz: boolean) => {
      const response = await axios.post(`${bodhi_url}/app/setup`, { authz });
      return response.data;
    },
    {
      onSuccess: () => {
        queryClient.invalidateQueries('appInfo');
      },
    }
  );

  return {
    appInfo: appInfoQuery.data,
    isLoading: appInfoQuery.isLoading,
    isError: appInfoQuery.isError,
    error: appInfoQuery.error,
    refetch: appInfoQuery.refetch,
    setup: setupMutation.mutateAsync,
    isSettingUp: setupMutation.isLoading,
    setupError: setupMutation.error,
  };
}
