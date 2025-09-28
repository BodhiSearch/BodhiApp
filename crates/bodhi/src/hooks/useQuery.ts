import apiClient from '@/lib/apiClient';
import {
  Alias,
  AppInfo,
  CreateAliasRequest,
  DownloadRequest,
  NewDownloadRequest,
  OpenAiApiError,
  PaginatedAliasResponse,
  PaginatedDownloadResponse,
  PaginatedLocalModelResponse,
  RedirectResponse,
  SettingInfo,
  SetupRequest,
  SetupResponse,
  UpdateAliasRequest,
  UserInfo,
  UserResponse,
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

// backend endpoints
export const BODHI_API_BASE = '/bodhi/v1';

export const ENDPOINT_UI_LOGIN = '/ui/login';

export const ENDPOINT_APP_INFO = `${BODHI_API_BASE}/info`;
export const ENDPOINT_APP_SETUP = `${BODHI_API_BASE}/setup`;
export const ENDPOINT_USER_INFO = `${BODHI_API_BASE}/user`;
export const ENDPOINT_LOGOUT = `${BODHI_API_BASE}/logout`;
export const ENDPOINT_AUTH_INITIATE = `${BODHI_API_BASE}/auth/initiate`;
export const ENDPOINT_AUTH_CALLBACK = `${BODHI_API_BASE}/auth/callback`;
export const ENDPOINT_MODEL_FILES = `${BODHI_API_BASE}/modelfiles`;
export const ENDPOINT_MODEL_FILES_PULL = `${BODHI_API_BASE}/modelfiles/pull`;
export const ENDPOINT_MODELS = `${BODHI_API_BASE}/models`;

export const API_TOKENS_ENDPOINT = `${BODHI_API_BASE}/tokens`;
export const ENDPOINT_SETTINGS = `${BODHI_API_BASE}/settings`;

// User endpoints
export const ENDPOINT_USERS = `${BODHI_API_BASE}/users`;
export const ENDPOINT_USER_ROLE = `${BODHI_API_BASE}/users/{user_id}/role`;
export const ENDPOINT_USER_ID = `${BODHI_API_BASE}/users/{user_id}`;

// Model endpoints
export const ENDPOINT_MODEL_ALIAS = `${BODHI_API_BASE}/models/{alias}`;
export const ENDPOINT_MODEL_ID = `${BODHI_API_BASE}/models/{id}`;

// Token endpoints
export const ENDPOINT_TOKEN_ID = `${BODHI_API_BASE}/tokens/{id}`;

// Settings endpoints
export const ENDPOINT_SETTING_KEY = `${BODHI_API_BASE}/settings/{key}`;

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

export function useAppInfo() {
  return useQuery<AppInfo>('appInfo', ENDPOINT_APP_INFO);
}

export function useUser(options?: { enabled?: boolean }) {
  return useQuery<UserResponse | null>('user', ENDPOINT_USER_INFO, undefined, {
    retry: false,
    enabled: options?.enabled ?? true,
  });
}

export function useSetupApp(options?: {
  onSuccess?: (appInfo: SetupResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<SetupResponse>, AxiosError<ErrorResponse>, SetupRequest> {
  const queryClient = useQueryClient();
  return useMutationQuery<SetupResponse, SetupRequest>(ENDPOINT_APP_SETUP, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries('appInfo');
      queryClient.invalidateQueries('user');
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to setup app';
      options?.onError?.(message);
    },
  });
}

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

export function useModel(alias: string) {
  return useQuery<Alias>(['model', alias], `${ENDPOINT_MODELS}/${alias}`, undefined, {
    enabled: !!alias,
  });
}

export function useCreateModel(options?: {
  onSuccess?: (model: Alias) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<Alias>, AxiosError<ErrorResponse>, CreateAliasRequest> {
  const queryClient = useQueryClient();
  return useMutation<AxiosResponse<Alias>, AxiosError<ErrorResponse>, CreateAliasRequest>(
    async (apiData) => {
      const response = await apiClient.post<Alias>(ENDPOINT_MODELS, apiData, {
        headers: {
          'Content-Type': 'application/json',
        },
      });
      return response;
    },
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(ENDPOINT_MODELS);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to create model';
        options?.onError?.(message);
      },
    }
  );
}

export function useUpdateModel(
  alias: string,
  options?: {
    onSuccess?: (model: Alias) => void;
    onError?: (message: string) => void;
  }
): UseMutationResult<AxiosResponse<Alias>, AxiosError<ErrorResponse>, UpdateAliasRequest> {
  const queryClient = useQueryClient();
  return useMutation<AxiosResponse<Alias>, AxiosError<ErrorResponse>, UpdateAliasRequest>(
    async (apiData) => {
      const response = await apiClient.put<Alias>(`${ENDPOINT_MODELS}/${alias}`, apiData, {
        headers: {
          'Content-Type': 'application/json',
        },
      });
      return response;
    },
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['model', alias]);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to update model';
        options?.onError?.(message);
      },
    }
  );
}

export function useLogout(
  options?: UseMutationOptions<AxiosResponse<RedirectResponse>, AxiosError<ErrorResponse>, void, unknown>
): UseMutationResult<AxiosResponse<RedirectResponse>, AxiosError<ErrorResponse>, void, unknown> {
  const queryClient = useQueryClient();
  return useMutationQuery<RedirectResponse, void>(ENDPOINT_LOGOUT, 'post', {
    ...options,
    onSuccess: (data, variables, context) => {
      queryClient.invalidateQueries();
      if (options?.onSuccess) {
        options.onSuccess(data, variables, context);
      }
    },
  });
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

export function useSettings(): UseQueryResult<SettingInfo[], AxiosError<ErrorResponse>> {
  return useQuery<SettingInfo[]>('settings', ENDPOINT_SETTINGS);
}

export function useUpdateSetting(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<SettingInfo>,
  AxiosError<ErrorResponse>,
  { key: string; value: string | number | boolean }
> {
  const queryClient = useQueryClient();
  return useMutationQuery<SettingInfo, { key: string; value: string | number | boolean }>(
    (vars) => `${ENDPOINT_SETTINGS}/${vars.key}`,
    'put',
    {
      onSuccess: () => {
        queryClient.invalidateQueries('settings');
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to update setting';
        options?.onError?.(message);
      },
    }
  );
}

export function useDeleteSetting(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<SettingInfo>, AxiosError<ErrorResponse>, { key: string }> {
  const queryClient = useQueryClient();
  return useMutationQuery<SettingInfo, { key: string }>((vars) => `${ENDPOINT_SETTINGS}/${vars.key}`, 'delete', {
    onSuccess: () => {
      queryClient.invalidateQueries('settings');
      options?.onSuccess?.();
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to delete setting';
      options?.onError?.(message);
    },
  });
}
