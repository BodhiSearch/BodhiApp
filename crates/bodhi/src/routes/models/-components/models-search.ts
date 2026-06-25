import type { ModelsFilter } from '@/hooks/models';

import type { ModelsSearch } from '../index';

export const PAGE_SIZE = 30;

/** URL search → the facet state the sidebar renders from (inverse of {@link facetsToSearch}). */
export function searchToFilter(s: ModelsSearch): ModelsFilter {
  return {
    ...(s.type?.length ? { types: s.type } : {}),
    ...(s.api_format?.length ? { apiFormats: s.api_format } : {}),
    ...(s.capability?.length ? { capabilities: s.capability } : {}),
    ...(s.size_min != null ? { sizeMin: s.size_min } : {}),
    ...(s.size_max != null ? { sizeMax: s.size_max } : {}),
    ...(s.q ? { search: s.q } : {}),
  };
}

/** Facet state → the facet slice of the URL search (defaults/empties omitted). `search` lives in `q`. */
export function facetsToSearch(f: ModelsFilter): Partial<ModelsSearch> {
  return {
    ...(f.types?.length ? { type: f.types } : {}),
    ...(f.apiFormats?.length ? { api_format: f.apiFormats } : {}),
    ...(f.capabilities?.length ? { capability: f.capabilities } : {}),
    ...(f.sizeMin != null ? { size_min: f.sizeMin } : {}),
    ...(f.sizeMax != null ? { size_max: f.sizeMax } : {}),
  };
}

/** True when any sidebar facet (not the search query) is active. */
export function hasActiveFilter(f: ModelsFilter): boolean {
  return Boolean(
    f.types?.length || f.apiFormats?.length || f.capabilities?.length || f.sizeMin != null || f.sizeMax != null
  );
}

/**
 * URL search + the effective (resolved) sort → the args `useListModels(page, pageSize, sort,
 * order, filter)` consumes. The backend sorts by alias name for `sort='name'`; the derived
 * `provider`/`base_url` columns are sorted client-side in the screen, so here they map to the
 * server's name sort (a stable, deterministic base order) and the page reorders in-memory.
 */
export function searchToListArgs(
  s: ModelsSearch,
  effective?: { sort?: string; order?: string }
): { page: number; pageSize: number; sort: string; sortOrder: string; filter: ModelsFilter } {
  const sort = effective?.sort ?? s.sort;
  const order = effective?.order ?? s.order ?? 'asc';
  // 'name' → backend 'alias'; the derived columns have no server sort, so fall back to 'alias'.
  const backendSort = sort === 'name' ? 'alias' : 'alias';
  return {
    page: s.page ?? 1,
    pageSize: PAGE_SIZE,
    sort: backendSort,
    sortOrder: order,
    filter: searchToFilter(s),
  };
}
