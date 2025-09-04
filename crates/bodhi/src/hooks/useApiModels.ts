import apiClient from '@/lib/apiClient';
import {
  ApiModelResponse,
  CreateApiModelRequest,
  UpdateApiModelRequest,
  TestPromptRequest,
  TestPromptResponse,
  FetchModelsRequest,
  FetchModelsResponse,
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
  useQuery as useReactQuery,
} from 'react-query';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// API endpoints
export const ENDPOINT_API_MODELS = '/bodhi/v1/api-models';
export const ENDPOINT_API_MODELS_TEST = '/bodhi/v1/api-models/test';
export const ENDPOINT_API_MODELS_FETCH = '/bodhi/v1/api-models/fetch-models';

/**
 * Hook to fetch a single API model by id
 */
export function useApiModel(
  id: string,
  options?: UseQueryOptions<ApiModelResponse, AxiosError<ErrorResponse>>
): UseQueryResult<ApiModelResponse, AxiosError<ErrorResponse>> {
  return useReactQuery<ApiModelResponse, AxiosError<ErrorResponse>>(
    ['api-models', id],
    async () => {
      const { data } = await apiClient.get<ApiModelResponse>(`${ENDPOINT_API_MODELS}/${id}`);
      return data;
    },
    {
      enabled: !!id,
      refetchOnWindowFocus: false,
      staleTime: 5 * 60 * 1000,
      ...options,
    }
  );
}

/**
 * Hook to create a new API model
 */
export function useCreateApiModel(
  options?: UseMutationOptions<AxiosResponse<ApiModelResponse>, AxiosError<ErrorResponse>, CreateApiModelRequest>
): UseMutationResult<AxiosResponse<ApiModelResponse>, AxiosError<ErrorResponse>, CreateApiModelRequest> {
  const queryClient = useQueryClient();

  return useMutation<AxiosResponse<ApiModelResponse>, AxiosError<ErrorResponse>, CreateApiModelRequest>(
    async (data) => {
      const response = await apiClient.post<ApiModelResponse>(ENDPOINT_API_MODELS, data);
      return response;
    },
    {
      ...options,
      onSuccess: (data, variables, context) => {
        // Invalidate and refetch API models list
        queryClient.invalidateQueries(['api-models']);
        // Also invalidate models list since we'll be showing API models there
        queryClient.invalidateQueries(['models']);
        options?.onSuccess?.(data, variables, context);
      },
    }
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
  return useMutation<AxiosResponse<TestPromptResponse>, AxiosError<ErrorResponse>, TestPromptRequest>(async (data) => {
    const response = await apiClient.post<TestPromptResponse>(ENDPOINT_API_MODELS_TEST, data);
    return response;
  }, options);
}

/**
 * Hook to fetch available models from an API provider
 */
export function useFetchApiModels(
  options?: UseMutationOptions<AxiosResponse<FetchModelsResponse>, AxiosError<ErrorResponse>, FetchModelsRequest>
): UseMutationResult<AxiosResponse<FetchModelsResponse>, AxiosError<ErrorResponse>, FetchModelsRequest> {
  return useMutation<AxiosResponse<FetchModelsResponse>, AxiosError<ErrorResponse>, FetchModelsRequest>(
    async (data) => {
      const response = await apiClient.post<FetchModelsResponse>(ENDPOINT_API_MODELS_FETCH, data);
      return response;
    },
    options
  );
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
    'provider' in model
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
