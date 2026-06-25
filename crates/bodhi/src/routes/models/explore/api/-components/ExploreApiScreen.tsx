import { useCallback, useEffect, useMemo, useState } from 'react';

import type { ModelLite } from '@bodhiapp/reference-api-types';
import { getRouteApi } from '@tanstack/react-router';

import {
  LinkRow,
  ShellIcon,
  ShellPagination,
  ShellSearch,
  useListKeyNav,
  useShell,
  useShellChrome,
} from '@/components/shell';
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
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

import type { ExploreApiSearch } from '../index';

import {
  DEFAULT_ORDER,
  DEFAULT_SORT,
  facetsToSearch,
  PAGE_SIZE,
  searchToFacets,
  searchToParams,
} from './explore-api-search';
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

function ColSort({
  col,
  label,
  sort,
  order,
  align,
  onSort,
}: {
  col: ModelSort;
  label: string;
  sort: ModelSort;
  order: SortOrder;
  align: ColumnAlign;
  onSort: (c: ModelSort) => void;
}) {
  const active = sort === col;
  const icon = !active ? 'chevrons-up-down' : order === 'asc' ? 'arrow-up' : 'arrow-down';
  return (
    <button
      type="button"
      className={`cat-colsort${align === 'left' ? ' cat-colsort--left' : ''}${active ? ' on' : ''}`}
      onClick={() => onSort(col)}
      data-testid={`cat-model-sort-${col}`}
      data-test-state={active ? 'active' : 'idle'}
    >
      <span className="cat-colsort-label">{label}</span>
      <ShellIcon name={icon} size={10} />
    </button>
  );
}

function modelKey(m: ModelLite): string {
  return `${m.slug}/${m.model_id}`;
}

type ColumnAlign = 'left' | 'right';

// Column model: headers, row cells, and the <colgroup> all derive from this so the column picker
// (show/hide) and any sortable header stay in sync. `#` + MODEL are mandatory; the rest are toggleable.
// `width` is a <col> width (px); an empty string means "no explicit width" — that column absorbs the
// table's slack under table-layout:fixed (only MODEL does this). `sort` (when set) makes the header a
// sortable ColSort. `align` keeps the header label justified like the cell content (text left, numeric
// right).
interface Column {
  key: string;
  label: string;
  width: string;
  align?: ColumnAlign;
  sort?: ModelSort;
  optional?: boolean;
  cell: (m: ModelLite) => React.ReactNode;
}

