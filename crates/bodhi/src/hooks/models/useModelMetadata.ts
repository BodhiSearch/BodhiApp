import { BodhiErrorResponse, AliasResponse, RefreshResponse } from '@bodhiapp/ts-client';
import { AxiosError } from 'axios';

import { useMutationQuery, useQueryClient } from '@/hooks/useQuery';

import { modelKeys, modelFileKeys, ENDPOINT_MODELS_REFRESH } from './constants';

// Async bulk mode: refresh metadata for all local models.
export function useRefreshAllMetadata(options?: {
  onSuccess?: (response: RefreshResponse) => void;
  onError?: (message: string) => void;
}) {
  const queryClient = useQueryClient();

  return useMutationQuery<RefreshResponse, { source: 'all' }>(ENDPOINT_MODELS_REFRESH, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: modelKeys.all });
      queryClient.invalidateQueries({ queryKey: modelFileKeys.all });
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<BodhiErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to refresh metadata';
      options?.onError?.(message);
    },
  });
}

// Sync single mode: refresh metadata for one model by GGUF file.
export function useRefreshSingleMetadata(options?: {
  onSuccess?: (response: AliasResponse) => void;
  onError?: (message: string) => void;
}) {
  const queryClient = useQueryClient();

  return useMutationQuery<AliasResponse, { source: 'model'; repo: string; filename: string; snapshot: string }>(
    ENDPOINT_MODELS_REFRESH,
    'post',
    {
      onSuccess: (response) => {
        options?.onSuccess?.(response.data);
        // Delay invalidation so mutation state updates settle first.
        setTimeout(() => {
          queryClient.invalidateQueries({ queryKey: modelKeys.all });
          queryClient.invalidateQueries({ queryKey: modelFileKeys.all });
        }, 100);
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to refresh metadata';
        options?.onError?.(message);
      },
    }
  );
}
