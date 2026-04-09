// External imports
import { Alias, BodhiApiError, PaginatedAliasResponse, UserAliasRequest } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

// Internal imports
import { UseMutationResult, useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';

import { modelKeys, ENDPOINT_MODELS, ENDPOINT_ALIAS } from './constants';

// Type alias
type ErrorResponse = BodhiApiError;

// Model-related hooks

export function useListModels(page: number, pageSize: number, sort: string, sortOrder: string) {
  return useQuery<PaginatedAliasResponse>(modelKeys.list(page, pageSize, sort, sortOrder), ENDPOINT_MODELS, {
    page,
    page_size: pageSize,
    sort,
    sort_order: sortOrder,
  });
}

export function useGetModel(id: string) {
  return useQuery<Alias>(modelKeys.detail(id), `${ENDPOINT_MODELS}/${id}`, undefined, {
    enabled: !!id,
  });
}

export function useCreateModel(options?: {
  onSuccess?: (model: Alias) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<Alias>, AxiosError<ErrorResponse>, UserAliasRequest> {
  const queryClient = useQueryClient();
  return useMutationQuery<Alias, UserAliasRequest>(ENDPOINT_ALIAS, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: modelKeys.all });
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
): UseMutationResult<AxiosResponse<Alias>, AxiosError<ErrorResponse>, UserAliasRequest> {
  const queryClient = useQueryClient();
  return useMutationQuery<Alias, UserAliasRequest>(() => `${ENDPOINT_ALIAS}/${id}`, 'put', {
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: modelKeys.all });
      queryClient.invalidateQueries({ queryKey: modelKeys.detail(id) });
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to update model';
      options?.onError?.(message);
    },
  });
}
