import { useCallback, useEffect, useMemo, useState } from 'react';

import type { ProviderSummary } from '@bodhiapp/reference-api-types';
import { getRouteApi } from '@tanstack/react-router';

import { LinkRow, ShellIcon, ShellPagination, ShellSearch, useListKeyNav, useShellChrome } from '@/components/shell';
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
import { useCatalogProviderDetail, useCatalogProviderModels, useCatalogProviders } from '@/hooks/reference';
import { useViewTransition } from '@/hooks/useViewTransition';
import { exploreBreadcrumb } from '@/routes/models/explore/-shared/breadcrumbs';
import {
  CAP_LABELS,
  CAP_TONE,
  fmtPrice,
  isFree,
  monogram,
  tintIndex,
} from '@/routes/models/explore/-shared/catalog-format';
import { persistSortPreference, resolveSortPreference } from '@/routes/models/explore/-shared/useSortPreference';

import type { ExploreProvidersSearch } from '../index';

import { facetsToSearch, PAGE_SIZE, searchToFacets, searchToParams } from './explore-providers-search';
import { ExploreProvidersRail, ExploreProvidersRailHeader } from './ExploreProvidersRail';
import { ExploreProvidersSidebar, hasActiveProviderFacets, type ProviderFacets } from './ExploreProvidersSidebar';
import '@/components/shell/list.css';
import '@/routes/models/-components/models.css';
import '@/routes/models/explore/-shared/catalog.css';

const BREADCRUMB = exploreBreadcrumb('Explore · API Providers');

const routeApi = getRouteApi('/models/explore/providers/');

type ProviderSort = NonNullable<ExploreProvidersSearch['sort']>;
type SortOrder = NonNullable<ExploreProvidersSearch['order']>;

const NATURAL_ORDER: Record<ProviderSort, SortOrder> = {
  model_count: 'desc',
  name: 'asc',
  api_format: 'asc',
};

const SORT_STORAGE_KEY = 'bodhi.explore.providers.sort';
const PERSISTED_SORTS = ['name', 'model_count', 'api_format'] as const;
const VALID_ORDERS = ['asc', 'desc'] as const;

function ColSort({
  col,
  label,
  sort,
  order,
  align,
  onSort,
}: {
  col: ProviderSort;
  label: string;
  sort: ProviderSort | undefined;
  order: SortOrder | undefined;
  align: 'left' | 'right';
  onSort: (c: ProviderSort) => void;
}) {
  const active = sort === col;
  const icon = !active ? 'chevrons-up-down' : order === 'asc' ? 'arrow-up' : 'arrow-down';
  return (
    <button
      type="button"
      className={`cat-colsort${align === 'left' ? ' cat-colsort--left' : ''}${active ? ' on' : ''}`}
      onClick={() => onSort(col)}
      data-testid={`cat-prov-sort-${col}`}
      data-test-state={active ? 'active' : 'idle'}
    >
      <span className="cat-colsort-label">{label}</span>
      <ShellIcon name={icon} size={10} />
    </button>
  );
}

// Toggleable table columns (the `#`, logo, and PROVIDER columns are always shown).
const OPTIONAL_COLUMNS: { key: string; label: string }[] = [
  { key: 'api_format', label: 'Format' },
  { key: 'model_count', label: 'Models' },
];

