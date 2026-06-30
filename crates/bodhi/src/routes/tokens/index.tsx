import { useCallback, useEffect, useMemo, useState } from 'react';

import { TokenDetail, TokenGrants } from '@bodhiapp/ts-client';
import { createFileRoute, useNavigate } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { EmptyState } from '@/components/EmptyState';
import {
  ShellFilterTabs,
  ShellIcon,
  ShellPagination,
  useCollapsibleSearch,
  useListKeyNav,
  useShellChrome,
} from '@/components/shell';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import '@/components/shell/api-keys.css';
import '@/components/shell/list.css';
import '@/components/shell/tokens.css';
import { useGetAppInfo } from '@/hooks/info';
import { useDeleteToken, useListTokens, useUpdateToken } from '@/hooks/tokens';
import { useToastMessages } from '@/hooks/useToastMessages';
import { useViewTransition } from '@/hooks/useViewTransition';
import { type CatalogColumn, CatalogTable } from '@/routes/models/explore/-shared/catalog-table';

// `select` carries the open detail rail (the token id). Written with replace (no history entries);
// browser Back/Forward restores the rail from whatever the target URL holds.
const tokensSearchSchema = z.object({
  select: z.string().optional(),
});

export const Route = createFileRoute('/tokens/')({
  staticData: { section: 'api-keys', subPage: 'api-tokens' },
  validateSearch: tokensSearchSchema,
  component: TokenPage,
});

const TOKEN_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Access Tokens', href: '/tokens/' },
  { label: 'API Tokens', current: true },
];

type TokenFilter = 'all' | 'active' | 'inactive';

const FILTER_TABS: { id: TokenFilter; label: string }[] = [
  { id: 'all', label: 'All' },
  { id: 'active', label: 'Active' },
  { id: 'inactive', label: 'Inactive' },
];

const scopeLabel = (scopes: string) => (scopes.includes('power') ? 'Power User' : 'User');
const fmtDate = (iso: string) =>
  new Date(iso).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });

const readSelectFromUrl = () => new URLSearchParams(window.location.search).get('select');

