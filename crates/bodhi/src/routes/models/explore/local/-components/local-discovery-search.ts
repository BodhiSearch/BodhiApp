import type { ListModelsQuery, SortKey } from '@bodhiapp/reference-api-types';

import type { LocalDiscoverySearch } from '../index';

import { facetsToQuery, type DiscoveryFacets } from './LocalDiscoverySidebar';

/** Page size; raised when searching (search disables cursor pagination server-side). */
export const PAGE_SIZE = 30;
export const SEARCH_PAGE_SIZE = 100;
/** The catalog's natural sort when none is set (also the Browse default). */
export const DEFAULT_SORT: SortKey = 'downloads';

/** URL search → the facet state the sidebar renders from (inverse of {@link facetsToSearch}). */
export function searchToFacets(s: LocalDiscoverySearch): DiscoveryFacets {
  return {
    ...(s.specialisation?.length ? { specialisation: s.specialisation } : {}),
    ...(s.pipeline_tag ? { pipeline_tag: s.pipeline_tag } : {}),
    ...(s.tag?.length ? { tag: s.tag } : {}),
    ...(s.language?.length ? { language: s.language } : {}),
    ...(s.license?.length ? { license: s.license } : {}),
    ...(s.author?.length ? { author: s.author } : {}),
  };
}

/** Facet state → the facet slice of the URL search (defaults/empties omitted). */
export function facetsToSearch(f: DiscoveryFacets): Partial<LocalDiscoverySearch> {
  return facetsToQuery(f) as Partial<LocalDiscoverySearch>;
}

/**
 * URL search + the component-held cursor → the catalog API params (`useDiscoverModels` input).
 * Search raises the page size and disables cursor pagination (the server ignores `cursor` while
 * `q` is set). There is no `order` param — the catalog is descending-only.
 */
export function searchToParams(s: LocalDiscoverySearch, cursor: string | undefined): ListModelsQuery {
  const searching = (s.q ?? '').trim() !== '';
  return {
    sort: s.sort ?? DEFAULT_SORT,
    limit: searching ? SEARCH_PAGE_SIZE : PAGE_SIZE,
    ...facetsToQuery(searchToFacets(s)),
    ...(searching ? { q: (s.q ?? '').trim() } : {}),
    ...(cursor && !searching ? { cursor } : {}),
  };
}
