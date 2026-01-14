import { OpenAiApiError, AliasResponse, RefreshResponse } from '@bodhiapp/ts-client';
import { AxiosError } from 'axios';
import { useQueryClient } from 'react-query';

import { useMutationQuery } from './useQuery';

type ErrorResponse = OpenAiApiError;

export const ENDPOINT_MODELS_REFRESH = '/bodhi/v1/models/refresh';
export const ENDPOINT_QUEUE = '/bodhi/v1/queue';

/**
 * Hook to trigger metadata refresh for all local models (async bulk mode)
 */
export function useRefreshAllMetadata(options?: {
  onSuccess?: (response: RefreshResponse) => void;
  onError?: (message: string) => void;
}) {
  return useMutationQuery<RefreshResponse, { source: 'all' }>(
    ENDPOINT_MODELS_REFRESH,
    'post',
    {
      onSuccess: (response) => {
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to refresh metadata';
        options?.onError?.(message);
      },
    }
  );
}

/**
 * Hook to trigger metadata refresh for a model by GGUF file (sync single mode)
 */
export function useRefreshSingleMetadata(options?: {
  onSuccess?: (response: AliasResponse) => void;
  onError?: (message: string) => void;
}) {
  const queryClient = useQueryClient();

  return useMutationQuery<
    AliasResponse,
    { source: 'model'; repo: string; filename: string; snapshot: string }
  >(
    ENDPOINT_MODELS_REFRESH,
    'post',
    {
      onSuccess: (response) => {
        // Call user callback first
        options?.onSuccess?.(response.data);
        // Delay query invalidation to ensure mutation state updates complete first
        setTimeout(() => {
          queryClient.invalidateQueries({ queryKey: ['models'] });
          queryClient.invalidateQueries({ queryKey: ['modelFiles'] });
        }, 100);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to refresh metadata';
        options?.onError?.(message);
      },
    },
    { skipCacheInvalidation: true }
  );
}
