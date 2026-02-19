'use client';

import { useCallback, useEffect, useMemo, useState } from 'react';

import { zodResolver } from '@hookform/resolvers/zod';
import { KeyRound, Loader2 } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useForm } from 'react-hook-form';
import * as z from 'zod';

import AppInitializer from '@/components/AppInitializer';
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
  useCreateMcp,
  useFetchMcpTools,
  useMcp,
  useMcpServers,
  useUpdateMcp,
  type McpAuth,
  type McpServerResponse,
  type McpTool,
} from '@/hooks/useMcps';
import { useUser } from '@/hooks/useUsers';
import { isAdminRole } from '@/lib/roles';
import McpServerSelector from '@/app/ui/mcps/new/McpServerSelector';
import ToolSelection from '@/app/ui/mcps/new/ToolSelection';

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
    auth_type: z.enum(['public', 'header']).default('public'),
    auth_header_key: z.string().optional(),
    auth_header_value: z.string().optional(),
  })
  .refine(
    (data) => {
      if (data.auth_type === 'header') {
        return !!data.auth_header_key && data.auth_header_key.length > 0;
      }
      return true;
    },
    { message: 'Header name is required', path: ['auth_header_key'] }
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

function NewMcpPageContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const editId = searchParams.get('id');
  const { data: userInfo } = useUser();
  const isAdmin = userInfo?.auth_status === 'logged_in' && userInfo.role ? isAdminRole(userInfo.role) : false;

  const {
    data: existingMcp,
    isLoading: loadingExisting,
    error: existingError,
  } = useMcp(editId || '', { enabled: !!editId });

  const { data: serversData, isLoading: loadingServers } = useMcpServers({ enabled: true }, { enabled: !editId });

  const enabledServers = useMemo(() => serversData?.mcp_servers || [], [serversData]);

  const [comboboxOpen, setComboboxOpen] = useState(false);
  const [selectedServer, setSelectedServer] = useState<McpServerResponse | null>(null);
  const [fetchedTools, setFetchedTools] = useState<McpTool[]>([]);
  const [selectedTools, setSelectedTools] = useState<Set<string>>(new Set());
  const [toolsFetched, setToolsFetched] = useState(false);

  const createMutation = useCreateMcp({
    onSuccess: () => {
      toast({ title: 'MCP created successfully' });
      router.push('/ui/mcps');
    },
    onError: (message) => {
      toast({ title: 'Failed to create MCP', description: message, variant: 'destructive' });
    },
  });

  const updateMutation = useUpdateMcp({
    onSuccess: () => {
      toast({ title: 'MCP updated successfully' });
      router.push('/ui/mcps');
    },
    onError: (message) => {
      toast({ title: 'Failed to update MCP', description: message, variant: 'destructive' });
    },
  });

  const fetchToolsMutation = useFetchMcpTools({
    onSuccess: (response) => {
      const tools = response.tools || [];
      setFetchedTools(tools);
      setSelectedTools(new Set(tools.map((t) => t.name)));
      setToolsFetched(true);
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
        auth_type: (existingMcp.auth_type as 'public' | 'header') || 'public',
        auth_header_key: existingMcp.auth_header_key || '',
        auth_header_value: '',
      });
      setSelectedServer({
        id: existingMcp.mcp_server.id,
        url: existingMcp.mcp_server.url,
        name: existingMcp.mcp_server.name,
        enabled: existingMcp.mcp_server.enabled,
      } as McpServerResponse);
      if (existingMcp.tools_cache) {
        setFetchedTools(existingMcp.tools_cache);
        setSelectedTools(new Set(existingMcp.tools_filter || []));
        setToolsFetched(true);
      }
    }
  }, [existingMcp, editId, form]);

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

      setFetchedTools([]);
      setSelectedTools(new Set());
      setToolsFetched(false);
      setComboboxOpen(false);
    },
    [form]
  );

  const handleToolToggle = (toolName: string) => {
    setSelectedTools((prev) => {
      const next = new Set(prev);
      if (next.has(toolName)) {
        next.delete(toolName);
      } else {
        next.add(toolName);
      }
      return next;
    });
  };

  const handleSelectAll = () => {
    setSelectedTools(new Set(fetchedTools.map((t) => t.name)));
  };

  const handleDeselectAll = () => {
    setSelectedTools(new Set());
  };

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
      } else {
        fetchToolsMutation.mutate({ mcp_server_id: serverId, auth: { type: 'public' } });
      }
    }
  };

  const onSubmit = (data: CreateMcpFormData) => {
    if (!editId && data.auth_type === 'header' && !data.auth_header_value) {
      form.setError('auth_header_value', { message: 'Header value is required' });
      return;
    }

    const authObj: McpAuth =
      data.auth_type === 'header'
        ? { type: 'header', header_key: data.auth_header_key || '', header_value: data.auth_header_value || '' }
        : { type: 'public' };

    if (editId) {
      let updateAuth: McpAuth | undefined;
      if (data.auth_type === 'header' && data.auth_header_value) {
        updateAuth = { type: 'header', header_key: data.auth_header_key || '', header_value: data.auth_header_value };
      } else if (data.auth_type === 'public' && existingMcp?.auth_type === 'header') {
        updateAuth = { type: 'public' };
      }
      updateMutation.mutate({
        id: editId,
        name: data.name,
        slug: data.slug,
        description: data.description || undefined,
        enabled: data.enabled,
        tools_filter: Array.from(selectedTools),
        tools_cache: fetchedTools.length > 0 ? fetchedTools : undefined,
        auth: updateAuth,
      });
      return;
    }

    createMutation.mutate({
      mcp_server_id: data.mcp_server_id,
      name: data.name,
      slug: data.slug,
      description: data.description || undefined,
      enabled: data.enabled,
      tools_cache: fetchedTools.length > 0 ? fetchedTools : undefined,
      tools_filter: Array.from(selectedTools),
      auth: authObj,
    });
  };

  const isSubmitting = createMutation.isLoading || updateMutation.isLoading;
  const canCreate = toolsFetched && !isSubmitting;

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
                        </SelectContent>
                      </Select>
                      <FormDescription>
                        {authType === 'header'
                          ? 'A custom header will be sent with every request to this MCP server.'
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
                                editId && existingMcp?.has_auth_header_value
                                  ? 'Leave empty to keep existing'
                                  : 'e.g. Bearer sk-...'
                              }
                              disabled={isSubmitting}
                              data-testid="mcp-auth-header-value"
                            />
                          </FormControl>
                          {editId && existingMcp?.has_auth_header_value && (
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
              </div>

              <ToolSelection
                selectedServer={selectedServer}
                isFetchingTools={fetchToolsMutation.isLoading}
                toolsFetched={toolsFetched}
                fetchedTools={fetchedTools}
                selectedTools={selectedTools}
                onToolToggle={handleToolToggle}
                onSelectAll={handleSelectAll}
                onDeselectAll={handleDeselectAll}
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
