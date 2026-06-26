import { useCallback, useEffect, useMemo, useState } from 'react';

import type { McpServerSummary } from '@bodhiapp/reference-api-types';
import { getRouteApi } from '@tanstack/react-router';

import { ShellIcon, ShellPagination, ShellSearch, useShellChrome } from '@/components/shell';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { useMcpServers } from '@/hooks/reference';
import { exploreMcpBreadcrumb } from '@/routes/mcps/explore/-shared/breadcrumbs';
import { monogram, tintIndex } from '@/routes/models/explore/-shared/catalog-format';
import { type CatalogColumn, CatalogTable } from '@/routes/models/explore/-shared/catalog-table';

import type { ExploreMcpSearch } from '../index';

import { McpServerLogo } from './McpServerLogo';
import '@/components/shell/list.css';
import '@/routes/models/-components/models.css';
import '@/routes/models/explore/-shared/catalog.css';
import '@/routes/mcps/explore/-shared/explore-mcp.css';

const BREADCRUMB = exploreMcpBreadcrumb('Explore · MCP Servers');
const routeApi = getRouteApi('/mcps/explore/');

type McpSort = 'name';
type SortOrder = 'asc' | 'desc';
const PAGE_SIZE = 50;

function serverKey(s: McpServerSummary): string {
  return s.id;
}

const COLUMNS: CatalogColumn<McpServerSummary, McpSort>[] = [
  { key: 'num', label: '#', width: '44px', cell: () => null },
  {
    key: 'logo',
    label: '',
    width: '44px',
    cell: (s) => (
      <McpServerLogo
        src={s.logo_url}
        className={`cat-logo cat-tint-${tintIndex(s.slug)}`}
        fallback={monogram(s.name)}
      />
    ),
  },
  {
    key: 'server',
    label: 'SERVER',
    width: '',
    sort: 'name',
    cell: (s) => (
      <div className="cat-body">
        <div className="cat-name">
          {s.name}
          {s.featured && <span className="cat-status cat-status-stable">Featured</span>}
        </div>
        {s.description && <div className="cat-sub">{s.description}</div>}
      </div>
    ),
  },
  {
    key: 'auth',
    label: 'AUTH',
    width: '110px',
    cell: (s) => <span className="cat-cell-text mono">{s.auth_type}</span>,
  },
];

export function ExploreMcpScreen() {
  const search = routeApi.useSearch();
  const navigate = routeApi.useNavigate();

  const sort: McpSort = 'name';
  const order: SortOrder = search.order ?? 'asc';
  const page = search.page ?? 1;
  const committedSearch = search.q ?? '';

  const [searchInput, setSearchInput] = useState(committedSearch);
  useEffect(() => {
    setSearchInput(committedSearch);
  }, [committedSearch]);

  const params = useMemo(
    () => ({ q: committedSearch || undefined, sort, order, page, page_size: PAGE_SIZE }),
    [committedSearch, order, page]
  );
  const { data, isLoading, error } = useMcpServers(params);

  const rows = data?.items ?? [];
  const total = data?.total ?? rows.length;

  const commitSearch = useCallback(
    (value: string) => {
      const next = value.trim();
      navigate({
        search: (prev: ExploreMcpSearch) => {
          const out: ExploreMcpSearch = { ...prev };
          delete out.page;
          if (next) out.q = next;
          else delete out.q;
          return out;
        },
      });
    },
    [navigate]
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
    (_next: McpSort) => {
      // Only `name` is sortable; clicking the header toggles direction (asc is natural → omit it).
      const nextOrder: SortOrder = order === 'asc' ? 'desc' : 'asc';
      navigate({
        search: (prev: ExploreMcpSearch) => {
          const out: ExploreMcpSearch = { ...prev };
          delete out.page;
          if (nextOrder === 'asc') delete out.order;
          else out.order = nextOrder;
          return out;
        },
      });
    },
    [navigate, order]
  );
  const onPage = useCallback(
    (p: number) =>
      navigate({ search: (prev: ExploreMcpSearch) => (p === 1 ? { ...prev, page: undefined } : { ...prev, page: p }) }),
    [navigate]
  );

  useShellChrome({
    breadcrumb: useMemo(() => BREADCRUMB, []),
  });

  if (error) {
    return <ErrorPage message={error instanceof Error ? error.message : 'Failed to load the MCP catalog'} />;
  }

  return (
    <div
      className="cat-screen l-page"
      data-testid="explore-mcp-content"
      data-pagestatus={isLoading ? 'loading' : 'ready'}
    >
      <div className="l-controls">
        <div className="m-toolbar">
          <div className="m-search" data-testid="cat-mcp-search">
            <ShellSearch
              value={searchInput}
              onChange={onSearchChange}
              onKeyDown={onSearchKeyDown}
              placeholder="Search MCP servers"
              kbd="⌘K"
            />
          </div>
        </div>
      </div>

      <div className="l-scroll" data-testid="cat-mcp-list">
        {isLoading && rows.length === 0 ? (
          <div style={{ padding: 16 }} data-testid="cat-mcp-skeleton-container">
            {Array.from({ length: 6 }).map((_, i) => (
              <Skeleton key={i} className="h-14 w-full mb-3" data-testid="cat-mcp-skeleton" />
            ))}
          </div>
        ) : rows.length === 0 ? (
          <div className="empty-state" data-testid="cat-mcp-empty">
            <div className="empty-icon">
              <ShellIcon name="search-x" size={28} />
            </div>
            <div className="empty-title">No MCP servers found</div>
            <div className="empty-sub">Try a different search.</div>
          </div>
        ) : (
          <CatalogTable<McpServerSummary, McpSort>
            columns={COLUMNS}
            rows={rows}
            rowKey={serverKey}
            rowTestId={(s) => `cat-mcp-row-${s.id}`}
            rowLabel={(s) => `Open ${s.name}`}
            activeKey={null}
            onSelect={() => {}}
            sort={sort}
            order={order}
            onSort={onSort}
            startIndex={(page - 1) * PAGE_SIZE}
            testIdPrefix="cat-mcp"
          />
        )}
      </div>

      {total > PAGE_SIZE && (
        <ShellPagination total={total} page={page} onPage={onPage} pageSize={PAGE_SIZE} unit="servers" />
      )}
    </div>
  );
}
