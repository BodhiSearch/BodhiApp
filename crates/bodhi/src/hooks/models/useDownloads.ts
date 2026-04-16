// External imports
import {
  DownloadRequest,
  NewDownloadRequest,
  BodhiErrorResponse,
  PaginatedDownloadResponse,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

// Internal imports
import { UseMutationResult, useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';

import { downloadKeys, ENDPOINT_MODEL_FILES_PULL } from './constants';

export function useListDownloads(page: number, pageSize: number, options?: { enablePolling?: boolean }) {
  return useQuery<PaginatedDownloadResponse>(
    downloadKeys.list(page, pageSize),
    ENDPOINT_MODEL_FILES_PULL,
    { page, page_size: pageSize },
    {
      refetchInterval: options?.enablePolling ? 1000 : false, // Poll every 1 second if enabled
      refetchIntervalInBackground: true, // Continue polling when tab is not focused
    }
  );
}

export function usePullModel(options?: {
  onSuccess?: (response: DownloadRequest) => void;
  onError?: (message: string, code?: string) => void;
}): UseMutationResult<AxiosResponse<DownloadRequest>, AxiosError<BodhiErrorResponse>, NewDownloadRequest> {
  const queryClient = useQueryClient();
  return useMutationQuery<DownloadRequest, NewDownloadRequest>(ENDPOINT_MODEL_FILES_PULL, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: downloadKeys.all });
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<BodhiErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to pull model';
      const code = error?.response?.data?.error?.code ?? undefined;
      options?.onError?.(message, code);
    },
  });
}