export function TokenPageContent() {
  useListKeyNav();
  const { isLoading: appLoading } = useGetAppInfo();
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);
  const { showSuccess, showError } = useToastMessages();
  const navigate = useNavigate();

  const { mutate: updateToken } = useUpdateToken({
    onSuccess: (token) => showSuccess('Token Updated', `Token status changed to ${token.status}`),
    onError: (message) => showError('Error', message),
  });

  const { data: tokensData, isLoading: tokensLoading } = useListTokens(page, pageSize, {
    enabled: !appLoading,
  });

  const onStatusChange = useCallback(
    (token: TokenDetail, checked: boolean) =>
      updateToken({ id: token.id, name: token.name, status: checked ? 'active' : 'inactive' }),
    [updateToken]
  );

  const withViewTransition = useViewTransition();
  const [filter, setFilter] = useState<TokenFilter>('all');
  const [search, setSearch] = useState('');
  // Render source of truth is local; the URL is mirrored so links are shareable and browser
  // Back/Forward works (the popstate listener pulls the id back out of the URL).
  const [selectedId, setSelectedId] = useState<string | null>(() => readSelectFromUrl());

  useEffect(() => {
    const onPop = () => setSelectedId(readSelectFromUrl());
    window.addEventListener('popstate', onPop);
    return () => window.removeEventListener('popstate', onPop);
  }, []);

  const selectToken = useCallback(
    (id: string | null) => {
      withViewTransition(() => setSelectedId(id));
      navigate({ to: '/tokens/', search: (prev) => ({ ...prev, select: id ?? undefined }), replace: true });
    },
    [withViewTransition, navigate]
  );

  const searchNode = useCollapsibleSearch({
    value: search,
    onChange: setSearch,
    placeholder: 'Search tokens by name or id…',
    toggleTestId: 'tokens-search-toggle',
    closeTestId: 'tokens-search-close',
  });

  const { mutate: deleteToken } = useDeleteToken({
    onSuccess: () => {
      showSuccess('Token Deleted', 'The token has been permanently revoked.');
      selectToken(null);
    },
    onError: (message) => showError('Error', message),
  });
  const onDelete = useCallback((token: TokenDetail) => deleteToken(token.id), [deleteToken]);

  const tokens = tokensData?.data ?? [];
  const total = tokensData?.total ?? 0;
  const effPageSize = tokensData?.page_size ?? pageSize;

  // Counts reflect only the current page; counting from fetched rows avoids fetching all tokens.
  const counts = useMemo(() => {
    let active = 0;
    let inactive = 0;
    for (const t of tokens) {
      if (t.status === 'active') active++;
      else inactive++;
    }
    return { all: tokens.length, active, inactive };
  }, [tokens]);

  const filterTabs = useMemo(() => FILTER_TABS.map((t) => ({ ...t, count: counts[t.id] })), [counts]);

  const visible = useMemo(() => {
    const q = search.trim().toLowerCase();
    return tokens.filter((t) => {
      if (filter !== 'all' && t.status !== filter) return false;
      if (!q) return true;
      return (t.name ?? '').toLowerCase().includes(q) || t.token_prefix.toLowerCase().includes(q);
    });
  }, [tokens, filter, search]);

  const selected = useMemo(() => tokens.find((t) => t.id === selectedId) ?? null, [tokens, selectedId]);

  const columns = useMemo<CatalogColumn<TokenDetail>[]>(
    () => [
      { key: 'num', label: '', width: '52px', cell: () => null },
      {
        key: 'token',
        label: 'Token',
        width: '',
        cell: (t) => (
          <div className="tk-id-cell">
            <div className={'token-name' + (t.name ? '' : ' unnamed')} data-testid={`token-name-${t.id}`}>
              {t.name || 'Unnamed token'}
            </div>
            <div className="token-meta" data-testid={`token-scope-${t.id}`}>
              <ScopeChip scopes={t.scopes} />
            </div>
          </div>
        ),
      },
      {
        key: 'models',
        label: 'Models',
        width: '112px',
        cell: (t) => <span className="tk-grant">{modelGrantLabel(t.grants)}</span>,
      },
      {
        key: 'mcps',
        label: 'MCPs',
        width: '100px',
        cell: (t) => <span className="tk-grant">{mcpGrantLabel(t.grants)}</span>,
      },
      {
        key: 'created',
        label: 'Created',
        width: '116px',
        cell: (t) => <span className="tk-date-val">{fmtDate(t.created_at)}</span>,
      },
      {
        key: 'updated',
        label: 'Updated',
        width: '116px',
        cell: (t) => <span className="tk-date-val">{fmtDate(t.updated_at)}</span>,
      },
      {
        key: 'status',
        label: 'Status',
        width: '140px',
        cell: (t) => {
          const isActive = t.status === 'active';
          return (
            <span className="tk-status-cell" onClick={(e) => e.stopPropagation()}>
              <Switch
                checked={isActive}
                onCheckedChange={(checked) => onStatusChange(t, checked)}
                aria-label="Toggle token status"
                data-testid={`token-status-switch-${t.id}`}
              />
              <span className={'status-chip ' + (isActive ? 'status-active' : 'status-inactive')}>{t.status}</span>
            </span>
          );
        },
      },
    ],
    [onStatusChange]
  );

  const railHeader = useMemo(
    () => (selected ? <TokenRailHeader token={selected} onClose={() => selectToken(null)} /> : null),
    [selected, selectToken]
  );
  const rail = useMemo(
    () => (selected ? <TokenDetailPanel token={selected} onStatusChange={onStatusChange} onDelete={onDelete} /> : null),
    [selected, onStatusChange, onDelete]
  );

  useShellChrome({
    breadcrumb: TOKEN_BREADCRUMB,
    rail,
    railHeader,
    railDefaultOpen: false,
  });

  if (appLoading) {
    return (
      <div className="api-keys-screen cat-screen l-page" data-testid="tokens-page" data-pagestatus="loading">
        <div className="space-y-4 p-4">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-1/4" />
        </div>
      </div>
    );
  }

  return (
    <div
      className="api-keys-screen cat-screen l-page"
      data-testid="tokens-page"
      data-pagestatus={tokensLoading ? 'loading' : 'ready'}
    >
      <div className="l-controls">
        {searchNode.row}
        <div className="l-toolbar">
          <ShellFilterTabs
            tabs={filterTabs}
            value={filter}
            onChange={setFilter}
            label="Filter tokens"
            testIdPrefix="tokens-filter"
            loading={tokensLoading}
          />
          <div className="l-tb-actions">{searchNode.toggle}</div>
        </div>
      </div>

      <div className="l-scroll" data-testid="tokens-table">
        {tokensLoading ? (
          <div style={{ padding: 16 }} data-testid="loading-skeleton">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-12 w-full mb-3" />
            ))}
          </div>
        ) : visible.length === 0 ? (
          <EmptyState
            icon="key-round"
            title="No tokens"
            sub="Create an API token to access the Bodhi API programmatically."
            testId="tokens-empty"
          />
        ) : (
          <CatalogTable<TokenDetail, never>
            columns={columns}
            rows={visible}
            rowKey={(t) => t.id}
            rowTestId={(t) => `token-row-${t.id}`}
            rowLabel={(t) => `Open token ${t.name || 'Unnamed token'}`}
            activeKey={selectedId}
            onSelect={(t) => selectToken(t.id)}
            onSort={() => {}}
            testIdPrefix="tokens"
          />
        )}
        {total > effPageSize && (
          <ShellPagination minimal total={total} page={page} onPage={setPage} pageSize={effPageSize} />
        )}
      </div>
    </div>
  );
}

