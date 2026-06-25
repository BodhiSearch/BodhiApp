import { useCallback, useEffect, useMemo, useState } from 'react';

import type { ModelLite } from '@bodhiapp/reference-api-types';
import { getRouteApi } from '@tanstack/react-router';

import { ShellIcon, ShellPagination, ShellSearch, useListKeyNav, useShellChrome } from '@/components/shell';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { useCatalogModelDetail, useCatalogModels } from '@/hooks/reference';
import { useViewTransition } from '@/hooks/useViewTransition';
import { exploreBreadcrumb } from '@/routes/models/explore/-shared/breadcrumbs';
import {
  CAP_LABELS,
  CAP_TONE,
  fmtContext,
  fmtDate,
  fmtPrice,
  isFree,
  statusLabel,
} from '@/routes/models/explore/-shared/catalog-format';
import { type CatalogColumn, CatalogTable } from '@/routes/models/explore/-shared/catalog-table';
import { ColumnPicker, useHiddenColumns } from '@/routes/models/explore/-shared/ColumnPicker';
import { ResetButton } from '@/routes/models/explore/-shared/ResetButton';
import { persistSortPreference, resolveSortPreference } from '@/routes/models/explore/-shared/useSortPreference';

import type { ExploreApiSearch } from '../index';

import { facetsToSearch, PAGE_SIZE, searchToFacets, searchToParams } from './explore-api-search';
import { ExploreApiRail, ExploreApiRailHeader } from './ExploreApiRail';
import { ExploreApiSidebar, hasActiveModelFacets, type ModelFacetsState } from './ExploreApiSidebar';
import '@/components/shell/list.css';
import '@/routes/models/-components/models.css';
import '@/routes/models/explore/-shared/catalog.css';

const BREADCRUMB = exploreBreadcrumb('Explore · API Models');

const routeApi = getRouteApi('/models/explore/api/');

type ModelSort = NonNullable<ExploreApiSearch['sort']>;
type SortOrder = NonNullable<ExploreApiSearch['order']>;

// The backend's natural direction per sort key (docs: endpoints.md "Sorts"). Selecting a new column
// applies its natural default; clicking the active column toggles it.
const NATURAL_ORDER: Record<ModelSort, SortOrder> = {
  relevance: 'desc',
  updated: 'desc',
  context: 'desc',
  providers: 'desc',
  price: 'asc',
  price_out: 'asc',
  name: 'asc',
  family: 'asc',
};

// Persisted sort preference (URL > localStorage > none). `relevance` is search-driven and excluded
// from the persisted set so a stored pref never overrides the search→relevance behavior.
const SORT_STORAGE_KEY = 'bodhi.explore.api.sort';
const PERSISTED_SORTS = ['updated', 'context', 'providers', 'price', 'price_out', 'name', 'family'] as const;
const VALID_ORDERS = ['asc', 'desc'] as const;

function modelKey(m: ModelLite): string {
  return `${m.slug}/${m.model_id}`;
}

const COLUMNS: CatalogColumn<ModelLite, ModelSort>[] = [
  { key: 'num', label: '#', width: '44px', cell: () => null },
  {
    key: 'model',
    label: 'MODEL',
    width: '',
    sort: 'name',
    cell: (m) => (
      <div className="cat-body">
        <div className="cat-model-name">
          {m.name}
          {m.status && <span className={`cat-status cat-status-${m.status}`}>{statusLabel(m.status)}</span>}
        </div>
        {m.caps.length > 0 && (
          <div className="cat-caps cat-model-caps">
            {m.caps.map((c) => (
              <span className={`cap-chip cap-${CAP_TONE[c]}`} key={c}>
                {CAP_LABELS[c]}
              </span>
            ))}
          </div>
        )}
      </div>
    ),
  },
  {
    key: 'family',
    label: 'FAMILY',
    width: '120px',
    sort: 'family',
    optional: true,
    cell: (m) => <div className="cat-cell-text">{m.family ?? '—'}</div>,
  },
  {
    key: 'context',
    label: 'CONTEXT',
    width: '90px',
    align: 'right',
    sort: 'context',
    optional: true,
    cell: (m) => <div className="cat-num-cell">{fmtContext(m.context_limit)}</div>,
  },
  {
    key: 'price',
    label: 'INPUT $',
    width: '82px',
    align: 'right',
    sort: 'price',
    optional: true,
    cell: (m) => {
      const free = isFree(m.pricing.input_per_m, m.pricing.output_per_m);
      return (
        <div className={`cat-num-cell${free ? ' free' : ''}`}>{free ? 'Free' : fmtPrice(m.pricing.input_per_m)}</div>
      );
    },
  },
  {
    key: 'price_out',
    label: 'OUTPUT $',
    width: '94px',
    align: 'right',
    sort: 'price_out',
    optional: true,
    cell: (m) => {
      const free = isFree(m.pricing.input_per_m, m.pricing.output_per_m);
      return <div className={`cat-num-cell${free ? ' free' : ''}`}>{free ? '' : fmtPrice(m.pricing.output_per_m)}</div>;
    },
  },
  {
    key: 'updated',
    label: 'UPDATED',
    width: '90px',
    align: 'right',
    sort: 'updated',
    optional: true,
    cell: (m) => <div className="cat-num-cell">{fmtDate(m.last_updated)}</div>,
  },
  {
    key: 'providers',
    label: 'PROVIDERS',
    width: '106px',
    align: 'right',
    sort: 'providers',
    optional: true,
    cell: (m) => (
      <div className="cat-score">
        <div className="cat-score-num">{m.provider_count}</div>
      </div>
    ),
  },
];

