import {
  ApiAliasResponse,
  ApiModelRequest,
  TestPromptRequest,
  TestPromptResponse,
  FetchModelsRequest,
  FetchModelsResponse,
  ApiFormatsResponse,
  OpenAiApiError,
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

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

/**
 * Hook to fetch a single API model by id
 */
export function useGetApiModel(
  id: string,
  options?: Omit<UseQueryOptions<ApiAliasResponse, AxiosError<ErrorResponse>>, 'queryKey' | 'queryFn'>
): UseQueryResult<ApiAliasResponse, AxiosError<ErrorResponse>> {
  return useQuery<ApiAliasResponse>(apiModelKeys.detail(id), `${ENDPOINT_API_MODELS}/${id}`, undefined, {
    enabled: !!id,
    refetchOnWindowFocus: false,
    staleTime: 5 * 60 * 1000,
    ...options,
  });
}

/**
 * Hook to create a new API model
 */
export function useCreateApiModel(
  options?: UseMutationOptions<AxiosResponse<ApiAliasResponse>, AxiosError<ErrorResponse>, ApiModelRequest>
): UseMutationResult<AxiosResponse<ApiAliasResponse>, AxiosError<ErrorResponse>, ApiModelRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<ApiAliasResponse, ApiModelRequest>(ENDPOINT_API_MODELS, 'post', {
    ...options,
    onSuccess: (data, variables, onMutateResult, context) => {
      // Invalidate and refetch API models list
      queryClient.invalidateQueries({ queryKey: apiModelKeys.all });
      // Also invalidate models list since we'll be showing API models there
      queryClient.invalidateQueries({ queryKey: modelKeys.all });
      options?.onSuccess?.(data, variables, onMutateResult, context);
    },
  });
}

/**
 * Hook to update an existing API model
 */
export function useUpdateApiModel(
  options?: UseMutationOptions<
    AxiosResponse<ApiAliasResponse>,
    AxiosError<ErrorResponse>,
    { id: string; data: ApiModelRequest }
  >
): UseMutationResult<
  AxiosResponse<ApiAliasResponse>,
  AxiosError<ErrorResponse>,
  { id: string; data: ApiModelRequest }
> {
  const queryClient = useQueryClient();

  // Transform from: {id: string; data: ApiModelRequest} → endpoint: /models/api/${id}, body: data
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

/**
 * Hook to delete an API model
 */
export function useDeleteApiModel(
  options?: UseMutationOptions<AxiosResponse<void>, AxiosError<ErrorResponse>, string>
): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, string> {
  const queryClient = useQueryClient();

  // DELETE with path variables and no body
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

/**
 * Hook to test API model connectivity
 */
export function useTestApiModel(
  options?: UseMutationOptions<AxiosResponse<TestPromptResponse>, AxiosError<ErrorResponse>, TestPromptRequest>
): UseMutationResult<AxiosResponse<TestPromptResponse>, AxiosError<ErrorResponse>, TestPromptRequest> {
  return useMutationQuery<TestPromptResponse, TestPromptRequest>(ENDPOINT_API_MODELS_TEST, 'post', options);
}

/**
 * Hook to fetch available models from an API provider
 */
export function useFetchApiModels(
  options?: UseMutationOptions<AxiosResponse<FetchModelsResponse>, AxiosError<ErrorResponse>, FetchModelsRequest>
): UseMutationResult<AxiosResponse<FetchModelsResponse>, AxiosError<ErrorResponse>, FetchModelsRequest> {
  return useMutationQuery<FetchModelsResponse, FetchModelsRequest>(ENDPOINT_API_MODELS_FETCH, 'post', options);
}

/**
 * Hook to fetch available API formats
 */
export function useListApiFormats(
  options?: Omit<UseQueryOptions<ApiFormatsResponse, AxiosError<ErrorResponse>>, 'queryKey' | 'queryFn'>
): UseQueryResult<ApiFormatsResponse, AxiosError<ErrorResponse>> {
  return useQuery<ApiFormatsResponse>(apiFormatKeys.all, ENDPOINT_API_MODELS_FORMATS, undefined, {
    refetchOnWindowFocus: false,
    staleTime: 10 * 60 * 1000, // 10 minutes (formats don't change often)
    ...options,
  });
}

/**
 * Helper function to check if a model is an API model
 */
export function isApiModel(model: unknown): model is ApiAliasResponse {
  return (
    typeof model === 'object' &&
    model !== null &&
    'has_api_key' in model &&
    'base_url' in model &&
    'api_format' in model
  );
}

/**
 * Helper function to get API key mask for display (first 3, last 6 chars)
 */
export function maskApiKey(apiKey: string): string {
  if (!apiKey || apiKey.length < 10) {
    return '***';
  }
  const firstPart = apiKey.substring(0, 3);
  const lastPart = apiKey.substring(apiKey.length - 6);
  return `${firstPart}...${lastPart}`;
}
