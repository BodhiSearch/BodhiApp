import { useCallback, useEffect, useMemo, useState } from 'react';

import { AliasResponse, ApiAliasResponse } from '@bodhiapp/ts-client';
import { getRouteApi, useNavigate } from '@tanstack/react-router';

import { DownloadsPanel, DownloadsPanelHeader, isActive } from '@/components/downloads-panel/DownloadsPanel';
import { EmptyState } from '@/components/EmptyState';
import { ShellIcon, ShellPagination, ShellSearch, useListKeyNav, useShell, useShellChrome } from '@/components/shell';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import {
  ModelsFilter,
  useArchiveDownload,
  useDownloadsRefresh,
  useListDownloads,
  useListModels,
  useRetryDownload,
} from '@/hooks/models';
import { useToastMessages } from '@/hooks/useToastMessages';
import { useViewTransition } from '@/hooks/useViewTransition';
import { extractErrorMessage } from '@/lib/errorUtils';
import { isApiAlias, isModelRouterAlias, isUserAlias } from '@/lib/utils';
import { type CatalogColumn, CatalogTable } from '@/routes/models/explore/-shared/catalog-table';
import { ColumnPicker, useHiddenColumns } from '@/routes/models/explore/-shared/ColumnPicker';
import { ResetButton } from '@/routes/models/explore/-shared/ResetButton';
import { persistSortPreference, resolveSortPreference } from '@/routes/models/explore/-shared/useSortPreference';

import type { ModelsSearch } from '../index';

import { getAliasId, getAliasTitle, getAliasTypeMeta } from './aliasFormatters';
import { ModelDetailRail, ModelRailHeader } from './ModelDetailRail';
import { facetsToSearch, hasActiveFilter, PAGE_SIZE, searchToFilter, searchToListArgs } from './models-search';
import { ModelSidebarFacets } from './ModelSidebarFacets';
import '@/components/downloads-panel/downloads-panel.css';
import '@/components/shell/api-keys.css';
import '@/components/shell/list.css';
import '@/routes/models/explore/-shared/catalog.css';
import './models.css';

const MODELS_BREADCRUMB = [{ label: 'Bodhi' }, { label: 'Models' }, { label: 'My Models', current: true }];

const routeApi = getRouteApi('/models/');

type ModelSort = NonNullable<ModelsSearch['sort']>;
type SortOrder = NonNullable<ModelsSearch['order']>;

// Natural direction per sort key; selecting a new column applies its default, clicking the active
// column toggles it.
const NATURAL_ORDER: Record<ModelSort, SortOrder> = { name: 'asc', provider: 'asc', base_url: 'asc' };
const SORT_STORAGE_KEY = 'bodhi.models.sort';
const PERSISTED_SORTS = ['name', 'provider', 'base_url'] as const;
const VALID_ORDERS = ['asc', 'desc'] as const;

// ── Universal column derivations (one value per alias type) ──────────────────────────────────────
// Provider/Repo: repo (local+user-alias), api_format (api), first target's alias (router).
function aliasProvider(alias: AliasResponse): string {
  if (isApiAlias(alias)) return alias.api_format.toUpperCase();
  if (isModelRouterAlias(alias)) return alias.targets[0]?.alias ?? '—';
  return alias.repo;
}
// Base-URL/Filename: filename (local+user-alias), base_url (api), first target's model (router).
function aliasBaseUrl(alias: AliasResponse): string {
  if (isApiAlias(alias)) return alias.base_url;
  if (isModelRouterAlias(alias)) return alias.targets[0]?.model ?? '—';
  return alias.filename;
}

