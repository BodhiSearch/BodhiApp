import { BodhiApiError } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';
import {
  useMutation,
  UseMutationOptions,
  UseMutationResult,
  useQueryClient,
  UseQueryOptions,
  UseQueryResult,
  useQuery as useReactQuery,
} from '@tanstack/react-query';

import { BODHI_API_BASE } from '@/hooks/constants';
import apiClient from '@/lib/apiClient';

// Re-export for backward compatibility (test files import from here)
export { BODHI_API_BASE };

// Type alias for compatibility
type ErrorResponse = BodhiApiError;

export function useQuery<T>(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  key: string | readonly any[],
  endpoint: string,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  params?: Record<string, any>,
  options?: Omit<UseQueryOptions<T, AxiosError<ErrorResponse>>, 'queryKey' | 'queryFn'>
): UseQueryResult<T, AxiosError<ErrorResponse>> {
  const queryKey = Array.isArray(key) ? key : [key];
  return useReactQuery<T, AxiosError<ErrorResponse>>({
    queryKey,
    queryFn: async () => {
      const { data } = await apiClient.get<T>(endpoint, {
        params,
        headers: {
          'Content-Type': 'application/json',
        },
      });
      return data;
    },
    ...options,
  });
}

export function useMutationQuery<T, V>(
  endpoint: string | ((variables: V) => string),
  method: 'post' | 'put' | 'delete' = 'post',
  options?: UseMutationOptions<AxiosResponse<T>, AxiosError<ErrorResponse>, V>,
  axiosConfig?: {
    headers?: Record<string, string>;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    transformBody?: (variables: V) => any;
    noBody?: boolean;
  }
): UseMutationResult<AxiosResponse<T>, AxiosError<ErrorResponse>, V> {
  return useMutation<AxiosResponse<T>, AxiosError<ErrorResponse>, V>({
    mutationFn: async (variables) => {
      const _endpoint = typeof endpoint === 'function' ? endpoint(variables) : endpoint;

      // Handle body transformation or no body
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
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
    ...options,
  });
}

// Re-export types for other hooks
export type { UseMutationResult, UseQueryResult, UseMutationOptions, UseQueryOptions } from '@tanstack/react-query';

// Re-export components for ClientProviders and tests
export { QueryClient, QueryClientProvider } from '@tanstack/react-query';

// Re-export hooks for other use cases
export { useMutation } from '@tanstack/react-query';

// Export useQueryClient for consistency
export { useQueryClient } from '@tanstack/react-query';
