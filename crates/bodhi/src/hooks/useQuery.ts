import {
  useQuery as useReactQuery,
  useMutation,
  useQueryClient,
  UseQueryOptions,
  UseMutationOptions,
  UseMutationResult,
} from 'react-query';
import apiClient from '@/lib/apiClient';
import { AxiosError, AxiosResponse } from 'axios';
import {
  AppInfo,
  FeaturedModel,
  Model,
  ModelFile,
  UserInfo,
  Setting,
} from '@/types/models';
import { AliasFormData } from '@/schemas/alias';
import {
  ListDownloadsResponse,
  DownloadRequest,
  PullModelRequest,
} from '@/types/api';

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
  options?: UseQueryOptions<T, AxiosError>
) {
  return useReactQuery<T, AxiosError>(
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
  options?: UseMutationOptions<AxiosResponse<T>, AxiosError, V>
): UseMutationResult<AxiosResponse<T>, AxiosError, V> {
  const queryClient = useQueryClient();

  return useMutation<AxiosResponse<T>, AxiosError, V>(
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

type SetupRequest = {
  authz: boolean;
};

export function useSetupApp() {
  return useMutationQuery<AppInfo, SetupRequest>(ENDPOINT_APP_SETUP);
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

export function useCreateModel() {
  return useMutationQuery<Model, AliasFormData>(ENDPOINT_MODELS);
}

export function useUpdateModel(alias: string) {
  const queryClient = useQueryClient();
  return useMutationQuery<Model, AliasFormData>(
    `${ENDPOINT_MODELS}/${alias}`,
    'put',
    {
      onSuccess: () => {
        queryClient.invalidateQueries(['model', alias]);
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
  options?: UseMutationOptions<AxiosResponse, AxiosError, void, unknown>
): UseMutationResult<AxiosResponse, AxiosError, void, unknown> {
  const queryClient = useQueryClient();
  return useMutationQuery<AxiosResponse, void>(ENDPOINT_LOGOUT, 'post', {
    ...options,
    onSuccess: (data, variables, context) => {
      queryClient.invalidateQueries();
      if (options?.onSuccess) {
        options.onSuccess(data, variables, context);
      }
    },
  });
}

export function useUser(options?: { enabled: boolean }) {
  return useQuery<UserInfo | null>('user', ENDPOINT_USER_INFO, undefined, {
    retry: false,
    enabled: options?.enabled,
  });
}

export function useDownloads(page: number, pageSize: number) {
  return useQuery<ListDownloadsResponse>(
    ['downloads', page.toString(), pageSize.toString()],
    ENDPOINT_MODEL_FILES_PULL,
    { page, page_size: pageSize }
  );
}

export function usePullModel() {
  return useMutationQuery<DownloadRequest, PullModelRequest>(
    ENDPOINT_MODEL_FILES_PULL
  );
}

export function useSettings() {
  return useQuery<Setting[]>('settings', ENDPOINT_SETTINGS);
}

export function useUpdateSetting() {
  const queryClient = useQueryClient();
  return useMutationQuery<Setting, {key: string, value: string | number | boolean}>(
    (vars) => `${ENDPOINT_SETTINGS}/${vars.key}`,
    'put',
    {
      onSuccess: () => {
        queryClient.invalidateQueries('settings');
      },
    }
  );
}

export function useDeleteSetting() {
  const queryClient = useQueryClient();
  return useMutationQuery<Setting, {key: string}>(
    (vars) => `${ENDPOINT_SETTINGS}/${vars.key}`,
    'delete',
    {
      onSuccess: () => {
        queryClient.invalidateQueries('settings');
      },
    }
  );
}