function ColumnPicker({ hidden, onToggle }: { hidden: Set<string>; onToggle: (key: string) => void }) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          className="cat-sort-btn cat-toolbar-icon-btn"
          data-testid="cat-prov-columns"
          aria-label="Columns"
          title="Columns"
        >
          <ShellIcon name="columns-3" size={13} />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuLabel>Columns</DropdownMenuLabel>
        <DropdownMenuSeparator />
        {OPTIONAL_COLUMNS.map((col) => (
          <DropdownMenuCheckboxItem
            key={col.key}
            checked={!hidden.has(col.key)}
            onCheckedChange={() => onToggle(col.key)}
            onSelect={(e) => e.preventDefault()}
            data-testid={`cat-prov-col-${col.key}`}
          >
            {col.label}
          </DropdownMenuCheckboxItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function ProviderRow({
  provider,
  idx,
  active,
  hidden,
  onSelect,
}: {
  provider: ProviderSummary;
  idx: number;
  active: boolean;
  hidden: Set<string>;
  onSelect: () => void;
}) {
  const free = isFree(provider.pricing_summary.min_in_per_m, provider.pricing_summary.min_out_per_m);
  return (
    <tr
      className={`l-listrow cat-row${active ? ' active' : ''}`}
      onClick={onSelect}
      role="option"
      aria-selected={active}
      data-testid={`cat-prov-row-${provider.slug}`}
    >
      <td className="cat-num-td">
        <LinkRow onActivate={onSelect} label={`Open ${provider.name}`} />
        <span className="cat-num">#{idx}</span>
      </td>
      <td>
        <div className={`cat-logo cat-tint-${tintIndex(provider.slug)}`} aria-hidden="true">
          {monogram(provider.name)}
        </div>
      </td>
      <td>
        <div className="cat-body">
          <div className="cat-name">
            {provider.name}
            <span className="cat-shape">{provider.provider_shape}</span>
          </div>
          <div className="cat-caps" style={{ marginTop: 6 }}>
            {provider.capabilities_summary.map((c) => (
              <span className={`cap-chip cap-${CAP_TONE[c]}`} key={c}>
                {CAP_LABELS[c]}
              </span>
            ))}
          </div>
          <div className="cat-sub">
            {free ? 'Free tier available' : `from ${fmtPrice(provider.pricing_summary.min_in_per_m)}/M in`}
          </div>
        </div>
      </td>
      {!hidden.has('api_format') && (
        <td>
          <span className="cat-cell-text mono">{provider.api_format_hint}</span>
        </td>
      )}
      {!hidden.has('model_count') && (
        <td className="cat-td--right">
          <div className="cat-score">
            <div className="cat-score-num">{provider.model_count}</div>
            <div className="cat-score-lbl">MODELS</div>
          </div>
        </td>
      )}
    </tr>
  );
}

export function ExploreProvidersScreen() {
  useListKeyNav();

  // URL search is the single source of truth; the only effect below writes LOCAL searchInput
  // (URL→input), never the URL, so there is no read→write loop.
  const search = routeApi.useSearch();
  const navigate = routeApi.useNavigate();

  // Effective sort precedence: URL > localStorage (request-only, never written to URL) > none.
  const resolvedSort = resolveSortPreference<ProviderSort, SortOrder>({
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

  // The open detail rail is the URL's `select` (provider slug). Deriving it (not mirroring in state)
  // makes Back/Forward restoration and the ?select cross-link automatic.
  const selectedSlug = search.select ?? null;
  const [searchInput, setSearchInput] = useState(committedSearch);
  const [hiddenColumns, setHiddenColumns] = useState<Set<string>>(() => new Set());
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

  const params = useMemo(() => searchToParams(search, { sort, order }), [search, sort, order]);
  const { data, isLoading, error } = useCatalogProviders(params);

  const rows = data?.items ?? [];
  const total = data?.total ?? rows.length;

  const withViewTransition = useViewTransition();
  // Selection lives in the URL via replace (no history entries). The rail auto-opens/closes from its
  // content presence, so no openRail() call is needed.
  const select = useCallback(
    (slug: string | null) => {
      if ((slug ?? undefined) === search.select) return; // dedup
      withViewTransition(() => {
        navigate({
          search: (prev: ExploreProvidersSearch) => {
            const out: ExploreProvidersSearch = { ...prev };
            if (slug) out.select = slug;
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
  const nonFacetSlice = useCallback((prev: ExploreProvidersSearch): ExploreProvidersSearch => {
    const base: ExploreProvidersSearch = {};
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
        search: (prev: ExploreProvidersSearch) => {
          const out: ExploreProvidersSearch = { ...prev };
          delete out.page;
          if (next) out.q = next;
          else delete out.q;
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
    (next: ProviderSort) => {
      // Clicking the active sort toggles direction; a new sort adopts its natural default.
      const nextOrder: SortOrder = sort === next ? (order === 'asc' ? 'desc' : 'asc') : NATURAL_ORDER[next];
      persistSortPreference(SORT_STORAGE_KEY, next, nextOrder);
      navigate({
        search: (prev: ExploreProvidersSearch) => {
          const out: ExploreProvidersSearch = { ...prev };
          delete out.page;
          out.sort = next;
          if (nextOrder === NATURAL_ORDER[next]) delete out.order;
          else out.order = nextOrder;
          return out;
        },
      });
    },
    [navigate, sort, order]
  );
  const onFacetsChange = useCallback(
    (next: ProviderFacets) =>
      navigate({ search: (prev: ExploreProvidersSearch) => ({ ...nonFacetSlice(prev), ...facetsToSearch(next) }) }),
    [navigate, nonFacetSlice]
  );
  const onClearAllFacets = useCallback(
    () => navigate({ search: (prev: ExploreProvidersSearch) => nonFacetSlice(prev) }),
    [navigate, nonFacetSlice]
  );

  // The toolbar reset waterfalls: clear active filters first, else the search query, else inert.
  const hasFilters = hasActiveProviderFacets(facets);
  const hasQuery = committedSearch !== '';
  const resetMode: 'filters' | 'query' | 'none' = hasFilters ? 'filters' : hasQuery ? 'query' : 'none';
  const onReset = useCallback(() => {
    if (resetMode === 'filters') onClearAllFacets();
    else if (resetMode === 'query') commitSearch('');
  }, [resetMode, onClearAllFacets, commitSearch]);

  const onPage = useCallback(
    (p: number) =>
      navigate({
        search: (prev: ExploreProvidersSearch) => (p === 1 ? { ...prev, page: undefined } : { ...prev, page: p }),
      }),
    [navigate]
  );

  const sidebar = useMemo(
    () => (
      <ExploreProvidersSidebar
        facets={facets}
        capabilityCounts={data?.facets.capability ?? {}}
        apiFormatCounts={data?.facets.api_format ?? {}}
        onFacetsChange={onFacetsChange}
      />
    ),
    [facets, data?.facets.capability, data?.facets.api_format, onFacetsChange]
  );

  // The ?select cross-link (from the API Models "Served by" list) and deep links open the rail purely
  // by deriving selectedSlug above — no effect needed.

  const { data: detail, isLoading: detailLoading } = useCatalogProviderDetail(selectedSlug);
  const { data: providerModels, isLoading: modelsLoading } = useCatalogProviderModels(selectedSlug, {});

  // Prefer the list-row summary; fall back to one synthesized from the detail fetch when the
  // selected provider isn't on the currently-loaded list page (deep-link / cross-link case).
  const selectedProvider: ProviderSummary | null = useMemo(() => {
    const fromList = rows.find((p) => p.slug === selectedSlug);
    if (fromList) return fromList;
    if (selectedSlug && detail && detail.slug === selectedSlug) {
      return {
        slug: detail.slug,
        name: detail.name,
        logo_url: detail.logo_url,
        model_count: detail.model_count,
        rank: 0,
        is_lab: false,
        api_base_url: detail.api_base_url,
        provider_shape: detail.provider_shape,
        // BridgeApiFormat is a subset of ApiFormatHint (no 'other'); widen for the summary field.
        api_format_hint: detail.bridge.api_format as ProviderSummary['api_format_hint'],
        capabilities_summary: [],
        pricing_summary: { min_in_per_m: null, min_out_per_m: null },
      };
    }
    return null;
  }, [rows, selectedSlug, detail]);

  const railHeader = useMemo(
    () =>
      selectedProvider ? <ExploreProvidersRailHeader provider={selectedProvider} onClose={() => select(null)} /> : null,
    [selectedProvider, select]
  );

  const rail = useMemo(
    () =>
      selectedProvider ? (
        <ExploreProvidersRail
          provider={selectedProvider}
          detail={detail}
          detailLoading={detailLoading}
          models={providerModels?.items ?? []}
          modelsLoading={modelsLoading}
        />
      ) : null,
    [selectedProvider, detail, detailLoading, providerModels?.items, modelsLoading]
  );

  useShellChrome({
    breadcrumb: useMemo(() => BREADCRUMB, []),
    sidebar,
    rail,
    railHeader,
    railDefaultOpen: false,
  });

  if (error) {
    return <ErrorPage message={error instanceof Error ? error.message : 'Failed to load the provider catalog'} />;
  }

  return (
    <div
      className="cat-screen l-page"
      data-testid="explore-providers-content"
      data-pagestatus={isLoading ? 'loading' : 'ready'}
    >
      <div className="l-controls">
        <div className="m-toolbar">
          <div className="m-search" data-testid="cat-prov-search">
            <ShellSearch
              value={searchInput}
              onChange={onSearchChange}
              onKeyDown={onSearchKeyDown}
              placeholder="Search providers or the models they serve"
              kbd="⌘K"
            />
          </div>
          <button
            type="button"
            className="cat-sort-btn cat-toolbar-icon-btn"
            onClick={onReset}
            disabled={resetMode === 'none'}
            data-testid="cat-prov-clear-all"
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

      <div className="l-scroll" data-testid="cat-prov-list">
        {isLoading && rows.length === 0 ? (
          <div style={{ padding: 16 }} data-testid="cat-prov-skeleton-container">
            {Array.from({ length: 6 }).map((_, i) => (
              <Skeleton key={i} className="h-16 w-full mb-3" data-testid="cat-prov-skeleton" />
            ))}
          </div>
        ) : rows.length === 0 ? (
          <div className="empty-state" data-testid="cat-prov-empty">
            <div className="empty-icon">
              <ShellIcon name="search-x" size={28} />
            </div>
            <div className="empty-title">No providers found</div>
            <div className="empty-sub">The catalog returned no providers.</div>
          </div>
        ) : (
          <table className="cat-table">
            <colgroup>
              <col style={{ width: '44px' }} />
              <col style={{ width: '38px' }} />
              <col />
              {!hiddenColumns.has('api_format') && <col style={{ width: '120px' }} />}
              {!hiddenColumns.has('model_count') && <col style={{ width: '88px' }} />}
            </colgroup>
            <thead className="cat-listhead" data-testid="cat-listhead">
              <tr>
                <th scope="col">#</th>
                <th scope="col" aria-hidden="true" />
                <th scope="col">
                  <ColSort col="name" label="PROVIDER" sort={sort} order={order} align="left" onSort={onSort} />
                </th>
                {!hiddenColumns.has('api_format') && (
                  <th scope="col">
                    <ColSort col="api_format" label="FORMAT" sort={sort} order={order} align="left" onSort={onSort} />
                  </th>
                )}
                {!hiddenColumns.has('model_count') && (
                  <th scope="col" className="cat-th--right">
                    <ColSort col="model_count" label="MODELS" sort={sort} order={order} align="right" onSort={onSort} />
                  </th>
                )}
              </tr>
            </thead>
            <tbody className="l-listview">
              {rows.map((p, i) => (
                <ProviderRow
                  key={p.slug}
                  provider={p}
                  idx={(page - 1) * PAGE_SIZE + i + 1}
                  active={p.slug === selectedSlug}
                  hidden={hiddenColumns}
                  onSelect={() => select(p.slug)}
                />
              ))}
            </tbody>
          </table>
        )}
      </div>

      {total > PAGE_SIZE && (
        <ShellPagination minimal total={total} page={page} onPage={onPage} pageSize={PAGE_SIZE} unit="providers" />
      )}
    </div>
  );
}
