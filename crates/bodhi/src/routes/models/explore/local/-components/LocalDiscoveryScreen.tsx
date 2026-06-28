import { useCallback, useEffect, useMemo, useState } from 'react';

import type { Model, Quant, SortKey } from '@bodhiapp/reference-api-types';
import { getRouteApi } from '@tanstack/react-router';

import { DownloadsPanel, DownloadsPanelHeader, isActive } from '@/components/downloads-panel/DownloadsPanel';
import { ShellIcon, ShellSearch, useListKeyNav, useShell, useShellChrome } from '@/components/shell';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import {
  useArchiveDownload,
  useDownloadsRefresh,
  useListDownloads,
  usePullModel,
  useRetryDownload,
} from '@/hooks/models';
import { useDiscoverModels, useModelDetail } from '@/hooks/reference';
import { useToastMessages } from '@/hooks/useToastMessages';
import { useViewTransition } from '@/hooks/useViewTransition';
import { exploreBreadcrumb } from '@/routes/models/explore/-shared/breadcrumbs';
import { type CatalogColumn, CatalogTable } from '@/routes/models/explore/-shared/catalog-table';
import { ResetButton } from '@/routes/models/explore/-shared/ResetButton';

import type { LocalDiscoverySearch } from '../index';

import { DEFAULT_SORT, facetsToSearch, searchToFacets, searchToParams } from './local-discovery-search';
import { fmtDate, LocalDiscoveryRail, LocalDiscoveryRailHeader } from './LocalDiscoveryRail';
import { compact, LocalRepoCell } from './LocalDiscoveryRow';
import { hasActiveFacets, LocalDiscoverySidebar, type DiscoveryFacets } from './LocalDiscoverySidebar';
import '@/components/downloads-panel/downloads-panel.css';
import '@/components/shell/list.css';
import '@/routes/models/-components/models.css';
import '@/routes/models/explore/-shared/catalog.css';
import './local-discovery.css';

const BREADCRUMB = exploreBreadcrumb('Explore · Local Models');

const routeApi = getRouteApi('/models/explore/local/');

function modelKey(m: Model): string {
  return `${m.namespace}/${m.repo}`;
}

/** Stable string of the non-cursor request slice; a change resets the keyset accumulator. */
function requestSliceKey(s: LocalDiscoverySearch): string {
  return JSON.stringify({ sort: s.sort ?? DEFAULT_SORT, q: s.q ?? '', f: searchToFacets(s) });
}

// `created_at`/`trending` are sidebar Browse presets, not columns. The repo column isn't a server
// sort key, so it has no `sort`. Stats are descending-only (see CatalogTable `descendingOnly`).
const COLUMNS: CatalogColumn<Model, SortKey>[] = [
  { key: 'num', label: '#', width: '44px', cell: () => null },
  { key: 'repo', label: 'REPOSITORY', width: '', cell: (m) => <LocalRepoCell model={m} /> },
  {
    key: 'downloads',
    label: 'DOWNLOADS',
    width: '110px',
    align: 'right',
    sort: 'downloads',
    cell: (m) => <div className="cat-num-cell">{compact(m.downloads)}</div>,
  },
  {
    key: 'likes',
    label: 'LIKES',
    width: '90px',
    align: 'right',
    sort: 'likes',
    cell: (m) => <div className="cat-num-cell">{compact(m.likes)}</div>,
  },
  {
    key: 'last_modified',
    label: 'UPDATED',
    width: '110px',
    align: 'right',
    sort: 'last_modified',
    cell: (m) => <div className="cat-num-cell">{fmtDate(m.last_modified)}</div>,
  },
];

