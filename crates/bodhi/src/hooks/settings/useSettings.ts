import { SettingInfo, BodhiErrorResponse } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { UseMutationResult, UseQueryResult, useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';
import { extractErrorMessage } from '@/lib/errorUtils';

import { settingKeys, ENDPOINT_SETTINGS } from './constants';

export function useListSettings(): UseQueryResult<SettingInfo[], AxiosError<BodhiErrorResponse>> {
  return useQuery<SettingInfo[]>(settingKeys.all, ENDPOINT_SETTINGS);
}

export function useUpdateSetting(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<SettingInfo>,
  AxiosError<BodhiErrorResponse>,
  { key: string; value: string | number | boolean }
> {
  const queryClient = useQueryClient();
  return useMutationQuery<SettingInfo, { key: string; value: string | number | boolean }>(
    (vars) => `${ENDPOINT_SETTINGS}/${vars.key}`,
    'put',
    {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: settingKeys.all });
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        options?.onError?.(extractErrorMessage(error, 'Failed to update setting'));
      },
    }
  );
}

export function useDeleteSetting(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<SettingInfo>, AxiosError<BodhiErrorResponse>, { key: string }> {
  const queryClient = useQueryClient();
  return useMutationQuery<SettingInfo, { key: string }>((vars) => `${ENDPOINT_SETTINGS}/${vars.key}`, 'delete', {
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: settingKeys.all });
      options?.onSuccess?.();
    },
    onError: (error: AxiosError<BodhiErrorResponse>) => {
      options?.onError?.(extractErrorMessage(error, 'Failed to delete setting'));
    },
  });
}
