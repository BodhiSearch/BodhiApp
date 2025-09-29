import apiClient from '@/lib/apiClient';
import { OpenAiApiError } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';
import {
  useMutation,
  UseMutationOptions,
  UseMutationResult,
  useQueryClient,
  UseQueryOptions,
  UseQueryResult,
  useQuery as useReactQuery,
  QueryClient,
  QueryClientProvider,
} from 'react-query';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// Shared backend API base
export const BODHI_API_BASE = '/bodhi/v1';

export function useQuery<T>(
  key: string | string[],
  endpoint: string,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  params?: Record<string, any>,
  options?: UseQueryOptions<T, AxiosError<ErrorResponse>>
): UseQueryResult<T, AxiosError<ErrorResponse>> {
  return useReactQuery<T, AxiosError<ErrorResponse>>(
    key,
    async () => {
      const { data } = await apiClient.get<T>(endpoint, {
        params,
        headers: {
          'Content-Type': 'application/json',
        },
      });
      return data;
    },
    options
  );
}

export function useMutationQuery<T, V>(
  endpoint: string | ((variables: V) => string),
  method: 'post' | 'put' | 'delete' = 'post',
  options?: UseMutationOptions<AxiosResponse<T>, AxiosError<ErrorResponse>, V>,
  axiosConfig?: {
    headers?: Record<string, string>;
    skipCacheInvalidation?: boolean;
    transformBody?: (variables: V) => any;
    noBody?: boolean;
  }
): UseMutationResult<AxiosResponse<T>, AxiosError<ErrorResponse>, V> {
  const queryClient = useQueryClient();

  return useMutation<AxiosResponse<T>, AxiosError<ErrorResponse>, V>(
    async (variables) => {
      const _endpoint = typeof endpoint === 'function' ? endpoint(variables) : endpoint;

      // Handle body transformation or no body
      let requestBody: any;
      if (axiosConfig?.noBody) {
        requestBody = undefined;
      } else if (axiosConfig?.transformBody) {
        requestBody = axiosConfig.transformBody(variables);
      } else {
        requestBody = variables;
      }

      // For DELETE with no body, don't pass body parameter
      if (method === 'delete' && (axiosConfig?.noBody || requestBody === undefined)) {
        const response = await apiClient[method]<T>(_endpoint, {
          headers: {
            'Content-Type': 'application/json',
            ...axiosConfig?.headers,
          },
        });
        return response;
      } else {
        const response = await apiClient[method]<T>(_endpoint, requestBody, {
          headers: {
            'Content-Type': 'application/json',
            ...axiosConfig?.headers,
          },
        });
        return response;
      }
    },
    {
      ...options,
      onSuccess: (data, variables, context) => {
        if (!axiosConfig?.skipCacheInvalidation) {
          const _endpoint = typeof endpoint === 'function' ? endpoint(variables) : endpoint;
          queryClient.invalidateQueries(_endpoint);
        }
        if (options?.onSuccess) {
          options.onSuccess(data, variables, context);
        }
      },
    }
  );
}

// Re-export types for other hooks
export type { UseMutationResult, UseQueryResult, UseMutationOptions, UseQueryOptions } from 'react-query';

// Re-export components for ClientProviders and tests
export { QueryClient, QueryClientProvider } from 'react-query';

// Re-export hooks for other use cases
export { useMutation } from 'react-query';

// Export useQueryClient for consistency
export { useQueryClient } from 'react-query';
