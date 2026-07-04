import { useCallback, useEffect, useMemo, useState } from 'react';

import type { AppAccessSummary } from '@bodhiapp/ts-client';
import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { EmptyState } from '@/components/EmptyState';
import { ShellIcon, useCollapsibleSearch, useListKeyNav, useShellChrome } from '@/components/shell';
import { Skeleton } from '@/components/ui/skeleton';
import '@/components/shell/api-keys.css';
import '@/components/shell/list.css';
import '@/components/shell/tokens.css';
import { useListAppAccess, useRevokeAppAccess } from '@/hooks/apps';
import { useGetAppInfo } from '@/hooks/info';
import { useToastMessages } from '@/hooks/useToastMessages';
import { type CatalogColumn, CatalogTable } from '@/routes/models/explore/-shared/catalog-table';
import {
  DetailRow,
  fmtDate,
  GrantChips,
  grantSummary,
  useUrlMirroredSelection,
} from '@/routes/tokens/-shared/token-rail';

const appsSearchSchema = z.object({ select: z.string().optional() });

export const Route = createFileRoute('/tokens/apps/')({
  staticData: { section: 'api-keys', subPage: 'app-tokens' },
  validateSearch: appsSearchSchema,
  component: AppTokensPage,
});

const BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Access Tokens', href: '/tokens/' },
  { label: 'App Tokens', current: true },
];

const appDisplayName = (app: AppAccessSummary) => app.app_name || app.app_client_id;

export function AppTokensPageContent() {
  useListKeyNav();
  const { isLoading: appLoading } = useGetAppInfo();
  const { showSuccess, showError } = useToastMessages();

  const { data, isLoading } = useListAppAccess({ enabled: !appLoading });
  const [search, setSearch] = useState('');
  const { selectedId, select: selectApp } = useUrlMirroredSelection('/tokens/apps/');

  const { mutate: revoke } = useRevokeAppAccess({
    onSuccess: () => {
      showSuccess('Access Revoked', 'The app can no longer access your resources.');
      selectApp(null);
    },
    onError: (message) => showError('Error', message),
  });
  const onRevoke = useCallback((app: AppAccessSummary) => revoke({ id: app.id }), [revoke]);

  const searchNode = useCollapsibleSearch({
    value: search,
    onChange: setSearch,
    placeholder: 'Search apps by name…',
    toggleTestId: 'app-tokens-search-toggle',
    closeTestId: 'app-tokens-search-close',
  });

  const apps = useMemo(() => data?.data ?? [], [data]);
  const visible = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return apps;
    return apps.filter((a) => appDisplayName(a).toLowerCase().includes(q) || a.app_client_id.toLowerCase().includes(q));
  }, [apps, search]);

  const selected = useMemo(() => apps.find((a) => a.id === selectedId) ?? null, [apps, selectedId]);

  const columns = useMemo<CatalogColumn<AppAccessSummary, never>[]>(
    () => [
      { key: 'num', label: '', width: '52px', cell: () => null },
      {
        key: 'app',
        label: 'App',
        width: '',
        cell: (a) => (
          <div className="tk-id-cell">
            <div className="token-name" data-testid={`app-name-${a.id}`}>
              {appDisplayName(a)}
            </div>
            <div className="token-meta" data-testid={`app-client-${a.id}`}>
              {a.app_client_id}
            </div>
          </div>
        ),
      },
      {
        key: 'models',
        label: 'Models',
        width: '112px',
        cell: (a) => <span className="tk-grant">{grantSummary(a.models, 'model')}</span>,
      },
      {
        key: 'mcps',
        label: 'Tools',
        width: '100px',
        cell: (a) => <span className="tk-grant">{grantSummary(a.mcps, 'tool')}</span>,
      },
      {
        key: 'created',
        label: 'Granted',
        width: '116px',
        cell: (a) => <span className="tk-date-val">{fmtDate(a.created_at)}</span>,
      },
    ],
    []
  );

  const railHeader = useMemo(
    () => (selected ? <AppRailHeader app={selected} onClose={() => selectApp(null)} /> : null),
    [selected, selectApp]
  );
  const rail = useMemo(
    () => (selected ? <AppDetailPanel app={selected} onRevoke={onRevoke} /> : null),
    [selected, onRevoke]
  );

  useShellChrome({ breadcrumb: BREADCRUMB, rail, railHeader, railDefaultOpen: false });

  if (appLoading) {
    return (
      <div className="api-keys-screen cat-screen l-page" data-testid="app-tokens-page" data-pagestatus="loading">
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
      data-testid="app-tokens-page"
      data-pagestatus={isLoading ? 'loading' : 'ready'}
    >
      <div className="l-controls">
        {searchNode.row}
        <div className="l-toolbar">
          <div className="page-subtitle">Apps you've granted access to your resources.</div>
          <div className="l-tb-actions">{searchNode.toggle}</div>
        </div>
      </div>

      <div className="l-scroll" data-testid="app-tokens-table">
        {isLoading ? (
          <div style={{ padding: 16 }} data-testid="loading-skeleton">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-12 w-full mb-3" />
            ))}
          </div>
        ) : visible.length === 0 ? (
          <EmptyState
            icon="app-window"
            title="No connected apps"
            sub="When you approve a 3rd-party app's access request, it appears here."
            testId="app-tokens-empty"
          />
        ) : (
          <CatalogTable<AppAccessSummary, never>
            columns={columns}
            rows={visible}
            rowKey={(a) => a.id}
            rowTestId={(a) => `app-row-${a.id}`}
            rowAttrs={(a) => ({ 'data-test-client-id': a.app_client_id })}
            rowLabel={(a) => `Open app ${appDisplayName(a)}`}
            activeKey={selectedId}
            onSelect={(a) => selectApp(a.id)}
            onSort={() => {}}
            testIdPrefix="app-tokens"
          />
        )}
      </div>
    </div>
  );
}

