import { useMemo } from 'react';

import type { ProviderSummary } from '@bodhiapp/reference-api-types';
import { getRouteApi } from '@tanstack/react-router';

import { EmptyState } from '@/components/EmptyState';
import { ShellPagination, ShellSearch, useListKeyNav, useShellChrome } from '@/components/shell';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { useCatalogProviderDetail, useCatalogProviderModels, useCatalogProviders } from '@/hooks/reference';
import { exploreBreadcrumb } from '@/routes/models/explore/-shared/breadcrumbs';
import {
  CAP_LABELS,
  CAP_TONE,
  fmtPrice,
  isFree,
  monogram,
  tintIndex,
} from '@/routes/models/explore/-shared/catalog-format';
import { type CatalogColumn, CatalogTable } from '@/routes/models/explore/-shared/catalog-table';
import { ColumnPicker, useHiddenColumns } from '@/routes/models/explore/-shared/ColumnPicker';
import { ResetButton } from '@/routes/models/explore/-shared/ResetButton';
import { useCatalogScreenState } from '@/routes/models/explore/-shared/useCatalogScreenState';

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

const COLUMNS: CatalogColumn<ProviderSummary, ProviderSort>[] = [
  { key: 'num', label: '#', width: '44px', cell: () => null },
  {
    key: 'logo',
    label: '',
    width: '38px',
    cell: (p) => (
      <div className={`cat-logo cat-tint-${tintIndex(p.slug)}`} aria-hidden="true">
        {monogram(p.name)}
      </div>
    ),
  },
  {
    key: 'provider',
    label: 'PROVIDER',
    width: '',
    sort: 'name',
    cell: (p) => {
      const free = isFree(p.pricing_summary.min_in_per_m, p.pricing_summary.min_out_per_m);
      return (
        <div className="cat-body">
          <div className="cat-name">
            {p.name}
            <span className="cat-shape">{p.provider_shape}</span>
          </div>
          <div className="cat-caps" style={{ marginTop: 6 }}>
            {p.capabilities_summary.map((c) => (
              <span className={`cap-chip cap-${CAP_TONE[c]}`} key={c}>
                {CAP_LABELS[c]}
              </span>
            ))}
          </div>
          <div className="cat-sub">
            {free ? 'Free tier available' : `from ${fmtPrice(p.pricing_summary.min_in_per_m)}/M in`}
          </div>
        </div>
      );
    },
  },
  {
    key: 'api_format',
    label: 'FORMAT',
    width: '120px',
    sort: 'api_format',
    optional: true,
    cell: (p) => <span className="cat-cell-text mono">{p.api_format_hint}</span>,
  },
  {
    key: 'model_count',
    label: 'MODELS',
    width: '88px',
    align: 'right',
    sort: 'model_count',
    optional: true,
    cell: (p) => (
      <div className="cat-score">
        <div className="cat-score-num">{p.model_count}</div>
        <div className="cat-score-lbl">MODELS</div>
      </div>
    ),
  },
];

export function ExploreProvidersScreen() {
  useListKeyNav();

  // URL search is the single source of truth; the only effect below writes LOCAL searchInput
  // (URL→input), never the URL, so there is no read→write loop.
  const search = routeApi.useSearch();
  const navigate = routeApi.useNavigate();

  const {
    facets,
    sort,
    order,
    page,
    committedSearch,
    selectedKey: selectedSlug,
    searchInput,
    onSearchChange,
    onSearchKeyDown,
    onSort,
    onFacetsChange,
    resetMode,
    onReset,
    onPage,
    select,
  } = useCatalogScreenState<ExploreProvidersSearch, ProviderFacets, ProviderSort>({
    search,
    navigate,
    searchToFacets,
    facetsToSearch,
    hasActiveFacets: hasActiveProviderFacets,
    sortConfig: {
      storageKey: SORT_STORAGE_KEY,
      persistedSorts: PERSISTED_SORTS,
      validOrders: VALID_ORDERS,
      naturalOrder: (s) => NATURAL_ORDER[s],
    },
  });

  const { hidden: hiddenColumns, toggle: toggleColumn, visibleColumns: filterVisible } = useHiddenColumns();
  const visibleColumns = useMemo(() => filterVisible(COLUMNS), [filterVisible]);

  const params = useMemo(() => searchToParams(search, { sort, order }), [search, sort, order]);
  const { data, isLoading, error } = useCatalogProviders(params);

  const rows = data?.items ?? [];
  const total = data?.total ?? rows.length;

  const sidebar = useMemo(
    () => (
      <ExploreProvidersSidebar
        facets={facets}
        capabilityValues={data?.facets.capability ?? []}
        apiFormatValues={data?.facets.api_format ?? []}
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
          <ResetButton mode={resetMode} onReset={onReset} testId="cat-prov-clear-all" />
          <div className="cat-sortbar">
            <ColumnPicker columns={COLUMNS} hidden={hiddenColumns} onToggle={toggleColumn} testIdPrefix="cat-prov" />
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
          <EmptyState
            icon="search-x"
            title="No providers found"
            sub="The catalog returned no providers."
            testId="cat-prov-empty"
          />
        ) : (
          <CatalogTable<ProviderSummary, ProviderSort>
            columns={visibleColumns}
            rows={rows}
            rowKey={(p) => p.slug}
            rowTestId={(p) => `cat-prov-row-${p.slug}`}
            rowLabel={(p) => `Open ${p.name}`}
            activeKey={selectedSlug}
            onSelect={(p) => select(p.slug)}
            sort={sort}
            order={order}
            onSort={onSort}
            startIndex={(page - 1) * PAGE_SIZE}
            testIdPrefix="cat-prov"
          />
        )}
      </div>

      {total > PAGE_SIZE && (
        <ShellPagination minimal total={total} page={page} onPage={onPage} pageSize={PAGE_SIZE} unit="providers" />
      )}
    </div>
  );
}
