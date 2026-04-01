import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { safeNavigate } from '@/lib/safeNavigate';
import { zodResolver } from '@hookform/resolvers/zod';
import { CheckCircle2, ExternalLink, Eye, EyeOff, KeyRound, Loader2, Unplug } from 'lucide-react';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useForm } from 'react-hook-form';
import * as z from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { toast } from '@/hooks/use-toast';
import {
  useCreateMcp,
  useDeleteOAuthToken,
  useListAuthConfigs,
  useGetMcp,
  useListMcpServers,
  useOAuthLogin,
  useUpdateMcp,
  type McpAuthConfigResponse,
  type McpServerResponse,
} from '@/hooks/mcps';
import { useGetUser } from '@/hooks/users';
import { isAdminRole } from '@/lib/roles';
import { authConfigTypeLabel } from '@/lib/mcpUtils';
import McpServerSelector from '@/app/mcps/new/McpServerSelector';
import { useMcpFormStore } from '@/stores/mcpFormStore';
import type { McpAuthParamInput, McpAuthType } from '@bodhiapp/ts-client';

const safeOrigin = (urlStr: string): string => {
  try {
    return new URL(urlStr).origin;
  } catch {
    return urlStr;
  }
};

const createMcpSchema = z.object({
  mcp_server_id: z.string().min(1, 'Please select an MCP server'),
  name: z.string().min(1, 'Name is required').max(100, 'Name must be 100 characters or less'),
  slug: z
    .string()
    .min(1, 'Slug is required')
    .max(24, 'Slug must be 24 characters or less')
    .regex(/^[a-zA-Z0-9-]+$/, 'Slug can only contain letters, numbers, and hyphens'),
  description: z.string().max(255).optional(),
  enabled: z.boolean().default(true),
  auth_type: z.enum(['public', 'header', 'oauth']).default('public'),
});

type CreateMcpFormData = z.infer<typeof createMcpSchema>;

const extractSlugFromUrl = (url: string): string => {
  try {
    const hostname = new URL(url).hostname;
    const parts = hostname.split('.');
    if (parts.length >= 2) {
      return parts[parts.length - 2];
    }
    return parts[0];
  } catch {
    return '';
  }
};

type AuthConfigOption = {
  id: string;
  name: string;
  type: 'header' | 'oauth';
  config: McpAuthConfigResponse;
};

function OAuthConnectedCard({
  config,
  onDisconnect,
  isDisconnecting,
}: {
  config: McpAuthConfigResponse | null;
  onDisconnect: () => void;
  isDisconnecting: boolean;
}) {
  return (
    <div
      className="rounded-lg border border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-950/30 p-4 space-y-3"
      data-testid="oauth-connected-card"
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <CheckCircle2 className="h-4 w-4 text-green-600 dark:text-green-400" />
          <Badge
            variant="outline"
            className="bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-300 border-green-300 dark:border-green-700"
            data-testid="oauth-connected-badge"
          >
            Connected
          </Badge>
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={onDisconnect}
          disabled={isDisconnecting}
          data-testid="oauth-disconnect-button"
        >
          {isDisconnecting ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Unplug className="mr-2 h-4 w-4" />}
          Disconnect
        </Button>
      </div>
      {config && config.type !== 'header' && (
        <div className="text-sm text-muted-foreground space-y-1" data-testid="oauth-connected-info">
          <p>
            <span className="font-medium">Client ID:</span> {config.client_id}
          </p>
          <p>
            <span className="font-medium">Auth Server:</span> {safeOrigin(config.authorization_endpoint)}
          </p>
          {config.scopes && (
            <p>
              <span className="font-medium">Scopes:</span> {config.scopes}
            </p>
          )}
        </div>
      )}
    </div>
  );
}