const COLUMNS: Column[] = [
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

function ColumnPicker({ hidden, onToggle }: { hidden: Set<string>; onToggle: (key: string) => void }) {
  const optional = COLUMNS.filter((c) => c.optional);
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          className="cat-sort-btn cat-toolbar-icon-btn"
          data-testid="cat-model-columns"
          aria-label="Columns"
          title="Columns"
        >
          <ShellIcon name="columns-3" size={13} />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuLabel>Columns</DropdownMenuLabel>
        <DropdownMenuSeparator />
        {optional.map((col) => (
          <DropdownMenuCheckboxItem
            key={col.key}
            checked={!hidden.has(col.key)}
            onCheckedChange={() => onToggle(col.key)}
            onSelect={(e) => e.preventDefault()}
            data-testid={`cat-model-col-${col.key}`}
          >
            {col.label}
          </DropdownMenuCheckboxItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function ModelRow({
  model,
  idx,
  active,
  columns,
  onSelect,
}: {
  model: ModelLite;
  idx: number;
  active: boolean;
  columns: Column[];
  onSelect: () => void;
}) {
  return (
    <tr
      className={`l-listrow cat-row${active ? ' active' : ''}`}
      onClick={onSelect}
      role="option"
      aria-selected={active}
      data-testid={`cat-model-row-${model.slug}-${model.model_id}`}
    >
      {columns.map((col) =>
        col.key === 'num' ? (
          <td className="cat-num-td" key="num">
            <LinkRow onActivate={onSelect} label={`Open ${model.name}`} />
            <span className="cat-num">#{idx}</span>
          </td>
        ) : (
          <td key={col.key} className={col.align === 'right' ? 'cat-td--right' : undefined}>
            {col.cell(model)}
          </td>
        )
      )}
    </tr>
  );
}

export function ExploreApiScreen() {
  useListKeyNav();

  // The URL search is the single source of truth: sort/order/page/q/facets are DERIVED from it each
  // render, and user actions write back via navigate(). No useState mirrors the URL, and the only
  // effect below writes LOCAL state (searchInput) — never the URL — so there is no read→write loop.
  const search = routeApi.useSearch();
  const navigate = routeApi.useNavigate();

  const sort = search.sort ?? DEFAULT_SORT;
  const order = search.order ?? DEFAULT_ORDER;
  const page = search.page ?? 1;
  const committedSearch = search.q ?? '';
  const facets = useMemo(() => searchToFacets(search), [search]);

  // Local-only UI state: uncommitted search text, the open detail rail, and column visibility are
  // ephemeral per page-load and deliberately NOT in the URL.
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const [searchInput, setSearchInput] = useState(committedSearch);
  const [hiddenColumns, setHiddenColumns] = useState<Set<string>>(() => new Set());
  // Sync the input down from the URL on Back/Forward (URL→input only; never writes back).
  useEffect(() => {
    setSearchInput(committedSearch);
  }, [committedSearch]);
  const toggleColumn = useCallback((key: string) => {
    setHiddenColumns((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }, []);

  const visibleColumns = useMemo(
    () => COLUMNS.filter((c) => !c.optional || !hiddenColumns.has(c.key)),
    [hiddenColumns]
  );

  const params = useMemo(() => searchToParams(search), [search]);
  const { data, isLoading, error } = useCatalogModels(params);

  // keepPreviousData (in the hook) avoids a flash on page change; `params` is stable per distinct URL
  // via useMemo([search]). Filter/sort/search writes drop `page` (→ resets to 1); only the pager sets it.
  const rows = data?.items ?? [];
  const total = data?.total ?? rows.length;

  const { openRail } = useShell();
  const withViewTransition = useViewTransition();
  const select = useCallback(
    (key: string | null) =>
      withViewTransition(() => {
        setSelectedKey(key);
        if (key) openRail();
      }),
    [withViewTransition, openRail]
  );

  // The non-facet slice (q/sort/order) carried across a facet change, with defaults stripped so they
  // never serialize. `page` is intentionally omitted → facet changes reset to page 1.
  const nonFacetSlice = useCallback((prev: ExploreApiSearch): ExploreApiSearch => {
    const base: ExploreApiSearch = {};
    if (prev.q) base.q = prev.q;
    if (prev.sort && prev.sort !== DEFAULT_SORT) base.sort = prev.sort;
    if (prev.order && prev.order !== DEFAULT_ORDER) base.order = prev.order;
    return base;
  }, []);

  const commitSearch = useCallback(
    (value: string) => {
      const next = value.trim();
      navigate({
        search: (prev: ExploreApiSearch) => {
          const out: ExploreApiSearch = { ...prev };
          delete out.page; // reset paging
          delete out.order; // both relevance and updated use their natural (desc) order
          // Search-as-you-type ranks by best text match; clearing reverts to recency (default sort).
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
      navigate({
        search: (prev: ExploreApiSearch) => {
          const prevSort = prev.sort ?? DEFAULT_SORT;
          const prevOrder = prev.order ?? DEFAULT_ORDER;
          // Clicking the active column toggles direction; a new column adopts its natural default.
          const nextOrder = prevSort === next ? (prevOrder === 'asc' ? 'desc' : 'asc') : NATURAL_ORDER[next];
          const out: ExploreApiSearch = { ...prev };
          delete out.page; // reset paging
          if (next === DEFAULT_SORT) delete out.sort;
          else out.sort = next;
          if (nextOrder === DEFAULT_ORDER) delete out.order;
          else out.order = nextOrder;
          return out;
        },
      });
    },
    [navigate]
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
          <button
            type="button"
            className="cat-sort-btn cat-toolbar-icon-btn"
            onClick={onReset}
            disabled={resetMode === 'none'}
            data-testid="cat-model-clear-all"
            data-test-state={resetMode}
            aria-label={
              resetMode === 'filters'
                ? 'Clear all filters'
                : resetMode === 'query'
                  ? 'Clear search'
                  : 'Nothing to reset'
            }
            title={
              resetMode === 'filters'
                ? 'Clear all filters'
                : resetMode === 'query'
                  ? 'Clear search'
                  : 'Nothing to reset'
            }
          >
            <ShellIcon name="rotate-ccw" size={13} />
          </button>
          <div className="cat-sortbar">
            <ColumnPicker hidden={hiddenColumns} onToggle={toggleColumn} />
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
          <table className="cat-table">
            <colgroup>
              {visibleColumns.map((col) => (
                <col key={col.key} style={col.width ? { width: col.width } : undefined} />
              ))}
            </colgroup>
            <thead className="cat-listhead" data-testid="cat-listhead">
              <tr>
                {visibleColumns.map((col) =>
                  col.sort ? (
                    <th key={col.key} scope="col" className={col.align === 'right' ? 'cat-th--right' : undefined}>
                      <ColSort
                        col={col.sort}
                        label={col.label}
                        sort={sort}
                        order={order}
                        align={col.align ?? 'left'}
                        onSort={onSort}
                      />
                    </th>
                  ) : (
                    <th
                      key={col.key}
                      scope="col"
                      className={`cat-colhead${col.align === 'right' ? ' cat-colhead--right' : ''}`}
                    >
                      {col.label}
                    </th>
                  )
                )}
              </tr>
            </thead>
            <tbody className="l-listview">
              {rows.map((m, i) => (
                <ModelRow
                  key={modelKey(m)}
                  model={m}
                  idx={(page - 1) * PAGE_SIZE + i + 1}
                  active={modelKey(m) === selectedKey}
                  columns={visibleColumns}
                  onSelect={() => select(modelKey(m))}
                />
              ))}
            </tbody>
          </table>
        )}
      </div>

      {total > PAGE_SIZE && (
        <ShellPagination total={total} page={page} onPage={onPage} pageSize={PAGE_SIZE} unit="models" />
      )}
    </div>
  );
}
