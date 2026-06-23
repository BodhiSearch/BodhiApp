import { useCallback } from 'react';

import {
  DownloadRequest,
  NewDownloadRequest,
  BodhiErrorResponse,
  PaginatedDownloadResponse,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { UseMutationResult, useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';
import { extractErrorMessage, extractErrorCode } from '@/lib/errorUtils';

import {
  downloadKeys,
  ENDPOINT_MODEL_FILES_PULL,
  ENDPOINT_MODEL_FILES_PULL_ARCHIVE,
  ENDPOINT_MODEL_FILES_PULL_RETRY,
} from './constants';

/** One-minute freshness window; live progress is driven separately by `enablePolling`. */
const DOWNLOADS_STALE_TIME = 60_000;

export function useListDownloads(page: number, pageSize: number, options?: { enablePolling?: boolean }) {
  return useQuery<PaginatedDownloadResponse>(
    downloadKeys.list(page, pageSize),
    ENDPOINT_MODEL_FILES_PULL,
    { page, page_size: pageSize },
    {
      staleTime: DOWNLOADS_STALE_TIME,
      refetchInterval: options?.enablePolling ? 1000 : false,
      refetchIntervalInBackground: true,
    }
  );
}

/** Invalidates the downloads cache; call this when opening the Downloads panel to cache-bust. */
export function useDownloadsRefresh() {
  const queryClient = useQueryClient();
  return useCallback(() => {
    queryClient.invalidateQueries({ queryKey: downloadKeys.all });
  }, [queryClient]);
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
      const message = extractErrorMessage(error, 'Failed to pull model');
      const code = extractErrorCode(error);
      options?.onError?.(message, code);
    },
  });
}

/** Archive a terminal/queued download so it leaves the panel and the list API. */
export function useArchiveDownload(options?: {
  onSuccess?: (response: DownloadRequest) => void;
  onError?: (message: string, code?: string) => void;
}): UseMutationResult<AxiosResponse<DownloadRequest>, AxiosError<BodhiErrorResponse>, { id: string }> {
  const queryClient = useQueryClient();
  return useMutationQuery<DownloadRequest, { id: string }>(
    ({ id }) => ENDPOINT_MODEL_FILES_PULL_ARCHIVE.replace('{id}', id),
    'post',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries({ queryKey: downloadKeys.all });
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = extractErrorMessage(error, 'Failed to archive download');
        const code = extractErrorCode(error);
        options?.onError?.(message, code);
      },
    },
    { noBody: true }
  );
}

/** Retry a failed download; the server resumes from the partial file. */
export function useRetryDownload(options?: {
  onSuccess?: (response: DownloadRequest) => void;
  onError?: (message: string, code?: string) => void;
}): UseMutationResult<AxiosResponse<DownloadRequest>, AxiosError<BodhiErrorResponse>, { id: string }> {
  const queryClient = useQueryClient();
  return useMutationQuery<DownloadRequest, { id: string }>(
    ({ id }) => ENDPOINT_MODEL_FILES_PULL_RETRY.replace('{id}', id),
    'post',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries({ queryKey: downloadKeys.all });
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = extractErrorMessage(error, 'Failed to retry download');
        const code = extractErrorCode(error);
        options?.onError?.(message, code);
      },
    },
    { noBody: true }
  );
}
