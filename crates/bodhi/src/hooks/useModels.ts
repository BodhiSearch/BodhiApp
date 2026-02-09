// External imports
import {
  Alias,
  CreateAliasRequest,
  DownloadRequest,
  NewDownloadRequest,
  OpenAiApiError,
  PaginatedAliasResponse,
  PaginatedDownloadResponse,
  PaginatedLocalModelResponse,
  UpdateAliasRequest,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

// Internal imports
import { UseMutationResult, useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';

// Constants at top
export const BODHI_API_BASE = '/bodhi/v1';
export const ENDPOINT_MODEL_FILES = `${BODHI_API_BASE}/modelfiles`;
export const ENDPOINT_MODEL_FILES_PULL = `${BODHI_API_BASE}/modelfiles/pull`;
export const ENDPOINT_MODELS = `${BODHI_API_BASE}/models`;
export const ENDPOINT_MODEL_ID = `${BODHI_API_BASE}/models/{id}`;

// Type alias
type ErrorResponse = OpenAiApiError;

// Model-related hooks

export function useModelFiles(page?: number, pageSize?: number, sort: string = 'repo', sortOrder: string = 'asc') {
  return useQuery<PaginatedLocalModelResponse>(
    ['modelFiles', page?.toString() ?? '-1', pageSize?.toString() ?? '-1', sort, sortOrder],
    ENDPOINT_MODEL_FILES,
    { page, page_size: pageSize, sort, sort_order: sortOrder }
  );
}

export function useModels(page: number, pageSize: number, sort: string, sortOrder: string) {
  return useQuery<PaginatedAliasResponse>(
    ['models', page.toString(), pageSize.toString(), sort, sortOrder],
    ENDPOINT_MODELS,
    { page, page_size: pageSize, sort, sort_order: sortOrder }
  );
}

export function useModel(id: string) {
  return useQuery<Alias>(['model', id], `${ENDPOINT_MODELS}/${id}`, undefined, {
    enabled: !!id,
  });
}

export function useCreateModel(options?: {
  onSuccess?: (model: Alias) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<Alias>, AxiosError<ErrorResponse>, CreateAliasRequest> {
  const queryClient = useQueryClient();
  return useMutationQuery<Alias, CreateAliasRequest>(ENDPOINT_MODELS, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries(ENDPOINT_MODELS);
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to create model';
      options?.onError?.(message);
    },
  });
}

export function useUpdateModel(
  id: string,
  options?: {
    onSuccess?: (model: Alias) => void;
    onError?: (message: string) => void;
  }
): UseMutationResult<AxiosResponse<Alias>, AxiosError<ErrorResponse>, UpdateAliasRequest> {
  const queryClient = useQueryClient();
  return useMutationQuery<Alias, UpdateAliasRequest>(
    () => `${ENDPOINT_MODELS}/${id}`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['model', id]);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to update model';
        options?.onError?.(message);
      },
    },
    { skipCacheInvalidation: true }
  );
}

export function useDownloads(page: number, pageSize: number, options?: { enablePolling?: boolean }) {
  return useQuery<PaginatedDownloadResponse>(
    ['downloads', page.toString(), pageSize.toString()],
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
}): UseMutationResult<AxiosResponse<DownloadRequest>, AxiosError<ErrorResponse>, NewDownloadRequest> {
  const queryClient = useQueryClient();
  return useMutationQuery<DownloadRequest, NewDownloadRequest>(ENDPOINT_MODEL_FILES_PULL, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries('downloads');
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to pull model';
      const code = error?.response?.data?.error?.code ?? undefined;
      options?.onError?.(message, code);
    },
  });
}
