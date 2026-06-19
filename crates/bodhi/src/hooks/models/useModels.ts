import { Alias, BodhiErrorResponse, PaginatedAliasResponse, UserAliasRequest } from '@bodhiapp/ts-client';
import { keepPreviousData } from '@tanstack/react-query';
import { AxiosError, AxiosResponse } from 'axios';

import { UseMutationResult, useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';

import { modelKeys, ENDPOINT_MODELS, ENDPOINT_ALIAS } from './constants';

/** Alias-type facet tokens (mirror the backend `type` query param). */
export type ModelTypeFacet = 'local_file' | 'model_alias' | 'api_model' | 'fallback';
/** API-format facet tokens (API rows only; mirror the backend `api_format` query param). */
export type ApiFormatFacet = 'openai' | 'responses' | 'anthropic' | 'gemini' | 'liberty';
/** Capability facet tokens (local rows only; mirror the backend `capability` query param). */
export type CapabilityFacet = 'vision' | 'tool_use' | 'reasoning';

/** Faceted filter selection for the All-Models list. Empty arrays / undefined = no filter. */
export interface ModelsFilter {
  types?: ModelTypeFacet[];
  apiFormats?: ApiFormatFacet[];
  capabilities?: CapabilityFacet[];
  /** Local-file size range in BYTES, inclusive. Omit a bound to leave it open. */
  sizeMin?: number;
  sizeMax?: number;
}

/** Build the backend query params for a filter (CSV for multi-value facets; omits empties). */
export function buildModelsFilterParams(filter?: ModelsFilter): Record<string, string | number> {
  const params: Record<string, string | number> = {};
  if (filter?.types?.length) params.type = filter.types.join(',');
  if (filter?.apiFormats?.length) params.api_format = filter.apiFormats.join(',');
  if (filter?.capabilities?.length) params.capability = filter.capabilities.join(',');
  if (filter?.sizeMin != null) params.size_min = filter.sizeMin;
  if (filter?.sizeMax != null) params.size_max = filter.sizeMax;
  return params;
}

/** Stable cache-key fragment for a filter (sorted, so token order doesn't fragment the cache). */
function filterCacheKey(filter?: ModelsFilter): string {
  const params = buildModelsFilterParams(filter);
  return Object.keys(params)
    .sort()
    .map((k) => `${k}=${params[k]}`)
    .join('&');
}

export function useListModels(page: number, pageSize: number, sort: string, sortOrder: string, filter?: ModelsFilter) {
  return useQuery<PaginatedAliasResponse>(
    modelKeys.list(page, pageSize, sort, sortOrder, filterCacheKey(filter)),
    ENDPOINT_MODELS,
    {
      page,
      page_size: pageSize,
      sort,
      sort_order: sortOrder,
      ...buildModelsFilterParams(filter),
    },
    // Keep the prior page visible during a facet/page refetch so the list doesn't flash empty.
    { placeholderData: keepPreviousData }
  );
}

export function useGetModel(id: string) {
  return useQuery<Alias>(modelKeys.detail(id), `${ENDPOINT_MODELS}/${id}`, undefined, {
    enabled: !!id,
  });
}

export function useCreateModel(options?: {
  onSuccess?: (model: Alias) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<Alias>, AxiosError<BodhiErrorResponse>, UserAliasRequest> {
  const queryClient = useQueryClient();
  return useMutationQuery<Alias, UserAliasRequest>(ENDPOINT_ALIAS, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: modelKeys.all });
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<BodhiErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to create model';
      options?.onError?.(message);
    },
  });
}

export function useUpdateModel(
  id: string,
  options?: {
    onSuccess?: (model: Alias) => void;
    onError?: (message: string) => void;
  }
): UseMutationResult<AxiosResponse<Alias>, AxiosError<BodhiErrorResponse>, UserAliasRequest> {
  const queryClient = useQueryClient();
  return useMutationQuery<Alias, UserAliasRequest>(() => `${ENDPOINT_ALIAS}/${id}`, 'put', {
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: modelKeys.all });
      queryClient.invalidateQueries({ queryKey: modelKeys.detail(id) });
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<BodhiErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to update model';
      options?.onError?.(message);
    },
  });
}
