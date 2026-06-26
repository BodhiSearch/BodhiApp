import { useCallback, useEffect, useMemo, useState } from 'react';

import type { McpServerSummary } from '@bodhiapp/reference-api-types';
import { getRouteApi } from '@tanstack/react-router';

import { ShellIcon, ShellPagination, ShellSearch, useListKeyNav, useShellChrome } from '@/components/shell';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { useMcpServerDetail, useMcpServers } from '@/hooks/reference';
import { useViewTransition } from '@/hooks/useViewTransition';
import { exploreMcpBreadcrumb } from '@/routes/mcps/explore/-shared/breadcrumbs';
import { monogram, tintIndex } from '@/routes/models/explore/-shared/catalog-format';
import { type CatalogColumn, CatalogTable } from '@/routes/models/explore/-shared/catalog-table';
import { ColumnPicker, useHiddenColumns } from '@/routes/models/explore/-shared/ColumnPicker';
import { ResetButton } from '@/routes/models/explore/-shared/ResetButton';

import type { ExploreMcpSearch } from '../index';

import { ExploreMcpRail, ExploreMcpRailHeader } from './ExploreMcpRail';
import { type McpFacetsState, ExploreMcpSidebar, hasActiveMcpFacets } from './ExploreMcpSidebar';
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
    optional: true,
    cell: (s) => <span className="cat-cell-text mono">{s.auth_type}</span>,
  },
];

type McpAuthFacet = NonNullable<McpFacetsState['auth']>[number];

function searchToFacets(search: ExploreMcpSearch): McpFacetsState {
  return {
    category: search.category,
    auth: search.auth as McpAuthFacet[] | undefined,
    verified: search.verified,
  };
}

export function ExploreMcpScreen() {
  useListKeyNav();

  const search = routeApi.useSearch();
  const navigate = routeApi.useNavigate();

  const sort: McpSort = 'name';
  const order: SortOrder = search.order ?? 'asc';
  const page = search.page ?? 1;
  const committedSearch = search.q ?? '';
  const selectedKey = search.select ?? null;
  const facets = useMemo(() => searchToFacets(search), [search]);

  const [searchInput, setSearchInput] = useState(committedSearch);
  const { hidden: hiddenColumns, toggle: toggleColumn, visibleColumns: filterVisible } = useHiddenColumns();
  useEffect(() => {
    setSearchInput(committedSearch);
  }, [committedSearch]);

  const visibleColumns = useMemo(() => filterVisible(COLUMNS), [filterVisible]);

  // category + auth are server-side params (repeatable OR). verified has no API param — filtered
  // client-side below.
  const params = useMemo(
    () => ({
      q: committedSearch || undefined,
      sort,
      order,
      page,
      page_size: PAGE_SIZE,
      ...(facets.category?.length ? { category: facets.category } : {}),
      ...(facets.auth?.length ? { auth: facets.auth } : {}),
    }),
    [committedSearch, order, page, facets.category, facets.auth]
  );
  const { data, isLoading, error } = useMcpServers(params);

  // verified is a client-side cut on the current page (the API has no `verified` query param).
  const rows = useMemo(() => {
    const items = data?.items ?? [];
    return facets.verified ? items.filter((s) => s.verified) : items;
  }, [data?.items, facets.verified]);
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

  // Carry the non-facet slice (q/sort/order/select) across a facet change; `page` is omitted so a
  // facet change resets to page 1.
  const nonFacetSlice = useCallback((prev: ExploreMcpSearch): ExploreMcpSearch => {
    const base: ExploreMcpSearch = {};
    if (prev.q) base.q = prev.q;
    if (prev.sort) base.sort = prev.sort;
    if (prev.order) base.order = prev.order;
    if (prev.select) base.select = prev.select;
    return base;
  }, []);
  const onFacetsChange = useCallback(
    (next: McpFacetsState) =>
      navigate({
        search: (prev: ExploreMcpSearch) => ({
          ...nonFacetSlice(prev),
          ...(next.category?.length ? { category: next.category } : {}),
          ...(next.auth?.length ? { auth: next.auth } : {}),
          ...(next.verified ? { verified: true } : {}),
        }),
      }),
    [navigate, nonFacetSlice]
  );
  const onClearAllFacets = useCallback(
    () => navigate({ search: (prev: ExploreMcpSearch) => nonFacetSlice(prev) }),
    [navigate, nonFacetSlice]
  );
  // Toolbar reset, three states in precedence order: clear filters → clear query → nothing (inert).
  const hasFilters = hasActiveMcpFacets(facets);
  const hasQuery = committedSearch !== '';
  const resetMode: 'filters' | 'query' | 'none' = hasFilters ? 'filters' : hasQuery ? 'query' : 'none';
  const onReset = useCallback(() => {
    if (resetMode === 'filters') onClearAllFacets();
    else if (resetMode === 'query') commitSearch('');
  }, [resetMode, onClearAllFacets, commitSearch]);

  const sidebar = useMemo(
    () => <ExploreMcpSidebar facets={facets} facetValues={data?.facets} onFacetsChange={onFacetsChange} />,
    [facets, data?.facets, onFacetsChange]
  );

  const withViewTransition = useViewTransition();
  // Selection lives in the URL via replace (no history entries) — Back/Forward skips past selections.
  // The rail auto-opens/closes from its content presence.
  const select = useCallback(
    (key: string | null) => {
      if ((key ?? undefined) === search.select) return;
      withViewTransition(() => {
        navigate({
          search: (prev: ExploreMcpSearch) => {
            const out: ExploreMcpSearch = { ...prev };
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

  // The selected row by id; if it isn't on the current page (filtered/paged out) the rail closes.
  const selectedServer = useMemo(() => rows.find((s) => serverKey(s) === selectedKey) ?? null, [rows, selectedKey]);
  const { data: detail, isLoading: detailLoading } = useMcpServerDetail(selectedServer ? selectedServer.id : null);

  const railHeader = useMemo(
    () => (selectedServer ? <ExploreMcpRailHeader server={selectedServer} onClose={() => select(null)} /> : null),
    [selectedServer, select]
  );
  const rail = useMemo(
    () => (selectedServer ? <ExploreMcpRail server={selectedServer} detail={detail} loading={detailLoading} /> : null),
    [selectedServer, detail, detailLoading]
  );

  useShellChrome({
    breadcrumb: useMemo(() => BREADCRUMB, []),
    sidebar,
    rail,
    railHeader,
    railDefaultOpen: false,
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
          <ResetButton mode={resetMode} onReset={onReset} testId="cat-mcp-clear-all" />
          <div className="cat-sortbar">
            <ColumnPicker columns={COLUMNS} hidden={hiddenColumns} onToggle={toggleColumn} testIdPrefix="cat-mcp" />
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
            columns={visibleColumns}
            rows={rows}
            rowKey={serverKey}
            rowTestId={(s) => `cat-mcp-row-${s.id}`}
            rowLabel={(s) => `Open ${s.name}`}
            activeKey={selectedKey}
            onSelect={(s) => select(serverKey(s))}
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
