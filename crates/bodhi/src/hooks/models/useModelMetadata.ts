import { OpenAiApiError, AliasResponse, RefreshResponse } from '@bodhiapp/ts-client';
import { AxiosError } from 'axios';

import { useMutationQuery, useQueryClient } from '@/hooks/useQuery';

import { modelKeys, modelFileKeys, ENDPOINT_MODELS_REFRESH } from './constants';

type ErrorResponse = OpenAiApiError;

/**
 * Hook to trigger metadata refresh for all local models (async bulk mode)
 */
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
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to refresh metadata';
      options?.onError?.(message);
    },
  });
}

/**
 * Hook to trigger metadata refresh for a model by GGUF file (sync single mode)
 */
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
        // Call user callback first
        options?.onSuccess?.(response.data);
        // Delay query invalidation to ensure mutation state updates complete first
        setTimeout(() => {
          queryClient.invalidateQueries({ queryKey: modelKeys.all });
          queryClient.invalidateQueries({ queryKey: modelFileKeys.all });
        }, 100);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to refresh metadata';
        options?.onError?.(message);
      },
    }
  );
}