const COLUMNS: CatalogColumn<AliasResponse, ModelSort>[] = [
  { key: 'num', label: '#', width: '44px', cell: () => null },
  {
    key: 'name',
    label: 'NAME',
    width: '',
    sort: 'name',
    cell: (alias) => {
      const meta = getAliasTypeMeta(alias);
      const id = getAliasId(alias);
      return (
        <div className="m-cell-name">
          <span className={`m-row-icon ${meta.iconCls}`}>
            <ShellIcon name={meta.icon} size={14} />
          </span>
          <span className="m-cell-title" data-testid={`model-title-${id}`}>
            {getAliasTitle(alias)}
          </span>
          {isApiAlias(alias) ? (
            <span className="m-provider-badge" data-testid={`model-type-${id}`}>
              {(alias as ApiAliasResponse).api_format.toUpperCase()}
            </span>
          ) : (
            <span className={`m-badge ${meta.badgeCls}`} data-testid={`model-type-${id}`}>
              {meta.label}
            </span>
          )}
        </div>
      );
    },
  },
  {
    key: 'provider',
    label: 'PROVIDER / REPO',
    width: '180px',
    sort: 'provider',
    optional: true,
    cell: (alias) => <div className="cat-cell-text mono">{aliasProvider(alias)}</div>,
  },
  {
    key: 'base_url',
    label: 'BASE URL / FILE',
    width: '260px',
    sort: 'base_url',
    optional: true,
    cell: (alias) => <div className="cat-cell-text mono">{aliasBaseUrl(alias)}</div>,
  },
];

