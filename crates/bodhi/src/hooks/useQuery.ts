import apiClient from '@/lib/apiClient';
import { AliasFormData } from '@/schemas/alias';
import {
  DownloadRequest,
  ListDownloadsResponse,
  PullModelRequest,
} from '@/types/api';
import {
  AppInfo,
  ErrorResponse,
  FeaturedModel,
  Model,
  ModelFile,
  Setting,
  UserInfo,
} from '@/types/models';
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

// backend endpoints
export const ENDPOINT_APP_LOGIN = '/app/login';

export const BODHI_API_BASE = '/bodhi/v1';

export const ENDPOINT_APP_INFO = `${BODHI_API_BASE}/info`;
export const ENDPOINT_APP_SETUP = `${BODHI_API_BASE}/setup`;
export const ENDPOINT_USER_INFO = `${BODHI_API_BASE}/user`;
export const ENDPOINT_LOGOUT = `${BODHI_API_BASE}/logout`;
export const ENDPOINT_MODEL_FILES = `${BODHI_API_BASE}/modelfiles`;
export const ENDPOINT_MODEL_FILES_PULL = `${BODHI_API_BASE}/modelfiles/pull`;
export const ENDPOINT_MODELS = `${BODHI_API_BASE}/models`;
export const ENDPOINT_CHAT_TEMPLATES = `${BODHI_API_BASE}/chat_templates`;
export const API_TOKENS_ENDPOINT = `${BODHI_API_BASE}/tokens`;
export const ENDPOINT_SETTINGS = `${BODHI_API_BASE}/settings`;

export const ENDPOINT_OAI_CHAT_COMPLETIONS = '/v1/chat/completions';

type PagedApiResponse<T> = {
  data: T;
  total?: number;
  page?: number;
  page_size?: number;
};

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
  options?: UseMutationOptions<AxiosResponse<T>, AxiosError<ErrorResponse>, V>
): UseMutationResult<AxiosResponse<T>, AxiosError<ErrorResponse>, V> {
  const queryClient = useQueryClient();

  return useMutation<AxiosResponse<T>, AxiosError<ErrorResponse>, V>(
    async (variables) => {
      const _endpoint =
        typeof endpoint === 'function' ? endpoint(variables) : endpoint;
      const response = await apiClient[method]<T>(_endpoint, variables, {
        headers: {
          'Content-Type': 'application/json',
        },
        validateStatus: (status) => status >= 200 && status < 400,
      });
      return response;
    },
    {
      ...options,
      onSuccess: (data, variables, context) => {
        const _endpoint =
          typeof endpoint === 'function' ? endpoint(variables) : endpoint;
        queryClient.invalidateQueries(_endpoint);
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
  return useQuery<UserInfo | null>('user', ENDPOINT_USER_INFO, undefined, {
    retry: false,
    enabled: options?.enabled ?? true,
  });
}

type SetupRequest = {
  authz: boolean;
};

export function useSetupApp(options?: {
  onSuccess?: (appInfo: AppInfo) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<AppInfo>,
  AxiosError<ErrorResponse>,
  SetupRequest
> {
  const queryClient = useQueryClient();
  return useMutationQuery<AppInfo, SetupRequest>(ENDPOINT_APP_SETUP, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries('appInfo');
      queryClient.invalidateQueries('user');
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message =
        error?.response?.data?.error?.message || 'Failed to setup app';
      options?.onError?.(message);
    },
  });
}

export function useModelFiles(
  page?: number,
  pageSize?: number,
  sort: string = 'repo',
  sortOrder: string = 'asc'
) {
  return useQuery<PagedApiResponse<ModelFile[]>>(
    [
      'modelFiles',
      page?.toString() ?? '-1',
      pageSize?.toString() ?? '-1',
      sort,
      sortOrder,
    ],
    ENDPOINT_MODEL_FILES,
    { page, page_size: pageSize, sort, sort_order: sortOrder }
  );
}

export function useModels(
  page: number,
  pageSize: number,
  sort: string,
  sortOrder: string
) {
  return useQuery<PagedApiResponse<Model[]>>(
    ['models', page.toString(), pageSize.toString(), sort, sortOrder],
    ENDPOINT_MODELS,
    { page, page_size: pageSize, sort, sort_order: sortOrder }
  );
}

export function useModel(alias: string) {
  return useQuery<Model>(
    ['model', alias],
    `${ENDPOINT_MODELS}/${alias}`,
    undefined,
    {
      enabled: !!alias,
    }
  );
}

export function useCreateModel(options?: {
  onSuccess?: (model: Model) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<Model>,
  AxiosError<ErrorResponse>,
  AliasFormData
> {
  const queryClient = useQueryClient();
  return useMutationQuery<Model, AliasFormData>(ENDPOINT_MODELS, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries(ENDPOINT_MODELS);
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message =
        error?.response?.data?.error?.message || 'Failed to create model';
      options?.onError?.(message);
    },
  });
}

export function useUpdateModel(
  alias: string,
  options?: {
    onSuccess?: (model: Model) => void;
    onError?: (message: string) => void;
  }
): UseMutationResult<
  AxiosResponse<Model>,
  AxiosError<ErrorResponse>,
  AliasFormData
> {
  const queryClient = useQueryClient();
  return useMutationQuery<Model, AliasFormData>(
    `${ENDPOINT_MODELS}/${alias}`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['model', alias]);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message =
          error?.response?.data?.error?.message || 'Failed to update model';
        options?.onError?.(message);
      },
    }
  );
}

