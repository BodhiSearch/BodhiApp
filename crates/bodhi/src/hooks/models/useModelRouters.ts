import { ModelRouterRequest, ModelRouterResponse, BodhiErrorResponse } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationOptions, UseMutationResult, UseQueryOptions, UseQueryResult } from '@/hooks/useQuery';

import { modelRouterKeys, modelKeys, ENDPOINT_MODEL_ROUTERS } from './constants';

/**
 * Hook to fetch a single model-router by id
 */
export function useGetModelRouter(
  id: string,
  options?: Omit<UseQueryOptions<ModelRouterResponse, AxiosError<BodhiErrorResponse>>, 'queryKey' | 'queryFn'>
): UseQueryResult<ModelRouterResponse, AxiosError<BodhiErrorResponse>> {
  return useQuery<ModelRouterResponse>(modelRouterKeys.detail(id), `${ENDPOINT_MODEL_ROUTERS}/${id}`, undefined, {
    enabled: !!id,
    refetchOnWindowFocus: false,
    staleTime: 5 * 60 * 1000,
    ...options,
  });
}

/**
 * Hook to create a new model-router
 */
export function useCreateModelRouter(
  options?: UseMutationOptions<AxiosResponse<ModelRouterResponse>, AxiosError<BodhiErrorResponse>, ModelRouterRequest>
): UseMutationResult<AxiosResponse<ModelRouterResponse>, AxiosError<BodhiErrorResponse>, ModelRouterRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<ModelRouterResponse, ModelRouterRequest>(ENDPOINT_MODEL_ROUTERS, 'post', {
    ...options,
    onSuccess: (data, variables, onMutateResult, context) => {
      queryClient.invalidateQueries({ queryKey: modelRouterKeys.all });
      // Routers appear in the aggregate models list / chat picker.
      queryClient.invalidateQueries({ queryKey: modelKeys.all });
      options?.onSuccess?.(data, variables, onMutateResult, context);
    },
  });
}

/**
 * Hook to update an existing model-router
 */
export function useUpdateModelRouter(
  options?: UseMutationOptions<
    AxiosResponse<ModelRouterResponse>,
    AxiosError<BodhiErrorResponse>,
    { id: string; data: ModelRouterRequest }
  >
): UseMutationResult<
  AxiosResponse<ModelRouterResponse>,
  AxiosError<BodhiErrorResponse>,
  { id: string; data: ModelRouterRequest }
> {
  const queryClient = useQueryClient();

  return useMutationQuery<ModelRouterResponse, { id: string; data: ModelRouterRequest }>(
    ({ id }) => `${ENDPOINT_MODEL_ROUTERS}/${id}`,
    'put',
    {
      ...options,
      onSuccess: (data, variables, onMutateResult, context) => {
        queryClient.invalidateQueries({ queryKey: modelRouterKeys.all });
        queryClient.invalidateQueries({ queryKey: modelRouterKeys.detail(variables.id) });
        queryClient.invalidateQueries({ queryKey: modelKeys.all });
        options?.onSuccess?.(data, variables, onMutateResult, context);
      },
    },
    {
      transformBody: ({ data }) => data,
    }
  );
}

/**
 * Hook to delete a model-router
 */
export function useDeleteModelRouter(
  options?: UseMutationOptions<AxiosResponse<void>, AxiosError<BodhiErrorResponse>, string>
): UseMutationResult<AxiosResponse<void>, AxiosError<BodhiErrorResponse>, string> {
  const queryClient = useQueryClient();

  return useMutationQuery<void, string>(
    (id) => `${ENDPOINT_MODEL_ROUTERS}/${id}`,
    'delete',
    {
      ...options,
      onSuccess: (data, variables, onMutateResult, context) => {
        queryClient.invalidateQueries({ queryKey: modelRouterKeys.all });
        queryClient.removeQueries({ queryKey: modelRouterKeys.detail(variables) });
        queryClient.invalidateQueries({ queryKey: modelKeys.all });
        options?.onSuccess?.(data, variables, onMutateResult, context);
      },
    },
    {
      noBody: true,
    }
  );
}
