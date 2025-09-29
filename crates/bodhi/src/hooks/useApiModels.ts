import { useQuery, useMutationQuery } from '@/hooks/useQuery';
import apiClient from '@/lib/apiClient';
import {
  ApiModelResponse,
  CreateApiModelRequest,
  UpdateApiModelRequest,
  TestPromptRequest,
  TestPromptResponse,
  FetchModelsRequest,
  FetchModelsResponse,
  ApiFormatsResponse,
  OpenAiApiError,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';
import {
  useMutation,
  UseMutationOptions,
  UseMutationResult,
  useQueryClient,
  UseQueryOptions,
  UseQueryResult,
} from 'react-query';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// API endpoints
export const ENDPOINT_API_MODELS = '/bodhi/v1/api-models';
export const ENDPOINT_API_MODEL_ID = '/bodhi/v1/api-models/{id}';
export const ENDPOINT_API_MODELS_TEST = '/bodhi/v1/api-models/test';
export const ENDPOINT_API_MODELS_FETCH = '/bodhi/v1/api-models/fetch-models';
export const ENDPOINT_API_MODELS_FORMATS = '/bodhi/v1/api-models/api-formats';
/**
 * Hook to fetch a single API model by id
 */
export function useApiModel(
  id: string,
  options?: UseQueryOptions<ApiModelResponse, AxiosError<ErrorResponse>>
): UseQueryResult<ApiModelResponse, AxiosError<ErrorResponse>> {
  return useQuery<ApiModelResponse>(['api-models', id], `${ENDPOINT_API_MODELS}/${id}`, undefined, {
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
  options?: UseMutationOptions<AxiosResponse<ApiModelResponse>, AxiosError<ErrorResponse>, CreateApiModelRequest>
): UseMutationResult<AxiosResponse<ApiModelResponse>, AxiosError<ErrorResponse>, CreateApiModelRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<ApiModelResponse, CreateApiModelRequest>(
    ENDPOINT_API_MODELS,
    'post',
    {
      ...options,
      onSuccess: (data, variables, context) => {
        // Invalidate and refetch API models list
        queryClient.invalidateQueries(['api-models']);
        // Also invalidate models list since we'll be showing API models there
        queryClient.invalidateQueries(['models']);
        options?.onSuccess?.(data, variables, context);
      },
    },
    { skipCacheInvalidation: true }
  );
}

/**
 * Hook to update an existing API model
 */
export function useUpdateApiModel(
  options?: UseMutationOptions<
    AxiosResponse<ApiModelResponse>,
    AxiosError<ErrorResponse>,
    { id: string; data: UpdateApiModelRequest }
  >
): UseMutationResult<
  AxiosResponse<ApiModelResponse>,
  AxiosError<ErrorResponse>,
  { id: string; data: UpdateApiModelRequest }
> {
  const queryClient = useQueryClient();

  // Using traditional useMutation for complex case with path variables and body transformation
  // Variables: { id: string; data: UpdateApiModelRequest } need to be split into URL path and request body
  return useMutation<
    AxiosResponse<ApiModelResponse>,
    AxiosError<ErrorResponse>,
    { id: string; data: UpdateApiModelRequest }
  >(
    async ({ id, data }) => {
      const response = await apiClient.put<ApiModelResponse>(`${ENDPOINT_API_MODELS}/${id}`, data);
      return response;
    },
    {
      ...options,
      onSuccess: (data, variables, context) => {
        // Invalidate and refetch API models list
        queryClient.invalidateQueries(['api-models']);
        // Invalidate specific API model
        queryClient.invalidateQueries(['api-models', variables.id]);
        // Also invalidate models list
        queryClient.invalidateQueries(['models']);
        options?.onSuccess?.(data, variables, context);
      },
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

  // Using traditional useMutation for DELETE with path variable and no request body
  // Variable is just the ID string which needs to be in the URL path, not request body
  return useMutation<AxiosResponse<void>, AxiosError<ErrorResponse>, string>(
    async (id) => {
      const response = await apiClient.delete<void>(`${ENDPOINT_API_MODELS}/${id}`);
      return response;
    },
    {
      ...options,
      onSuccess: (data, variables, context) => {
        // Invalidate and refetch API models list
        queryClient.invalidateQueries(['api-models']);
        // Remove specific API model from cache
        queryClient.removeQueries(['api-models', variables]);
        // Also invalidate models list
        queryClient.invalidateQueries(['models']);
        options?.onSuccess?.(data, variables, context);
      },
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
export function useApiFormats(
  options?: UseQueryOptions<ApiFormatsResponse, AxiosError<ErrorResponse>>
): UseQueryResult<ApiFormatsResponse, AxiosError<ErrorResponse>> {
  return useQuery<ApiFormatsResponse>(['api-formats'], ENDPOINT_API_MODELS_FORMATS, undefined, {
    refetchOnWindowFocus: false,
    staleTime: 10 * 60 * 1000, // 10 minutes (formats don't change often)
    ...options,
  });
}

/**
 * Helper function to check if a model is an API model
 */
export function isApiModel(model: unknown): model is ApiModelResponse {
  return (
    typeof model === 'object' &&
    model !== null &&
    'api_key_masked' in model &&
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
