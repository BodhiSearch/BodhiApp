import { useMemo, useState } from 'react';

import { TokenDetail } from '@bodhiapp/ts-client';
import { createFileRoute, useNavigate } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';
import { Pagination } from '@/components/DataTable';
import { ShellIcon, useShellChrome } from '@/components/shell';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import '@/components/shell/api-keys.css';
import '@/components/shell/list.css';
import { useGetAppInfo } from '@/hooks/info';
import { useListTokens, useUpdateToken } from '@/hooks/tokens';
import { useToastMessages } from '@/hooks/use-toast-messages';

export const Route = createFileRoute('/tokens/')({
  component: TokenPage,
});

function StatusBadge({ status }: { status: string }) {
  const variant = status === 'active' ? 'default' : 'secondary';
  return <Badge variant={variant}>{status}</Badge>;
}

const TOKEN_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'API Keys', href: '/tokens/' },
  { label: 'App Tokens', current: true },
];

type TokenFilter = 'all' | 'active' | 'inactive';

const FILTER_TABS: { id: TokenFilter; label: string }[] = [
  { id: 'all', label: 'All' },
  { id: 'active', label: 'Active' },
  { id: 'inactive', label: 'Revoked' },
];

/* Presentation: real fields only (name, scope, status, prefix, created/updated).
   The prototype's per-token model/MCP-access badges + "last used" have no backing
   data and are intentionally omitted until those backend features land. */

