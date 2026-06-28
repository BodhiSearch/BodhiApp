import { useCallback, useEffect, useMemo, useState } from 'react';

import type { Mcp } from '@bodhiapp/ts-client';
import { getRouteApi } from '@tanstack/react-router';

import { EmptyState } from '@/components/EmptyState';
import { ShellPagination, ShellSearch, useListKeyNav, useShellChrome } from '@/components/shell';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { useDeleteMcp, useListAuthConfigs, useListMcpServers, useListMcps } from '@/hooks/mcps';
import { useMcpServerDetail, useMcpServers } from '@/hooks/reference';
import { toast } from '@/hooks/use-toast';
import { useGetUser } from '@/hooks/users';
import { useViewTransition } from '@/hooks/useViewTransition';
import { isAdminRole } from '@/lib/roles';
import { exploreMcpBreadcrumb } from '@/routes/mcps/explore/-shared/breadcrumbs';
import {
  type McpJoinedRow,
  INSTALL_LABEL,
  indexInstances,
  indexRegisteredServers,
  joinInstances,
} from '@/routes/mcps/explore/-shared/instance-join';
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

function serverKey(s: McpJoinedRow): string {
  return s.id;
}

const COLUMNS: CatalogColumn<McpJoinedRow, McpSort>[] = [
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
    key: 'status',
    label: 'STATUS',
    width: '130px',
    cell: (s) => (
      <span className={`mcp-install mcp-install-${s.install}`} data-testid={`cat-mcp-install-${s.id}`}>
        {INSTALL_LABEL[s.install]}
      </span>
    ),
  },
  {
    key: 'auth',
    label: 'AUTH',
    width: '100px',
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
    installed: search.installed,
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

  // Join the user's own instances (no per-user state in the catalog API) → derive install status, and
  // resolve each catalog endpoint to a REGISTERED server so the rail can offer connect/configure.
  const { data: instancesData } = useListMcps();
  const { data: serversData } = useListMcpServers();
  const byUrl = useMemo(() => indexInstances(instancesData?.mcps), [instancesData?.mcps]);
  const registeredByUrl = useMemo(() => indexRegisteredServers(serversData?.mcp_servers), [serversData?.mcp_servers]);

  const { data: userInfo } = useGetUser();
  const isAdmin = userInfo?.auth_status === 'logged_in' && userInfo.role ? isAdminRole(userInfo.role) : false;

  // verified + installed are client-side cuts on the current page (the API has neither param).
  const rows = useMemo(() => {
    let items = joinInstances(data?.items ?? [], byUrl, registeredByUrl);
    if (facets.verified) items = items.filter((s) => s.verified);
    if (facets.installed === 'installed') items = items.filter((s) => s.install !== 'none');
    else if (facets.installed === 'not_installed') items = items.filter((s) => s.install === 'none');
    return items;
  }, [data?.items, byUrl, registeredByUrl, facets.verified, facets.installed]);
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
          ...(next.installed ? { installed: next.installed } : {}),
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

  // When the selected catalog row resolves to a registered server, the rail offers the same
  // connect/configure/instance actions as My MCPs — keyed by the REGISTERED server id.
  const registeredId = selectedServer?.registered?.id ?? null;
  const selectedInstances = useMemo(() => {
    if (!registeredId) return [];
    return (instancesData?.mcps ?? []).filter((m) => m.mcp_server?.id === registeredId);
  }, [registeredId, instancesData?.mcps]);
  const { data: authConfigsData, isLoading: authConfigsLoading } = useListAuthConfigs(registeredId ?? '', {
    enabled: !!registeredId,
  });

  const [deleteTarget, setDeleteTarget] = useState<Mcp | null>(null);
  const deleteMutation = useDeleteMcp({
    onSuccess: () => {
      toast({ title: 'MCP instance deleted' });
      setDeleteTarget(null);
    },
    onError: (message) => {
      toast({ title: 'Failed to delete MCP instance', description: message, variant: 'destructive' });
      setDeleteTarget(null);
    },
  });

  const railHeader = useMemo(
    () => (selectedServer ? <ExploreMcpRailHeader server={selectedServer} onClose={() => select(null)} /> : null),
    [selectedServer, select]
  );
  const rail = useMemo(
    () =>
      selectedServer ? (
        <ExploreMcpRail
          server={selectedServer}
          detail={detail}
          loading={detailLoading}
          instances={selectedInstances}
          authConfigs={authConfigsData?.auth_configs}
          authConfigsLoading={authConfigsLoading}
          isAdmin={isAdmin}
          onDeleteInstance={setDeleteTarget}
        />
      ) : null,
    [
      selectedServer,
      detail,
      detailLoading,
      selectedInstances,
      authConfigsData?.auth_configs,
      authConfigsLoading,
      isAdmin,
    ]
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
          <EmptyState
            icon="search-x"
            title="No MCP servers found"
            sub="Try a different search."
            testId="cat-mcp-empty"
          />
        ) : (
          <CatalogTable<McpJoinedRow, McpSort>
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

      <AlertDialog open={!!deleteTarget} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <AlertDialogContent data-testid="cat-mcp-delete-dialog">
          <AlertDialogHeader>
            <AlertDialogTitle>Delete MCP instance</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete &quot;{deleteTarget?.name}&quot;? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => deleteTarget && deleteMutation.mutate({ id: deleteTarget.id })}
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending ? 'Deleting...' : 'Delete'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
