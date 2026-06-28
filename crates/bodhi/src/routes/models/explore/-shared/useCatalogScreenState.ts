import { useCallback, useEffect, useMemo, useState } from 'react';

import { useViewTransition } from '@/hooks/useViewTransition';
import { persistSortPreference, resolveSortPreference } from '@/routes/models/explore/-shared/useSortPreference';

type SortOrder = 'asc' | 'desc';

/**
 * The screen-level orchestration shared by every Explore catalog screen (API Models, Providers,
 * Local Discovery, MCP). The four screens differ in their columns, sidebar, rail, and data hook —
 * but the URL-as-truth wiring (search commit, sort resolution + persistence, facet replace, reset
 * precedence, pagination, rail selection) was duplicated verbatim. This hook captures that, leaving
 * each screen to own only its domain-specific rendering.
 *
 * `Search` is the route's URL search shape; `Facets` is the sidebar facet state; `Sort` is the
 * sort-key union. The screen supplies the (pure) conversions between them.
 */
export interface CatalogScreenConfig<Search, Facets, Sort extends string> {
  /** The current URL search (route's useSearch()). */
  search: Search;
  /** The route's navigate(); called with a functional `search` updater. */
  navigate: (opts: { search: (prev: Search) => Search; replace?: boolean }) => void;
  /** URL search → sidebar facet state. */
  searchToFacets: (search: Search) => Facets;
  /** Facet state → the subset of URL search it controls (omit empty facets). */
  facetsToSearch: (facets: Facets) => Partial<Search>;
  /** Whether any facet is active (drives the reset button's "filters" mode). */
  hasActiveFacets: (facets: Facets) => boolean;
  /** Sort resolution/persistence config. Omit for screens with a single fixed sort. */
  sortConfig?: {
    storageKey: string;
    persistedSorts: readonly Sort[];
    validOrders: readonly SortOrder[];
    naturalOrder: (sort: Sort) => SortOrder;
    /**
     * Only some catalogs rank by text-match: when set (e.g. `'relevance'`), committing a non-empty
     * search switches to this sort and clears order, and clearing search drops both q and sort.
     * When omitted, committing a search only toggles `q` (sort/order untouched) — for catalogs
     * whose sort keys don't include a relevance option.
     */
    searchRelevanceSort?: Sort;
  };
}

export interface CatalogScreenState<Facets, Sort extends string> {
  facets: Facets;
  sort: Sort | undefined;
  order: SortOrder | undefined;
  page: number;
  committedSearch: string;
  selectedKey: string | null;
  searchInput: string;
  setSearchInput: (v: string) => void;
  commitSearch: (value: string) => void;
  onSearchChange: (value: string) => void;
  onSearchKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
  onSort: (next: Sort) => void;
  onFacetsChange: (next: Facets) => void;
  onClearAllFacets: () => void;
  resetMode: 'filters' | 'query' | 'none';
  onReset: () => void;
  onPage: (p: number) => void;
  select: (key: string | null) => void;
}

// The hook reads/writes a handful of well-known keys on the search object. They are typed loosely
// here (the public generic surface keeps callers type-safe); internally we treat Search as a bag.
type SearchBag = {
  q?: string;
  sort?: string;
  order?: SortOrder;
  page?: number;
  select?: string;
};