export function TokenPageContent() {
  const { isLoading: appLoading } = useGetAppInfo();
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);
  const { showSuccess, showError } = useToastMessages();

  const { mutate: updateToken } = useUpdateToken({
    onSuccess: (token) => showSuccess('Token Updated', `Token status changed to ${token.status}`),
    onError: (message) => showError('Error', message),
  });

  const { data: tokensData, isLoading: tokensLoading } = useListTokens(page, pageSize, {
    enabled: !appLoading,
  });

  const onStatusChange = (token: TokenDetail, checked: boolean) => {
    updateToken({ id: token.id, name: token.name, status: checked ? 'active' : 'inactive' });
  };

  const navigate = useNavigate();
  const [filter, setFilter] = useState<TokenFilter>('all');
  const [search, setSearch] = useState('');
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const tokens = tokensData?.data ?? [];
  const total = tokensData?.total ?? 0;
  const effPageSize = tokensData?.page_size ?? pageSize;

  // Counts derived during render from the fetched page (no extra fetch / state).
  const counts = useMemo(() => {
    let active = 0;
    let inactive = 0;
    for (const t of tokens) {
      if (t.status === 'active') active++;
      else inactive++;
    }
    return { all: tokens.length, active, inactive };
  }, [tokens]);

  const visible = useMemo(() => {
    const q = search.trim().toLowerCase();
    return tokens.filter((t) => {
      if (filter !== 'all' && t.status !== filter) return false;
      if (!q) return true;
      return (t.name ?? '').toLowerCase().includes(q) || t.token_prefix.toLowerCase().includes(q);
    });
  }, [tokens, filter, search]);

  const selected = useMemo(() => tokens.find((t) => t.id === selectedId) ?? null, [tokens, selectedId]);

  const headerActions = useMemo(
    () => (
      <button className="btn-accent" data-testid="new-token-button" onClick={() => navigate({ to: '/tokens/new/' })}>
        <ShellIcon name="plus" size={14} />
        New Token
      </button>
    ),
    [navigate]
  );

  const rail = useMemo(
    () => (selected ? <TokenDetailRail token={selected} onClose={() => setSelectedId(null)} /> : null),
    [selected]
  );

  useShellChrome({ breadcrumb: TOKEN_BREADCRUMB, headerActions, rail, railDefaultOpen: false });

  if (appLoading) {
    return (
      <div className="api-keys-screen l-page" data-testid="tokens-page" data-pagestatus="loading">
        <div className="space-y-4 p-4">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-1/4" />
        </div>
      </div>
    );
  }

  return (
    <div
      className="api-keys-screen l-page"
      data-testid="tokens-page"
      data-pagestatus={tokensLoading ? 'loading' : 'ready'}
    >
      <div className="l-controls">
        <div className="l-searchrow">
          <div className="search-wrap" style={{ flex: 1, maxWidth: 280 }}>
            <span className="search-icon">
              <ShellIcon name="search" size={14} />
            </span>
            <input
              className="search-input"
              style={{ width: '100%' }}
              placeholder="Search tokens…"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              data-testid="tokens-search"
            />
          </div>
        </div>
        <div className="l-toolbar">
          <div className="filter-tabs" role="tablist" aria-label="Filter tokens">
            {FILTER_TABS.map((tab) => (
              <button
                key={tab.id}
                role="tab"
                aria-selected={filter === tab.id}
                className={'filter-tab' + (filter === tab.id ? ' active' : '')}
                onClick={() => setFilter(tab.id)}
                data-testid={`tokens-filter-${tab.id}`}
              >
                {tab.label}
                <span className="tab-count">{counts[tab.id]}</span>
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="l-scroll" data-testid="tokens-table">
        {visible.length === 0 ? (
          <div className="empty-state" data-testid="tokens-empty">
            <div className="empty-icon">
              <ShellIcon name="key-round" size={28} />
            </div>
            <div className="empty-title">No tokens</div>
            <div className="empty-sub">Create an API token to access the Bodhi API programmatically.</div>
          </div>
        ) : (
          <div className="l-listview">
            {visible.map((token) => (
              <TokenRow
                key={token.id}
                token={token}
                active={token.id === selectedId}
                onSelect={() => setSelectedId(token.id)}
                onStatusChange={onStatusChange}
              />
            ))}
          </div>
        )}
        {total > effPageSize && (
          <div style={{ padding: '14px 16px' }} data-testid="pagination">
            <Pagination page={page} totalPages={Math.ceil(total / effPageSize)} onPageChange={setPage} />
          </div>
        )}
      </div>
    </div>
  );
}

interface TokenRowProps {
  token: TokenDetail;
  active: boolean;
  onSelect: () => void;
  onStatusChange: (token: TokenDetail, checked: boolean) => void;
}

function TokenRow({ token, active, onSelect, onStatusChange }: TokenRowProps) {
  return (
    <div className={'l-listrow' + (active ? ' active' : '')} onClick={onSelect} data-testid={`token-row-${token.id}`}>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div data-testid={`token-name-${token.id}`} style={{ fontWeight: 600, fontSize: 13.5 }}>
          {token.name || '-'}
        </div>
        <div data-testid={`token-scope-${token.id}`}>
          <span className="tag tag-muted">{token.scopes}</span>
        </div>
      </div>
      <div className="flex items-center gap-2" onClick={(e) => e.stopPropagation()}>
        <Switch
          checked={token.status === 'active'}
          onCheckedChange={(checked) => onStatusChange(token, checked)}
          aria-label="Toggle token status"
          data-testid={`token-status-switch-${token.id}`}
        />
        <StatusBadge status={token.status} />
      </div>
    </div>
  );
}

function TokenDetailRail({ token, onClose }: { token: TokenDetail; onClose: () => void }) {
  return (
    <div className="dp-panel" data-testid="token-detail-rail">
      <div className="dp-status-row">
        <StatusBadge status={token.status} />
        <button className="dp-close" onClick={onClose} title="Close" style={{ marginLeft: 'auto' }}>
          <ShellIcon name="x" size={15} />
        </button>
      </div>
      <div className="dp-body">
        <div className="dp-section">
          <div className="dp-sec-lbl">Details</div>
          <div className="dp-rows">
            <div className="dp-row">
              <span className="dp-row-k">Name</span>
              <span className="dp-row-v">{token.name || '-'}</span>
            </div>
            <div className="dp-row">
              <span className="dp-row-k">Scope</span>
              <span className="dp-row-v">{token.scopes}</span>
            </div>
            <div className="dp-row">
              <span className="dp-row-k">Prefix</span>
              <span className="dp-row-v mono">{token.token_prefix}</span>
            </div>
            <div className="dp-row">
              <span className="dp-row-k">Created</span>
              <span className="dp-row-v">{new Date(token.created_at).toLocaleDateString()}</span>
            </div>
            <div className="dp-row">
              <span className="dp-row-k">Updated</span>
              <span className="dp-row-v">{new Date(token.updated_at).toLocaleDateString()}</span>
            </div>
          </div>
        </div>
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
