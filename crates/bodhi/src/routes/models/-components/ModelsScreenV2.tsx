import { useCallback, useMemo, useState } from 'react';

import { AliasResponse, ApiAliasResponse } from '@bodhiapp/ts-client';
import { useNavigate } from '@tanstack/react-router';

import { DownloadsPanel, DownloadsPanelHeader, isActive } from '@/components/downloads-panel/DownloadsPanel';
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
import {
  ModelsFilter,
  ModelTypeFacet,
  useArchiveDownload,
  useDownloadsRefresh,
  useListDownloads,
  useListModels,
  useRetryDownload,
} from '@/hooks/models';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useViewTransition } from '@/hooks/useViewTransition';
import { isApiAlias, isModelRouterAlias, isUserAlias } from '@/lib/utils';

import { ModelDetailRail, ModelRailHeader } from './ModelDetailRail';
import { ModelSidebarFacets } from './ModelSidebarFacets';
import '@/components/downloads-panel/downloads-panel.css';
import '@/components/shell/api-keys.css';
import '@/components/shell/list.css';
import './models.css';

const MODELS_BREADCRUMB = [{ label: 'Bodhi' }, { label: 'Models' }, { label: 'My Models', current: true }];

const PAGE_SIZE = 30;

/** A stable identity for any alias row (api/router use id, local aliases use the alias name). */
function aliasId(alias: AliasResponse): string {
  if (isApiAlias(alias) || isModelRouterAlias(alias)) return alias.id;
  return alias.alias;
}

/** TYPE-facet token + display label + badge + icon-tile classes (color-coded per type). */
function typeMeta(alias: AliasResponse): {
  token: ModelTypeFacet;
  label: string;
  badgeCls: string;
  iconCls: string;
  icon: string;
} {
  switch (alias.source) {
    case 'model':
      return {
        token: 'local_file',
        label: 'Local File',
        badgeCls: 'm-badge-local',
        iconCls: 'm-icon-local',
        icon: 'hard-drive',
      };
    case 'user':
      return {
        token: 'model_alias',
        label: 'Model Alias',
        badgeCls: 'm-badge-alias',
        iconCls: 'm-icon-alias',
        icon: 'tag',
      };
    case 'api':
      return {
        token: 'api_model',
        label: 'API Model',
        badgeCls: 'm-badge-api',
        iconCls: 'm-icon-api',
        icon: 'at-sign',
      };
    default:
      return {
        token: 'fallback',
        label: 'Router',
        badgeCls: 'm-badge-fallback',
        iconCls: 'm-icon-fallback',
        icon: 'route',
      };
  }
}

/** Secondary line under a row's title — repo/file, API base+model-count, or the fallback chain. */
function rowSubtitle(alias: AliasResponse): string {
  if (isApiAlias(alias)) {
    const count = alias.models.length;
    return `${alias.base_url} · ${count} model${count === 1 ? '' : 's'} exposed`;
  }
  if (isModelRouterAlias(alias)) {
    const names = alias.targets.map((t) => t.alias);
    const chain = names.length > 2 ? `${names[0]} → … → ${names[names.length - 1]}` : names.join(' → ');
    return chain || 'no targets';
  }
  return alias.filename;
}

function rowTitle(alias: AliasResponse): string {
  if (isApiAlias(alias)) return alias.name || alias.id;
  return alias.alias;
}

interface ModelRowProps {
  alias: AliasResponse;
  active: boolean;
  query: string;
  onSelect: () => void;
}

function ModelRow({ alias, active, query, onSelect }: ModelRowProps) {
  const meta = typeMeta(alias);
  const title = rowTitle(alias);
  const subtitle = rowSubtitle(alias);

  const id = aliasId(alias);
  // API rows show the provider (api_format) as a green badge + a connection status, mirroring the
  // design; other rows show the type badge. The model-type testid is preserved on both.
  const api = isApiAlias(alias) ? (alias as ApiAliasResponse) : null;
  return (
    <div
      className={`l-listrow m-row${active ? ' active' : ''}`}
      onClick={onSelect}
      role="option"
      aria-selected={active}
      data-testid={`model-row-${id}`}
      data-model-type={alias.source}
    >
      <LinkRow onActivate={onSelect} label={`Open model ${title}`} />
      <div className={`m-row-icon ${meta.iconCls}`}>
        <ShellIcon name={meta.icon} size={15} />
      </div>
      <div className="m-row-body">
        <div className="m-row-title" data-testid={`model-title-${id}`}>
          {highlight(title, query)}
        </div>
        <div className="m-row-sub mono">{highlight(subtitle, query)}</div>
      </div>
      <div className="m-row-meta">
        {api ? (
          <span className="m-provider-badge" data-testid={`model-type-${id}`}>
            {api.api_format.toUpperCase()}
          </span>
        ) : (
          <span className={`m-badge ${meta.badgeCls}`} data-testid={`model-type-${id}`}>
            {meta.label}
          </span>
        )}
        {api && (
          <span className={`m-conn ${api.has_api_key ? 'ok' : 'warn'}`}>
            <ShellIcon name={api.has_api_key ? 'check-circle' : 'key'} size={10} />
            {api.has_api_key ? 'connected' : 'no key'}
          </span>
        )}
      </div>
    </div>
  );
}