function ScopeChip({ scopes }: { scopes: string }) {
  const power = scopes.includes('power');
  return <span className={power ? 'scope-power' : 'scope-user'}>{scopes}</span>;
}

function TokenRailHeader({ token, onClose }: { token: TokenDetail; onClose: () => void }) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: 'var(--c-indigo-text)' }}>
        <ShellIcon name="key-round" size={15} />
      </div>
      <div className="dp-head-body">
        <div className={'dp-head-title' + (token.name ? ' mono' : '')}>{token.name || 'Unnamed token'}</div>
        <div className="dp-head-sub">{token.token_prefix}</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close">
        <ShellIcon name="x" size={15} />
      </button>
    </div>
  );
}

function DetailRow({ icon, label, value }: { icon: string; label: string; value: string }) {
  return (
    <div className="dp-row">
      <span className="dp-row-k">
        <ShellIcon name={icon} size={13} /> {label}
      </span>
      <span className="dp-row-v">{value}</span>
    </div>
  );
}

function modelGrantLabel(grants: TokenGrants): string {
  if (grants.models.type === 'all') return 'All models';
  const n = grants.models.ids.length;
  return n ? `${n} model${n === 1 ? '' : 's'}` : 'No models';
}

function mcpGrantLabel(grants: TokenGrants): string {
  if (grants.mcps.type === 'all') return 'All MCPs';
  const n = grants.mcps.ids.length;
  return n ? `${n} MCP${n === 1 ? '' : 's'}` : 'No MCPs';
}

function TokenDetailPanel({
  token,
  onStatusChange,
  onDelete,
}: {
  token: TokenDetail;
  onStatusChange: (token: TokenDetail, checked: boolean) => void;
  onDelete: (token: TokenDetail) => void;
}) {
  const isActive = token.status === 'active';
  const grants = token.grants;
  const [confirmDelete, setConfirmDelete] = useState(false);
  useEffect(() => setConfirmDelete(false), [token.id]);

  return (
    <div className="dp-panel" data-testid="token-detail-rail">
      <div className="dp-status-row">
        <span className={'status-chip ' + (isActive ? 'status-active' : 'status-inactive')}>{token.status}</span>
        <span className={isActive ? 'scope-power' : 'scope-user'}>{token.scopes}</span>
      </div>
      <div className="dp-body">
        <div className="dp-section">
          <div className="dp-sec-lbl">Models</div>
          <div className="dp-rows">
            {grants.models_list && <DetailRow icon="list" label="List all models" value="/v1/models" />}
            <DetailRow icon="cpu" label="Inference" value={modelGrantLabel(grants)} />
          </div>
          {grants.models.type === 'specific' && grants.models.ids.length > 0 && (
            <div className="dp-chips">
              {grants.models.ids.map((m) => (
                <span key={m} className="dp-chip" data-testid={`token-model-grant-${m}`}>
                  {m}
                </span>
              ))}
            </div>
          )}
        </div>
        <div className="dp-section">
          <div className="dp-sec-lbl">MCP servers</div>
          <div className="dp-rows">
            {grants.mcps_list && <DetailRow icon="list" label="List all MCPs" value="/v1/mcps" />}
            <DetailRow icon="plug" label="Connect" value={mcpGrantLabel(grants)} />
          </div>
          {grants.mcps.type === 'specific' && grants.mcps.ids.length > 0 && (
            <div className="dp-chips">
              {grants.mcps.ids.map((m) => (
                <span key={m} className="dp-chip" data-testid={`token-mcp-grant-${m}`}>
                  {m}
                </span>
              ))}
            </div>
          )}
        </div>
        <div className="dp-section">
          <div className="dp-sec-lbl">Details</div>
          <div className="dp-rows">
            <DetailRow icon="hash" label="Token ID" value={token.token_prefix} />
            <DetailRow icon="shield" label="Scope" value={scopeLabel(token.scopes)} />
            <DetailRow icon="calendar" label="Created" value={fmtDate(token.created_at)} />
            <DetailRow icon="activity" label="Updated" value={fmtDate(token.updated_at)} />
          </div>
        </div>
      </div>
      <div className="dp-foot">
        <div className="dp-toggle-row">
          <span className="dp-toggle-label">Token active</span>
          <Switch
            checked={isActive}
            onCheckedChange={(checked) => onStatusChange(token, checked)}
            aria-label="Toggle token status"
            data-testid="token-detail-status-switch"
          />
        </div>
        {confirmDelete ? (
          <button
            className="dp-btn dp-btn-danger is-confirm"
            onClick={() => onDelete(token)}
            data-testid="token-delete-confirm"
          >
            Confirm delete
          </button>
        ) : (
          <button className="dp-btn dp-btn-danger" onClick={() => setConfirmDelete(true)} data-testid="token-delete">
            Delete token
          </button>
        )}
      </div>
    </div>
  );
}

export function TokenPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <TokenPageContent />
    </AppInitializer>
  );
}
