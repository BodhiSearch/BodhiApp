import { useCallback, useEffect, useMemo, useState } from 'react';

import type { ListProvidersQuery, ProviderSummary } from '@bodhiapp/reference-api-types';
import { useSearch } from '@tanstack/react-router';

import {
  LinkRow,
  ShellIcon,
  ShellPagination,
  ShellSearch,
  useListKeyNav,
  useShell,
  useShellChrome,
} from '@/components/shell';
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

import { ExploreProvidersRail, ExploreProvidersRailHeader } from './ExploreProvidersRail';
import { ExploreProvidersSidebar, providerFacetsToQuery, type ProviderFacets } from './ExploreProvidersSidebar';
import '@/components/shell/list.css';
import '@/routes/models/-components/models.css';
import '@/routes/models/explore/-shared/catalog.css';

const BREADCRUMB = exploreBreadcrumb('Explore · API Providers');

const PAGE_SIZE = 30;

type ProviderSort = NonNullable<ListProvidersQuery['sort']>;
type SortOrder = NonNullable<ListProvidersQuery['order']>;
const SORT_LABELS: Record<ProviderSort, string> = {
  rank: 'Rank',
  name: 'Name',
  model_count: 'Models',
  api_format: 'Format',
  pricing: 'Cheapest',
};

// Backend natural direction per provider sort key (docs: endpoints.md "Sorts").
const NATURAL_ORDER: Record<ProviderSort, SortOrder> = {
  rank: 'desc',
  model_count: 'desc',
  name: 'asc',
  api_format: 'asc',
  pricing: 'asc',
};

function ProviderRow({
  provider,
  idx,
  active,
  onSelect,
}: {
  provider: ProviderSummary;
  idx: number;
  active: boolean;
  onSelect: () => void;
}) {
  const free = isFree(provider.pricing_summary.min_in_per_m, provider.pricing_summary.min_out_per_m);
  return (
    <div
      className={`l-listrow cat-row cat-prov-grid${active ? ' active' : ''}`}
      onClick={onSelect}
      role="option"
      aria-selected={active}
      data-testid={`cat-prov-row-${provider.slug}`}
    >
      <LinkRow onActivate={onSelect} label={`Open ${provider.name}`} />
      <div className="cat-num">#{idx}</div>
      <div className={`cat-logo cat-tint-${tintIndex(provider.slug)}`} aria-hidden="true">
        {monogram(provider.name)}
      </div>
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
      <div className="cat-score">
        <div className="cat-score-num">{provider.model_count}</div>
        <div className="cat-score-lbl">MODELS</div>
      </div>
    </div>
  );
}

export function ExploreProvidersScreen() {
  useListKeyNav();

  const [page, setPage] = useState(1);
  const [selectedSlug, setSelectedSlug] = useState<string | null>(null);
  const [searchInput, setSearchInput] = useState('');
  const [search, setSearch] = useState('');
  const [sort, setSort] = useState<ProviderSort>('rank');
  const [order, setOrder] = useState<SortOrder>('desc');
  const [facets, setFacets] = useState<ProviderFacets>({});

  const params: ListProvidersQuery = useMemo(
    () => ({
      sort,
      order,
      page,
      page_size: PAGE_SIZE,
      ...(search ? { q: search } : {}),
      ...providerFacetsToQuery(facets),
    }),
    [sort, order, page, search, facets]
  );
  const { data, isLoading, error } = useCatalogProviders(params);

  // Numbered pagination: render the current page directly (keepPreviousData avoids a flash on page
  // change). Reset to page 1 on any filter/sort/search change.
  const resetPaging = useCallback(() => setPage(1), []);
  const rows = data?.items ?? [];
  const total = data?.total ?? rows.length;

  const { openRail } = useShell();
  const withViewTransition = useViewTransition();
  const select = useCallback(
    (slug: string | null) =>
      withViewTransition(() => {
        setSelectedSlug(slug);
        if (slug) openRail();
      }),
    [withViewTransition, openRail]
  );

  const commitSearch = useCallback(
    (value: string) => {
      setSearch(value.trim());
      resetPaging();
    },
    [resetPaging]
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
      setOrder((prev) => (sort === next ? (prev === 'asc' ? 'desc' : 'asc') : NATURAL_ORDER[next]));
      setSort(next);
      resetPaging();
    },
    [resetPaging, sort]
  );
  const onFacetsChange = useCallback(
    (next: ProviderFacets) => {
      setFacets(next);
      resetPaging();
    },
    [resetPaging]
  );
  const onClearAllFacets = useCallback(() => {
    setFacets({});
    resetPaging();
  }, [resetPaging]);

  const sidebar = useMemo(
    () => (
      <ExploreProvidersSidebar
        facets={facets}
        capabilityCounts={data?.facets.capability ?? {}}
        apiFormatCounts={data?.facets.api_format ?? {}}
        onFacetsChange={onFacetsChange}
        onClearAll={onClearAllFacets}
      />
    ),
    [facets, data?.facets.capability, data?.facets.api_format, onFacetsChange, onClearAllFacets]
  );

  // Cross-link entry: /api-providers?select=<slug> (from the API Models "Served by" list) opens
  // that provider's rail on mount.
  const selectParam = useSearch({
    strict: false,
    select: (s: Record<string, unknown>) => s.select as string | undefined,
  });
  useEffect(() => {
    if (selectParam) {
      setSelectedSlug(selectParam);
      openRail();
    }
  }, [selectParam, openRail]);

  const { data: detail, isLoading: detailLoading } = useCatalogProviderDetail(selectedSlug);
  const { data: providerModels, isLoading: modelsLoading } = useCatalogProviderModels(selectedSlug);

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
        api_base_url: detail.api_base_url,
        provider_shape: detail.provider_shape,
        api_format_hint: detail.bridge.api_format,
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
              placeholder="Search providers"
              kbd="⌘K"
            />
          </div>
          <div className="cat-sortbar">
            {(Object.keys(SORT_LABELS) as ProviderSort[]).map((s) => (
              <button
                key={s}
                type="button"
                className={`cat-sort-btn${sort === s ? ' on' : ''}`}
                aria-pressed={sort === s}
                onClick={() => onSort(s)}
                data-testid={`cat-prov-sort-${s}`}
                data-test-state={sort === s ? 'active' : 'idle'}
              >
                {SORT_LABELS[s]}
                {sort === s && <ShellIcon name={order === 'asc' ? 'arrow-up' : 'arrow-down'} size={10} />}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="cat-resultbar" data-testid="cat-prov-resultbar">
        <span className="cat-count">
          Showing {rows.length} of {total}
        </span>
        <span>
          sorted by <strong>{SORT_LABELS[sort]}</strong> ({order === 'asc' ? 'asc' : 'desc'})
        </span>
      </div>

      <div className="cat-listhead cat-prov-grid">
        <div>#</div>
        <div />
        <div>PROVIDER</div>
        <div style={{ textAlign: 'right' }}>MODELS</div>
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
          <div className="l-listview">
            {rows.map((p, i) => (
              <ProviderRow
                key={p.slug}
                provider={p}
                idx={(page - 1) * PAGE_SIZE + i + 1}
                active={p.slug === selectedSlug}
                onSelect={() => select(p.slug)}
              />
            ))}
          </div>
        )}
      </div>

      {total > PAGE_SIZE && (
        <ShellPagination minimal total={total} page={page} onPage={setPage} pageSize={PAGE_SIZE} unit="providers" />
      )}
    </div>
  );
}
