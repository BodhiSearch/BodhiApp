import type { ListCatalogModelsQuery } from '@bodhiapp/reference-api-types';

import type { ExploreApiSearch } from '../index';

import { modelFacetsToQuery, type ModelFacetsState } from './ExploreApiSidebar';

// Defaults the URL never carries (stripped before navigate). Mirrors the screen's prior useState defaults.
export const DEFAULT_SORT = 'updated' as const;
export const DEFAULT_ORDER = 'desc' as const;
export const PAGE_SIZE = 30;

/**
 * URL search → the facet state the sidebar renders from. Inverse of `modelFacetsToQuery` for the
 * facet fields; the non-facet fields (q/sort/order/page) are read directly off `search`.
 */
export function searchToFacets(s: ExploreApiSearch): ModelFacetsState {
  return {
    ...(s.capability?.length ? { capability: s.capability } : {}),
    ...(s.modality?.length ? { modality: s.modality } : {}),
    ...(s.status?.length ? { status: s.status } : {}),
    ...(s.provider?.length ? { provider: s.provider } : {}),
    ...(s.family?.length ? { family: s.family } : {}),
    ...(s.open_weights ? { open_weights: s.open_weights } : {}),
    ...(s.pricing ? { pricing: s.pricing } : {}),
    ...(s.pricing_in_min != null ? { pricing_in_min: s.pricing_in_min } : {}),
    ...(s.pricing_in_max != null ? { pricing_in_max: s.pricing_in_max } : {}),
    ...(s.pricing_out_min != null ? { pricing_out_min: s.pricing_out_min } : {}),
    ...(s.pricing_out_max != null ? { pricing_out_max: s.pricing_out_max } : {}),
    ...(s.context_min != null ? { context_min: s.context_min } : {}),
  };
}

/**
 * Facet state → the facet slice of the URL search (defaults/empties omitted). This is exactly the
 * shape `modelFacetsToQuery` produces — the URL facet slice and the API facet params are identical.
 */
export function facetsToSearch(f: ModelFacetsState): Partial<ExploreApiSearch> {
  return modelFacetsToQuery(f) as Partial<ExploreApiSearch>;
}

/** URL search → the catalog API params object (what `useCatalogModels` consumes). Applies defaults. */
export function searchToParams(s: ExploreApiSearch): ListCatalogModelsQuery {
  return {
    sort: s.sort ?? DEFAULT_SORT,
    order: s.order ?? DEFAULT_ORDER,
    page: s.page ?? 1,
    page_size: PAGE_SIZE,
    ...(s.q ? { q: s.q } : {}),
    ...modelFacetsToQuery(searchToFacets(s)),
  };
}