export function useChatTemplates() {
  return useQuery<string[]>('chatTemplates', ENDPOINT_CHAT_TEMPLATES);
}

export function useFeaturedModels() {
  return useQuery<FeaturedModel[]>(
    'featuredModels',
    'https://api.getbodhi.app/featured-models'
  );
}

export function useLogout(
  options?: UseMutationOptions<
    AxiosResponse<void>,
    AxiosError<ErrorResponse>,
    void,
    unknown
  >
): UseMutationResult<
  AxiosResponse<void>,
  AxiosError<ErrorResponse>,
  void,
  unknown
> {
  const queryClient = useQueryClient();
  return useMutationQuery<void, void>(ENDPOINT_LOGOUT, 'post', {
    ...options,
    onSuccess: (data, variables, context) => {
      queryClient.invalidateQueries();
      if (options?.onSuccess) {
        options.onSuccess(data, variables, context);
      }
    },
  });
}

export function useDownloads(page: number, pageSize: number) {
  return useQuery<ListDownloadsResponse>(
    ['downloads', page.toString(), pageSize.toString()],
    ENDPOINT_MODEL_FILES_PULL,
    { page, page_size: pageSize }
  );
}

export function usePullModel(options?: {
  onSuccess?: (response: DownloadRequest) => void;
  onError?: (message: string, code?: string) => void;
}): UseMutationResult<
  AxiosResponse<DownloadRequest>,
  AxiosError<ErrorResponse>,
  PullModelRequest
> {
  const queryClient = useQueryClient();
  return useMutationQuery<DownloadRequest, PullModelRequest>(
    ENDPOINT_MODEL_FILES_PULL,
    'post',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries('downloads');
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message =
          error?.response?.data?.error?.message || 'Failed to pull model';
        const code = error?.response?.data?.error?.code;
        options?.onError?.(message, code);
      },
    }
  );
}

export function useSettings(): UseQueryResult<
  Setting[],
  AxiosError<ErrorResponse>
> {
  return useQuery<Setting[]>('settings', ENDPOINT_SETTINGS);
}

export function useUpdateSetting(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<Setting>,
  AxiosError<ErrorResponse>,
  { key: string; value: string | number | boolean }
> {
  const queryClient = useQueryClient();
  return useMutationQuery<
    Setting,
    { key: string; value: string | number | boolean }
  >((vars) => `${ENDPOINT_SETTINGS}/${vars.key}`, 'put', {
    onSuccess: () => {
      queryClient.invalidateQueries('settings');
      options?.onSuccess?.();
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message =
        error?.response?.data?.error?.message || 'Failed to update setting';
      options?.onError?.(message);
    },
  });
}

export function useDeleteSetting(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<Setting>,
  AxiosError<ErrorResponse>,
  { key: string }
> {
  const queryClient = useQueryClient();
  return useMutationQuery<Setting, { key: string }>(
    (vars) => `${ENDPOINT_SETTINGS}/${vars.key}`,
    'delete',
    {
      onSuccess: () => {
        queryClient.invalidateQueries('settings');
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message =
          error?.response?.data?.error?.message || 'Failed to delete setting';
        options?.onError?.(message);
      },
    }
  );
}
