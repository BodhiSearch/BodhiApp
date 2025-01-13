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
} from '@/types/models';
import { AliasFormData } from '@/schemas/alias';
import {
  ListDownloadsResponse,
  DownloadRequest,
  PullModelRequest,
} from '@/types/api';

export const ENDPOINT_APP_INFO = '/app/info';
export const ENDPOINT_USER_INFO = '/api/ui/user';

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
  endpoint: string,
  method: 'post' | 'put' | 'delete' = 'post',
  options?: UseMutationOptions<AxiosResponse<T>, AxiosError, V>
): UseMutationResult<AxiosResponse<T>, AxiosError, V> {
  const queryClient = useQueryClient();

  return useMutation<AxiosResponse<T>, AxiosError, V>(
    async (variables) => {
      const response = await apiClient[method]<T>(endpoint, variables, {
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
        queryClient.invalidateQueries(endpoint);
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
  return useMutationQuery<AppInfo, SetupRequest>('/app/setup');
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
    '/api/ui/modelfiles',
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
    '/api/ui/models',
    { page, page_size: pageSize, sort, sort_order: sortOrder }
  );
}

export function useModel(alias: string) {
  return useQuery<Model>(
    ['model', alias],
    `/api/ui/models/${alias}`,
    undefined,
    {
      enabled: !!alias,
    }
  );
}

export function useCreateModel() {
  return useMutationQuery<Model, AliasFormData>('/api/ui/models');
}

export function useUpdateModel(alias: string) {
  const queryClient = useQueryClient();
  return useMutationQuery<Model, AliasFormData>(
    `/api/ui/models/${alias}`,
    'put',
    {
      onSuccess: () => {
        queryClient.invalidateQueries(['model', alias]);
      },
    }
  );
}

export function useChatTemplates() {
  return useQuery<string[]>('chatTemplates', '/api/ui/chat_templates');
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
  return useMutationQuery<AxiosResponse, void>('/api/ui/logout', 'post', {
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
    '/api/ui/modelfiles/pull/downloads',
    { page, page_size: pageSize }
  );
}

export function usePullModel() {
  return useMutationQuery<DownloadRequest, PullModelRequest>(
    '/api/ui/modelfiles/pull'
  );
}
