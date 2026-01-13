import { OpenAiApiError, AliasResponse, RefreshResponse, QueueStatusResponse } from '@bodhiapp/ts-client';
import { AxiosError } from 'axios';
import { useQuery as useReactQuery, UseQueryOptions, useQueryClient } from 'react-query';

import apiClient from '@/lib/apiClient';

import { useMutationQuery } from './useQuery';

type ErrorResponse = OpenAiApiError;

export const ENDPOINT_MODELS_REFRESH = '/bodhi/v1/models/refresh';
export const ENDPOINT_QUEUE = '/bodhi/v1/queue';

/**
 * Hook to trigger metadata refresh for all local models
 */
export function useRefreshAllMetadata(options?: {
  onSuccess?: (response: RefreshResponse) => void;
  onError?: (message: string) => void;
}) {
  return useMutationQuery<RefreshResponse, void>(
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
    },
    { noBody: true }
  );
}

/**
 * Hook to trigger metadata refresh for a single model (synchronous)
 */
export function useRefreshSingleMetadata(options?: {
  onSuccess?: (response: AliasResponse) => void;
  onError?: (message: string) => void;
}) {
  const queryClient = useQueryClient();

  return useMutationQuery<AliasResponse, string>(
    (alias) => `/bodhi/v1/models/${encodeURIComponent(alias)}/refresh`,
    'post',
    {
      onSuccess: (response) => {
        // Invalidate models query to refetch with updated metadata
        queryClient.invalidateQueries({ queryKey: ['models'] });
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to refresh metadata';
        options?.onError?.(message);
      },
    },
    { noBody: true }
  );
}

/**
 * Hook to query queue status
 */
export function useQueueStatus(options?: UseQueryOptions<QueueStatusResponse, AxiosError<ErrorResponse>>) {
  return useReactQuery<QueueStatusResponse, AxiosError<ErrorResponse>>(
    ['queue-status'],
    async () => {
      const { data } = await apiClient.get<QueueStatusResponse>(ENDPOINT_QUEUE);
      return data;
    },
    {
      refetchInterval: false, // Don't auto-refetch, let caller control polling
      ...options,
    }
  );
}
