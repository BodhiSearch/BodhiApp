import { useEffect, useMemo, useState } from 'react';

import type { McpAuthConfigParamInput, McpAuthConfigResponse } from '@bodhiapp/ts-client';
import { createFileRoute, useNavigate, useSearch } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { ShellIcon, useShellChrome } from '@/components/shell';
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
import { Button } from '@/components/ui/button';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import {
  useCreateAuthConfig,
  useDeleteAuthConfig,
  useListAuthConfigs,
  useGetMcpServer,
  useStandaloneDynamicRegister,
  useUpdateMcpServer,
} from '@/hooks/mcps';
import { toast } from '@/hooks/use-toast';
import { ROUTE_MCPS } from '@/lib/constants';
import { extractErrorMessage } from '@/lib/errorUtils';
import { authConfigDetail } from '@/lib/mcpUtils';
import { authKind } from '@/routes/mcps/-shared/auth-badges';
import { AuthConfigForm } from '@/routes/mcps/servers/-components/AuthConfigForm';
import '@/routes/mcps/servers/-components/server-config.css';
import '@/routes/mcps/-shared/auth-badges.css';

export const Route = createFileRoute('/mcps/servers/view/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: ServerViewPage,
});

const SERVER_VIEW_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'MCP', href: ROUTE_MCPS },
  { label: 'Configure server', current: true },
];

const KIND_ICON: Record<string, string> = { oauth: 'lock', key: 'key', public: 'unlock', http: 'shield' };
const KIND_LABEL: Record<string, string> = { oauth: 'OAuth', key: 'API Key', public: 'Public', http: 'HTTP' };

