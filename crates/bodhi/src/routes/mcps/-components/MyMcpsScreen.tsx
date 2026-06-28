import { useCallback, useEffect, useMemo, useState } from 'react';

import type { Mcp, McpServerResponse } from '@bodhiapp/ts-client';
import { getRouteApi, Link } from '@tanstack/react-router';

import { EmptyState } from '@/components/EmptyState';
import { ShellSearch, useListKeyNav, useShellChrome } from '@/components/shell';
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
import { useDeleteMcp, useListAuthConfigs, useListMcps, useListMcpServers } from '@/hooks/mcps';
import { toast } from '@/hooks/use-toast';
import { useGetUser } from '@/hooks/users';
import { useViewTransition } from '@/hooks/useViewTransition';
import { ROUTE_MCPS } from '@/lib/constants';
import { isAdminRole } from '@/lib/roles';
import { monogram, tintIndex } from '@/routes/models/explore/-shared/catalog-format';
import { type CatalogColumn, CatalogTable } from '@/routes/models/explore/-shared/catalog-table';
import { ResetButton } from '@/routes/models/explore/-shared/ResetButton';

import type { MyMcpsSearch } from '../index';

import { MyMcpsRail, MyMcpsRailHeader } from './MyMcpsRail';
import { type MyMcpsFacetsState, MyMcpsSidebar, hasActiveMyMcpsFacets } from './MyMcpsSidebar';
import './my-mcps.css';
import '@/components/shell/list.css';
import '@/routes/models/-components/models.css';
import '@/routes/models/explore/-shared/catalog.css';

const BREADCRUMB = [{ label: 'Bodhi' }, { label: 'MCP', href: ROUTE_MCPS }, { label: 'My MCPs', current: true }];
const routeApi = getRouteApi('/mcps/');

type ServerSort = 'name';
type SortOrder = 'asc' | 'desc';

function serverKey(s: McpServerResponse): string {
  return s.id;
}

function instanceCount(s: McpServerResponse): number {
  return s.enabled_mcp_count + s.disabled_mcp_count;
}

const COLUMNS: CatalogColumn<McpServerResponse, ServerSort>[] = [
  { key: 'num', label: '#', width: '44px', cell: () => null },
  {
    key: 'logo',
    label: '',
    width: '44px',
    cell: (s) => <div className={`cat-logo cat-tint-${tintIndex(s.id)}`}>{monogram(s.name)}</div>,
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
          {!s.enabled && <span className="cat-status">Disabled</span>}
        </div>
        <div className="cat-sub mono">{s.url}</div>
      </div>
    ),
  },
  {
    key: 'status',
    label: 'STATUS',
    width: '130px',
    cell: (s) => {
      const count = instanceCount(s);
      const cls = !s.enabled ? 'none' : count > 0 ? 'enabled' : 'none';
      const label = !s.enabled ? 'Disabled' : count > 0 ? `${count} instance${count === 1 ? '' : 's'}` : 'Available';
      return (
        <span className={`mcp-install mcp-install-${cls}`} data-testid={`my-mcps-status-${s.id}`}>
          {label}
        </span>
      );
    },
  },
];

function searchToFacets(search: MyMcpsSearch): MyMcpsFacetsState {
  return { scope: search.scope };
}

