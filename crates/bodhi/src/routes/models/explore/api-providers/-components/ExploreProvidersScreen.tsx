import { useCallback, useMemo, useState } from 'react';

import type { ListProvidersQuery, ProviderSummary } from '@bodhiapp/reference-api-types';

import { LinkRow, ShellIcon, useListKeyNav, useShellChrome } from '@/components/shell';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { useCatalogProviders } from '@/hooks/reference';

import { CAP_LABELS, CAP_TONE, fmtPrice, isFree, monogram, tintIndex } from '../../-shared/catalog-format';
import '@/components/shell/list.css';
import '@/routes/models/-components/models.css';
import '../../-shared/catalog.css';

const BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Models', href: '/models/' },
  { label: 'Explore · API Providers', current: true },
];

const PAGE_SIZE = 30;

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

  const [accumulated, setAccumulated] = useState<ProviderSummary[]>([]);
  const [page, setPage] = useState(1);
  const [selectedSlug, setSelectedSlug] = useState<string | null>(null);

  const params: ListProvidersQuery = useMemo(() => ({ sort: 'rank', page, page_size: PAGE_SIZE }), [page]);
  const { data, isLoading, error } = useCatalogProviders(params);

  // Page-based "Load more": prepend accumulated earlier pages, dedup by slug. (Catalog is page-based
  // with a real total — unlike Local's cursor.) Param changes (search/sort, added in B3) reset both
  // `page` and `accumulated` synchronously so a stale page-2 never lands on a new filter's page-1.
  const rows = useMemo(() => {
    const seen = new Set<string>();
    const out: ProviderSummary[] = [];
    for (const p of [...accumulated, ...(data?.items ?? [])]) {
      if (seen.has(p.slug)) continue;
      seen.add(p.slug);
      out.push(p);
    }
    return out;
  }, [accumulated, data?.items]);

  const total = data?.total ?? rows.length;
  const showLoadMore = rows.length < total;

  const loadMore = useCallback(() => {
    setAccumulated((prev) => {
      const seen = new Set(prev.map((p) => p.slug));
      return [...prev, ...(data?.items ?? []).filter((p) => !seen.has(p.slug))];
    });
    setPage((p) => p + 1);
  }, [data?.items]);

  const select = useCallback((slug: string | null) => setSelectedSlug(slug), []);

  useShellChrome({
    breadcrumb: useMemo(() => BREADCRUMB, []),
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
      <div className="cat-resultbar" data-testid="cat-prov-resultbar">
        <span className="cat-count">
          Showing {rows.length} of {total}
        </span>
        <span>
          sorted by <strong>Rank</strong>
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
                idx={i + 1}
                active={p.slug === selectedSlug}
                onSelect={() => select(p.slug)}
              />
            ))}
            {showLoadMore && (
              <button type="button" className="cat-loadmore" onClick={loadMore} data-testid="cat-prov-load-more">
                <ShellIcon name="chevrons-down" size={14} /> Load more
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