function AppRailHeader({ app, onClose }: { app: AppAccessSummary; onClose: () => void }) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: 'var(--c-indigo-text)' }}>
        <ShellIcon name="app-window" size={15} />
      </div>
      <div className="dp-head-body">
        <div className="dp-head-title">{appDisplayName(app)}</div>
        <div className="dp-head-sub">{app.app_client_id}</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close">
        <ShellIcon name="x" size={15} />
      </button>
    </div>
  );
}

function AppDetailPanel({ app, onRevoke }: { app: AppAccessSummary; onRevoke: (app: AppAccessSummary) => void }) {
  const [confirmRevoke, setConfirmRevoke] = useState(false);
  useEffect(() => setConfirmRevoke(false), [app.id]);

  return (
    <div className="dp-panel" data-testid="app-detail-rail">
      <div className="dp-status-row">
        <span className={`status-chip ${app.status === 'approved' ? 'status-active' : 'status-inactive'}`}>
          {app.status === 'approved' ? 'active' : app.status}
        </span>
        {app.approved_role && (
          <span className={app.approved_role.includes('power') ? 'scope-power' : 'scope-user'}>
            {app.approved_role}
          </span>
        )}
      </div>
      <div className="dp-body">
        <div className="dp-section">
          <div className="dp-sec-lbl">Models</div>
          <div className="dp-rows">
            {app.models.list && <DetailRow icon="list" label="List all models" value="/v1/models" />}
            <DetailRow icon="cpu" label="Inference" value={grantSummary(app.models, 'model')} />
          </div>
          {app.models.type === 'specific' && <GrantChips ids={app.models.ids} testIdPrefix="app-model-grant" />}
        </div>
        <div className="dp-section">
          <div className="dp-sec-lbl">Connected tools</div>
          <div className="dp-rows">
            {app.mcps.list && <DetailRow icon="list" label="List all tools" value="/v1/mcps" />}
            <DetailRow icon="plug" label="Connect" value={grantSummary(app.mcps, 'tool')} />
          </div>
          {app.mcps.type === 'specific' && <GrantChips ids={app.mcps.ids} testIdPrefix="app-mcp-grant" />}
        </div>
        <div className="dp-section">
          <div className="dp-sec-lbl">Details</div>
          <div className="dp-rows">
            <DetailRow icon="hash" label="Client ID" value={app.app_client_id} />
            <DetailRow icon="calendar" label="Granted" value={fmtDate(app.created_at)} />
            <DetailRow icon="activity" label="Updated" value={fmtDate(app.updated_at)} />
          </div>
        </div>
      </div>
      <div className="dp-foot">
        {confirmRevoke ? (
          <button
            className="dp-btn dp-btn-danger is-confirm"
            onClick={() => onRevoke(app)}
            data-testid="app-revoke-confirm"
          >
            Confirm revoke
          </button>
        ) : (
          <button className="dp-btn dp-btn-danger" onClick={() => setConfirmRevoke(true)} data-testid="app-revoke">
            Revoke access
          </button>
        )}
      </div>
    </div>
  );
}

export function AppTokensPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <AppTokensPageContent />
    </AppInitializer>
  );
}