function highlight(text: string, query: string) {
  if (!query) return text;
  const idx = text.toLowerCase().indexOf(query);
  if (idx < 0) return text;
  return (
    <>
      {text.slice(0, idx)}
      <mark>{text.slice(idx, idx + query.length)}</mark>
      {text.slice(idx + query.length)}
    </>
  );
}

export function ModelsScreenV2() {
  useListKeyNav();
  const navigate = useNavigate();
  const { openRail } = useShell();

  const [filter, setFilter] = useState<ModelsFilter>({});
  // `searchInput` is the live text box; it's committed to `filter.search` (the backend param) on
  // Enter, or when cleared to empty.
  const [searchInput, setSearchInput] = useState('');
  const [page, setPage] = useState(1);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  // Right-rail content: 'model' = selected-alias detail, 'downloads' = Downloads panel.
  const [railMode, setRailMode] = useState<'model' | 'downloads' | null>(null);

  const withViewTransition = useViewTransition();
  const select = useCallback(
    (id: string | null) =>
      withViewTransition(() => {
        setSelectedId(id);
        setRailMode(id ? 'model' : null);
      }),
    [withViewTransition]
  );

  const { data, isLoading, error } = useListModels(page, PAGE_SIZE, 'alias', 'asc', filter);

  // Rows come pre-filtered from the server (facets + search). Dedup by id — be resilient to the
  // backend returning the same alias twice (Batch-2 gotcha) which would dup React keys.
  const rows = useMemo(() => {
    const seen = new Set<string>();
    return (data?.data ?? []).filter((a) => {
      const id = aliasId(a);
      if (seen.has(id)) return false;
      seen.add(id);
      return true;
    });
  }, [data?.data]);

  // Highlight matches the committed server query (lowercased), not the in-progress text box.
  const q = (filter.search ?? '').trim().toLowerCase();

  const selected = useMemo(() => rows.find((a) => aliasId(a) === selectedId) ?? null, [rows, selectedId]);

  const onFilterChange = useCallback((next: ModelsFilter) => {
    setFilter(next);
    setPage(1);
  }, []);

  // Commit the search box to the backend filter (Enter to run; clearing to empty resets it).
  const commitSearch = useCallback(
    (value: string) => {
      const trimmed = value.trim();
      onFilterChange({ ...filter, search: trimmed || undefined });
    },
    [filter, onFilterChange]
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

  const jumpToList = useCallback(() => {
    const target = document.querySelector<HTMLElement>('.l-listrow.active .l-rowlink, .l-listrow .l-rowlink');
    target?.focus();
  }, []);

  const sidebar = useMemo(
    () => <ModelSidebarFacets filter={filter} onChange={onFilterChange} />,
    [filter, onFilterChange]
  );

  const railHeader = useMemo(() => {
    if (railMode === 'downloads') return <DownloadsPanelHeader onClose={closeRail} />;
    if (railMode === 'model' && selected) return <ModelRailHeader alias={selected} onClose={() => select(null)} />;
    return null;
  }, [railMode, selected, select, closeRail]);

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
    if (railMode === 'model' && selected) return <ModelDetailRail alias={selected} onEdit={() => onEdit(selected)} />;
    return null;
  }, [
    railMode,
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
    const message = error.response?.data?.error?.message || error.message;
    return <ErrorPage message={message} />;
  }

  const total = data?.total ?? 0;
  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

  return (
    <div
      className="models-screen l-page"
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
          <button
            type="button"
            className={`l-iconbtn ld-dl-iconbtn${railMode === 'downloads' ? ' on' : ''}`}
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
          <div className="empty-state" data-testid="no-models">
            <div className="empty-icon">
              <ShellIcon name="search-x" size={28} />
            </div>
            <div className="empty-title">No models match</div>
            <div className="empty-sub">
              {q || hasActiveFacet(filter)
                ? 'Try a different search term or clear the filters.'
                : 'No models configured yet.'}
            </div>
          </div>
        ) : (
          <div className="l-listview">
            {rows.map((alias) => (
              <ModelRow
                key={aliasId(alias)}
                alias={alias}
                active={aliasId(alias) === selectedId}
                query={q}
                onSelect={() => select(aliasId(alias))}
              />
            ))}
          </div>
        )}
      </div>

      {totalPages > 1 && <ShellPagination minimal total={total} page={page} onPage={setPage} pageSize={PAGE_SIZE} />}
    </div>
  );
}

function hasActiveFacet(filter: ModelsFilter): boolean {
  return Boolean(
    filter.types?.length ||
      filter.apiFormats?.length ||
      filter.capabilities?.length ||
      filter.sizeMin != null ||
      filter.sizeMax != null ||
      filter.search
  );
}
