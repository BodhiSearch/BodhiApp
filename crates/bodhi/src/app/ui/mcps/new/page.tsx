'use client';

import { useCallback, useEffect, useMemo, useState } from 'react';

import { zodResolver } from '@hookform/resolvers/zod';
import { CheckCircle2, KeyRound, Loader2, Search, ExternalLink, Unplug } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useForm } from 'react-hook-form';
import * as z from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { PasswordInput } from '@/components/ui/password-input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { toast } from '@/hooks/use-toast';
import {
  useAuthHeader,
  useCreateAuthHeader,
  useCreateMcp,
  useCreateOAuthConfig,
  useDeleteOAuthToken,
  useFetchMcpTools,
  useListOAuthConfigs,
  useMcp,
  useMcpServers,
  useOAuthDiscover,
  useOAuthLogin,
  useUpdateAuthHeader,
  useUpdateMcp,
  type McpServerResponse,
  type OAuthConfigResponse,
} from '@/hooks/useMcps';
import { useUser } from '@/hooks/useUsers';
import { isAdminRole } from '@/lib/roles';
import McpServerSelector from '@/app/ui/mcps/new/McpServerSelector';
import ToolSelection from '@/app/ui/mcps/new/ToolSelection';
import { useMcpFormStore } from '@/stores/mcpFormStore';
import type { McpAuthType } from '@bodhiapp/ts-client';

const OAUTH_FORM_STORAGE_KEY = 'mcp_oauth_form_state';

const safeOrigin = (urlStr: string): string => {
  try {
    return new URL(urlStr).origin;
  } catch {
    return urlStr;
  }
};

const createMcpSchema = z
  .object({
    mcp_server_id: z.string().min(1, 'Please select an MCP server'),
    name: z.string().min(1, 'Name is required').max(100, 'Name must be 100 characters or less'),
    slug: z
      .string()
      .min(1, 'Slug is required')
      .max(24, 'Slug must be 24 characters or less')
      .regex(/^[a-zA-Z0-9-]+$/, 'Slug can only contain letters, numbers, and hyphens'),
    description: z.string().max(255).optional(),
    enabled: z.boolean().default(true),
    auth_type: z.enum(['public', 'header', 'oauth-pre-registered']).default('public'),
    auth_header_key: z.string().optional(),
    auth_header_value: z.string().optional(),
    oauth_server_url: z.string().optional(),
    oauth_client_id: z.string().optional(),
    oauth_client_secret: z.string().optional(),
    oauth_authorization_endpoint: z.string().optional(),
    oauth_token_endpoint: z.string().optional(),
    oauth_scopes: z.string().optional(),
  })
  .refine(
    (data) => {
      if (data.auth_type === 'header') {
        return !!data.auth_header_key && data.auth_header_key.length > 0;
      }
      return true;
    },
    { message: 'Header name is required', path: ['auth_header_key'] }
  )
  .refine(
    (data) => {
      if (data.auth_type === 'oauth-pre-registered') {
        return !!data.oauth_client_id && data.oauth_client_id.length > 0;
      }
      return true;
    },
    { message: 'Client ID is required for OAuth', path: ['oauth_client_id'] }
  )
  .refine(
    (data) => {
      if (data.auth_type === 'oauth-pre-registered') {
        return !!data.oauth_client_secret && data.oauth_client_secret.length > 0;
      }
      return true;
    },
    { message: 'Client Secret is required for OAuth', path: ['oauth_client_secret'] }
  )
  .refine(
    (data) => {
      if (data.auth_type === 'oauth-pre-registered') {
        return !!data.oauth_authorization_endpoint && data.oauth_authorization_endpoint.length > 0;
      }
      return true;
    },
    { message: 'Authorization endpoint is required for OAuth', path: ['oauth_authorization_endpoint'] }
  )
  .refine(
    (data) => {
      if (data.auth_type === 'oauth-pre-registered') {
        return !!data.oauth_token_endpoint && data.oauth_token_endpoint.length > 0;
      }
      return true;
    },
    { message: 'Token endpoint is required for OAuth', path: ['oauth_token_endpoint'] }
  );
const HEADER_SUGGESTIONS = ['Authorization', 'X-API-Key', 'Api-Key'];

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

