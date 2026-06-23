import { useCallback, useMemo, useState } from 'react';

import type { ListCatalogModelsQuery, ModelLite } from '@bodhiapp/reference-api-types';

import { LinkRow, ShellIcon, ShellSearch, useListKeyNav, useShell, useShellChrome } from '@/components/shell';
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
const SORT_LABELS: Record<ModelSort, string> = {
  updated: 'Newest',
  context: 'Context',
  price: 'Price',
  name: 'Name',
  providers: 'Providers',
};
function ColSort({
  col,
  label,
  sort,
  onSort,
}: {
  col: ModelSort;
  label: string;
  sort: ModelSort;
  onSort: (c: ModelSort) => void;
}) {
  const active = sort === col;
  return (
    <button
      type="button"
      className={`cat-colsort${active ? ' on' : ''}`}
      onClick={() => onSort(col)}
      data-testid={`cat-model-sort-${col}`}
      data-test-state={active ? 'active' : 'idle'}
    >
      {label}
      <ShellIcon name={active ? 'arrow-down' : 'chevrons-up-down'} size={10} />
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

  const [accumulated, setAccumulated] = useState<ModelLite[]>([]);
  const [page, setPage] = useState(1);
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const [searchInput, setSearchInput] = useState('');
  const [search, setSearch] = useState('');
  const [sort, setSort] = useState<ModelSort>('updated');
  const [facets, setFacets] = useState<ModelFacetsState>({});

  const params: ListCatalogModelsQuery = useMemo(
    () => ({
      sort,
      page,
      page_size: PAGE_SIZE,
      ...(search ? { q: search } : {}),
      ...modelFacetsToQuery(facets),
    }),
    [sort, page, search, facets]
  );
  const { data, isLoading, error } = useCatalogModels(params);

  // Reset paging synchronously on any filter/sort/search change so keepPreviousData can't append a
  // stale page-2 onto a new query's page-1.
  const resetPaging = useCallback(() => {
    setAccumulated([]);
    setPage(1);
  }, []);

  // Page-based "Load more": accumulate earlier pages, dedup by slug/model_id. (Catalog is page-based
  // with a real total — unlike Local's cursor.)
  const rows = useMemo(() => {
    const seen = new Set<string>();
    const out: ModelLite[] = [];
    for (const m of [...accumulated, ...(data?.items ?? [])]) {
      const k = modelKey(m);
      if (seen.has(k)) continue;
      seen.add(k);
      out.push(m);
    }
    return out;
  }, [accumulated, data?.items]);

  const total = data?.total ?? rows.length;
  const showLoadMore = rows.length < total;

  const loadMore = useCallback(() => {
    setAccumulated((prev) => {
      const seen = new Set(prev.map(modelKey));
      return [...prev, ...(data?.items ?? []).filter((m) => !seen.has(modelKey(m)))];
    });
    setPage((p) => p + 1);
  }, [data?.items]);

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
    (next: ModelSort) => {
      setSort(next);
      resetPaging();
    },
    [resetPaging]
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
            {(['updated', 'name'] as ModelSort[]).map((s) => (
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
          sorted by <strong>{SORT_LABELS[sort]}</strong>
        </span>
      </div>

      <div className="cat-listhead cat-model-grid">
        <div>#</div>
        <div>MODEL</div>
        <ColSort col="context" label="CONTEXT" sort={sort} onSort={onSort} />
        <ColSort col="price" label="INPUT $" sort={sort} onSort={onSort} />
        <div style={{ textAlign: 'right' }}>OUTPUT $</div>
        <div>CAPABILITIES</div>
        <ColSort col="providers" label="PROVIDERS" sort={sort} onSort={onSort} />
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
                idx={i + 1}
                active={modelKey(m) === selectedKey}
                onSelect={() => select(modelKey(m))}
              />
            ))}
            {showLoadMore && (
              <button type="button" className="cat-loadmore" onClick={loadMore} data-testid="cat-model-load-more">
                <ShellIcon name="chevrons-down" size={14} /> Load more
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
