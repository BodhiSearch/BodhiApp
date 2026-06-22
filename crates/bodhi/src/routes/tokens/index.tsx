import { useCallback, useMemo, useState } from 'react';

import { TokenDetail } from '@bodhiapp/ts-client';
import { createFileRoute } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';
import {
  LinkRow,
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
import { useListTokens, useUpdateToken } from '@/hooks/tokens';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useViewTransition } from '@/hooks/useViewTransition';

export const Route = createFileRoute('/tokens/')({
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

/* Presentation matches the App Tokens design, bound to real data only: no per-token
   model/MCP-access chips and no "last used" (the backend has neither). The status
   toggle drives the real active/inactive update; there is no revoke/delete endpoint. */

export function TokenPageContent() {
  useListKeyNav();
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

  const onStatusChange = useCallback(
    (token: TokenDetail, checked: boolean) =>
      updateToken({ id: token.id, name: token.name, status: checked ? 'active' : 'inactive' }),
    [updateToken]
  );

  const withViewTransition = useViewTransition();
  const [filter, setFilter] = useState<TokenFilter>('all');
  const [search, setSearch] = useState('');
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const searchNode = useCollapsibleSearch({
    value: search,
    onChange: setSearch,
    placeholder: 'Search tokens by name or id…',
    toggleTestId: 'tokens-search-toggle',
    closeTestId: 'tokens-search-close',
  });

  const selectToken = useCallback(
    (id: string | null) => withViewTransition(() => setSelectedId(id)),
    [withViewTransition]
  );

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

  const railHeader = useMemo(
    () => (selected ? <TokenRailHeader token={selected} onClose={() => selectToken(null)} /> : null),
    [selected, selectToken]
  );
  const rail = useMemo(
    () => (selected ? <TokenDetailPanel token={selected} onStatusChange={onStatusChange} /> : null),
    [selected, onStatusChange]
  );

  useShellChrome({
    breadcrumb: TOKEN_BREADCRUMB,
    rail,
    railHeader,
    railDefaultOpen: false,
  });

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
          <div className="empty-state" data-testid="tokens-empty">
            <div className="empty-icon">
              <ShellIcon name="key-round" size={28} />
            </div>
            <div className="empty-title">No tokens</div>
            <div className="empty-sub">Create an API token to access the Bodhi API programmatically.</div>
          </div>
        ) : (
          <div className="l-listview">
            <div className="l-listhead">
              <div className="l-lh tk-icon" />
              <div className="l-lh tk-id">Token</div>
              <div className="l-lh tk-created">Created</div>
              <div className="l-lh tk-used">Updated</div>
              <div className="l-lh tk-status">Status</div>
            </div>
            {visible.map((token) => (
              <TokenRow
                key={token.id}
                token={token}
                active={token.id === selectedId}
                onSelect={() => selectToken(token.id)}
                onStatusChange={onStatusChange}
              />
            ))}
          </div>
        )}
        {total > effPageSize && (
          <ShellPagination minimal total={total} page={page} onPage={setPage} pageSize={effPageSize} />
        )}
      </div>
    </div>
  );
}

function TokenKeyIcon() {
  return (
    <span className="token-key-icon">
      <ShellIcon name="key-round" size={16} />
    </span>
  );
}

function ScopeChip({ scopes }: { scopes: string }) {
  const power = scopes.includes('power');
  return <span className={power ? 'scope-power' : 'scope-user'}>{scopes}</span>;
}

interface TokenRowProps {
  token: TokenDetail;
  active: boolean;
  onSelect: () => void;
  onStatusChange: (token: TokenDetail, checked: boolean) => void;
}

function TokenRow({ token, active, onSelect, onStatusChange }: TokenRowProps) {
  const isActive = token.status === 'active';
  return (
    <div
      className={'l-listrow tk-row' + (active ? ' active' : '')}
      onClick={onSelect}
      role="option"
      aria-selected={active}
      data-testid={`token-row-${token.id}`}
    >
      <LinkRow onActivate={onSelect} label={`Open token ${token.name || 'Unnamed token'}`} />
      <div className="tk-icon">
        <TokenKeyIcon />
      </div>
      <div className="tk-id">
        <div className={'token-name' + (token.name ? '' : ' unnamed')} data-testid={`token-name-${token.id}`}>
          {token.name || 'Unnamed token'}
        </div>
        <div className="token-meta" data-testid={`token-scope-${token.id}`}>
          <ScopeChip scopes={token.scopes} />
        </div>
      </div>
      <div className="tk-created">
        <span className="tk-date-lbl">Created</span>
        <span className="tk-date-val">{fmtDate(token.created_at)}</span>
      </div>
      <div className="tk-used">
        <span className="tk-date-lbl">Updated</span>
        <span className="tk-date-val">{fmtDate(token.updated_at)}</span>
      </div>
      <div className="tk-status" onClick={(e) => e.stopPropagation()}>
        <Switch
          checked={isActive}
          onCheckedChange={(checked) => onStatusChange(token, checked)}
          aria-label="Toggle token status"
          data-testid={`token-status-switch-${token.id}`}
        />
        <span className={'status-chip ' + (isActive ? 'status-active' : 'status-inactive')}>{token.status}</span>
      </div>
    </div>
  );
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

function TokenDetailPanel({
  token,
  onStatusChange,
}: {
  token: TokenDetail;
  onStatusChange: (token: TokenDetail, checked: boolean) => void;
}) {
  const isActive = token.status === 'active';
  return (
    <div className="dp-panel" data-testid="token-detail-rail">
      <div className="dp-status-row">
        <span className={'status-chip ' + (isActive ? 'status-active' : 'status-inactive')}>{token.status}</span>
        <span className={isActive ? 'scope-power' : 'scope-user'}>{token.scopes}</span>
      </div>
      <div className="dp-body">
        <div className="dp-section">
          <div className="dp-sec-lbl">Details</div>
          <div className="dp-rows">
            <div className="dp-row">
              <span className="dp-row-k">
                <ShellIcon name="hash" size={13} /> Token ID
              </span>
              <span className="dp-row-v mono">{token.token_prefix}</span>
            </div>
            <div className="dp-row">
              <span className="dp-row-k">
                <ShellIcon name="shield" size={13} /> Scope
              </span>
              <span className="dp-row-v">{scopeLabel(token.scopes)}</span>
            </div>
            <div className="dp-row">
              <span className="dp-row-k">
                <ShellIcon name="calendar" size={13} /> Created
              </span>
              <span className="dp-row-v">{fmtDate(token.created_at)}</span>
            </div>
            <div className="dp-row">
              <span className="dp-row-k">
                <ShellIcon name="activity" size={13} /> Updated
              </span>
              <span className="dp-row-v">{fmtDate(token.updated_at)}</span>
            </div>
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