function OAuthConnectedCard({
  config,
  onDisconnect,
  isDisconnecting,
}: {
  config: OAuthConfigResponse | null;
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
      {config && (
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

function OAuthConfigDropdown({
  configs,
  selectedConfigId,
  onSelect,
  onNewConfig,
  isNewConfig,
  disabled,
}: {
  configs: OAuthConfigResponse[];
  selectedConfigId: string | null;
  onSelect: (id: string) => void;
  onNewConfig: () => void;
  isNewConfig: boolean;
  disabled: boolean;
}) {
  return (
    <div className="space-y-2" data-testid="oauth-config-dropdown">
      <FormLabel>OAuth Configuration</FormLabel>
      <Select
        value={isNewConfig ? '__new__' : selectedConfigId || ''}
        onValueChange={(val) => {
          if (val === '__new__') {
            onNewConfig();
          } else {
            onSelect(val);
          }
        }}
        disabled={disabled}
      >
        <SelectTrigger data-testid="oauth-config-select">
          <SelectValue placeholder="Select an OAuth configuration" />
        </SelectTrigger>
        <SelectContent>
          {configs.map((c) => (
            <SelectItem key={c.id} value={c.id} data-testid={`oauth-config-option-${c.id}`}>
              {c.client_id} - {safeOrigin(c.authorization_endpoint)}
            </SelectItem>
          ))}
          <SelectItem value="__new__" data-testid="oauth-config-option-new">
            + New OAuth Config
          </SelectItem>
        </SelectContent>
      </Select>
      <FormDescription>Choose an existing OAuth configuration or create a new one</FormDescription>
    </div>
  );
}

function OAuthConfigSummary({
  config,
  onAuthorize,
  isAuthorizing,
}: {
  config: OAuthConfigResponse;
  onAuthorize: () => void;
  isAuthorizing: boolean;
}) {
  return (
    <div className="space-y-3" data-testid="oauth-config-summary">
      <div className="rounded-lg border p-3 text-sm space-y-1 bg-muted/50">
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
      <Button
        type="button"
        variant="outline"
        size="sm"
        onClick={onAuthorize}
        disabled={isAuthorizing}
        data-testid="oauth-authorize-existing"
      >
        {isAuthorizing ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <ExternalLink className="mr-2 h-4 w-4" />}
        Authorize
      </Button>
    </div>
  );
}

function NewMcpPageContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const editId = searchParams.get('id');
  const { data: userInfo } = useUser();
  const isAdmin = userInfo?.auth_status === 'logged_in' && userInfo.role ? isAdminRole(userInfo.role) : false;

  const store = useMcpFormStore();

  const {
    data: existingMcp,
    isLoading: loadingExisting,
    error: existingError,
  } = useMcp(editId || '', { enabled: !!editId });

  const { data: existingAuthHeader } = useAuthHeader(existingMcp?.auth_uuid || '', {
    enabled: !!existingMcp?.auth_uuid && existingMcp?.auth_type === 'header',
  });

  const { data: serversData, isLoading: loadingServers } = useMcpServers({ enabled: true }, { enabled: !editId });

  const enabledServers = useMemo(() => serversData?.mcp_servers || [], [serversData]);

  const [comboboxOpen, setComboboxOpen] = useState(false);
  const [selectedServer, setSelectedServer] = useState<McpServerResponse | null>(null);

  const { data: oauthConfigsData } = useListOAuthConfigs(selectedServer?.id || '', {
    enabled: !!selectedServer?.id && !editId,
  });
  const oauthConfigsList = useMemo(() => oauthConfigsData?.oauth_configs || [], [oauthConfigsData]);

  useEffect(() => {
    store.setOAuthConfigs(oauthConfigsList);
  }, [oauthConfigsList]); // eslint-disable-line react-hooks/exhaustive-deps

  const selectedOAuthConfig = useMemo(
    () => oauthConfigsList.find((c) => c.id === store.selectedOAuthConfigId) || null,
    [oauthConfigsList, store.selectedOAuthConfigId]
  );

  const createAuthHeaderMutation = useCreateAuthHeader();
  const updateAuthHeaderMutation = useUpdateAuthHeader();
  const createOAuthConfigMutation = useCreateOAuthConfig();
  const oauthDiscoverMutation = useOAuthDiscover();
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
      router.push('/ui/mcps');
    },
    onError: (message) => {
      toast({ title: 'Failed to create MCP', description: message, variant: 'destructive' });
    },
  });

  const updateMutation = useUpdateMcp({
    onSuccess: () => {
      toast({ title: 'MCP updated successfully' });
      store.reset();
      router.push('/ui/mcps');
    },
    onError: (message) => {
      toast({ title: 'Failed to update MCP', description: message, variant: 'destructive' });
    },
  });

  const fetchToolsMutation = useFetchMcpTools({
    onSuccess: (response) => {
      const tools = response.tools || [];
      store.setFetchedTools(tools);
      store.setSelectedTools(new Set(tools.map((t) => t.name)));
      store.setToolsFetched(true);
      toast({ title: `Fetched ${tools.length} tool${tools.length !== 1 ? 's' : ''}` });
    },
    onError: (message) => {
      toast({ title: 'Failed to fetch tools', description: message, variant: 'destructive' });
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
      auth_header_key: '',
      auth_header_value: '',
      oauth_server_url: '',
      oauth_client_id: '',
      oauth_client_secret: '',
      oauth_authorization_endpoint: '',
      oauth_token_endpoint: '',
      oauth_scopes: '',
    },
  });

  const authType = form.watch('auth_type');
  const authHeaderKey = form.watch('auth_header_key');
  const authHeaderValue = form.watch('auth_header_value');

  useEffect(() => {
    if (existingMcp && editId) {
      form.reset({
        mcp_server_id: existingMcp.mcp_server.id,
        name: existingMcp.name,
        slug: existingMcp.slug,
        description: existingMcp.description || '',
        enabled: existingMcp.enabled,
        auth_type: existingMcp.auth_type || 'public',
        auth_header_key: '',
        auth_header_value: '',
        oauth_server_url: '',
        oauth_client_id: '',
        oauth_client_secret: '',
        oauth_authorization_endpoint: '',
        oauth_token_endpoint: '',
        oauth_scopes: '',
      });
      setSelectedServer({
        id: existingMcp.mcp_server.id,
        url: existingMcp.mcp_server.url,
        name: existingMcp.mcp_server.name,
        enabled: existingMcp.mcp_server.enabled,
      } as McpServerResponse);
      if (existingMcp.tools_cache) {
        store.setFetchedTools(existingMcp.tools_cache);
        store.setSelectedTools(new Set(existingMcp.tools_filter || []));
        store.setToolsFetched(true);
      }
      if (existingMcp.auth_type === 'oauth-pre-registered' && existingMcp.auth_uuid) {
        store.completeOAuthFlow(existingMcp.auth_uuid);
      }
    }
  }, [existingMcp, editId, form]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    if (authType === 'oauth-pre-registered' && selectedServer?.url && !form.getValues('oauth_server_url')) {
      try {
        const origin = new URL(selectedServer.url).origin;
        form.setValue('oauth_server_url', origin);
      } catch {
        // invalid URL, leave empty
      }
    }
  }, [authType, selectedServer, form]);

  useEffect(() => {
    if (existingAuthHeader && editId) {
      form.setValue('auth_header_key', existingAuthHeader.header_key);
    }
  }, [existingAuthHeader, editId, form]);

  useEffect(() => {
    if (editId) return;
    const state = store.restoreFromSession();
    if (!state) return;

    form.reset({
      mcp_server_id: (state.mcp_server_id as string) || '',
      name: (state.name as string) || '',
      slug: (state.slug as string) || '',
      description: (state.description as string) || '',
      enabled: (state.enabled as boolean) ?? true,
      auth_type: (state.auth_type as McpAuthType) || 'public',
      auth_header_key: '',
      auth_header_value: '',
      oauth_server_url: (state.oauth_server_url as string) || '',
      oauth_client_id: '',
      oauth_client_secret: '',
      oauth_authorization_endpoint: '',
      oauth_token_endpoint: '',
      oauth_scopes: '',
    });
    if (state.tools_cache) {
      store.setFetchedTools(state.tools_cache as typeof store.fetchedTools);
      store.setSelectedTools(new Set((state.tools_filter as string[]) || []));
      store.setToolsFetched(true);
    }
    if (state.oauth_token_id) {
      store.completeOAuthFlow(state.oauth_token_id as string);
    }
    if (state.oauth_config_id) {
      store.selectOAuthConfig(state.oauth_config_id as string);
    }
    if (state.mcp_server_id && state.server_url && state.server_name) {
      setSelectedServer({
        id: state.mcp_server_id as string,
        url: state.server_url as string,
        name: state.server_name as string,
        enabled: true,
      } as McpServerResponse);
    }
  }, [editId, form]); // eslint-disable-line react-hooks/exhaustive-deps

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

      store.setFetchedTools([]);
      store.setSelectedTools(new Set());
      store.setToolsFetched(false);
      store.disconnect();
      store.selectOAuthConfig(null);
      store.setNewOAuthConfig(false);
      setComboboxOpen(false);
    },
    [form, store]
  );

  const handleFetchTools = () => {
    const serverId = form.getValues('mcp_server_id');
    if (serverId) {
      const currentAuthType = form.getValues('auth_type');
      if (currentAuthType === 'header') {
        fetchToolsMutation.mutate({
          mcp_server_id: serverId,
          auth: {
            type: 'header',
            header_key: form.getValues('auth_header_key') || '',
            header_value: form.getValues('auth_header_value') || '',
          },
        });
      } else if (currentAuthType === 'oauth-pre-registered') {
        const authUuid = store.oauthTokenId || existingMcp?.auth_uuid;
        fetchToolsMutation.mutate({
          mcp_server_id: serverId,
          ...(authUuid ? { auth_uuid: authUuid } : {}),
        });
      } else {
        fetchToolsMutation.mutate({ mcp_server_id: serverId });
      }
    }
  };

  const handleOAuthAutoDetect = () => {
    const oauthServerUrl = form.getValues('oauth_server_url');
    if (!oauthServerUrl) {
      toast({ title: 'Please enter an authorization server URL', variant: 'destructive' });
      return;
    }
    oauthDiscoverMutation.mutate(
      { url: oauthServerUrl },
      {
        onSuccess: (response) => {
          const data = response.data;
          form.setValue('oauth_authorization_endpoint', data.authorization_endpoint);
          form.setValue('oauth_token_endpoint', data.token_endpoint);
          if (data.scopes_supported?.length) {
            form.setValue('oauth_scopes', data.scopes_supported.join(' '));
          }
          toast({ title: 'OAuth endpoints auto-detected' });
        },
        onError: (error) => {
          const message = error?.response?.data?.error?.message || 'Failed to auto-detect OAuth endpoints';
          toast({ title: 'Auto-detect failed', description: message, variant: 'destructive' });
        },
      }
    );
  };

  const handleOAuthAuthorize = async (configId?: string) => {
    const serverId = form.getValues('mcp_server_id');
    if (!serverId) {
      toast({ title: 'Please select an MCP server first', variant: 'destructive' });
      return;
    }

    let oauthConfigId = configId;

    try {
      if (!oauthConfigId && (store.isNewOAuthConfig || oauthConfigsList.length === 0)) {
        const clientId = form.getValues('oauth_client_id');
        const clientSecret = form.getValues('oauth_client_secret');
        const authEndpoint = form.getValues('oauth_authorization_endpoint');
        const tokenEndpoint = form.getValues('oauth_token_endpoint');
        const scopes = form.getValues('oauth_scopes');

        if (!clientId || !clientSecret || !authEndpoint || !tokenEndpoint) {
          toast({ title: 'Please fill in all required OAuth fields', variant: 'destructive' });
          return;
        }

        const configResponse = await createOAuthConfigMutation.mutateAsync({
          serverId,
          client_id: clientId,
          client_secret: clientSecret,
          authorization_endpoint: authEndpoint,
          token_endpoint: tokenEndpoint,
          scopes: scopes || undefined,
        });
        oauthConfigId = configResponse.data.id;
        store.selectOAuthConfig(oauthConfigId);
      }

      if (!oauthConfigId && store.selectedOAuthConfigId) {
        oauthConfigId = store.selectedOAuthConfigId;
      }

      if (!oauthConfigId) {
        toast({ title: 'No OAuth configuration selected', variant: 'destructive' });
        return;
      }

      store.saveToSession(
        {
          name: form.getValues('name'),
          slug: form.getValues('slug'),
          description: form.getValues('description'),
          enabled: form.getValues('enabled'),
          mcp_server_id: serverId,
          auth_type: 'oauth-pre-registered',
          oauth_config_id: oauthConfigId,
          oauth_server_url: form.getValues('oauth_server_url'),
        },
        selectedServer ? { url: selectedServer.url, name: selectedServer.name } : undefined
      );

      const redirectUri = `${window.location.origin}/ui/mcps/oauth/callback`;
      const loginResponse = await oauthLoginMutation.mutateAsync({
        id: oauthConfigId,
        serverId,
        redirect_uri: redirectUri,
      });
      window.location.href = loginResponse.data.authorization_url;
    } catch {
      // Errors from mutateAsync are surfaced via React Query mutation state
    }
  };

  const handleDisconnect = () => {
    if (store.oauthTokenId) {
      deleteOAuthTokenMutation.mutate({ tokenId: store.oauthTokenId });
    }
  };

  const onSubmit = async (data: CreateMcpFormData) => {
    if (!editId && data.auth_type === 'header' && !data.auth_header_value) {
      form.setError('auth_header_value', { message: 'Header value is required' });
      return;
    }

    if (data.auth_type === 'oauth-pre-registered' && !store.isConnected && !existingMcp?.auth_uuid) {
      toast({ title: 'Please complete OAuth authorization first', variant: 'destructive' });
      return;
    }

    if (editId) {
      await handleEditSubmit(data);
    } else {
      await handleCreateSubmit(data);
    }
  };

  const handleCreateSubmit = async (data: CreateMcpFormData) => {
    if (data.auth_type === 'header') {
      try {
        const authResponse = await createAuthHeaderMutation.mutateAsync({
          header_key: data.auth_header_key || '',
          header_value: data.auth_header_value || '',
        });
        createMutation.mutate({
          mcp_server_id: data.mcp_server_id,
          name: data.name,
          slug: data.slug,
          description: data.description || undefined,
          enabled: data.enabled,
          tools_cache: store.fetchedTools.length > 0 ? store.fetchedTools : undefined,
          tools_filter: Array.from(store.selectedTools),
          auth_type: 'header',
          auth_uuid: authResponse.data.id,
        });
      } catch {
        toast({ title: 'Failed to create auth header config', variant: 'destructive' });
      }
    } else if (data.auth_type === 'oauth-pre-registered' && store.oauthTokenId) {
      createMutation.mutate({
        mcp_server_id: data.mcp_server_id,
        name: data.name,
        slug: data.slug,
        description: data.description || undefined,
        enabled: data.enabled,
        tools_cache: store.fetchedTools.length > 0 ? store.fetchedTools : undefined,
        tools_filter: Array.from(store.selectedTools),
        auth_type: 'oauth-pre-registered',
        auth_uuid: store.oauthTokenId,
      });
    } else {
      createMutation.mutate({
        mcp_server_id: data.mcp_server_id,
        name: data.name,
        slug: data.slug,
        description: data.description || undefined,
        enabled: data.enabled,
        tools_cache: store.fetchedTools.length > 0 ? store.fetchedTools : undefined,
        tools_filter: Array.from(store.selectedTools),
      });
    }
  };

  const handleEditSubmit = async (data: CreateMcpFormData) => {
    const existingAuthType = existingMcp?.auth_type || 'public';
    const existingAuthUuid = existingMcp?.auth_uuid;

    if (data.auth_type === 'header' && data.auth_header_value) {
      try {
        if (existingAuthUuid && existingAuthType === 'header') {
          await updateAuthHeaderMutation.mutateAsync({
            id: existingAuthUuid,
            header_key: data.auth_header_key || '',
            header_value: data.auth_header_value,
          });
          updateMutation.mutate({
            id: editId!,
            name: data.name,
            slug: data.slug,
            description: data.description || undefined,
            enabled: data.enabled,
            tools_filter: Array.from(store.selectedTools),
            tools_cache: store.fetchedTools.length > 0 ? store.fetchedTools : undefined,
            auth_type: 'header',
            auth_uuid: existingAuthUuid,
          });
        } else {
          const authResponse = await createAuthHeaderMutation.mutateAsync({
            header_key: data.auth_header_key || '',
            header_value: data.auth_header_value,
          });
          updateMutation.mutate({
            id: editId!,
            name: data.name,
            slug: data.slug,
            description: data.description || undefined,
            enabled: data.enabled,
            tools_filter: Array.from(store.selectedTools),
            tools_cache: store.fetchedTools.length > 0 ? store.fetchedTools : undefined,
            auth_type: 'header',
            auth_uuid: authResponse.data.id,
          });
        }
      } catch {
        toast({ title: 'Failed to save auth header config', variant: 'destructive' });
      }
    } else if (data.auth_type === 'oauth-pre-registered') {
      const authUuid = store.oauthTokenId || existingAuthUuid;
      updateMutation.mutate({
        id: editId!,
        name: data.name,
        slug: data.slug,
        description: data.description || undefined,
        enabled: data.enabled,
        tools_filter: Array.from(store.selectedTools),
        tools_cache: store.fetchedTools.length > 0 ? store.fetchedTools : undefined,
        auth_type: 'oauth-pre-registered',
        auth_uuid: authUuid,
      });
    } else if (data.auth_type === 'public' && existingAuthType !== 'public') {
      updateMutation.mutate({
        id: editId!,
        name: data.name,
        slug: data.slug,
        description: data.description || undefined,
        enabled: data.enabled,
        tools_filter: Array.from(store.selectedTools),
        tools_cache: store.fetchedTools.length > 0 ? store.fetchedTools : undefined,
        auth_type: 'public',
      });
    } else {
      updateMutation.mutate({
        id: editId!,
        name: data.name,
        slug: data.slug,
        description: data.description || undefined,
        enabled: data.enabled,
        tools_filter: Array.from(store.selectedTools),
        tools_cache: store.fetchedTools.length > 0 ? store.fetchedTools : undefined,
      });
    }
  };

  const isSubmitting =
    createMutation.isLoading ||
    updateMutation.isLoading ||
    createAuthHeaderMutation.isLoading ||
    updateAuthHeaderMutation.isLoading ||
    createOAuthConfigMutation.isLoading ||
    oauthLoginMutation.isLoading;
  const canCreate = store.toolsFetched && !isSubmitting;

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

                <FormField
                  control={form.control}
                  name="auth_type"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Auth Type</FormLabel>
                      <Select
                        key={field.value}
                        onValueChange={field.onChange}
                        value={field.value}
                        disabled={isSubmitting}
                      >
                        <FormControl>
                          <SelectTrigger data-testid="mcp-auth-type-select" data-test-state={field.value}>
                            <SelectValue placeholder="Select auth type" />
                          </SelectTrigger>
                        </FormControl>
                        <SelectContent>
                          <SelectItem value="public" data-testid="mcp-auth-type-public">
                            Public
                          </SelectItem>
                          <SelectItem value="header" data-testid="mcp-auth-type-header">
                            Header
                          </SelectItem>
                          <SelectItem value="oauth-pre-registered" data-testid="mcp-auth-type-oauth">
                            OAuth 2.1 (Pre-registered)
                          </SelectItem>
                        </SelectContent>
                      </Select>
                      <FormDescription>
                        {authType === 'header'
                          ? 'A custom header will be sent with every request to this MCP server.'
                          : authType === 'oauth-pre-registered'
                            ? 'OAuth 2.1 pre-registered client credentials for this MCP server.'
                            : 'No authentication required for this MCP server.'}
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                {authType === 'header' && (
                  <>
                    <FormField
                      control={form.control}
                      name="auth_header_key"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>Header Name</FormLabel>
                          <FormControl>
                            <div className="relative">
                              <Input
                                {...field}
                                placeholder="e.g. Authorization"
                                disabled={isSubmitting}
                                list="header-suggestions"
                                data-testid="mcp-auth-header-key"
                              />
                              <datalist id="header-suggestions">
                                {HEADER_SUGGESTIONS.map((s) => (
                                  <option key={s} value={s} />
                                ))}
                              </datalist>
                            </div>
                          </FormControl>
                          <FormDescription>Common: Authorization, X-API-Key, Api-Key</FormDescription>
                          <FormMessage />
                        </FormItem>
                      )}
                    />

                    <FormField
                      control={form.control}
                      name="auth_header_value"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>Header Value</FormLabel>
                          <FormControl>
                            <PasswordInput
                              {...field}
                              placeholder={
                                editId && existingMcp?.auth_uuid ? 'Leave empty to keep existing' : 'e.g. Bearer sk-...'
                              }
                              disabled={isSubmitting}
                              data-testid="mcp-auth-header-value"
                            />
                          </FormControl>
                          {editId && existingMcp?.auth_uuid && (
                            <FormDescription>An auth header value is currently configured.</FormDescription>
                          )}
                          {authHeaderKey === 'Authorization' &&
                            authHeaderValue &&
                            !authHeaderValue.startsWith('Bearer ') && (
                              <p
                                className="text-sm text-yellow-600 dark:text-yellow-500"
                                data-testid="mcp-auth-bearer-warning"
                              >
                                Header value does not start with &apos;Bearer &apos;
                              </p>
                            )}
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                  </>
                )}

                {authType === 'oauth-pre-registered' && (
                  <div className="space-y-4" data-testid="oauth-fields-section">
                    {store.isConnected ? (
                      <OAuthConnectedCard
                        config={selectedOAuthConfig}
                        onDisconnect={handleDisconnect}
                        isDisconnecting={deleteOAuthTokenMutation.isLoading}
                      />
                    ) : (
                      <>
                        {oauthConfigsList.length > 0 && !editId && (
                          <OAuthConfigDropdown
                            configs={oauthConfigsList}
                            selectedConfigId={store.selectedOAuthConfigId}
                            onSelect={(id) => store.selectOAuthConfig(id)}
                            onNewConfig={() => store.setNewOAuthConfig(true)}
                            isNewConfig={store.isNewOAuthConfig}
                            disabled={isSubmitting}
                          />
                        )}

                        {store.selectedOAuthConfigId && selectedOAuthConfig && !store.isNewOAuthConfig && (
                          <OAuthConfigSummary
                            config={selectedOAuthConfig}
                            onAuthorize={() => handleOAuthAuthorize(store.selectedOAuthConfigId!)}
                            isAuthorizing={oauthLoginMutation.isLoading}
                          />
                        )}

                        {(store.isNewOAuthConfig || oauthConfigsList.length === 0 || editId) && (
                          <>
                            <FormField
                              control={form.control}
                              name="oauth_server_url"
                              render={({ field }) => (
                                <FormItem>
                                  <FormLabel>Authorization Server URL</FormLabel>
                                  <FormControl>
                                    <Input
                                      {...field}
                                      placeholder="https://example.com"
                                      disabled={isSubmitting}
                                      data-testid="oauth-server-url"
                                    />
                                  </FormControl>
                                  <FormDescription>
                                    Base URL of the OAuth authorization server (used for .well-known discovery)
                                  </FormDescription>
                                  <FormMessage />
                                </FormItem>
                              )}
                            />

                            <FormField
                              control={form.control}
                              name="oauth_client_id"
                              render={({ field }) => (
                                <FormItem>
                                  <FormLabel>Client ID</FormLabel>
                                  <FormControl>
                                    <Input
                                      {...field}
                                      placeholder="OAuth client ID"
                                      disabled={isSubmitting}
                                      data-testid="oauth-client-id"
                                    />
                                  </FormControl>
                                  <FormMessage />
                                </FormItem>
                              )}
                            />

                            <FormField
                              control={form.control}
                              name="oauth_client_secret"
                              render={({ field }) => (
                                <FormItem>
                                  <FormLabel>Client Secret</FormLabel>
                                  <FormControl>
                                    <PasswordInput
                                      {...field}
                                      placeholder="OAuth client secret"
                                      disabled={isSubmitting}
                                      data-testid="oauth-client-secret"
                                    />
                                  </FormControl>
                                  <FormMessage />
                                </FormItem>
                              )}
                            />

                            <FormField
                              control={form.control}
                              name="oauth_authorization_endpoint"
                              render={({ field }) => (
                                <FormItem>
                                  <FormLabel>Authorization Endpoint</FormLabel>
                                  <FormControl>
                                    <Input
                                      {...field}
                                      placeholder="https://auth.example.com/authorize"
                                      disabled={isSubmitting}
                                      data-testid="oauth-authorization-endpoint"
                                    />
                                  </FormControl>
                                  <FormMessage />
                                </FormItem>
                              )}
                            />

                            <FormField
                              control={form.control}
                              name="oauth_token_endpoint"
                              render={({ field }) => (
                                <FormItem>
                                  <FormLabel>Token Endpoint</FormLabel>
                                  <FormControl>
                                    <Input
                                      {...field}
                                      placeholder="https://auth.example.com/token"
                                      disabled={isSubmitting}
                                      data-testid="oauth-token-endpoint"
                                    />
                                  </FormControl>
                                  <FormMessage />
                                </FormItem>
                              )}
                            />

                            <FormField
                              control={form.control}
                              name="oauth_scopes"
                              render={({ field }) => (
                                <FormItem>
                                  <FormLabel>Scopes (Optional)</FormLabel>
                                  <FormControl>
                                    <Input
                                      {...field}
                                      placeholder="e.g. mcp:tools mcp:read"
                                      disabled={isSubmitting}
                                      data-testid="oauth-scopes"
                                    />
                                  </FormControl>
                                  <FormDescription>Space-separated list of OAuth scopes</FormDescription>
                                  <FormMessage />
                                </FormItem>
                              )}
                            />

                            <div className="flex gap-2">
                              <Button
                                type="button"
                                variant="outline"
                                size="sm"
                                onClick={handleOAuthAutoDetect}
                                disabled={
                                  isSubmitting || oauthDiscoverMutation.isLoading || !form.watch('oauth_server_url')
                                }
                                data-testid="oauth-auto-detect"
                                data-test-state={
                                  oauthDiscoverMutation.isLoading
                                    ? 'loading'
                                    : oauthDiscoverMutation.isSuccess
                                      ? 'success'
                                      : oauthDiscoverMutation.isError
                                        ? 'error'
                                        : 'idle'
                                }
                              >
                                {oauthDiscoverMutation.isLoading ? (
                                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                ) : (
                                  <Search className="mr-2 h-4 w-4" />
                                )}
                                Auto-Detect
                              </Button>
                              <Button
                                type="button"
                                variant="outline"
                                size="sm"
                                onClick={() => handleOAuthAuthorize()}
                                disabled={isSubmitting || createOAuthConfigMutation.isLoading}
                                data-testid="oauth-authorize"
                                data-test-state={
                                  createOAuthConfigMutation.isLoading || oauthLoginMutation.isLoading
                                    ? 'loading'
                                    : 'idle'
                                }
                              >
                                {createOAuthConfigMutation.isLoading || oauthLoginMutation.isLoading ? (
                                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                ) : (
                                  <ExternalLink className="mr-2 h-4 w-4" />
                                )}
                                Authorize
                              </Button>
                            </div>
                          </>
                        )}

                        {editId && existingMcp?.auth_type === 'oauth-pre-registered' && existingMcp?.auth_uuid && (
                          <p className="text-sm text-muted-foreground" data-testid="oauth-existing-config">
                            An OAuth configuration is currently active. Use Authorize to re-authenticate.
                          </p>
                        )}
                      </>
                    )}
                  </div>
                )}
              </div>

              <ToolSelection
                selectedServer={selectedServer}
                isFetchingTools={fetchToolsMutation.isLoading}
                toolsFetched={store.toolsFetched}
                fetchedTools={store.fetchedTools}
                selectedTools={store.selectedTools}
                onToolToggle={(name) => store.toggleTool(name)}
                onSelectAll={() => store.selectAllTools()}
                onDeselectAll={() => store.deselectAllTools()}
                onFetchTools={handleFetchTools}
              />

              <div className="flex gap-4">
                {editId ? (
                  <Button type="submit" disabled={isSubmitting} data-testid="mcp-update-button">
                    {isSubmitting ? 'Updating...' : 'Update MCP'}
                  </Button>
                ) : (
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <span>
                          <Button type="submit" disabled={!canCreate} data-testid="mcp-create-button">
                            {isSubmitting ? 'Creating...' : 'Create MCP'}
                          </Button>
                        </span>
                      </TooltipTrigger>
                      {!canCreate && !isSubmitting && (
                        <TooltipContent>
                          <p>Fetch tools from server first</p>
                        </TooltipContent>
                      )}
                    </Tooltip>
                  </TooltipProvider>
                )}
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => router.push('/ui/mcps')}
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
