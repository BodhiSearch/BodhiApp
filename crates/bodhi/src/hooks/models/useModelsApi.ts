import {
  ApiAliasResponse,
  ApiModelRequest,
  TestPromptRequest,
  TestPromptResponse,
  FetchModelsRequest,
  FetchModelsResponse,
  ApiFormatsResponse,
  BodhiErrorResponse,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationOptions, UseMutationResult, UseQueryOptions, UseQueryResult } from '@/hooks/useQuery';

import {
  apiModelKeys,
  apiFormatKeys,
  modelKeys,
  ENDPOINT_API_MODELS,
  ENDPOINT_API_MODELS_TEST,
  ENDPOINT_API_MODELS_FETCH,
  ENDPOINT_API_MODELS_FORMATS,
} from './constants';

export function useGetApiModel(
  id: string,
  options?: Omit<UseQueryOptions<ApiAliasResponse, AxiosError<BodhiErrorResponse>>, 'queryKey' | 'queryFn'>
): UseQueryResult<ApiAliasResponse, AxiosError<BodhiErrorResponse>> {
  return useQuery<ApiAliasResponse>(apiModelKeys.detail(id), `${ENDPOINT_API_MODELS}/${id}`, undefined, {
    enabled: !!id,
    refetchOnWindowFocus: false,
    staleTime: 5 * 60 * 1000,
    ...options,
  });
}

export function useCreateApiModel(
  options?: UseMutationOptions<AxiosResponse<ApiAliasResponse>, AxiosError<BodhiErrorResponse>, ApiModelRequest>
): UseMutationResult<AxiosResponse<ApiAliasResponse>, AxiosError<BodhiErrorResponse>, ApiModelRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<ApiAliasResponse, ApiModelRequest>(ENDPOINT_API_MODELS, 'post', {
    ...options,
    onSuccess: (data, variables, onMutateResult, context) => {
      queryClient.invalidateQueries({ queryKey: apiModelKeys.all });
      // API models also surface in the aggregate models list.
      queryClient.invalidateQueries({ queryKey: modelKeys.all });
      options?.onSuccess?.(data, variables, onMutateResult, context);
    },
  });
}

export function useUpdateApiModel(
  options?: UseMutationOptions<
    AxiosResponse<ApiAliasResponse>,
    AxiosError<BodhiErrorResponse>,
    { id: string; data: ApiModelRequest }
  >
): UseMutationResult<
  AxiosResponse<ApiAliasResponse>,
  AxiosError<BodhiErrorResponse>,
  { id: string; data: ApiModelRequest }
> {
  const queryClient = useQueryClient();

  return useMutationQuery<ApiAliasResponse, { id: string; data: ApiModelRequest }>(
    ({ id }) => `${ENDPOINT_API_MODELS}/${id}`,
    'put',
    {
      ...options,
      onSuccess: (data, variables, onMutateResult, context) => {
        queryClient.invalidateQueries({ queryKey: apiModelKeys.all });
        queryClient.invalidateQueries({ queryKey: apiModelKeys.detail(variables.id) });
        queryClient.invalidateQueries({ queryKey: modelKeys.all });
        options?.onSuccess?.(data, variables, onMutateResult, context);
      },
    },
    {
      transformBody: ({ data }) => data,
    }
  );
}

export function useDeleteApiModel(
  options?: UseMutationOptions<AxiosResponse<void>, AxiosError<BodhiErrorResponse>, string>
): UseMutationResult<AxiosResponse<void>, AxiosError<BodhiErrorResponse>, string> {
  const queryClient = useQueryClient();

  return useMutationQuery<void, string>(
    (id) => `${ENDPOINT_API_MODELS}/${id}`,
    'delete',
    {
      ...options,
      onSuccess: (data, variables, onMutateResult, context) => {
        queryClient.invalidateQueries({ queryKey: apiModelKeys.all });
        queryClient.removeQueries({ queryKey: apiModelKeys.detail(variables) });
        queryClient.invalidateQueries({ queryKey: modelKeys.all });
        options?.onSuccess?.(data, variables, onMutateResult, context);
      },
    },
    {
      noBody: true,
    }
  );
}

export function useTestApiModel(
  options?: UseMutationOptions<AxiosResponse<TestPromptResponse>, AxiosError<BodhiErrorResponse>, TestPromptRequest>
): UseMutationResult<AxiosResponse<TestPromptResponse>, AxiosError<BodhiErrorResponse>, TestPromptRequest> {
  return useMutationQuery<TestPromptResponse, TestPromptRequest>(ENDPOINT_API_MODELS_TEST, 'post', options);
}

export function useFetchApiModels(
  options?: UseMutationOptions<AxiosResponse<FetchModelsResponse>, AxiosError<BodhiErrorResponse>, FetchModelsRequest>
): UseMutationResult<AxiosResponse<FetchModelsResponse>, AxiosError<BodhiErrorResponse>, FetchModelsRequest> {
  return useMutationQuery<FetchModelsResponse, FetchModelsRequest>(ENDPOINT_API_MODELS_FETCH, 'post', options);
}

export function useListApiFormats(
  options?: Omit<UseQueryOptions<ApiFormatsResponse, AxiosError<BodhiErrorResponse>>, 'queryKey' | 'queryFn'>
): UseQueryResult<ApiFormatsResponse, AxiosError<BodhiErrorResponse>> {
  return useQuery<ApiFormatsResponse>(apiFormatKeys.all, ENDPOINT_API_MODELS_FORMATS, undefined, {
    refetchOnWindowFocus: false,
    staleTime: 10 * 60 * 1000, // formats rarely change
    ...options,
  });
}

export function isApiModel(model: unknown): model is ApiAliasResponse {
  return (
    typeof model === 'object' &&
    model !== null &&
    'has_api_key' in model &&
    'base_url' in model &&
    'api_format' in model
  );
}

export function maskApiKey(apiKey: string): string {
  if (!apiKey || apiKey.length < 10) {
    return '***';
  }
  const firstPart = apiKey.substring(0, 3);
  const lastPart = apiKey.substring(apiKey.length - 6);
  return `${firstPart}...${lastPart}`;
}
