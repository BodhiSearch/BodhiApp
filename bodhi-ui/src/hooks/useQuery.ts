import {
  useQuery as useReactQuery,
  useMutation,
  useQueryClient,
  UseQueryOptions,
  UseMutationOptions,
} from 'react-query';
import apiClient from '@/lib/apiClient';
import { AxiosError } from 'axios';
import { AppInfo, FeaturedModel, Model, ModelFile } from '@/types/models';
import { AliasFormData } from '@/schemas/alias';

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
  options?: UseMutationOptions<T, AxiosError, V>
) {
  const queryClient = useQueryClient();

  return useMutation<T, AxiosError, V>(
    async (variables) => {
      const { data } = await apiClient[method]<T>(endpoint, variables, {
        headers: {
          'Content-Type': 'application/json',
        },
      });
      return data;
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
  return useQuery<AppInfo>('appInfo', '/app/info');
}

type SetupRequest = {
  authz: boolean;
}

export function useSetupApp() {
  return useMutationQuery<AppInfo, SetupRequest>('/app/setup');
}

export function useModelFiles(
  page: number,
  pageSize: number,
  sort: string,
  sortOrder: string
) {
  return useQuery<PagedApiResponse<ModelFile[]>>(
    ['modelFiles', page.toString(), pageSize.toString(), sort, sortOrder],
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
  return useMutationQuery<Model, AliasFormData>(
    `/api/ui/models/${alias}`,
    'put'
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