function ServerViewContent() {
  useShellChrome({ breadcrumb: useMemo(() => SERVER_VIEW_BREADCRUMB, []) });
  const search = useSearch({ from: '/mcps/servers/view/' });
  const navigate = useNavigate();
  const serverId = search.id || '';

  const {
    data: server,
    isLoading: serverLoading,
    error: serverError,
  } = useGetMcpServer(serverId, { enabled: !!serverId });
  const { data: authConfigsData, isLoading: configsLoading } = useListAuthConfigs(serverId);

  // ── Basic-information inline edit (per-section, URL locked) ──
  const [editingBasic, setEditingBasic] = useState(false);
  const [savedBasic, setSavedBasic] = useState(false);
  const [draftName, setDraftName] = useState('');
  const [draftDesc, setDraftDesc] = useState('');
  const [draftEnabled, setDraftEnabled] = useState(true);
  useEffect(() => {
    if (server) {
      setDraftName(server.name);
      setDraftDesc(server.description ?? '');
      setDraftEnabled(server.enabled);
    }
  }, [server]);

  const updateServer = useUpdateMcpServer({
    onSuccess: () => {
      setEditingBasic(false);
      setSavedBasic(true);
      setTimeout(() => setSavedBasic(false), 2200);
    },
    onError: (message) => toast({ title: 'Failed to update server', description: message, variant: 'destructive' }),
  });
  const saveBasic = () => {
    if (!server || !draftName.trim()) return;
    updateServer.mutate({
      id: serverId,
      url: server.url,
      name: draftName.trim(),
      description: draftDesc.trim() || undefined,
      enabled: draftEnabled,
    });
  };

  // ── Auth mechanisms: inline add + delete ──
  const [deleteTarget, setDeleteTarget] = useState<McpAuthConfigResponse | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [formType, setFormType] = useState<'header' | 'oauth'>('header');
  const [formName, setFormName] = useState('');
  const [formEntries, setFormEntries] = useState<McpAuthConfigParamInput[]>([{ param_type: 'header', param_key: '' }]);
  const [formRegistrationType, setFormRegistrationType] = useState<'pre_registered' | 'dynamic_registration'>(
    'pre_registered'
  );
  const [formClientId, setFormClientId] = useState('');
  const [formClientSecret, setFormClientSecret] = useState('');
  const [formAuthEndpoint, setFormAuthEndpoint] = useState('');
  const [formTokenEndpoint, setFormTokenEndpoint] = useState('');
  const [formScopes, setFormScopes] = useState('');
  const [formRegistrationEndpoint, setFormRegistrationEndpoint] = useState('');

  const standaloneDcr = useStandaloneDynamicRegister({
    onError: (message) => toast({ title: 'Dynamic registration failed', description: message, variant: 'destructive' }),
  });
  const deleteAuthConfig = useDeleteAuthConfig({
    onSuccess: () => {
      toast({ title: 'Auth mechanism deleted' });
      setDeleteTarget(null);
    },
    onError: (message) => {
      toast({ title: 'Failed to delete auth mechanism', description: message, variant: 'destructive' });
      setDeleteTarget(null);
    },
  });
  const createAuthConfig = useCreateAuthConfig({
    onSuccess: () => {
      toast({ title: 'Auth mechanism added' });
      setShowForm(false);
      resetForm();
    },
    onError: (message) =>
      toast({ title: 'Failed to add auth mechanism', description: message, variant: 'destructive' }),
  });

  const authConfigs = authConfigsData?.auth_configs ?? [];

  const resetForm = () => {
    setFormName('');
    setFormEntries([{ param_type: 'header', param_key: '' }]);
    setFormRegistrationType('pre_registered');
    setFormClientId('');
    setFormClientSecret('');
    setFormAuthEndpoint('');
    setFormTokenEndpoint('');
    setFormScopes('');
    setFormRegistrationEndpoint('');
  };

  const handleCreateSubmit = async () => {
    if (formType === 'header') {
      createAuthConfig.mutate({
        mcp_server_id: serverId,
        type: 'header',
        name: formName,
        entries: formEntries.filter((e) => e.param_key.trim() !== ''),
      });
    } else if (formType === 'oauth' && formRegistrationType === 'dynamic_registration') {
      if (!formRegistrationEndpoint) {
        toast({ title: 'Registration endpoint is required for dynamic registration', variant: 'destructive' });
        return;
      }
      const redirectUri = `${window.location.origin}/ui/mcps/oauth/callback/`;
      try {
        const dcrResponse = await standaloneDcr.mutateAsync({
          registration_endpoint: formRegistrationEndpoint,
          redirect_uri: redirectUri,
          scopes: formScopes || undefined,
        });
        const dcrResult = dcrResponse.data;
        createAuthConfig.mutate({
          mcp_server_id: serverId,
          type: 'oauth',
          name: formName || 'OAuth',
          registration_type: 'dynamic_registration',
          authorization_endpoint: formAuthEndpoint,
          token_endpoint: formTokenEndpoint,
          registration_endpoint: formRegistrationEndpoint || undefined,
          scopes: formScopes || undefined,
          client_id: dcrResult.client_id,
          client_secret: dcrResult.client_secret ?? undefined,
          token_endpoint_auth_method: dcrResult.token_endpoint_auth_method ?? undefined,
          client_id_issued_at: dcrResult.client_id_issued_at ?? undefined,
          registration_access_token: dcrResult.registration_access_token ?? undefined,
        });
      } catch {
        // Error already handled by hook's onError
      }
    } else if (formType === 'oauth' && formRegistrationType === 'pre_registered') {
      createAuthConfig.mutate({
        mcp_server_id: serverId,
        type: 'oauth',
        name: formName,
        registration_type: 'pre_registered',
        client_id: formClientId,
        client_secret: formClientSecret || undefined,
        authorization_endpoint: formAuthEndpoint,
        token_endpoint: formTokenEndpoint,
        scopes: formScopes || undefined,
      });
    }
  };

  if (!serverId) return <ErrorPage message="No server ID provided" />;
  if (serverError) return <ErrorPage message={extractErrorMessage(serverError, 'Failed to load MCP server')} />;

  if (serverLoading || !server) {
    return (
      <div className="container mx-auto max-w-3xl px-4 py-6" data-testid="server-view-loading">
        <div className="sc-card">
          <div className="sc-card-head">
            <Skeleton className="h-6 w-48" />
          </div>
          <div className="sc-card-body space-y-3">
            <Skeleton className="h-4 w-full" />
            <Skeleton className="h-4 w-3/4" />
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="container mx-auto max-w-3xl px-4 py-6" data-testid="server-view-page">
      <div className="sc-card">
        <div className="sc-card-head">
          <h1 className="sc-card-title">Configure server</h1>
          <p className="sc-card-sub">
            Manage <strong>{server.name}</strong> — edit basic details or its auth mechanisms independently.
          </p>
        </div>

        <div className="sc-card-body">
          {/* ── Basic information ── */}
          <div className="sc-section" data-testid="server-info-section">
            <div className="sc-sec-head">
              <span className="sc-sec-lbl">Basic information</span>
              {!editingBasic && (
                <button
                  className="sc-edit-btn"
                  onClick={() => {
                    setDraftName(server.name);
                    setDraftDesc(server.description ?? '');
                    setDraftEnabled(server.enabled);
                    setEditingBasic(true);
                    setSavedBasic(false);
                  }}
                  data-testid="server-edit-button"
                >
                  <ShellIcon name="pencil" size={13} /> Edit
                </button>
              )}
              {savedBasic && (
                <span className="sc-saved" data-testid="server-saved">
                  <ShellIcon name="check" size={13} /> Saved
                </span>
              )}
            </div>

            {!editingBasic ? (
              <div className="sc-read">
                <div className="sc-read-row">
                  <span className="sc-read-k">Name</span>
                  <span className="sc-read-v" data-testid="server-name-value">
                    {server.name}
                  </span>
                </div>
                <div className="sc-read-row">
                  <span className="sc-read-k">URL</span>
                  <span className="sc-read-v mono">{server.url}</span>
                </div>
                <div className="sc-read-row">
                  <span className="sc-read-k">Description</span>
                  <span className="sc-read-v muted">{server.description || '—'}</span>
                </div>
                <div className="sc-read-row">
                  <span className="sc-read-k">Status</span>
                  <span className={`sc-read-v sc-status ${server.enabled ? 'on' : 'off'}`} data-testid="server-status">
                    <ShellIcon name={server.enabled ? 'circle-check' : 'circle-slash'} size={13} />
                    {server.enabled ? 'Enabled' : 'Disabled'}
                  </span>
                </div>
              </div>
            ) : (
              <div className="space-y-3" data-testid="server-edit-form">
                <div className="space-y-1.5">
                  <Label className="flex items-center gap-2">
                    URL
                    <span className="text-xs text-muted-foreground inline-flex items-center gap-1">
                      <ShellIcon name="lock" size={11} /> locked
                    </span>
                  </Label>
                  <Input value={server.url} disabled data-testid="mcp-server-url-input" />
                  <p className="text-xs text-muted-foreground">
                    The base URL is the server identity and can&apos;t be changed after creation.
                  </p>
                </div>
                <div className="space-y-1.5">
                  <Label>Name *</Label>
                  <Input
                    value={draftName}
                    onChange={(e) => setDraftName(e.target.value)}
                    data-testid="mcp-server-name-input"
                  />
                </div>
                <div className="space-y-1.5">
                  <Label>Description</Label>
                  <Textarea
                    value={draftDesc}
                    onChange={(e) => setDraftDesc(e.target.value)}
                    data-testid="mcp-server-description-input"
                  />
                </div>
                <div className="flex items-center gap-2">
                  <Switch
                    checked={draftEnabled}
                    onCheckedChange={setDraftEnabled}
                    data-testid="mcp-server-enabled-switch"
                  />
                  <Label>Enabled</Label>
                </div>
                <div className="flex gap-2 justify-end">
                  <Button variant="outline" size="sm" onClick={() => setEditingBasic(false)}>
                    Cancel
                  </Button>
                  <Button
                    size="sm"
                    onClick={saveBasic}
                    disabled={!draftName.trim() || updateServer.isPending}
                    data-testid="mcp-server-save-button"
                  >
                    {updateServer.isPending ? 'Saving...' : 'Save changes'}
                  </Button>
                </div>
              </div>
            )}
          </div>

          <div className="sc-divider" />

          {/* ── Auth mechanisms ── */}
          <div className="sc-section" data-testid="auth-configs-section">
            <span className="sc-sec-lbl">Auth mechanisms</span>
            <div className="sc-sec-desc">
              Public is always available. Add OAuth or header/query keys for servers that require it. Mechanisms can be
              deleted but not edited — delete and re-add to change one.
            </div>

            {configsLoading ? (
              <div className="space-y-2">
                <Skeleton className="h-14 w-full" />
                <Skeleton className="h-14 w-full" />
              </div>
            ) : (
              <div className="sc-auth-list">
                {/* Synthetic Public row — always present, built-in, not deletable. */}
                <div className="sc-auth-row" data-testid="auth-config-row-public">
                  <div className="sc-auth-ico auth-badge-public">
                    <ShellIcon name="unlock" size={14} />
                  </div>
                  <div className="sc-auth-body">
                    <div className="sc-auth-name">
                      Public <span className="sc-builtin">Built-in</span>
                    </div>
                    <div className="sc-auth-detail">No authentication required</div>
                  </div>
                </div>

                {authConfigs.map((config) => {
                  const kind = authKind(config.type);
                  return (
                    <div className="sc-auth-row" key={config.id} data-testid={`auth-config-row-${config.id}`}>
                      <div className={`sc-auth-ico auth-badge-${kind}`}>
                        <ShellIcon name={KIND_ICON[kind]} size={14} />
                      </div>
                      <div className="sc-auth-body">
                        <div className="sc-auth-name" data-testid={`auth-config-type-badge-${config.id}`}>
                          {KIND_LABEL[kind]} <span className="sc-auth-cfgname">{config.name}</span>
                        </div>
                        <div className="sc-auth-detail">{authConfigDetail(config)}</div>
                      </div>
                      <button
                        className="sc-del"
                        title="Delete auth mechanism"
                        onClick={() => setDeleteTarget(config)}
                        data-testid={`auth-config-delete-button-${config.id}`}
                      >
                        <ShellIcon name="trash-2" size={14} />
                      </button>
                    </div>
                  );
                })}
              </div>
            )}

            {showForm ? (
              <div className="sc-auth-row" style={{ display: 'block', marginTop: 8 }} data-testid="auth-config-form">
                <AuthConfigForm
                  serverUrl={server.url}
                  type={formType}
                  name={formName}
                  onTypeChange={setFormType}
                  onNameChange={setFormName}
                  entries={formEntries}
                  onEntriesChange={setFormEntries}
                  registrationType={formRegistrationType}
                  clientId={formClientId}
                  clientSecret={formClientSecret}
                  authEndpoint={formAuthEndpoint}
                  tokenEndpoint={formTokenEndpoint}
                  registrationEndpoint={formRegistrationEndpoint}
                  scopes={formScopes}
                  onRegistrationTypeChange={setFormRegistrationType}
                  onClientIdChange={setFormClientId}
                  onClientSecretChange={setFormClientSecret}
                  onAuthEndpointChange={setFormAuthEndpoint}
                  onTokenEndpointChange={setFormTokenEndpoint}
                  onRegistrationEndpointChange={setFormRegistrationEndpoint}
                  onScopesChange={setFormScopes}
                  onSubmit={handleCreateSubmit}
                  onCancel={() => {
                    setShowForm(false);
                    resetForm();
                  }}
                  isSubmitting={createAuthConfig.isPending || standaloneDcr.isPending}
                />
              </div>
            ) : (
              <button className="sc-add-mech" onClick={() => setShowForm(true)} data-testid="add-auth-config-button">
                <ShellIcon name="plus" size={15} /> Add auth mechanism
              </button>
            )}
          </div>
        </div>

        <div className="sc-foot">
          <Button variant="outline" onClick={() => navigate({ to: ROUTE_MCPS })} data-testid="server-back-button">
            <ShellIcon name="arrow-left" size={15} /> Back to My MCPs
          </Button>
        </div>
      </div>

      <AlertDialog open={!!deleteTarget} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <AlertDialogContent data-testid="delete-auth-config-dialog">
          <AlertDialogHeader>
            <AlertDialogTitle>Delete auth mechanism</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete &quot;{deleteTarget?.name}&quot;? All associated OAuth tokens will also be
              deleted, and existing instances using it will break.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => deleteTarget && deleteAuthConfig.mutate({ configId: deleteTarget.id })}
              disabled={deleteAuthConfig.isPending}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {deleteAuthConfig.isPending ? 'Deleting...' : 'Delete'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

export default function ServerViewPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <ServerViewContent />
    </AppInitializer>
  );
}
