import type { ListProvidersQuery } from '@bodhiapp/reference-api-types';

import type { ExploreProvidersSearch } from '../index';

import { providerFacetsToQuery, type ProviderFacets } from './ExploreProvidersSidebar';

export const DEFAULT_ORDER = 'desc' as const;
export const PAGE_SIZE = 30;

/** URL search → the facet state the sidebar renders from. Inverse of `providerFacetsToQuery`. */
export function searchToFacets(s: ExploreProvidersSearch): ProviderFacets {
  return {
    ...(s.capability?.length ? { capability: s.capability } : {}),
    ...(s.api_format?.length ? { api_format: s.api_format } : {}),
    ...(s.pricing ? { pricing: s.pricing } : {}),
    ...(s.is_lab === 'true' ? { is_lab: true } : {}),
  };
}

/** Facet state → the facet slice of the URL search (defaults/empties omitted). */
export function facetsToSearch(f: ProviderFacets): Partial<ExploreProvidersSearch> {
  return providerFacetsToQuery(f) as Partial<ExploreProvidersSearch>;
}

/**
 * URL search → the provider catalog API params (what `useCatalogProviders` consumes). `sort`/`order`
 * are resolved by the screen (URL > localStorage > none) and passed in via `effective`; when absent
 * they are omitted so the API uses its natural order.
 */
export function searchToParams(
  s: ExploreProvidersSearch,
  effective?: { sort?: string; order?: string }
): ListProvidersQuery {
  const sort = effective?.sort ?? s.sort;
  const order = effective?.order ?? s.order;
  return {
    ...(sort ? { sort: sort as ListProvidersQuery['sort'] } : {}),
    ...(order ? { order: order as ListProvidersQuery['order'] } : {}),
    page: s.page ?? 1,
    page_size: PAGE_SIZE,
    ...(s.q ? { q: s.q } : {}),
    ...providerFacetsToQuery(searchToFacets(s)),
  };
}
