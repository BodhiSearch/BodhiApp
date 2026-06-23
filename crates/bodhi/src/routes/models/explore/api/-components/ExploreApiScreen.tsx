import { useCallback, useMemo, useState } from 'react';

import type { ListCatalogModelsQuery, ModelLite } from '@bodhiapp/reference-api-types';

import { LinkRow, ShellIcon, useListKeyNav, useShellChrome } from '@/components/shell';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { useCatalogModels } from '@/hooks/reference';

import { CAP_LABELS, CAP_TONE, fmtContext, fmtPrice, isFree, statusLabel } from '../../-shared/catalog-format';
import '@/components/shell/list.css';
import '@/routes/models/-components/models.css';
import '../../-shared/catalog.css';

const BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Models', href: '/models/' },
  { label: 'Explore · API Models', current: true },
];

const PAGE_SIZE = 30;

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

  const params: ListCatalogModelsQuery = useMemo(() => ({ sort: 'updated', page, page_size: PAGE_SIZE }), [page]);
  const { data, isLoading, error } = useCatalogModels(params);

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

  const select = useCallback((key: string | null) => setSelectedKey(key), []);

  useShellChrome({
    breadcrumb: useMemo(() => BREADCRUMB, []),
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
      <div className="cat-resultbar" data-testid="cat-model-resultbar">
        <span className="cat-count">
          Showing {rows.length} of {total}
        </span>
        <span>
          sorted by <strong>Newest</strong>
        </span>
      </div>

      <div className="cat-listhead cat-model-grid">
        <div>#</div>
        <div>MODEL</div>
        <div style={{ textAlign: 'right' }}>CONTEXT</div>
        <div style={{ textAlign: 'right' }}>INPUT $</div>
        <div style={{ textAlign: 'right' }}>OUTPUT $</div>
        <div>CAPABILITIES</div>
        <div style={{ textAlign: 'right' }}>PROVIDERS</div>
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