export function ExploreApiScreen() {
  useListKeyNav();

  // The URL search is the single source of truth; the only effect below writes LOCAL searchInput
  // (URL→input), never the URL, so there is no read→write loop.
  const search = routeApi.useSearch();
  const navigate = routeApi.useNavigate();

  // Effective sort precedence: URL > localStorage (request-only, never written to URL) > none.
  const resolvedSort = resolveSortPreference<ModelSort, SortOrder>({
    urlSort: search.sort,
    urlOrder: search.order,
    storageKey: SORT_STORAGE_KEY,
    validSorts: PERSISTED_SORTS,
    validOrders: VALID_ORDERS,
    naturalOrder: (s) => NATURAL_ORDER[s],
  });
  const sort = resolvedSort.sort;
  const order = resolvedSort.order;
  const page = search.page ?? 1;
  const committedSearch = search.q ?? '';
  const facets = useMemo(() => searchToFacets(search), [search]);

  // The open detail rail is the URL's `select` (composite `${slug}/${model_id}`). Deriving it (not
  // mirroring in state) makes Back/Forward restoration automatic.
  const selectedKey = search.select ?? null;
  // Local-only UI state: uncommitted search text and column visibility are ephemeral per page-load.
  const [searchInput, setSearchInput] = useState(committedSearch);
  const { hidden: hiddenColumns, toggle: toggleColumn, visibleColumns: filterVisible } = useHiddenColumns();
  // Sync the input down from the URL on Back/Forward (URL→input only; never writes back).
  useEffect(() => {
    setSearchInput(committedSearch);
  }, [committedSearch]);

  const visibleColumns = useMemo(() => filterVisible(COLUMNS), [filterVisible]);

  const params = useMemo(() => searchToParams(search, { sort, order }), [search, sort, order]);
  const { data, isLoading, error } = useCatalogModels(params);

  // keepPreviousData (in the hook) avoids a flash on page change; `params` is stable per distinct URL
  // via useMemo([search]). Filter/sort/search writes drop `page` (→ resets to 1); only the pager sets it.
  const rows = data?.items ?? [];
  const total = data?.total ?? rows.length;

  const withViewTransition = useViewTransition();
  // Selection lives in the URL via replace (no history entries) — Back/Forward skips past selections.
  // The rail auto-opens/closes from its content presence, so no openRail() call is needed.
  const select = useCallback(
    (key: string | null) => {
      if ((key ?? undefined) === search.select) return; // dedup
      withViewTransition(() => {
        navigate({
          search: (prev: ExploreApiSearch) => {
            const out: ExploreApiSearch = { ...prev };
            if (key) out.select = key;
            else delete out.select;
            return out;
          },
          replace: true,
        });
      });
    },
    [navigate, withViewTransition, search.select]
  );

  // The non-facet slice (q/sort/order) carried across a facet change; `page` is omitted so facet
  // changes reset to page 1.
  const nonFacetSlice = useCallback((prev: ExploreApiSearch): ExploreApiSearch => {
    const base: ExploreApiSearch = {};
    if (prev.q) base.q = prev.q;
    if (prev.sort) base.sort = prev.sort;
    if (prev.order) base.order = prev.order;
    if (prev.select) base.select = prev.select; // keep the open rail across facet changes
    return base;
  }, []);

  const commitSearch = useCallback(
    (value: string) => {
      const next = value.trim();
      navigate({
        search: (prev: ExploreApiSearch) => {
          const out: ExploreApiSearch = { ...prev };
          delete out.page;
          delete out.order;
          // Searching ranks by text match; clearing drops the sort → natural order (or stored pref).
          if (next) {
            out.q = next;
            out.sort = 'relevance';
          } else {
            delete out.q;
            delete out.sort;
          }
          return out;
        },
      });
    },
    [navigate]
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
    (next: ModelSort) => {
      // Clicking the active column toggles direction; a new column adopts its natural default.
      const nextOrder: SortOrder = sort === next ? (order === 'asc' ? 'desc' : 'asc') : NATURAL_ORDER[next];
      // Persist the explicit pick so it applies on a later clean-URL visit (relevance excluded).
      if (next !== 'relevance') persistSortPreference(SORT_STORAGE_KEY, next, nextOrder);
      navigate({
        search: (prev: ExploreApiSearch) => {
          const out: ExploreApiSearch = { ...prev };
          delete out.page;
          out.sort = next;
          // Omit order when it matches the sort's natural direction; the resolver refills it on read.
          if (nextOrder === NATURAL_ORDER[next]) delete out.order;
          else out.order = nextOrder;
          return out;
        },
      });
    },
    [navigate, sort, order]
  );
  const onFacetsChange = useCallback(
    (next: ModelFacetsState) =>
      // Replace the whole facet slice (a shallow merge can't delete a removed facet); keep q/sort/order.
      navigate({ search: (prev: ExploreApiSearch) => ({ ...nonFacetSlice(prev), ...facetsToSearch(next) }) }),
    [navigate, nonFacetSlice]
  );
  const onClearAllFacets = useCallback(
    () => navigate({ search: (prev: ExploreApiSearch) => nonFacetSlice(prev) }),
    [navigate, nonFacetSlice]
  );
  // The toolbar reset is always visible with three states (in precedence order): clear active filters
  // first, else clear the search query, else nothing to reset (inert noop).
  const hasFilters = hasActiveModelFacets(facets);
  const hasQuery = committedSearch !== '';
  const resetMode: 'filters' | 'query' | 'none' = hasFilters ? 'filters' : hasQuery ? 'query' : 'none';
  const onReset = useCallback(() => {
    if (resetMode === 'filters') onClearAllFacets();
    else if (resetMode === 'query') commitSearch('');
  }, [resetMode, onClearAllFacets, commitSearch]);
  const onPage = useCallback(
    (p: number) =>
      navigate({ search: (prev: ExploreApiSearch) => (p === 1 ? { ...prev, page: undefined } : { ...prev, page: p }) }),
    [navigate]
  );

  const sidebar = useMemo(
    () => <ExploreApiSidebar facets={facets} facetCounts={data?.facets} onFacetsChange={onFacetsChange} />,
    [facets, data?.facets, onFacetsChange]
  );

  // Find the selected row by composite key; if it isn't on the current page (filtered/paged out) the
  // rail closes. The detail ref reads the row's real fields — never parse the key (model_id has '/').
  const selectedModel = useMemo(() => rows.find((m) => modelKey(m) === selectedKey) ?? null, [rows, selectedKey]);
  const selectedRef = selectedModel ? { slug: selectedModel.slug, modelId: selectedModel.model_id } : null;
  const { data: detail, isLoading: detailLoading } = useCatalogModelDetail(selectedRef);

  const railHeader = useMemo(
    () => (selectedModel ? <ExploreApiRailHeader model={selectedModel} onClose={() => select(null)} /> : null),
    [selectedModel, select]
  );
  const rail = useMemo(
    () => (selectedModel ? <ExploreApiRail model={selectedModel} detail={detail} loading={detailLoading} /> : null),
    [selectedModel, detail, detailLoading]
  );

  useShellChrome({
    breadcrumb: useMemo(() => BREADCRUMB, []),
    sidebar,
    rail,
    railHeader,
    railDefaultOpen: false,
  });

  if (error) {
    return <ErrorPage message={error instanceof Error ? error.message : 'Failed to load the model catalog'} />;
  }

  return (
    <div
      className="cat-screen l-page"
      data-testid="explore-api-content"
      data-pagestatus={isLoading ? 'loading' : 'ready'}
    >
      <div className="l-controls">
        <div className="m-toolbar">
          <div className="m-search" data-testid="cat-model-search">
            <ShellSearch
              value={searchInput}
              onChange={onSearchChange}
              onKeyDown={onSearchKeyDown}
              placeholder="Search models"
              kbd="⌘K"
            />
          </div>
          <ResetButton mode={resetMode} onReset={onReset} testId="cat-model-clear-all" />
          <div className="cat-sortbar">
            <ColumnPicker columns={COLUMNS} hidden={hiddenColumns} onToggle={toggleColumn} testIdPrefix="cat-model" />
          </div>
        </div>
      </div>

      <div className="l-scroll" data-testid="cat-model-list">
        {isLoading && rows.length === 0 ? (
          <div style={{ padding: 16 }} data-testid="cat-model-skeleton-container">
            {Array.from({ length: 6 }).map((_, i) => (
              <Skeleton key={i} className="h-14 w-full mb-3" data-testid="cat-model-skeleton" />
            ))}
          </div>
        ) : rows.length === 0 ? (
          <div className="empty-state" data-testid="cat-model-empty">
            <div className="empty-icon">
              <ShellIcon name="search-x" size={28} />
            </div>
            <div className="empty-title">No models found</div>
            <div className="empty-sub">Try a different search or filters.</div>
          </div>
        ) : (
          <CatalogTable<ModelLite, ModelSort>
            columns={visibleColumns}
            rows={rows}
            rowKey={modelKey}
            rowTestId={(m) => `cat-model-row-${m.slug}-${m.model_id}`}
            rowLabel={(m) => `Open ${m.name}`}
            activeKey={selectedKey}
            onSelect={(m) => select(modelKey(m))}
            sort={sort}
            order={order}
            onSort={onSort}
            startIndex={(page - 1) * PAGE_SIZE}
            testIdPrefix="cat-model"
          />
        )}
      </div>

      {total > PAGE_SIZE && (
        <ShellPagination total={total} page={page} onPage={onPage} pageSize={PAGE_SIZE} unit="models" />
      )}
    </div>
  );
}
