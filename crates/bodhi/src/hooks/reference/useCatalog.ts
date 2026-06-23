import type {
  ListCatalogModelsQuery,
  ListProviderModelsQuery,
  ListProvidersQuery,
  ModelDetailResponse,
  ModelsListResponse,
  ProviderDetailResponse,
  ProviderListResponse,
  ProviderModelsResponse,
} from '@bodhiapp/reference-api-types';
import { keepPreviousData, useQuery } from '@tanstack/react-query';

import {
  REF_ENDPOINT_CATALOG_MODELS,
  REF_ENDPOINT_CATALOG_PROVIDERS,
  refEndpointCatalogModel,
  refEndpointCatalogProvider,
  refEndpointCatalogProviderModels,
  referenceKeys,
} from './constants';
import { useAnonymousReferenceApi } from './useReferenceApi';

/**
 * Hooks for the API-model catalog (`/api/v1/catalog/*`) — the "models.dev inside Bodhi" Explore
 * pages (API Models + API Providers).
 *
 * The catalog is **publicly readable**, so all reads go through the ANONYMOUS reference client (no
 * id_token): a present-but-invalid token (e.g. from a Keycloak env the API doesn't trust) is
 * rejected 401, which would break the public read. Typed by `@bodhiapp/reference-api-types` (the
 * API's own published wire types) so query objects can only carry supported params.
 */

/**
 * Build a query string from a typed catalog params object, omitting empty values. Array-valued
 * params (`capability`, `modality`, `status`, `provider`, `api_format`) serialize as repeated keys.
 */
export function buildCatalogModelsQuery(
  params: ListCatalogModelsQuery | ListProvidersQuery | ListProviderModelsQuery
): string {
  const sp = new URLSearchParams();
  const isEmpty = (v: unknown) => v === undefined || v === null || v === '';
  (Object.keys(params) as Array<keyof typeof params>).forEach((key) => {
    const value = params[key];
    if (isEmpty(value)) return;
    if (Array.isArray(value)) {
      value.forEach((v) => {
        if (!isEmpty(v)) sp.append(key, String(v));
      });
    } else {
      sp.append(key, String(value));
    }
  });
  return sp.toString();
}

/** `GET /api/v1/catalog/providers` — provider list + facets, page-based. */
export function useCatalogProviders(params: ListProvidersQuery) {
  const client = useAnonymousReferenceApi();
  const query = buildCatalogModelsQuery(params);
  return useQuery<ProviderListResponse>({
    queryKey: referenceKeys.catalogProviders(query),
    queryFn: () => client!.get<ProviderListResponse>(`${REF_ENDPOINT_CATALOG_PROVIDERS}${query ? `?${query}` : ''}`),
    enabled: !!client,
    placeholderData: keepPreviousData,
  });
}

/** `GET /api/v1/catalog/providers/{slug}` — provider detail (env/base_url/doc/npm + bridge). */
export function useCatalogProviderDetail(slug: string | null) {
  const client = useAnonymousReferenceApi();
  return useQuery<ProviderDetailResponse>({
    queryKey: referenceKeys.catalogProvider(slug ?? ''),
    queryFn: () => client!.get<ProviderDetailResponse>(refEndpointCatalogProvider(slug!)),
    enabled: !!client && !!slug,
  });
}

/** `GET /api/v1/catalog/providers/{slug}/models` — the models a provider serves. */
export function useCatalogProviderModels(slug: string | null, params: ListProviderModelsQuery = {}) {
  const client = useAnonymousReferenceApi();
  const query = buildCatalogModelsQuery(params);
  return useQuery<ProviderModelsResponse>({
    queryKey: referenceKeys.catalogProviderModels(slug ?? '', query),
    queryFn: () =>
      client!.get<ProviderModelsResponse>(`${refEndpointCatalogProviderModels(slug!)}${query ? `?${query}` : ''}`),
    enabled: !!client && !!slug,
    placeholderData: keepPreviousData,
  });
}

/** `GET /api/v1/catalog/models` — global model list/search + facets, page-based. */
export function useCatalogModels(params: ListCatalogModelsQuery) {
  const client = useAnonymousReferenceApi();
  const query = buildCatalogModelsQuery(params);
  return useQuery<ModelsListResponse>({
    queryKey: referenceKeys.catalogModels(query),
    queryFn: () => client!.get<ModelsListResponse>(`${REF_ENDPOINT_CATALOG_MODELS}${query ? `?${query}` : ''}`),
    enabled: !!client,
    placeholderData: keepPreviousData,
  });
}

/** `GET /api/v1/catalog/models/{slug}/{model_id}` — full model detail (served_by + bridge). */
export function useCatalogModelDetail(selected: { slug: string; modelId: string } | null) {
  const client = useAnonymousReferenceApi();
  return useQuery<ModelDetailResponse>({
    queryKey: referenceKeys.catalogModel(selected?.slug ?? '', selected?.modelId ?? ''),
    queryFn: () => client!.get<ModelDetailResponse>(refEndpointCatalogModel(selected!.slug, selected!.modelId)),
    enabled: !!client && !!selected,
  });
}
