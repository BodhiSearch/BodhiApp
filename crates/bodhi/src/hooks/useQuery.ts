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
} from 'react-query';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// backend endpoints
export const BODHI_API_BASE = '/bodhi/v1';

export const ENDPOINT_UI_LOGIN = '/ui/login';

export const API_TOKENS_ENDPOINT = `${BODHI_API_BASE}/tokens`;
// Token endpoints
export const ENDPOINT_TOKEN_ID = `${BODHI_API_BASE}/tokens/{id}`;

export const ENDPOINT_OAI_CHAT_COMPLETIONS = '/v1/chat/completions';

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
  }
): UseMutationResult<AxiosResponse<T>, AxiosError<ErrorResponse>, V> {
  const queryClient = useQueryClient();

  return useMutation<AxiosResponse<T>, AxiosError<ErrorResponse>, V>(
    async (variables) => {
      const _endpoint = typeof endpoint === 'function' ? endpoint(variables) : endpoint;
      const response = await apiClient[method]<T>(_endpoint, variables, {
        headers: {
          'Content-Type': 'application/json',
          ...axiosConfig?.headers,
        },
      });
      return response;
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
