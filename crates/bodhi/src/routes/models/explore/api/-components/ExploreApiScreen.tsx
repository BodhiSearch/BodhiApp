import { useCallback, useMemo, useState } from 'react';

import type { ListCatalogModelsQuery, ModelLite } from '@bodhiapp/reference-api-types';

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
import { useCatalogModelDetail, useCatalogModels } from '@/hooks/reference';
import { useViewTransition } from '@/hooks/useViewTransition';
import { exploreBreadcrumb } from '@/routes/models/explore/-shared/breadcrumbs';
import {
  CAP_LABELS,
  CAP_TONE,
  fmtContext,
  fmtPrice,
  isFree,
  statusLabel,
} from '@/routes/models/explore/-shared/catalog-format';

import { ExploreApiRail, ExploreApiRailHeader } from './ExploreApiRail';
import { ExploreApiSidebar, modelFacetsToQuery, type ModelFacetsState } from './ExploreApiSidebar';
import '@/components/shell/list.css';
import '@/routes/models/-components/models.css';
import '@/routes/models/explore/-shared/catalog.css';

const BREADCRUMB = exploreBreadcrumb('Explore · API Models');

const PAGE_SIZE = 30;

type ModelSort = NonNullable<ListCatalogModelsQuery['sort']>;
type SortOrder = NonNullable<ListCatalogModelsQuery['order']>;
const SORT_LABELS: Record<ModelSort, string> = {
  relevance: 'Relevance',
  updated: 'Newest',
  context: 'Context',
  price: 'Input price',
  price_out: 'Output price',
  name: 'Name',
  family: 'Family',
  providers: 'Providers',
};

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
  onSort,
}: {
  col: ModelSort;
  label: string;
  sort: ModelSort;
  order: SortOrder;
  onSort: (c: ModelSort) => void;
}) {
  const active = sort === col;
  const icon = !active ? 'chevrons-up-down' : order === 'asc' ? 'arrow-up' : 'arrow-down';
  return (
    <button
      type="button"
      className={`cat-colsort${active ? ' on' : ''}`}
      onClick={() => onSort(col)}
      data-testid={`cat-model-sort-${col}`}
      data-test-state={active ? 'active' : 'idle'}
    >
      {label}
      <ShellIcon name={icon} size={10} />
    </button>
  );
}

function modelKey(m: ModelLite): string {
  return `${m.slug}/${m.model_id}`;
}

function ModelRow({
  model,
  idx,
  active,
  onSelect,
}: {
  model: ModelLite;
  idx: number;
  active: boolean;
  onSelect: () => void;
}) {
  const free = isFree(model.pricing.input_per_m, model.pricing.output_per_m);
  return (
    <div
      className={`l-listrow cat-row cat-model-grid${active ? ' active' : ''}`}
      onClick={onSelect}
      role="option"
      aria-selected={active}
      data-testid={`cat-model-row-${model.slug}-${model.model_id}`}
    >
      <LinkRow onActivate={onSelect} label={`Open ${model.name}`} />
      <div className="cat-num">#{idx}</div>
      <div className="cat-body">
        <div className="cat-model-name">
          {model.name}
          {model.status && <span className={`cat-status cat-status-${model.status}`}>{statusLabel(model.status)}</span>}
        </div>
        {model.family && <div className="cat-model-family">{model.family}</div>}
      </div>
      <div className="cat-num-cell">{fmtContext(model.context_limit)}</div>
      <div className={`cat-num-cell${free ? ' free' : ''}`}>{free ? 'Free' : fmtPrice(model.pricing.input_per_m)}</div>
      <div className={`cat-num-cell${free ? ' free' : ''}`}>{free ? '' : fmtPrice(model.pricing.output_per_m)}</div>
      <div className="cat-caps">
        {model.caps.map((c) => (
          <span className={`cap-chip cap-${CAP_TONE[c]}`} key={c}>
            {CAP_LABELS[c]}
          </span>
        ))}
      </div>
      <div className="cat-score">
        <div className="cat-score-num">{model.provider_count}</div>
        <div className="cat-score-lbl">PROVIDERS</div>
      </div>
    </div>
  );
}

