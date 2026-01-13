// External imports
import { SettingInfo, OpenAiApiError } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

// Internal imports
import { UseMutationResult, UseQueryResult, useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';

// Constants at top
export const BODHI_API_BASE = '/bodhi/v1';
export const ENDPOINT_SETTINGS = `${BODHI_API_BASE}/settings`;
export const ENDPOINT_SETTING_KEY = `${BODHI_API_BASE}/settings/{key}`;

// Type alias
type ErrorResponse = OpenAiApiError;

// Settings hooks
export function useSettings(): UseQueryResult<SettingInfo[], AxiosError<ErrorResponse>> {
  return useQuery<SettingInfo[]>('settings', ENDPOINT_SETTINGS);
}

export function useUpdateSetting(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<SettingInfo>,
  AxiosError<ErrorResponse>,
  { key: string; value: string | number | boolean }
> {
  const queryClient = useQueryClient();
  return useMutationQuery<SettingInfo, { key: string; value: string | number | boolean }>(
    (vars) => `${ENDPOINT_SETTINGS}/${vars.key}`,
    'put',
    {
      onSuccess: () => {
        queryClient.invalidateQueries('settings');
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to update setting';
        options?.onError?.(message);
      },
    }
  );
}

export function useDeleteSetting(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<SettingInfo>, AxiosError<ErrorResponse>, { key: string }> {
  const queryClient = useQueryClient();
  return useMutationQuery<SettingInfo, { key: string }>((vars) => `${ENDPOINT_SETTINGS}/${vars.key}`, 'delete', {
    onSuccess: () => {
      queryClient.invalidateQueries('settings');
      options?.onSuccess?.();
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to delete setting';
      options?.onError?.(message);
    },
  });
}
