import type { GetModelResponse, ListModelsQuery, ListModelsResponse } from '@bodhiapp/reference-api-types';
import { keepPreviousData, useQuery } from '@tanstack/react-query';

import { REF_ENDPOINT_MODELS, refEndpointModel, referenceKeys } from './constants';
import { useAnonymousReferenceApi } from './useReferenceApi';

/**
 * Discovery hooks for the external model catalog (the Reference API's `GET /api/v1/models`).
 *
 * Typed by `@bodhiapp/reference-api-types` — the API's own published wire types — so the request
 * object can only carry params the v1 API actually supports (any dropped filter is a compile error,
 * not a silent no-op).
 *
 * The catalog is **publicly readable**, so we use an ANONYMOUS reference client (no id_token): a
 * present-but-invalid token — e.g. one from a Keycloak env the API doesn't trust — is rejected 401,
 * which would break the public read. Per-user attribution is deferred until auth envs are aligned.
 * Calls go through the fetch-based reference client, NOT the same-origin axios `apiClient`, so we
 * drive `@tanstack/react-query` directly here.
 */

/**
 * Build a query string from a typed params object, omitting empty values. Array-valued params
 * (`author`, `specialisation`, `tag`, `language`, `license`) serialize as repeated keys.
 */
export function buildModelsQuery(params: ListModelsQuery): string {
  const sp = new URLSearchParams();
  (Object.keys(params) as Array<keyof ListModelsQuery>).forEach((key) => {
    const value = params[key];
    if (value === undefined || value === null || value === '') return;
    if (Array.isArray(value)) {
      value.forEach((v) => {
        if (v !== undefined && v !== null && v !== '') sp.append(key, String(v));
      });
    } else {
      sp.append(key, String(value));
    }
  });
  return sp.toString();
}

/**
 * `GET /api/v1/models` — list/search/filter/sort with keyset pagination.
 *
 * Gated on a ready client (`enabled: !!client`); `keepPreviousData` avoids an empty-flash on
 * facet/sort/page change. Setting `q` disables the cursor server-side (`next_cursor` is null),
 * so callers raise `limit` instead of paging during search.
 */
export function useDiscoverModels(params: ListModelsQuery) {
  const client = useAnonymousReferenceApi();
  const query = buildModelsQuery(params);
  return useQuery<ListModelsResponse>({
    queryKey: referenceKeys.discoverList(query),
    queryFn: () => client!.get<ListModelsResponse>(`${REF_ENDPOINT_MODELS}${query ? `?${query}` : ''}`),
    enabled: !!client,
    placeholderData: keepPreviousData,
  });
}

/**
 * `GET /api/v1/models/{source}/{namespace}/{repo}` — a single model with its `quants[]` table and
 * the detail-only fields (context_max, architecture, sizes) that are null on list rows.
 *
 * Gated on a ready client and a non-null selection. No `?include=` — README is not surfaced in v1
 * and quant sizes come back on the base detail response.
 */
export function useModelDetail(selected: { source: string; namespace: string; repo: string } | null) {
  const client = useAnonymousReferenceApi();
  return useQuery<GetModelResponse>({
    queryKey: selected
      ? referenceKeys.discoverDetail(selected.source, selected.namespace, selected.repo)
      : referenceKeys.discoverDetail('', '', ''),
    queryFn: () =>
      client!.get<GetModelResponse>(refEndpointModel(selected!.source, selected!.namespace, selected!.repo)),
    enabled: !!client && !!selected,
  });
}