export function LocalDiscoveryScreen() {
  useListKeyNav();
  const { openRail } = useShell();

  // URL is the single source of truth for sort/facets/search/select; the cursor + Load-More
  // accumulation stay in component state (cursors are opaque/volatile, never URL-synced).
  const search = routeApi.useSearch();
  const navigate = routeApi.useNavigate();

  const sort = search.sort ?? DEFAULT_SORT;
  const committedSearch = search.q ?? '';
  const facets = useMemo(() => searchToFacets(search), [search]);
  const selectedKey = search.select ?? null;

  const [searchInput, setSearchInput] = useState(committedSearch);
  const [cursor, setCursor] = useState<string | undefined>(undefined);
  const [extraPages, setExtraPages] = useState<Model[]>([]);
  // `downloadsOpen` is the only ephemeral rail state; the model-detail rail is driven by ?select.
  const [downloadsOpen, setDownloadsOpen] = useState(false);

  useEffect(() => {
    setSearchInput(committedSearch);
  }, [committedSearch]);

  // Reset the keyset accumulator whenever the non-cursor request slice changes (sort/facets/search),
  // so a filter change restarts the cursor stream from page 1.
  const sliceKey = requestSliceKey(search);
  useEffect(() => {
    setCursor(undefined);
    setExtraPages([]);
  }, [sliceKey]);

  const withViewTransition = useViewTransition();
  // Selection lives in the URL via replace (no history entries) — Back/Forward skips past selections.
  const select = useCallback(
    (key: string | null) => {
      if ((key ?? undefined) === search.select) return; // dedup
      withViewTransition(() => {
        setDownloadsOpen(false);
        navigate({
          search: (prev: LocalDiscoverySearch) => {
            const out: LocalDiscoverySearch = { ...prev };
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

  const searching = committedSearch.trim() !== '';
  const params = useMemo(() => searchToParams(search, cursor), [search, cursor]);

  const { data, isLoading, error } = useDiscoverModels(params);

  // Append accumulated "Load more" pages ahead of the current page, dedup by namespace/repo.
  const rows = useMemo(() => {
    const seen = new Set<string>();
    const out: Model[] = [];
    for (const m of [...extraPages, ...(data?.items ?? [])]) {
      const k = modelKey(m);
      if (seen.has(k)) continue;
      seen.add(k);
      out.push(m);
    }
    return out;
  }, [extraPages, data?.items]);

  const selectedModel = useMemo(() => rows.find((m) => modelKey(m) === selectedKey) ?? null, [rows, selectedKey]);
  const selectedRef = selectedModel
    ? { source: selectedModel.source, namespace: selectedModel.namespace, repo: selectedModel.repo }
    : null;
  const { data: detail, isLoading: detailLoading } = useModelDetail(selectedRef);

  // Carry the non-facet slice (q/sort/select) across a facet change; the keyset accumulator resets via
  // the sliceKey effect, so no imperative paging reset is needed here.
  const nonFacetSlice = useCallback((prev: LocalDiscoverySearch): LocalDiscoverySearch => {
    const base: LocalDiscoverySearch = {};
    if (prev.q) base.q = prev.q;
    if (prev.sort) base.sort = prev.sort;
    if (prev.select) base.select = prev.select;
    return base;
  }, []);

  // Selecting a sort omits the catalog default (downloads) so the URL stays clean.
  const setSortInUrl = useCallback(
    (col: SortKey) =>
      navigate({
        search: (prev: LocalDiscoverySearch) => {
          const out: LocalDiscoverySearch = { ...prev };
          if (col === DEFAULT_SORT) delete out.sort;
          else out.sort = col;
          return out;
        },
      }),
    [navigate]
  );
  const onSort = setSortInUrl;
  // Browse presets set the sort key (Trending / New); order is always descending.
  const onBrowse = setSortInUrl;

  const onFacetsChange = useCallback(
    (next: DiscoveryFacets) =>
      navigate({ search: (prev: LocalDiscoverySearch) => ({ ...nonFacetSlice(prev), ...facetsToSearch(next) }) }),
    [navigate, nonFacetSlice]
  );

  const onClearAllFacets = useCallback(
    () => navigate({ search: (prev: LocalDiscoverySearch) => nonFacetSlice(prev) }),
    [navigate, nonFacetSlice]
  );

  const commitSearch = useCallback(
    (value: string) => {
      const next = value.trim();
      navigate({
        search: (prev: LocalDiscoverySearch) => {
          const out: LocalDiscoverySearch = { ...prev };
          if (next) out.q = next;
          else delete out.q;
          return out;
        },
      });
    },
    [navigate]
  );

  // The toolbar reset waterfalls: clear active facets first, else the search query, else inert.
  const resetMode: 'filters' | 'query' | 'none' = hasActiveFacets(facets)
    ? 'filters'
    : committedSearch !== ''
      ? 'query'
      : 'none';
  const onReset = useCallback(() => {
    if (resetMode === 'filters') onClearAllFacets();
    else if (resetMode === 'query') commitSearch('');
  }, [resetMode, onClearAllFacets, commitSearch]);

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

  const loadMore = useCallback(() => {
    if (!data?.next_cursor) return;
    setExtraPages((prev) => {
      const seen = new Set(prev.map(modelKey));
      return [...prev, ...(data.items ?? []).filter((m) => !seen.has(modelKey(m)))];
    });
    setCursor(data.next_cursor);
  }, [data]);

  const { showSuccess, showError } = useToastMessages();
  const { mutate: pullModel, isPending: pullPending } = usePullModel({
    onSuccess: () => showSuccess('Download started', 'Track progress in the Downloads panel.'),
    onError: (message) => showError('Download failed', message),
  });

  const onPull = useCallback(
    (quant: Quant) => {
      if (!selectedModel) return;
      pullModel({ repo: `${selectedModel.namespace}/${selectedModel.repo}`, filename: quant.filename });
    },
    [selectedModel, pullModel]
  );

  // ── Downloads panel ──────────────────────────────────────────────
  const { data: downloadsData, isLoading: downloadsLoading } = useListDownloads(1, 100, {
    enablePolling: downloadsOpen,
  });
  const downloads = useMemo(() => downloadsData?.data ?? [], [downloadsData?.data]);
  const activeCount = useMemo(() => downloads.filter(isActive).length, [downloads]);

  const refreshDownloads = useDownloadsRefresh();
  const { mutate: archiveDownload, isPending: archivePending } = useArchiveDownload({
    onError: (message) => showError('Could not dismiss download', message),
  });
  const { mutate: retryDownload, isPending: retryPending } = useRetryDownload({
    onSuccess: () => showSuccess('Retrying download', 'Resuming from where it stopped.'),
    onError: (message) => showError('Retry failed', message),
  });
  const downloadsBusy = archivePending || retryPending;

  // Toggle: clicking while Downloads is shown closes it (falling back to the selected-model rail, if
  // any); otherwise it opens Downloads (taking over the rail). `openRail` un-collapses the column.
  const toggleDownloads = useCallback(() => {
    setDownloadsOpen((open) => {
      if (open) return false;
      refreshDownloads();
      openRail();
      return true;
    });
  }, [openRail, refreshDownloads]);

  const closeDownloads = useCallback(() => setDownloadsOpen(false), []);

  const onArchiveDownload = useCallback((id: string) => archiveDownload({ id }), [archiveDownload]);
  const onRetryDownload = useCallback((id: string) => retryDownload({ id }), [retryDownload]);

  // Down-arrow from the Downloads rail hands focus to the main list (up-arrow does not return).
  const jumpToList = useCallback(() => {
    const target = document.querySelector<HTMLElement>('.l-listrow.active .l-rowlink, .l-listrow .l-rowlink');
    target?.focus();
  }, []);

  const sidebar = useMemo(
    () => <LocalDiscoverySidebar facets={facets} sort={sort} onFacetsChange={onFacetsChange} onBrowse={onBrowse} />,
    [facets, sort, onFacetsChange, onBrowse]
  );

  const railHeader = useMemo(() => {
    if (downloadsOpen) return <DownloadsPanelHeader onClose={closeDownloads} />;
    if (selectedModel) return <LocalDiscoveryRailHeader model={selectedModel} onClose={() => select(null)} />;
    return null;
  }, [downloadsOpen, selectedModel, select, closeDownloads]);

  const rail = useMemo(() => {
    if (downloadsOpen)
      return (
        <DownloadsPanel
          items={downloads}
          loading={downloadsLoading}
          onArchive={onArchiveDownload}
          onRetry={onRetryDownload}
          busy={downloadsBusy}
          onJumpToList={jumpToList}
        />
      );
    if (selectedModel)
      return (
        <LocalDiscoveryRail
          model={selectedModel}
          detail={detail}
          loading={detailLoading}
          onPull={onPull}
          pullPending={pullPending}
        />
      );
    return null;
  }, [
    downloadsOpen,
    downloads,
    downloadsLoading,
    onArchiveDownload,
    onRetryDownload,
    downloadsBusy,
    jumpToList,
    selectedModel,
    detail,
    detailLoading,
    onPull,
    pullPending,
  ]);

  useShellChrome({
    breadcrumb: useMemo(() => BREADCRUMB, []),
    sidebar,
    rail,
    railHeader,
    railDefaultOpen: false,
  });

  if (error) {
    return <ErrorPage message={error instanceof Error ? error.message : 'Failed to load model catalog'} />;
  }

  const showLoadMore = !searching && !!data?.next_cursor && rows.length > 0;

  return (
    <div
      className="ld-screen cat-screen l-page"
      data-testid="local-discovery-content"
      data-pagestatus={isLoading ? 'loading' : 'ready'}
    >
      <div className="l-controls">
        <div className="m-toolbar">
          <div className="m-search" data-testid="ld-search">
            <ShellSearch
              value={searchInput}
              onChange={onSearchChange}
              onKeyDown={onSearchKeyDown}
              placeholder="Search HuggingFace repos"
              kbd="⌘K"
            />
          </div>
          <ResetButton mode={resetMode} onReset={onReset} testId="ld-clear-all" />
          <button
            type="button"
            className={`l-iconbtn ld-dl-iconbtn${downloadsOpen ? ' on' : ''}`}
            onClick={toggleDownloads}
            data-testid="ld-downloads-button"
            title="Open Downloads"
            aria-label="Open Downloads"
          >
            <ShellIcon name="download" size={15} />
            {activeCount > 0 && (
              <span className="ld-dl-badge" data-testid="ld-downloads-badge">
                {activeCount}
              </span>
            )}
          </button>
        </div>
      </div>

      <div className="l-scroll" data-testid="ld-list">
        {isLoading && rows.length === 0 ? (
          <div style={{ padding: 16 }} data-testid="ld-skeleton-container">
            {Array.from({ length: 6 }).map((_, i) => (
              <Skeleton key={i} className="h-14 w-full mb-3" data-testid="ld-skeleton" />
            ))}
          </div>
        ) : rows.length === 0 ? (
          <div className="empty-state" data-testid="ld-empty">
            <div className="empty-icon">
              <ShellIcon name="search-x" size={28} />
            </div>
            <div className="empty-title">No repositories match</div>
            <div className="empty-sub">Try a different search term.</div>
          </div>
        ) : (
          <>
            <CatalogTable<Model, SortKey>
              columns={COLUMNS}
              rows={rows}
              rowKey={modelKey}
              rowTestId={(m) => `ld-row-${m.namespace}-${m.repo}`}
              rowLabel={(m) => `Open ${m.namespace}/${m.repo}`}
              activeKey={selectedKey}
              onSelect={(m) => select(modelKey(m))}
              sort={sort}
              onSort={onSort}
              descendingOnly
              testIdPrefix="ld"
            />
            <div className="ld-listfoot">
              <span className="ld-count" data-testid="ld-count">
                Showing {rows.length}
              </span>
              {showLoadMore && (
                <button type="button" className="ld-loadmore" onClick={loadMore} data-testid="ld-load-more">
                  <ShellIcon name="chevrons-down" size={14} /> Load more
                </button>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