export function MyMcpsScreen() {
  useListKeyNav();

  const search = routeApi.useSearch();
  const navigate = routeApi.useNavigate();

  const sort: ServerSort = 'name';
  const order: SortOrder = search.order ?? 'asc';
  const committedSearch = search.q ?? '';
  const selectedKey = search.select ?? null;
  const facets = useMemo(() => searchToFacets(search), [search]);

  const [searchInput, setSearchInput] = useState(committedSearch);
  useEffect(() => {
    setSearchInput(committedSearch);
  }, [committedSearch]);

  const { data: userInfo } = useGetUser();
  const isAdmin = userInfo?.auth_status === 'logged_in' && userInfo.role ? isAdminRole(userInfo.role) : false;

  const { data: serversData, isLoading, error } = useListMcpServers();
  const { data: instancesData } = useListMcps();

  // Group the user's instances by their server id for per-server "My Instances" + Connected scope.
  const instancesByServer = useMemo(() => {
    const map = new Map<string, Mcp[]>();
    for (const m of instancesData?.mcps ?? []) {
      const key = m.mcp_server?.id;
      if (!key) continue;
      const list = map.get(key) ?? [];
      list.push(m);
      map.set(key, list);
    }
    return map;
  }, [instancesData?.mcps]);

  const rows = useMemo(() => {
    let items = serversData?.mcp_servers ?? [];
    const q = committedSearch.trim().toLowerCase();
    if (q) {
      items = items.filter((s) => s.name.toLowerCase().includes(q) || s.url.toLowerCase().includes(q));
    }
    if (facets.scope === 'mine') {
      items = items.filter((s) => (instancesByServer.get(s.id)?.length ?? 0) > 0);
    }
    const sorted = [...items].sort((a, b) => a.name.localeCompare(b.name));
    if (order === 'desc') sorted.reverse();
    return sorted;
  }, [serversData?.mcp_servers, committedSearch, facets.scope, instancesByServer, order]);

  const commitSearch = useCallback(
    (value: string) => {
      const next = value.trim();
      navigate({
        search: (prev: MyMcpsSearch) => {
          const out: MyMcpsSearch = { ...prev };
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
    (_next: ServerSort) => {
      const nextOrder: SortOrder = order === 'asc' ? 'desc' : 'asc';
      navigate({
        search: (prev: MyMcpsSearch) => {
          const out: MyMcpsSearch = { ...prev };
          if (nextOrder === 'asc') delete out.order;
          else out.order = nextOrder;
          return out;
        },
      });
    },
    [navigate, order]
  );

  const nonFacetSlice = useCallback((prev: MyMcpsSearch): MyMcpsSearch => {
    const base: MyMcpsSearch = {};
    if (prev.q) base.q = prev.q;
    if (prev.order) base.order = prev.order;
    if (prev.select) base.select = prev.select;
    return base;
  }, []);
  const onFacetsChange = useCallback(
    (next: MyMcpsFacetsState) =>
      navigate({
        search: (prev: MyMcpsSearch) => ({ ...nonFacetSlice(prev), ...(next.scope ? { scope: next.scope } : {}) }),
      }),
    [navigate, nonFacetSlice]
  );

  const hasFilters = hasActiveMyMcpsFacets(facets);
  const hasQuery = committedSearch !== '';
  const resetMode: 'filters' | 'query' | 'none' = hasFilters ? 'filters' : hasQuery ? 'query' : 'none';
  const onReset = useCallback(() => {
    if (resetMode === 'filters') navigate({ search: (prev: MyMcpsSearch) => nonFacetSlice(prev) });
    else if (resetMode === 'query') commitSearch('');
  }, [resetMode, navigate, nonFacetSlice, commitSearch]);

  const sidebar = useMemo(
    () => <MyMcpsSidebar facets={facets} onFacetsChange={onFacetsChange} />,
    [facets, onFacetsChange]
  );

  const withViewTransition = useViewTransition();
  const select = useCallback(
    (key: string | null) => {
      if ((key ?? undefined) === search.select) return;
      withViewTransition(() => {
        navigate({
          search: (prev: MyMcpsSearch) => {
            const out: MyMcpsSearch = { ...prev };
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

  const selectedServer = useMemo(() => rows.find((s) => serverKey(s) === selectedKey) ?? null, [rows, selectedKey]);
  const selectedInstances = useMemo(
    () => (selectedServer ? (instancesByServer.get(selectedServer.id) ?? []) : []),
    [selectedServer, instancesByServer]
  );
  const { data: authConfigsData, isLoading: authConfigsLoading } = useListAuthConfigs(selectedServer?.id ?? '', {
    enabled: !!selectedServer?.id,
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
    () => (selectedServer ? <MyMcpsRailHeader server={selectedServer} onClose={() => select(null)} /> : null),
    [selectedServer, select]
  );
  const rail = useMemo(
    () =>
      selectedServer ? (
        <MyMcpsRail
          server={selectedServer}
          instances={selectedInstances}
          authConfigs={authConfigsData?.auth_configs}
          authConfigsLoading={authConfigsLoading}
          isAdmin={isAdmin}
          onDeleteInstance={setDeleteTarget}
        />
      ) : null,
    [selectedServer, selectedInstances, authConfigsData?.auth_configs, authConfigsLoading, isAdmin]
  );

  useShellChrome({
    breadcrumb: useMemo(() => BREADCRUMB, []),
    sidebar,
    rail,
    railHeader,
    railDefaultOpen: false,
  });

  if (error) {
    return <ErrorPage message={error instanceof Error ? error.message : 'Failed to load your MCP servers'} />;
  }

  return (
    <div className="cat-screen l-page" data-testid="my-mcps-content" data-pagestatus={isLoading ? 'loading' : 'ready'}>
      <div className="l-controls">
        <div className="m-toolbar">
          <div className="m-search" data-testid="my-mcps-search">
            <ShellSearch
              value={searchInput}
              onChange={onSearchChange}
              onKeyDown={onSearchKeyDown}
              placeholder="Search your MCPs by name or URL"
              kbd="⌘K"
            />
          </div>
          <ResetButton mode={resetMode} onReset={onReset} testId="my-mcps-clear-all" />
        </div>
      </div>

      <div className="l-scroll" data-testid="my-mcps-list">
        {isLoading && rows.length === 0 ? (
          <div style={{ padding: 16 }} data-testid="my-mcps-skeleton-container">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-14 w-full mb-3" data-testid="my-mcps-skeleton" />
            ))}
          </div>
        ) : rows.length === 0 ? (
          <EmptyState
            icon="plug"
            title="No MCP servers yet"
            sub={
              <>
                Browse the <Link to="/mcps/explore/">catalog</Link> to add one.
              </>
            }
            testId="my-mcps-empty"
          />
        ) : (
          <CatalogTable<McpServerResponse, ServerSort>
            columns={COLUMNS}
            rows={rows}
            rowKey={serverKey}
            rowTestId={(s) => `my-mcps-row-${s.id}`}
            rowLabel={(s) => `Open ${s.name}`}
            rowAttrs={(s) => ({ 'data-test-server-name': s.name, 'data-test-uuid': s.id })}
            activeKey={selectedKey}
            onSelect={(s) => select(serverKey(s))}
            sort={sort}
            order={order}
            onSort={onSort}
            startIndex={0}
            testIdPrefix="my-mcps"
          />
        )}
      </div>

      <AlertDialog open={!!deleteTarget} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <AlertDialogContent data-testid="my-mcps-delete-dialog">
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
