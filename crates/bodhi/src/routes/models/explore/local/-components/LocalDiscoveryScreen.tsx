import { useCallback, useMemo, useState } from 'react';

import type { ListModelsQuery, Model, Quant, SortKey } from '@bodhiapp/reference-api-types';

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
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useViewTransition } from '@/hooks/useViewTransition';
import { exploreBreadcrumb } from '@/routes/models/explore/-shared/breadcrumbs';

import { LocalDiscoveryRail, LocalDiscoveryRailHeader } from './LocalDiscoveryRail';
import { LocalRow, SortHeader } from './LocalDiscoveryRow';
import { LocalDiscoverySidebar, facetsToQuery, type DiscoveryFacets } from './LocalDiscoverySidebar';
import '@/components/downloads-panel/downloads-panel.css';
import '@/components/shell/list.css';
import '@/routes/models/-components/models.css';
import './local-discovery.css';

const BREADCRUMB = exploreBreadcrumb('Explore · Local Models');

/** Page size; raised when searching (search disables cursor pagination server-side). */
const PAGE_SIZE = 30;
const SEARCH_PAGE_SIZE = 100;

const SORT_LABELS: Record<SortKey, string> = {
  downloads: 'Downloads',
  likes: 'Likes',
  last_modified: 'Updated',
  created_at: 'Newest',
  trending: 'Trending',
};

function modelKey(m: Model): string {
  return `${m.namespace}/${m.repo}`;
}

export function LocalDiscoveryScreen() {
  useListKeyNav();
  const { openRail } = useShell();

  const [searchInput, setSearchInput] = useState('');
  const [search, setSearch] = useState('');
  const [sort, setSort] = useState<SortKey>('downloads');
  const [facets, setFacets] = useState<DiscoveryFacets>({});
  const [cursor, setCursor] = useState<string | undefined>(undefined);
  const [extraPages, setExtraPages] = useState<Model[]>([]);
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  // Which content the right rail shows. 'model' = selected-model detail, 'downloads' = Downloads panel.
  const [railMode, setRailMode] = useState<'model' | 'downloads' | null>(null);

  const withViewTransition = useViewTransition();
  const select = useCallback(
    (key: string | null) =>
      withViewTransition(() => {
        setSelectedKey(key);
        setRailMode(key ? 'model' : null);
      }),
    [withViewTransition]
  );

  const searching = search.trim() !== '';
  const params: ListModelsQuery = useMemo(
    () => ({
      sort,
      limit: searching ? SEARCH_PAGE_SIZE : PAGE_SIZE,
      ...facetsToQuery(facets),
      ...(searching ? { q: search.trim() } : {}),
      ...(cursor ? { cursor } : {}),
    }),
    [sort, searching, search, cursor, facets]
  );

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

  const resetPaging = useCallback(() => {
    setCursor(undefined);
    setExtraPages([]);
  }, []);

  const onSort = useCallback(
    (col: SortKey) => {
      setSort(col);
      resetPaging();
    },
    [resetPaging]
  );

  const onFacetsChange = useCallback(
    (next: DiscoveryFacets) => {
      setFacets(next);
      resetPaging();
    },
    [resetPaging]
  );

  // Browse presets set the sort key (Trending / New); order is always descending.
  const onBrowse = useCallback(
    (next: SortKey) => {
      setSort(next);
      resetPaging();
    },
    [resetPaging]
  );

  const onClearAllFacets = useCallback(() => {
    setFacets({});
    resetPaging();
  }, [resetPaging]);

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
    enablePolling: railMode === 'downloads',
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

  // Toggle: clicking while the Downloads panel is shown closes the rail; otherwise it shows
  // Downloads (swapping out the model-detail rail if that was open). Nulling the rail content
  // closes the column; `openRail` un-collapses it if the user had manually collapsed the rail.
  const toggleDownloads = useCallback(() => {
    setRailMode((mode) => {
      if (mode === 'downloads') return null;
      refreshDownloads();
      openRail();
      return 'downloads';
    });
  }, [openRail, refreshDownloads]);

  const closeRail = useCallback(() => setRailMode(null), []);

  const onArchiveDownload = useCallback((id: string) => archiveDownload({ id }), [archiveDownload]);
  const onRetryDownload = useCallback((id: string) => retryDownload({ id }), [retryDownload]);

  // Down-arrow from the Downloads rail hands focus to the main list (up-arrow does not return).
  const jumpToList = useCallback(() => {
    const target = document.querySelector<HTMLElement>('.l-listrow.active .l-rowlink, .l-listrow .l-rowlink');
    target?.focus();
  }, []);

  const sidebar = useMemo(
    () => (
      <LocalDiscoverySidebar
        facets={facets}
        sort={sort}
        onFacetsChange={onFacetsChange}
        onBrowse={onBrowse}
        onClearAll={onClearAllFacets}
      />
    ),
    [facets, sort, onFacetsChange, onBrowse, onClearAllFacets]
  );

  const railHeader = useMemo(() => {
    if (railMode === 'downloads') return <DownloadsPanelHeader onClose={closeRail} />;
    if (railMode === 'model' && selectedModel)
      return <LocalDiscoveryRailHeader model={selectedModel} onClose={() => select(null)} />;
    return null;
  }, [railMode, selectedModel, select, closeRail]);

  const rail = useMemo(() => {
    if (railMode === 'downloads')
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
    if (railMode === 'model' && selectedModel)
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
    railMode,
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
      className="ld-screen l-page"
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
          <button
            type="button"
            className={`l-iconbtn ld-dl-iconbtn${railMode === 'downloads' ? ' on' : ''}`}
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

      <div className="ld-resultbar" data-testid="ld-resultbar">
        <span className="ld-count">Showing {rows.length}</span>
        <span className="ld-sortlabel">
          sorted by <strong>{SORT_LABELS[sort]}</strong>
        </span>
      </div>

      <div className="ld-listhead">
        <div className="ld-lh-num">#</div>
        <div className="ld-lh-repo">REPOSITORY</div>
        <div className="ld-lh-stats">
          <SortHeader label="Downloads" col="downloads" sort={sort} onSort={onSort} />
          <SortHeader label="Likes" col="likes" sort={sort} onSort={onSort} />
          <SortHeader label="Updated" col="last_modified" sort={sort} onSort={onSort} />
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
          <div className="l-listview">
            {rows.map((m, i) => (
              <LocalRow
                key={modelKey(m)}
                model={m}
                idx={i + 1}
                sort={sort}
                active={modelKey(m) === selectedKey}
                onSelect={() => select(modelKey(m))}
              />
            ))}
            {showLoadMore && (
              <button type="button" className="ld-loadmore" onClick={loadMore} data-testid="ld-load-more">
                <ShellIcon name="chevrons-down" size={14} /> Load more
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
