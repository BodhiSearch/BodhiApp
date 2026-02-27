'use client';

import { useEffect, useState } from 'react';

import { Pencil, Plus, Trash2 } from 'lucide-react';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';

import AppInitializer from '@/components/AppInitializer';
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
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { toast } from '@/hooks/use-toast';
import {
  useCreateAuthConfig,
  useDeleteAuthConfig,
  useListAuthConfigs,
  useMcpServer,
  useStandaloneDynamicRegister,
  type McpAuthConfigResponse,
} from '@/hooks/useMcps';
import { authConfigTypeBadge, authConfigBadgeVariant, authConfigDetail } from '@/lib/mcpUtils';
import { AuthConfigForm } from '../components/AuthConfigForm';

function ServerViewContent() {
  const searchParams = useSearchParams();
  const serverId = searchParams.get('id') || '';

  const {
    data: server,
    isLoading: serverLoading,
    error: serverError,
  } = useMcpServer(serverId, { enabled: !!serverId });
  const { data: authConfigsData, isLoading: configsLoading } = useListAuthConfigs(serverId);

  const [deleteTarget, setDeleteTarget] = useState<McpAuthConfigResponse | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [formType, setFormType] = useState<'header' | 'oauth'>('header');
  const [formName, setFormName] = useState('');
  const [formHeaderKey, setFormHeaderKey] = useState('');
  const [formHeaderValue, setFormHeaderValue] = useState('');
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
    onError: (message) => {
      toast({ title: 'Dynamic registration failed', description: message, variant: 'destructive' });
    },
  });

  const deleteAuthConfig = useDeleteAuthConfig({
    onSuccess: () => {
      toast({ title: 'Auth config deleted' });
      setDeleteTarget(null);
    },
    onError: (message) => {
      toast({ title: 'Failed to delete auth config', description: message, variant: 'destructive' });
      setDeleteTarget(null);
    },
  });

  const createAuthConfig = useCreateAuthConfig({
    onSuccess: () => {
      toast({ title: 'Auth config created' });
      setShowForm(false);
      resetForm();
    },
    onError: (message) => {
      toast({ title: 'Failed to create auth config', description: message, variant: 'destructive' });
    },
  });

  const authConfigs = authConfigsData?.auth_configs ?? [];

  const resetForm = () => {
    setFormName('');
    setFormHeaderKey('');
    setFormHeaderValue('');
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
        header_key: formHeaderKey,
        header_value: formHeaderValue,
      });
    } else if (formType === 'oauth' && formRegistrationType === 'dynamic_registration') {
      // Dynamic registration: call standalone DCR first
      if (!formRegistrationEndpoint) {
        toast({ title: 'Registration endpoint is required for dynamic registration', variant: 'destructive' });
        return;
      }
      const redirectUri = `${window.location.origin}/ui/mcps/oauth/callback`;
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
      // Pre-registered OAuth: user provides client_id directly
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

  const handleDeleteConfirm = () => {
    if (!deleteTarget) return;
    deleteAuthConfig.mutate({ configId: deleteTarget.id });
  };

  if (!serverId) {
    return <ErrorPage message="No server ID provided" />;
  }

  if (serverError) {
    const errorMessage = serverError.response?.data?.error?.message || 'Failed to load MCP server';
    return <ErrorPage message={errorMessage} />;
  }

  if (serverLoading) {
    return (
      <div className="container mx-auto p-4 max-w-3xl" data-testid="server-view-loading">
        <Card>
          <CardHeader>
            <Skeleton className="h-6 w-48" />
          </CardHeader>
          <CardContent className="space-y-4">
            <Skeleton className="h-4 w-full" />
            <Skeleton className="h-4 w-3/4" />
            <Skeleton className="h-4 w-1/2" />
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="container mx-auto p-4 max-w-3xl" data-testid="server-view-page">
      <Card data-testid="server-info-section">
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle>{server?.name}</CardTitle>
          <Button variant="outline" size="sm" asChild>
            <Link href={`/ui/mcp-servers/edit?id=${serverId}`}>
              <Pencil className="h-4 w-4 mr-2" />
              Edit
            </Link>
          </Button>
        </CardHeader>
        <CardContent className="space-y-3">
          <div>
            <span className="text-sm text-muted-foreground">URL</span>
            <p className="font-mono text-sm">{server?.url}</p>
          </div>
          {server?.description && (
            <div>
              <span className="text-sm text-muted-foreground">Description</span>
              <p className="text-sm">{server.description}</p>
            </div>
          )}
          <div>
            <span className="text-sm text-muted-foreground">Status</span>
            <div className="mt-1">
              <Badge variant={server?.enabled ? 'default' : 'secondary'}>
                {server?.enabled ? 'Enabled' : 'Disabled'}
              </Badge>
            </div>
          </div>
        </CardContent>
      </Card>

      <div className="mt-6" data-testid="auth-configs-section">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold">Auth Configurations</h2>
          {!showForm && (
            <Button size="sm" onClick={() => setShowForm(true)} data-testid="add-auth-config-button">
              <Plus className="h-4 w-4 mr-2" />
              Add Auth Config
            </Button>
          )}
        </div>

        {showForm && (
          <Card className="mb-4" data-testid="auth-config-form">
            <CardContent className="pt-6">
              <AuthConfigForm
                serverUrl={server?.url || ''}
                type={formType}
                name={formName}
                onTypeChange={setFormType}
                onNameChange={setFormName}
                headerKey={formHeaderKey}
                headerValue={formHeaderValue}
                onHeaderKeyChange={setFormHeaderKey}
                onHeaderValueChange={setFormHeaderValue}
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
                enableAutoDcr={false}
                onSubmit={handleCreateSubmit}
                onCancel={() => {
                  setShowForm(false);
                  resetForm();
                }}
                isSubmitting={createAuthConfig.isLoading || standaloneDcr.isLoading}
              />
            </CardContent>
          </Card>
        )}

        {configsLoading ? (
          <div className="space-y-2">
            <Skeleton className="h-12 w-full" />
            <Skeleton className="h-12 w-full" />
          </div>
        ) : authConfigs.length === 0 ? (
          <Card>
            <CardContent className="py-8 text-center text-muted-foreground">No auth configurations yet.</CardContent>
          </Card>
        ) : (
          <div className="space-y-2">
            {authConfigs.map((config) => {
              const id = config.id;
              return (
                <Card key={id} data-testid={`auth-config-row-${id}`}>
                  <CardContent className="py-3 flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <span className="font-medium">{config.name}</span>
                      <Badge variant={authConfigBadgeVariant(config)} data-testid={`auth-config-type-badge-${id}`}>
                        {authConfigTypeBadge(config)}
                      </Badge>
                      <span className="text-sm text-muted-foreground">{authConfigDetail(config)}</span>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-8 w-8 p-0 text-destructive hover:text-destructive"
                      onClick={() => setDeleteTarget(config)}
                      data-testid={`auth-config-delete-button-${id}`}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </CardContent>
                </Card>
              );
            })}
          </div>
        )}
      </div>

      <AlertDialog
        open={!!deleteTarget}
        onOpenChange={(open) => {
          if (!open) setDeleteTarget(null);
        }}
      >
        <AlertDialogContent data-testid="delete-auth-config-dialog">
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Auth Config</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete &quot;{deleteTarget?.name}&quot;? All associated OAuth tokens will also be
              deleted. MCPs using this config will no longer have authentication.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDeleteConfirm}
              disabled={deleteAuthConfig.isLoading}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {deleteAuthConfig.isLoading ? 'Deleting...' : 'Delete'}
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