function NewMcpPageContent() {
  const navigate = useNavigate();
  const search = useSearch({ strict: false });
  const editId = search.id || null;
  const { data: userInfo } = useGetUser();
  const isAdmin = userInfo?.auth_status === 'logged_in' && userInfo.role ? isAdminRole(userInfo.role) : false;

  const store = useMcpFormStore();

  const {
    data: existingMcp,
    isLoading: loadingExisting,
    error: existingError,
  } = useGetMcp(editId || '', { enabled: !!editId });

  const { data: serversData, isLoading: loadingServers } = useListMcpServers({ enabled: true }, { enabled: !editId });

  const enabledServers = useMemo(() => serversData?.mcp_servers || [], [serversData]);

  const [comboboxOpen, setComboboxOpen] = useState(false);
  const [selectedServer, setSelectedServer] = useState<McpServerResponse | null>(null);
  const [showNewAuthRedirect, setShowNewAuthRedirect] = useState(false);
  const [pendingDeleteTokenId, setPendingDeleteTokenId] = useState<string | null>(null);
  const [visibleCredentials, setVisibleCredentials] = useState<Set<string>>(new Set());

  const { data: authConfigsData } = useListAuthConfigs(selectedServer?.id || '', {
    enabled: !!selectedServer?.id,
  });

  const authConfigOptions = useMemo<AuthConfigOption[]>(() => {
    const configs = authConfigsData?.auth_configs ?? [];
    return configs.map((c) => ({
      id: c.id,
      name: c.name,
      type: c.type,
      config: c,
    }));
  }, [authConfigsData]);

  const selectedAuthOption = useMemo(
    () => authConfigOptions.find((o) => o.id === store.selectedAuthConfigId) || null,
    [authConfigOptions, store.selectedAuthConfigId]
  );

  const oauthLoginMutation = useOAuthLogin();
  const deleteOAuthTokenMutation = useDeleteOAuthToken({
    onSuccess: () => {
      store.disconnect();
      toast({ title: 'OAuth connection removed' });
    },
    onError: (message) => {
      store.disconnect();
      toast({ title: 'Failed to disconnect', description: message, variant: 'destructive' });
    },
  });

  const createMutation = useCreateMcp({
    onSuccess: () => {
      toast({ title: 'MCP created successfully' });
      store.reset();
      navigate({ to: '/mcps' });
    },
    onError: (message) => {
      toast({ title: 'Failed to create MCP', description: message, variant: 'destructive' });
    },
  });

  const updateMutation = useUpdateMcp({
    onSuccess: () => {
      toast({ title: 'MCP updated successfully' });
      store.reset();
      navigate({ to: '/mcps' });
    },
    onError: (message) => {
      toast({ title: 'Failed to update MCP', description: message, variant: 'destructive' });
    },
  });

  const form = useForm<CreateMcpFormData>({
    resolver: zodResolver(createMcpSchema),
    defaultValues: {
      mcp_server_id: '',
      name: '',
      slug: '',
      description: '',
      enabled: true,
      auth_type: 'public',
    },
  });

  const sessionRestoredRef = useRef(false);

  useEffect(() => {
    if (!sessionRestoredRef.current) {
      const sessionState = store.restoreFromSession();
      if (sessionState) {
        sessionRestoredRef.current = true;
        form.reset({
          mcp_server_id: (sessionState.mcp_server_id as string) || '',
          name: (sessionState.name as string) || '',
          slug: (sessionState.slug as string) || '',
          description: (sessionState.description as string) || '',
          enabled: (sessionState.enabled as boolean) ?? true,
          auth_type: (sessionState.auth_type as McpAuthType) || 'public',
        });
        if (sessionState.oauth_token_id) {
          store.completeOAuthFlow(sessionState.oauth_token_id as string);
        }
        if (sessionState.selected_auth_config_id) {
          store.setSelectedAuthConfig(
            sessionState.selected_auth_config_id as string,
            (sessionState.selected_auth_config_type as string) || null
          );
        }
        if (sessionState.mcp_server_id && sessionState.server_url && sessionState.server_name) {
          setSelectedServer({
            id: sessionState.mcp_server_id as string,
            url: sessionState.server_url as string,
            name: sessionState.server_name as string,
            enabled: true,
          } as McpServerResponse);
        }
        return;
      }
    }

    if (sessionRestoredRef.current) return;

    if (editId && existingMcp) {
      form.reset({
        mcp_server_id: existingMcp.mcp_server.id,
        name: existingMcp.name,
        slug: existingMcp.slug,
        description: existingMcp.description || '',
        enabled: existingMcp.enabled,
        auth_type: existingMcp.auth_type || 'public',
      });
      setSelectedServer({
        id: existingMcp.mcp_server.id,
        url: existingMcp.mcp_server.url,
        name: existingMcp.mcp_server.name,
        enabled: existingMcp.mcp_server.enabled,
      } as McpServerResponse);
      if (existingMcp.auth_type === 'header' && existingMcp.auth_config_id) {
        store.setSelectedAuthConfig(existingMcp.auth_config_id, 'header');
      }
      if (existingMcp.auth_type === 'oauth' && existingMcp.auth_config_id) {
        store.completeOAuthFlow(existingMcp.auth_config_id);
      }
    }
  }, [existingMcp, editId, form]); // eslint-disable-line react-hooks/exhaustive-deps

  // Set the selected auth config for OAuth MCPs from Mcp.auth_config_id
  useEffect(() => {
    if (editId && existingMcp && existingMcp.auth_type === 'oauth' && existingMcp.auth_config_id) {
      store.setSelectedAuthConfig(existingMcp.auth_config_id, existingMcp.auth_type);
    }
  }, [editId, existingMcp]); // eslint-disable-line react-hooks/exhaustive-deps

  // Auto-select first auth config when configs load for a newly selected server (create mode only)
  useEffect(() => {
    if (
      !editId &&
      !sessionRestoredRef.current &&
      selectedServer &&
      authConfigOptions.length > 0 &&
      !store.selectedAuthConfigId
    ) {
      const first = authConfigOptions[0];
      store.setSelectedAuthConfig(first.id, first.type);
      form.setValue('auth_type', first.type as McpAuthType);
    }
  }, [authConfigOptions, selectedServer, editId]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleServerSelect = useCallback(
    (server: McpServerResponse) => {
      setSelectedServer(server);
      form.setValue('mcp_server_id', server.id);
      form.clearErrors('mcp_server_id');

      if (!form.getValues('name')) {
        form.setValue('name', server.name);
      }
      if (!form.getValues('description') && server.description) {
        form.setValue('description', server.description);
      }
      const slug = extractSlugFromUrl(server.url);
      if (!form.getValues('slug') && slug) {
        form.setValue('slug', slug);
      }

      store.disconnect();
      store.setSelectedAuthConfig(null, null);
      form.setValue('auth_type', 'public');
      setShowNewAuthRedirect(false);
      setComboboxOpen(false);
    },
    [form, store]
  );

  const handleAuthConfigChange = (val: string) => {
    setShowNewAuthRedirect(false);
    if (val === '__public__') {
      store.setSelectedAuthConfig(null, null);
      form.setValue('auth_type', 'public');
      store.disconnect();
      store.clearCredentialValues();
    } else if (val === '__new__') {
      setShowNewAuthRedirect(true);
    } else {
      const opt = authConfigOptions.find((o) => o.id === val);
      if (opt) {
        store.setSelectedAuthConfig(opt.id, opt.type);
        form.setValue('auth_type', opt.type as McpAuthType);
        store.disconnect();
        store.clearCredentialValues();
      }
    }
  };

  const buildCredentials = (): McpAuthParamInput[] | undefined => {
    const config = selectedAuthOption?.config;
    if (!config || config.type !== 'header' || !config.entries?.length) return undefined;
    const creds: McpAuthParamInput[] = [];
    for (const entry of config.entries) {
      const val = store.credentialValues[entry.param_key];
      if (val) {
        creds.push({ param_type: entry.param_type, param_key: entry.param_key, value: val });
      }
    }
    return creds.length > 0 ? creds : undefined;
  };

  const handleOAuthConnect = async () => {
    const serverId = selectedServer?.id;
    const configId = store.selectedAuthConfigId;
    if (!serverId || !configId) {
      toast({ title: 'Please select an OAuth configuration', variant: 'destructive' });
      return;
    }

    try {
      await deletePendingToken();
      store.saveToSession(
        { ...form.getValues(), mcp_id: editId || '' },
        selectedServer ? { url: selectedServer.url, name: selectedServer.name } : undefined
      );
      const redirectUri = `${window.location.origin}/ui/mcps/oauth/callback`;
      const loginResponse = await oauthLoginMutation.mutateAsync({
        id: configId,
        redirect_uri: redirectUri,
      });
      if (!safeNavigate(loginResponse.data.authorization_url)) {
        toast({
          title: 'Invalid authorization URL',
          description: `URL "${loginResponse.data.authorization_url}" must use http:// or https:// scheme`,
          variant: 'destructive',
        });
      }
    } catch {
      // Errors surfaced via React Query mutation state
    }
  };

  const handleDisconnect = () => {
    if (store.oauthTokenId) {
      setPendingDeleteTokenId(store.oauthTokenId);
    }
    store.disconnect();
  };

  const deletePendingToken = async () => {
    if (pendingDeleteTokenId) {
      try {
        await deleteOAuthTokenMutation.mutateAsync({ tokenId: pendingDeleteTokenId });
      } catch {
        // Best effort
      }
      setPendingDeleteTokenId(null);
    }
  };

  const onSubmit = async (data: CreateMcpFormData) => {
    const authType = data.auth_type;

    if (authType === 'oauth' && !store.isConnected && !existingMcp?.auth_config_id) {
      toast({ title: 'Please complete OAuth authorization first', variant: 'destructive' });
      return;
    }

    const basePayload = {
      name: data.name,
      slug: data.slug,
      mcp_server_id: data.mcp_server_id,
      description: data.description || undefined,
      enabled: data.enabled,
    };

    let authPayload: { auth_type?: McpAuthType; auth_config_id?: string; oauth_token_id?: string } = {};

    if (authType === 'header' && store.selectedAuthConfigId) {
      authPayload = { auth_type: 'header', auth_config_id: store.selectedAuthConfigId };
      const credentials = buildCredentials();
      if (credentials) {
        (authPayload as Record<string, unknown>).credentials = credentials;
      }
    } else if (authType === 'oauth') {
      if (editId && !store.isConnected && pendingDeleteTokenId) {
        await deletePendingToken();
        authPayload = { auth_type: 'oauth' };
      } else {
        const configId = store.selectedAuthConfigId || existingMcp?.auth_config_id;
        authPayload = {
          auth_type: 'oauth',
          auth_config_id: configId || undefined,
          oauth_token_id: store.oauthTokenId || undefined,
        };
      }
    } else {
      authPayload = { auth_type: 'public' };
    }

    const payload = { ...basePayload, ...authPayload };

    if (editId) {
      updateMutation.mutate({ id: editId, ...payload });
    } else {
      createMutation.mutate(payload);
    }
  };

  const isSubmitting = createMutation.isPending || updateMutation.isPending || oauthLoginMutation.isPending;

  const dropdownValue = showNewAuthRedirect ? '__new__' : store.selectedAuthConfigId || '__public__';

  if (editId && existingError) {
    const errorMessage = existingError.response?.data?.error?.message || existingError.message || 'Failed to load MCP';
    return <ErrorPage message={errorMessage} />;
  }

  if (editId && loadingExisting) {
    return (
      <div className="container mx-auto p-4 max-w-2xl" data-testid="new-mcp-loading">
        <Skeleton className="h-10 w-full mb-4" />
        <Skeleton className="h-64 w-full" />
      </div>
    );
  }

  return (
    <div className="container mx-auto p-4 max-w-2xl" data-testid="new-mcp-page">
      <Card>
        <CardHeader>
          <CardTitle>{editId ? 'Edit MCP' : 'New MCP'}</CardTitle>
          <CardDescription>
            {editId
              ? 'Update your MCP instance configuration.'
              : 'Create a new MCP instance by selecting a registered server.'}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
              <McpServerSelector
                control={form.control}
                name="mcp_server_id"
                editId={editId}
                selectedServer={selectedServer}
                comboboxOpen={comboboxOpen}
                onComboboxOpenChange={setComboboxOpen}
                onServerSelect={handleServerSelect}
                enabledServers={enabledServers}
                loadingServers={loadingServers}
                isAdmin={isAdmin}
                isSubmitting={isSubmitting}
              />

              <FormField
                control={form.control}
                name="name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Name</FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        placeholder="My MCP Instance"
                        disabled={isSubmitting}
                        data-testid="mcp-name-input"
                      />
                    </FormControl>
                    <FormDescription>A friendly name for this MCP instance</FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="slug"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Slug</FormLabel>
                    <FormControl>
                      <Input {...field} placeholder="my-mcp" disabled={isSubmitting} data-testid="mcp-slug-input" />
                    </FormControl>
                    <FormDescription>A unique identifier (letters, numbers, and hyphens only)</FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="description"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Description (Optional)</FormLabel>
                    <FormControl>
                      <Textarea
                        {...field}
                        placeholder="Describe what this MCP instance is for"
                        disabled={isSubmitting}
                        data-testid="mcp-description-input"
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="enabled"
                render={({ field }) => (
                  <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                    <div className="space-y-0.5">
                      <FormLabel className="text-base">Enable MCP</FormLabel>
                      <FormDescription>Make this MCP instance available for use</FormDescription>
                    </div>
                    <FormControl>
                      <Switch
                        checked={field.value}
                        onCheckedChange={field.onChange}
                        disabled={isSubmitting}
                        data-testid="mcp-enabled-switch"
                      />
                    </FormControl>
                  </FormItem>
                )}
              />

              <div className="space-y-4 rounded-lg border p-4" data-testid="mcp-auth-section">
                <div className="flex items-center gap-2 mb-2">
                  <KeyRound className="h-4 w-4" />
                  <h3 className="text-base font-semibold">Authentication</h3>
                </div>

                <div className="space-y-2">
                  <FormLabel>Auth Configuration</FormLabel>
                  <Select
                    value={dropdownValue}
                    onValueChange={handleAuthConfigChange}
                    disabled={isSubmitting || !selectedServer}
                  >
                    <SelectTrigger data-testid="auth-config-select" data-test-state={form.watch('auth_type')}>
                      <SelectValue placeholder="Select authentication configuration" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="__public__" data-testid="auth-config-option-public">
                        Public (No Auth)
                      </SelectItem>
                      {authConfigOptions.map((opt) => (
                        <SelectItem key={opt.id} value={opt.id} data-testid={`auth-config-option-${opt.id}`}>
                          {opt.name} [{authConfigTypeLabel(opt.type)}]
                        </SelectItem>
                      ))}
                      {isAdmin && (
                        <SelectItem value="__new__" data-testid="auth-config-option-new">
                          + New Auth Config
                        </SelectItem>
                      )}
                    </SelectContent>
                  </Select>
                  <FormDescription>
                    {store.selectedAuthConfigType === 'header'
                      ? 'A pre-configured header will be sent with every request to this MCP server.'
                      : store.selectedAuthConfigType === 'oauth'
                        ? 'OAuth authentication is required. Click Connect to authorize.'
                        : 'No authentication required for this MCP server.'}
                  </FormDescription>
                </div>

                {store.selectedAuthConfigType === 'header' &&
                  selectedAuthOption &&
                  selectedAuthOption.config.type === 'header' && (
                    <div
                      className="rounded-lg border p-3 text-sm space-y-3 bg-muted/50"
                      data-testid="auth-config-header-credentials"
                    >
                      <p>
                        <span className="font-medium">Config:</span> {selectedAuthOption.name}
                      </p>
                      {selectedAuthOption.config.entries.length > 0 ? (
                        selectedAuthOption.config.entries.map((entry) => {
                          const isVisible = visibleCredentials.has(entry.param_key);
                          return (
                            <div
                              key={entry.param_key}
                              className="space-y-1"
                              data-testid={`credential-field-${entry.param_key}`}
                            >
                              <label className="text-xs font-medium text-muted-foreground">
                                {entry.param_key} <span className="text-xs opacity-60">({entry.param_type})</span>
                              </label>
                              <div className="flex items-center gap-1">
                                <Input
                                  type={isVisible ? 'text' : 'password'}
                                  placeholder={`Enter ${entry.param_key} value`}
                                  value={store.credentialValues[entry.param_key] || ''}
                                  onChange={(e) => store.setCredentialValue(entry.param_key, e.target.value)}
                                  disabled={isSubmitting}
                                  data-testid={`credential-input-${entry.param_key}`}
                                  className="flex-1"
                                />
                                <Button
                                  type="button"
                                  variant="ghost"
                                  size="icon"
                                  className="h-9 w-9 shrink-0"
                                  onClick={() => {
                                    setVisibleCredentials((prev) => {
                                      const next = new Set(prev);
                                      if (next.has(entry.param_key)) next.delete(entry.param_key);
                                      else next.add(entry.param_key);
                                      return next;
                                    });
                                  }}
                                  data-testid={`credential-toggle-${entry.param_key}`}
                                >
                                  {isVisible ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                                </Button>
                              </div>
                            </div>
                          );
                        })
                      ) : (
                        <p className="text-muted-foreground">No credential keys defined for this config.</p>
                      )}
                    </div>
                  )}

                {store.selectedAuthConfigType === 'oauth' && store.isConnected && (
                  <OAuthConnectedCard
                    config={selectedAuthOption?.config ?? null}
                    onDisconnect={handleDisconnect}
                    isDisconnecting={deleteOAuthTokenMutation.isPending}
                  />
                )}

                {store.selectedAuthConfigType === 'oauth' && !store.isConnected && selectedAuthOption && (
                  <div className="space-y-3">
                    <div className="rounded-lg border p-3 text-sm space-y-1 bg-muted/50">
                      <p>
                        <span className="font-medium">Config:</span> {selectedAuthOption.name}
                      </p>
                      <p>
                        <span className="font-medium">Type:</span>{' '}
                        <Badge variant="secondary">{authConfigTypeLabel(selectedAuthOption.type)}</Badge>
                      </p>
                      <p>
                        <span className="font-medium">Auth Server:</span>{' '}
                        {selectedAuthOption.config.type !== 'header' &&
                          safeOrigin(selectedAuthOption.config.authorization_endpoint)}
                      </p>
                    </div>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={handleOAuthConnect}
                      disabled={oauthLoginMutation.isPending}
                      data-testid="auth-config-oauth-connect"
                    >
                      {oauthLoginMutation.isPending ? (
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      ) : (
                        <ExternalLink className="mr-2 h-4 w-4" />
                      )}
                      Connect
                    </Button>
                  </div>
                )}

                {showNewAuthRedirect && selectedServer && (
                  <div
                    className="rounded-lg border border-blue-200 dark:border-blue-800 bg-blue-50 dark:bg-blue-950/30 p-4 space-y-3"
                    data-testid="auth-config-new-redirect"
                  >
                    <p className="text-sm">You&apos;ll be redirected to create an auth config for this server.</p>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={() => navigate({ to: '/mcps/servers/view', search: { id: selectedServer.id } })}
                      data-testid="auth-config-new-redirect-button"
                    >
                      Go to Server Settings
                    </Button>
                  </div>
                )}
              </div>

              <div className="flex gap-4">
                {editId ? (
                  <Button type="submit" disabled={isSubmitting} data-testid="mcp-update-button">
                    {isSubmitting ? 'Updating...' : 'Update MCP'}
                  </Button>
                ) : (
                  <Button type="submit" disabled={isSubmitting} data-testid="mcp-create-button">
                    {isSubmitting ? 'Creating...' : 'Create MCP'}
                  </Button>
                )}
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => navigate({ to: '/mcps' })}
                  disabled={isSubmitting}
                  data-testid="mcp-cancel-button"
                >
                  Cancel
                </Button>
              </div>
            </form>
          </Form>
        </CardContent>
      </Card>
    </div>
  );
}

export default function NewMcpPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <NewMcpPageContent />
    </AppInitializer>
  );
}
