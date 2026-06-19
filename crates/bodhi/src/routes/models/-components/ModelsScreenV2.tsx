import { useCallback, useMemo, useState } from 'react';

import { AliasResponse, ApiAliasResponse } from '@bodhiapp/ts-client';
import { useNavigate } from '@tanstack/react-router';

import { Pagination } from '@/components/DataTable';
import { LinkRow, ShellFilterTabs, ShellIcon, useCollapsibleSearch, useShellChrome } from '@/components/shell';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { ModelsFilter, ModelTypeFacet, useListModels } from '@/hooks/models';
import { useViewTransition } from '@/hooks/useViewTransition';
import { isApiAlias, isModelRouterAlias, isUserAlias } from '@/lib/utils';

import { ModelDetailRail, ModelRailHeader } from './ModelDetailRail';
import { ModelSidebarFacets } from './ModelSidebarFacets';
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

/** TYPE-facet token + display label + badge class for an alias row. */
function typeMeta(alias: AliasResponse): { token: ModelTypeFacet; label: string; cls: string } {
  switch (alias.source) {
    case 'model':
      return { token: 'local_file', label: 'Local File', cls: 'm-badge-local' };
    case 'user':
      return { token: 'model_alias', label: 'Model Alias', cls: 'm-badge-alias' };
    case 'api':
      return { token: 'api_model', label: 'API Model', cls: 'm-badge-api' };
    default:
      return { token: 'fallback', label: 'Fallback', cls: 'm-badge-fallback' };
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

  // Connection status for API rows (no key vs configured), real data only.
  const apiStatus = isApiAlias(alias) ? (alias.has_api_key ? 'connected' : 'no key') : null;

  const id = aliasId(alias);
  return (
    <div
      className={`l-listrow m-row${active ? ' active' : ''}`}
      onClick={onSelect}
      data-testid={`model-row-${id}`}
      data-model-type={alias.source}
    >
      <LinkRow onActivate={onSelect} label={`Open model ${title}`} />
      <div className="m-row-icon">
        <ShellIcon name={rowIcon(alias)} size={15} />
      </div>
      <div className="m-row-body">
        <div className="m-row-title" data-testid={`model-title-${id}`}>
          {highlight(title, query)}
        </div>
        <div className="m-row-sub mono">{highlight(subtitle, query)}</div>
      </div>
      <div className="m-row-meta">
        <span className={`m-badge ${meta.cls}`} data-testid={`model-type-${id}`}>
          {meta.label}
        </span>
        {apiStatus && (
          <span
            className={`m-conn ${alias.source === 'api' && (alias as ApiAliasResponse).has_api_key ? 'ok' : 'warn'}`}
          >
            {apiStatus}
          </span>
        )}
      </div>
    </div>
  );
}

function rowIcon(alias: AliasResponse): string {
  if (isApiAlias(alias)) return 'cloud';
  if (isModelRouterAlias(alias)) return 'route';
  if (isUserAlias(alias)) return 'tag';
  return 'hard-drive';
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
  const navigate = useNavigate();

  const [filter, setFilter] = useState<ModelsFilter>({});
  const [search, setSearch] = useState('');
  const [page, setPage] = useState(1);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const withViewTransition = useViewTransition();
  const select = useCallback((id: string | null) => withViewTransition(() => setSelectedId(id)), [withViewTransition]);

  const { data, isLoading, error } = useListModels(page, PAGE_SIZE, 'alias', 'asc', filter);

  // Client-side search over the (already server-filtered) page — repo/filename/base-url/alias.
  const q = search.trim().toLowerCase();
  const rows = useMemo(() => {
    // Dedup by id — be resilient to the backend returning the same alias twice (Batch-2 gotcha),
    // which would otherwise produce duplicate React keys.
    const seen = new Set<string>();
    const all = (data?.data ?? []).filter((a) => {
      const id = aliasId(a);
      if (seen.has(id)) return false;
      seen.add(id);
      return true;
    });
    if (!q) return all;
    return all.filter((a) => {
      const hay = [rowTitle(a), rowSubtitle(a)].join(' ').toLowerCase();
      return hay.includes(q);
    });
  }, [data?.data, q]);

  const selected = useMemo(() => rows.find((a) => aliasId(a) === selectedId) ?? null, [rows, selectedId]);

  const onFilterChange = useCallback((next: ModelsFilter) => {
    setFilter(next);
    setPage(1);
  }, []);

  const onEdit = useCallback(
    (alias: AliasResponse) => {
      if (isApiAlias(alias)) navigate({ to: '/models/api/edit/', search: { id: alias.id } });
      else if (isModelRouterAlias(alias)) navigate({ to: '/models/router/edit/', search: { id: alias.id } });
      else if (isUserAlias(alias)) navigate({ to: '/models/alias/edit/', search: { id: alias.id } });
    },
    [navigate]
  );

  const searchNode = useCollapsibleSearch({
    value: search,
    onChange: setSearch,
    placeholder: 'Search by alias, repo, filename, base URL…',
    toggleTestId: 'models-search-toggle',
    closeTestId: 'models-search-close',
  });

  const sidebar = useMemo(
    () => <ModelSidebarFacets filter={filter} onChange={onFilterChange} />,
    [filter, onFilterChange]
  );

  const railHeader = useMemo(
    () => (selected ? <ModelRailHeader alias={selected} onClose={() => select(null)} /> : null),
    [selected, select]
  );

  const rail = useMemo(
    () => (selected ? <ModelDetailRail alias={selected} onEdit={() => onEdit(selected)} /> : null),
    [selected, onEdit]
  );

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
        {searchNode.row}
        <div className="l-toolbar">
          <ShellFilterTabs
            tabs={TYPE_TABS}
            value={primaryType(filter)}
            onChange={(id) => onFilterChange(applyPrimaryType(filter, id))}
            label="Filter by type"
            testIdPrefix="models-type"
            loading={isLoading}
          />
          <div className="l-tb-actions">{searchNode.toggle}</div>
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
              {search || hasActiveFacet(filter)
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

      {totalPages > 1 && (
        <div className="m-pagination">
          <Pagination page={page} totalPages={totalPages} onPageChange={setPage} />
        </div>
      )}
    </div>
  );
}

// --- Toolbar TYPE quick-tabs (mirror a subset of the sidebar TYPE facet for fast switching) ---

const TYPE_TABS: { id: 'all' | ModelTypeFacet; label: string }[] = [
  { id: 'all', label: 'All' },
  { id: 'local_file', label: 'Local' },
  { id: 'model_alias', label: 'Alias' },
  { id: 'api_model', label: 'API' },
  { id: 'fallback', label: 'Fallback' },
];

function primaryType(filter: ModelsFilter): 'all' | ModelTypeFacet {
  return filter.types?.length === 1 ? filter.types[0] : 'all';
}

function applyPrimaryType(filter: ModelsFilter, id: 'all' | ModelTypeFacet): ModelsFilter {
  return { ...filter, types: id === 'all' ? undefined : [id] };
}

function hasActiveFacet(filter: ModelsFilter): boolean {
  return Boolean(
    filter.types?.length ||
      filter.apiFormats?.length ||
      filter.capabilities?.length ||
      filter.sizeMin != null ||
      filter.sizeMax != null
  );
}
