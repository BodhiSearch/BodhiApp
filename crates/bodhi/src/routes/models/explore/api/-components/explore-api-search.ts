import type { ListCatalogModelsQuery } from '@bodhiapp/reference-api-types';

import type { ExploreApiSearch } from '../index';

import { modelFacetsToQuery, type ModelFacetsState } from './ExploreApiSidebar';

// Defaults the URL never carries (stripped before navigate). There is no implicit default SORT:
// when neither the URL nor a stored preference sets one, sort/order are omitted and the API returns
// its natural order. DEFAULT_ORDER is the fallback direction once a sort key is chosen.
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

/**
 * URL search → the catalog API params object (what `useCatalogModels` consumes). `sort`/`order` are
 * resolved by the screen (URL > localStorage > none) and passed in via `effective`; when absent they
 * are omitted entirely so the API uses its natural order.
 */
export function searchToParams(
  s: ExploreApiSearch,
  effective?: { sort?: string; order?: string }
): ListCatalogModelsQuery {
  const sort = effective?.sort ?? s.sort;
  const order = effective?.order ?? s.order;
  return {
    ...(sort ? { sort: sort as ListCatalogModelsQuery['sort'] } : {}),
    ...(order ? { order: order as ListCatalogModelsQuery['order'] } : {}),
    page: s.page ?? 1,
    page_size: PAGE_SIZE,
    ...(s.q ? { q: s.q } : {}),
    ...modelFacetsToQuery(searchToFacets(s)),
  };
}