export function useCatalogScreenState<Search, Facets, Sort extends string>(
  config: CatalogScreenConfig<Search, Facets, Sort>
): CatalogScreenState<Facets, Sort> {
  const { search, navigate, searchToFacets, facetsToSearch, hasActiveFacets, sortConfig } = config;
  const s = search as Search & SearchBag;

  const resolved = sortConfig
    ? resolveSortPreference<Sort, SortOrder>({
        urlSort: s.sort as Sort | undefined,
        urlOrder: s.order,
        storageKey: sortConfig.storageKey,
        validSorts: sortConfig.persistedSorts,
        validOrders: sortConfig.validOrders,
        naturalOrder: sortConfig.naturalOrder,
      })
    : { sort: s.sort as Sort | undefined, order: s.order };
  const sort = resolved.sort;
  const order = resolved.order;
  const page = s.page ?? 1;
  const committedSearch = s.q ?? '';
  const facets = useMemo(() => searchToFacets(search), [search, searchToFacets]);
  const selectedKey = s.select ?? null;

  const [searchInput, setSearchInput] = useState(committedSearch);
  useEffect(() => {
    setSearchInput(committedSearch);
  }, [committedSearch]);

  const withViewTransition = useViewTransition();

  const select = useCallback(
    (key: string | null) => {
      if ((key ?? undefined) === s.select) return;
      withViewTransition(() => {
        navigate({
          search: (prev) => {
            const out = { ...prev } as Search & SearchBag;
            if (key) out.select = key;
            else delete out.select;
            return out;
          },
          replace: true,
        });
      });
    },
    [navigate, withViewTransition, s.select]
  );

  // The non-facet slice (q/sort/order/select) carried across a facet change; `page` is dropped so a
  // facet change resets to page 1.
  const nonFacetSlice = useCallback((prev: Search): Search => {
    const p = prev as Search & SearchBag;
    const base = {} as Search & SearchBag;
    if (p.q) base.q = p.q;
    if (p.sort) base.sort = p.sort;
    if (p.order) base.order = p.order;
    if (p.select) base.select = p.select;
    return base;
  }, []);

  const relevanceSort = sortConfig?.searchRelevanceSort;
  const commitSearch = useCallback(
    (value: string) => {
      const next = value.trim();
      navigate({
        search: (prev) => {
          const out = { ...prev } as Search & SearchBag;
          delete out.page;
          if (relevanceSort) {
            // Text-ranked catalog: search switches to the relevance sort; clearing restores natural order.
            delete out.order;
            if (next) {
              out.q = next;
              out.sort = relevanceSort;
            } else {
              delete out.q;
              delete out.sort;
            }
          } else if (next) {
            out.q = next;
          } else {
            delete out.q;
          }
          return out;
        },
      });
    },
    [navigate, relevanceSort]
  );

  const onSearchChange = useCallback(
    (value: string) => {
      setSearchInput(value);
      if (value.trim() === '') commitSearch('');
    },
    [commitSearch]
  );

  const onSearchKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Enter') commitSearch(searchInput);
    },
    [commitSearch, searchInput]
  );

  const onSort = useCallback(
    (next: Sort) => {
      const naturalOrder = sortConfig ? sortConfig.naturalOrder(next) : 'asc';
      const nextOrder: SortOrder = sort === next ? (order === 'asc' ? 'desc' : 'asc') : naturalOrder;
      // Persist explicit picks (so they apply on a later clean-URL visit); the relevance sort is
      // search-driven and excluded.
      if (sortConfig && next !== sortConfig.searchRelevanceSort) {
        persistSortPreference(sortConfig.storageKey, next, nextOrder);
      }
      navigate({
        search: (prev) => {
          const out = { ...prev } as Search & SearchBag;
          delete out.page;
          out.sort = next;
          if (nextOrder === naturalOrder) delete out.order;
          else out.order = nextOrder;
          return out;
        },
      });
    },
    [navigate, sort, order, sortConfig]
  );

  const onFacetsChange = useCallback(
    (next: Facets) => navigate({ search: (prev) => ({ ...nonFacetSlice(prev), ...facetsToSearch(next) }) }),
    [navigate, nonFacetSlice, facetsToSearch]
  );

  const onClearAllFacets = useCallback(
    () => navigate({ search: (prev) => nonFacetSlice(prev) }),
    [navigate, nonFacetSlice]
  );

  const hasFilters = hasActiveFacets(facets);
  const hasQuery = committedSearch !== '';
  const resetMode: 'filters' | 'query' | 'none' = hasFilters ? 'filters' : hasQuery ? 'query' : 'none';
  const onReset = useCallback(() => {
    if (resetMode === 'filters') onClearAllFacets();
    else if (resetMode === 'query') commitSearch('');
  }, [resetMode, onClearAllFacets, commitSearch]);

  const onPage = useCallback(
    (p: number) =>
      navigate({
        search: (prev) => {
          const out = { ...prev } as Search & SearchBag;
          if (p === 1) delete out.page;
          else out.page = p;
          return out;
        },
      }),
    [navigate]
  );

  return {
    facets,
    sort,
    order,
    page,
    committedSearch,
    selectedKey,
    searchInput,
    setSearchInput,
    commitSearch,
    onSearchChange,
    onSearchKeyDown,
    onSort,
    onFacetsChange,
    onClearAllFacets,
    resetMode,
    onReset,
    onPage,
    select,
  };
}