export function ExploreApiScreen() {
  useListKeyNav();

  const [page, setPage] = useState(1);
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const [searchInput, setSearchInput] = useState('');
  const [search, setSearch] = useState('');
  const [sort, setSort] = useState<ModelSort>('updated');
  const [order, setOrder] = useState<SortOrder>('desc');
  const [facets, setFacets] = useState<ModelFacetsState>({});

  const params: ListCatalogModelsQuery = useMemo(
    () => ({
      sort,
      order,
      page,
      page_size: PAGE_SIZE,
      ...(search ? { q: search } : {}),
      ...modelFacetsToQuery(facets),
    }),
    [sort, order, page, search, facets]
  );
  const { data, isLoading, error } = useCatalogModels(params);

  // Numbered pagination: render the current page directly (keepPreviousData avoids a flash on page
  // change). Reset to page 1 on any filter/sort/search change.
  const resetPaging = useCallback(() => setPage(1), []);
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

  const commitSearch = useCallback(
    (value: string) => {
      const next = value.trim();
      setSearch(next);
      // Search-as-you-type ranks by best text match; clearing reverts to recency.
      if (next) {
        setSort('relevance');
        setOrder(NATURAL_ORDER.relevance);
      } else {
        setSort('updated');
        setOrder(NATURAL_ORDER.updated);
      }
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
    (next: ModelSort) => {
      // Clicking the active column toggles direction; a new column adopts its natural default.
      setOrder((prev) => (sort === next ? (prev === 'asc' ? 'desc' : 'asc') : NATURAL_ORDER[next]));
      setSort(next);
      resetPaging();
    },
    [resetPaging, sort]
  );
  const onFacetsChange = useCallback(
    (next: ModelFacetsState) => {
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
      <ExploreApiSidebar
        facets={facets}
        facetCounts={data?.facets}
        onFacetsChange={onFacetsChange}
        onClearAll={onClearAllFacets}
      />
    ),
    [facets, data?.facets, onFacetsChange, onClearAllFacets]
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
          <div className="cat-sortbar">
            {(['updated', 'name', 'family'] as ModelSort[]).map((s) => (
              <button
                key={s}
                type="button"
                className={`cat-sort-btn${sort === s ? ' on' : ''}`}
                aria-pressed={sort === s}
                onClick={() => onSort(s)}
                data-testid={`cat-model-sort-${s}`}
                data-test-state={sort === s ? 'active' : 'idle'}
              >
                {SORT_LABELS[s]}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="cat-resultbar" data-testid="cat-model-resultbar">
        <span className="cat-count">
          Showing {rows.length} of {total}
        </span>
        <span>
          sorted by <strong>{SORT_LABELS[sort]}</strong> ({order === 'asc' ? 'asc' : 'desc'})
        </span>
      </div>

      <div className="cat-listhead cat-model-grid">
        <div>#</div>
        <div>MODEL</div>
        <ColSort col="context" label="CONTEXT" sort={sort} order={order} onSort={onSort} />
        <ColSort col="price" label="INPUT $" sort={sort} order={order} onSort={onSort} />
        <ColSort col="price_out" label="OUTPUT $" sort={sort} order={order} onSort={onSort} />
        <div>CAPABILITIES</div>
        <ColSort col="providers" label="PROVIDERS" sort={sort} order={order} onSort={onSort} />
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
          <div className="l-listview">
            {rows.map((m, i) => (
              <ModelRow
                key={modelKey(m)}
                model={m}
                idx={(page - 1) * PAGE_SIZE + i + 1}
                active={modelKey(m) === selectedKey}
                onSelect={() => select(modelKey(m))}
              />
            ))}
          </div>
        )}
      </div>

      {total > PAGE_SIZE && (
        <ShellPagination total={total} page={page} onPage={setPage} pageSize={PAGE_SIZE} unit="models" />
      )}
    </div>
  );
}