export function ModelsScreenV2() {
  useListKeyNav();
  const navigate = useNavigate();
  const { openRail } = useShell();

  // The URL search is the single source of truth; the only effect below writes LOCAL searchInput
  // (URL→input), never the URL, so there is no read→write loop.
  const search = routeApi.useSearch();
  const urlNavigate = routeApi.useNavigate();

  // Effective sort precedence: URL > localStorage (request-only, never written to URL) > none.
  const resolvedSort = resolveSortPreference<ModelSort, SortOrder>({
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
  const filter = useMemo(() => searchToFilter(search), [search]);

  // The open detail rail is the URL's `select` (the alias id). Deriving it (not mirroring in state)
  // makes Back/Forward restoration automatic.
  const selectedId = search.select ?? null;
  // `searchInput` is the live text box (committed to ?q on Enter / clear).
  const [searchInput, setSearchInput] = useState(committedSearch);
  // `downloadsOpen` is the only ephemeral rail state; the model-detail rail is driven by ?select.
  // Opening Downloads takes over the rail; closing it falls back to the selected-model rail (if any).
  const [downloadsOpen, setDownloadsOpen] = useState(false);
  useEffect(() => {
    setSearchInput(committedSearch);
  }, [committedSearch]);

  const withViewTransition = useViewTransition();
  // Selection lives in the URL via replace (no history entries) — Back/Forward skips past selections.
  const select = useCallback(
    (id: string | null) => {
      if ((id ?? undefined) === search.select) return; // dedup
      withViewTransition(() => {
        setDownloadsOpen(false); // a row selection takes the rail back from Downloads
        urlNavigate({
          search: (prev: ModelsSearch) => {
            const out: ModelsSearch = { ...prev };
            if (id) out.select = id;
            else delete out.select;
            return out;
          },
          replace: true,
        });
      });
    },
    [urlNavigate, withViewTransition, search.select]
  );

  const listArgs = useMemo(() => searchToListArgs(search, { sort, order }), [search, sort, order]);
  const { data, isLoading, error } = useListModels(
    listArgs.page,
    listArgs.pageSize,
    listArgs.sort,
    listArgs.sortOrder,
    listArgs.filter
  );

  // Rows come pre-filtered from the server (facets + search). Dedup by id — be resilient to the
  // backend returning the same alias twice (Batch-2 gotcha) which would dup React keys.
  const dedupedRows = useMemo(() => {
    const seen = new Set<string>();
    return (data?.data ?? []).filter((a) => {
      const id = getAliasId(a);
      if (seen.has(id)) return false;
      seen.add(id);
      return true;
    });
  }, [data?.data]);

  // The derived Provider/Repo and Base-URL/Filename columns have no server sort (the API sorts only
  // name/repo/filename/source); sort them within the current page in-memory. Name goes to the server.
  // NOTE: this is per-page only — cross-page ordering for the derived columns is not guaranteed; a
  // follow-up could extend `sort_aliases` in routes_models.rs to make it server-side.
  const rows = useMemo(() => {
    if (sort !== 'provider' && sort !== 'base_url') return dedupedRows;
    const pick = sort === 'provider' ? aliasProvider : aliasBaseUrl;
    const dir = order === 'desc' ? -1 : 1;
    return [...dedupedRows].sort((a, b) => pick(a).localeCompare(pick(b)) * dir);
  }, [dedupedRows, sort, order]);

  const { hidden: hiddenColumns, toggle: toggleColumn, visibleColumns: filterVisible } = useHiddenColumns();
  const visibleColumns = useMemo(() => filterVisible(COLUMNS), [filterVisible]);

  const selected = useMemo(() => rows.find((a) => getAliasId(a) === selectedId) ?? null, [rows, selectedId]);

  // Replace the whole facet slice (a shallow merge can't delete a removed facet); keep q/sort/order
  // and the open rail; drop page so a facet change resets to page 1.
  const nonFacetSlice = useCallback((prev: ModelsSearch): ModelsSearch => {
    const base: ModelsSearch = {};
    if (prev.q) base.q = prev.q;
    if (prev.sort) base.sort = prev.sort;
    if (prev.order) base.order = prev.order;
    if (prev.select) base.select = prev.select;
    return base;
  }, []);

  const onFilterChange = useCallback(
    (next: ModelsFilter) =>
      urlNavigate({ search: (prev: ModelsSearch) => ({ ...nonFacetSlice(prev), ...facetsToSearch(next) }) }),
    [urlNavigate, nonFacetSlice]
  );

  // Commit the search box to ?q (Enter to run; clearing to empty resets it). Drops page → resets to 1.
  const commitSearch = useCallback(
    (value: string) => {
      const next = value.trim();
      urlNavigate({
        search: (prev: ModelsSearch) => {
          const out: ModelsSearch = { ...prev };
          delete out.page;
          if (next) out.q = next;
          else delete out.q;
          return out;
        },
      });
    },
    [urlNavigate]
  );

  const onSearchKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Enter') commitSearch(searchInput);
    },
    [commitSearch, searchInput]
  );

  const onSearchChange = useCallback(
    (value: string) => {
      setSearchInput(value);
      // Clearing the box live-resets the server search (so the full list returns without Enter).
      if (value.trim() === '') commitSearch('');
    },
    [commitSearch]
  );

  const onPage = useCallback(
    (p: number) =>
      urlNavigate({ search: (prev: ModelsSearch) => (p === 1 ? { ...prev, page: undefined } : { ...prev, page: p }) }),
    [urlNavigate]
  );

  const onSort = useCallback(
    (next: ModelSort) => {
      // Clicking the active column toggles direction; a new column adopts its natural default.
      const nextOrder: SortOrder = sort === next ? (order === 'asc' ? 'desc' : 'asc') : NATURAL_ORDER[next];
      persistSortPreference(SORT_STORAGE_KEY, next, nextOrder);
      urlNavigate({
        search: (prev: ModelsSearch) => {
          const out: ModelsSearch = { ...prev };
          delete out.page;
          out.sort = next;
          // Omit order when it matches the sort's natural direction; the resolver refills it on read.
          if (nextOrder === NATURAL_ORDER[next]) delete out.order;
          else out.order = nextOrder;
          return out;
        },
      });
    },
    [urlNavigate, sort, order]
  );

  // The toolbar reset waterfalls: clear active filters first, else the search query, else inert.
  const resetMode: 'filters' | 'query' | 'none' = hasActiveFilter(filter)
    ? 'filters'
    : committedSearch !== ''
      ? 'query'
      : 'none';
  const onReset = useCallback(() => {
    if (resetMode === 'filters') onFilterChange({});
    else if (resetMode === 'query') commitSearch('');
  }, [resetMode, onFilterChange, commitSearch]);

  const onEdit = useCallback(
    (alias: AliasResponse) => {
      if (isApiAlias(alias)) navigate({ to: '/models/api/edit/', search: { id: alias.id } });
      else if (isModelRouterAlias(alias)) navigate({ to: '/models/router/edit/', search: { id: alias.id } });
      else if (isUserAlias(alias)) navigate({ to: '/models/alias/edit/', search: { id: alias.id } });
    },
    [navigate]
  );

  // ── Downloads panel ──────────────────────────────────────────────
  const { showSuccess, showError } = useToastMessages();
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

  // Toggle: clicking while the Downloads panel is shown closes it (falling back to the selected-model
  // rail, if any); otherwise it opens Downloads (taking over the rail from the model detail).
  // `openRail` un-collapses the column if the user had manually collapsed the rail.
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

  const jumpToList = useCallback(() => {
    const target = document.querySelector<HTMLElement>('.l-listrow.active .l-rowlink, .l-listrow .l-rowlink');
    target?.focus();
  }, []);

  const sidebar = useMemo(
    () => <ModelSidebarFacets filter={filter} onChange={onFilterChange} />,
    [filter, onFilterChange]
  );

  const railHeader = useMemo(() => {
    if (downloadsOpen) return <DownloadsPanelHeader onClose={closeDownloads} />;
    if (selected) return <ModelRailHeader alias={selected} onClose={() => select(null)} />;
    return null;
  }, [downloadsOpen, selected, select, closeDownloads]);

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
    if (selected) return <ModelDetailRail alias={selected} onEdit={() => onEdit(selected)} />;
    return null;
  }, [
    downloadsOpen,
    downloads,
    downloadsLoading,
    onArchiveDownload,
    onRetryDownload,
    downloadsBusy,
    jumpToList,
    selected,
    onEdit,
  ]);

  useShellChrome({ breadcrumb: MODELS_BREADCRUMB, sidebar, rail, railHeader, railDefaultOpen: false });

  if (error) {
    const message = extractErrorMessage(error, 'An unexpected error occurred');
    return <ErrorPage message={message} />;
  }

  const total = data?.total ?? 0;

  return (
    <div
      className="models-screen cat-screen l-page"
      data-testid="models-content"
      data-pagestatus={isLoading ? 'loading' : 'ready'}
    >
      <div className="l-controls">
        <div className="m-toolbar">
          <div className="m-search" data-testid="models-search">
            <ShellSearch
              value={searchInput}
              onChange={onSearchChange}
              onKeyDown={onSearchKeyDown}
              placeholder="Search by alias, repo, filename, base URL"
              kbd="⌘K"
            />
          </div>
          <ResetButton mode={resetMode} onReset={onReset} testId="cat-mymodel-clear-all" />
          <ColumnPicker columns={COLUMNS} hidden={hiddenColumns} onToggle={toggleColumn} testIdPrefix="cat-mymodel" />
          <button
            type="button"
            className={`l-iconbtn ld-dl-iconbtn${downloadsOpen ? ' on' : ''}`}
            onClick={toggleDownloads}
            data-testid="models-downloads-button"
            title="Open Downloads"
            aria-label="Open Downloads"
          >
            <ShellIcon name="download" size={15} />
            {activeCount > 0 && (
              <span className="ld-dl-badge" data-testid="models-downloads-badge">
                {activeCount}
              </span>
            )}
          </button>
        </div>
      </div>

      <div className="l-scroll" data-testid="table-list-models">
        {isLoading ? (
          <div style={{ padding: 16 }} data-testid="models-skeleton-container">
            {Array.from({ length: 6 }).map((_, i) => (
              <Skeleton key={i} className="h-14 w-full mb-3" data-testid="models-skeleton" />
            ))}
          </div>
        ) : rows.length === 0 ? (
          <EmptyState
            icon="search-x"
            title="No models match"
            sub={
              committedSearch || hasActiveFilter(filter)
                ? 'Try a different search term or clear the filters.'
                : 'No models configured yet.'
            }
            testId="no-models"
          />
        ) : (
          <CatalogTable<AliasResponse, ModelSort>
            columns={visibleColumns}
            rows={rows}
            rowKey={getAliasId}
            rowTestId={(a) => `model-row-${getAliasId(a)}`}
            rowLabel={(a) => `Open model ${getAliasTitle(a)}`}
            rowAttrs={(a) => ({ 'data-model-type': a.source })}
            activeKey={selectedId}
            onSelect={(a) => select(getAliasId(a))}
            sort={sort}
            order={order}
            onSort={onSort}
            startIndex={(page - 1) * PAGE_SIZE}
            testIdPrefix="cat-mymodel"
          />
        )}
      </div>

      {total > PAGE_SIZE && <ShellPagination minimal total={total} page={page} onPage={onPage} pageSize={PAGE_SIZE} />}
    </div>
  );
}
