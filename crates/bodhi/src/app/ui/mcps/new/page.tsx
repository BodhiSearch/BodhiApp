'use client';

import { useCallback, useEffect, useState } from 'react';

import { zodResolver } from '@hookform/resolvers/zod';
import { AlertCircle, CheckCircle2, Loader2, RefreshCw } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useForm } from 'react-hook-form';
import * as z from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { toast } from '@/hooks/use-toast';
import {
  useCreateMcp,
  useEnableMcpServer,
  useMcp,
  useMcpServerCheck,
  useRefreshMcpTools,
  useUpdateMcp,
  type McpTool,
} from '@/hooks/useMcps';
import { useUser } from '@/hooks/useUsers';
import { isAdminRole } from '@/lib/roles';

const createMcpSchema = z.object({
  url: z.string().url('Must be a valid URL').min(1, 'URL is required'),
  name: z.string().min(1, 'Name is required').max(100, 'Name must be 100 characters or less'),
  slug: z
    .string()
    .min(1, 'Slug is required')
    .max(24, 'Slug must be 24 characters or less')
    .regex(/^[a-zA-Z0-9-]+$/, 'Slug can only contain letters, numbers, and hyphens'),
  description: z.string().max(255).optional(),
  enabled: z.boolean().default(true),
});

type CreateMcpFormData = z.infer<typeof createMcpSchema>;

type ServerStatus = 'idle' | 'checking' | 'enabled' | 'not_enabled' | 'error';

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
  } = useMcp(editId || '', {
    enabled: !!editId,
  });

  const [serverStatus, setServerStatus] = useState<ServerStatus>('idle');
  const [enableDialogOpen, setEnableDialogOpen] = useState(false);
  const [urlToCheck, setUrlToCheck] = useState('');
  const [fetchedTools, setFetchedTools] = useState<McpTool[]>([]);
  const [selectedTools, setSelectedTools] = useState<Set<string>>(new Set());
  const [toolsFetched, setToolsFetched] = useState(false);
  const [createdMcpId, setCreatedMcpId] = useState<string | null>(null);

  const { data: serverCheckData, isFetching: isCheckingServer } = useMcpServerCheck(urlToCheck, {
    enabled: !!urlToCheck,
  });

  const enableServerMutation = useEnableMcpServer({
    onSuccess: () => {
      setEnableDialogOpen(false);
      setServerStatus('enabled');
      toast({ title: 'MCP server URL enabled' });
    },
    onError: (message) => {
      toast({ title: 'Failed to enable MCP server', description: message, variant: 'destructive' });
    },
  });

  const createMutation = useCreateMcp({
    onSuccess: (mcp) => {
      setCreatedMcpId(mcp.id);
      toast({ title: 'MCP created, fetching tools...' });
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

  const refreshToolsMutation = useRefreshMcpTools({
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
      url: '',
      name: '',
      slug: '',
      description: '',
      enabled: true,
    },
  });

  useEffect(() => {
    if (existingMcp && editId) {
      form.reset({
        url: existingMcp.url,
        name: existingMcp.name,
        slug: existingMcp.slug,
        description: existingMcp.description || '',
        enabled: existingMcp.enabled,
      });
      setServerStatus('enabled');
      if (existingMcp.tools_cache) {
        setFetchedTools(existingMcp.tools_cache);
        setSelectedTools(new Set(existingMcp.tools_filter || []));
        setToolsFetched(true);
      }
    }
  }, [existingMcp, editId, form]);

  useEffect(() => {
    if (!urlToCheck || isCheckingServer) return;

    if (serverCheckData) {
      const servers = serverCheckData.mcp_servers || [];
      const found = servers.find((s) => s.url === urlToCheck);
      if (found && found.enabled) {
        setServerStatus('enabled');
      } else {
        setServerStatus('not_enabled');
      }
    }
  }, [serverCheckData, isCheckingServer, urlToCheck]);

  const handleCheckUrl = useCallback(() => {
    const url = form.getValues('url');
    if (!url) return;
    try {
      new URL(url);
    } catch {
      form.setError('url', { message: 'Must be a valid URL' });
      return;
    }
    setServerStatus('checking');
    setUrlToCheck(url);
  }, [form]);

  const handleEnableServer = () => {
    const url = form.getValues('url');
    enableServerMutation.mutate({ url, enabled: true });
  };

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
    if (createdMcpId) {
      refreshToolsMutation.mutate({ id: createdMcpId });
    } else if (editId) {
      refreshToolsMutation.mutate({ id: editId });
    }
  };

  const onSubmit = (data: CreateMcpFormData) => {
    if (editId) {
      updateMutation.mutate({
        id: editId,
        name: data.name,
        slug: data.slug,
        description: data.description || undefined,
        enabled: data.enabled,
      });
      return;
    }

    createMutation.mutate({
      url: data.url,
      name: data.name,
      slug: data.slug,
      description: data.description || undefined,
      enabled: data.enabled,
    });
  };

  const handleSaveAndFinish = () => {
    if (editId || createdMcpId) {
      router.push('/ui/mcps');
    }
  };

  const isSubmitting = createMutation.isLoading || updateMutation.isLoading;
  const isCreatedOrEditing = !!editId || !!createdMcpId;

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
          <CardTitle>{editId ? 'Edit MCP Server' : 'Add MCP Server'}</CardTitle>
          <CardDescription>
            {editId
              ? 'Update your MCP server configuration.'
              : 'Connect to an MCP server to add external tools to your AI assistant.'}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
              <FormField
                control={form.control}
                name="url"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Server URL</FormLabel>
                    <div className="flex gap-2">
                      <FormControl>
                        <Input
                          {...field}
                          placeholder="https://mcp.example.com/mcp"
                          disabled={isSubmitting || !!editId || !!createdMcpId}
                          data-testid="mcp-url-input"
                        />
                      </FormControl>
                      {!editId && !createdMcpId && (
                        <Button
                          type="button"
                          variant="outline"
                          onClick={handleCheckUrl}
                          disabled={isCheckingServer || serverStatus === 'checking'}
                          data-testid="mcp-check-url-button"
                        >
                          {isCheckingServer || serverStatus === 'checking' ? (
                            <Loader2 className="h-4 w-4 animate-spin" />
                          ) : (
                            'Check'
                          )}
                        </Button>
                      )}
                    </div>
                    <FormDescription>The MCP server endpoint URL (exact match required)</FormDescription>
                    <FormMessage />
                    {serverStatus === 'enabled' && (
                      <div className="flex items-center gap-2 text-sm text-green-600" data-testid="mcp-url-enabled">
                        <CheckCircle2 className="h-4 w-4" />
                        Server URL is enabled
                      </div>
                    )}
                    {serverStatus === 'not_enabled' && (
                      <div className="space-y-2">
                        <div
                          className="flex items-center gap-2 text-sm text-amber-600"
                          data-testid="mcp-url-not-enabled"
                        >
                          <AlertCircle className="h-4 w-4" />
                          This server URL is not in the allowlist
                        </div>
                        {isAdmin && (
                          <Button
                            type="button"
                            variant="outline"
                            size="sm"
                            onClick={() => setEnableDialogOpen(true)}
                            data-testid="mcp-enable-server-button"
                          >
                            Enable this server
                          </Button>
                        )}
                        {!isAdmin && (
                          <p className="text-sm text-muted-foreground">
                            Contact your administrator to enable this server URL.
                          </p>
                        )}
                      </div>
                    )}
                  </FormItem>
                )}
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
                        placeholder="My MCP Server"
                        disabled={isSubmitting}
                        data-testid="mcp-name-input"
                      />
                    </FormControl>
                    <FormDescription>A friendly name for this MCP connection</FormDescription>
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
                      <Input
                        {...field}
                        placeholder="my-mcp-server"
                        disabled={isSubmitting}
                        data-testid="mcp-slug-input"
                      />
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
                        placeholder="Describe what this MCP server provides"
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
                      <FormDescription>Make this MCP server available for use</FormDescription>
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

              {!isCreatedOrEditing && (
                <div className="flex gap-4">
                  <Button
                    type="submit"
                    disabled={isSubmitting || serverStatus !== 'enabled'}
                    data-testid="mcp-create-button"
                  >
                    {isSubmitting ? 'Creating...' : 'Create MCP'}
                  </Button>
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
              )}

              {editId && (
                <div className="flex gap-4">
                  <Button type="submit" disabled={isSubmitting} data-testid="mcp-update-button">
                    {isSubmitting ? 'Updating...' : 'Update MCP'}
                  </Button>
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
              )}
            </form>
          </Form>

          {isCreatedOrEditing && (
            <div className="mt-6 space-y-4" data-testid="mcp-tools-section">
              <div className="flex justify-between items-center">
                <h3 className="text-lg font-semibold">Tools</h3>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleFetchTools}
                  disabled={refreshToolsMutation.isLoading}
                  data-testid="mcp-fetch-tools-button"
                >
                  {refreshToolsMutation.isLoading ? (
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  ) : (
                    <RefreshCw className="h-4 w-4 mr-2" />
                  )}
                  {toolsFetched ? 'Refresh Tools' : 'Fetch Tools'}
                </Button>
              </div>

              {!toolsFetched && !refreshToolsMutation.isLoading && (
                <p className="text-sm text-muted-foreground">
                  Click &quot;Fetch Tools&quot; to discover available tools from this MCP server.
                </p>
              )}

              {refreshToolsMutation.isLoading && (
                <div className="space-y-2" data-testid="mcp-tools-loading">
                  <Skeleton className="h-8 w-full" />
                  <Skeleton className="h-8 w-full" />
                  <Skeleton className="h-8 w-full" />
                </div>
              )}

              {toolsFetched && fetchedTools.length > 0 && (
                <div className="space-y-3">
                  <div className="flex gap-2 text-sm">
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={handleSelectAll}
                      data-testid="mcp-select-all-tools"
                    >
                      Select All
                    </Button>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={handleDeselectAll}
                      data-testid="mcp-deselect-all-tools"
                    >
                      Deselect All
                    </Button>
                    <span className="ml-auto text-muted-foreground flex items-center">
                      {selectedTools.size}/{fetchedTools.length} selected
                    </span>
                  </div>
                  <div
                    className="border rounded-lg divide-y max-h-[400px] overflow-y-auto"
                    data-testid="mcp-tools-list"
                  >
                    {fetchedTools.map((tool) => (
                      <label
                        key={tool.name}
                        className="flex items-start gap-3 p-3 hover:bg-muted/50 cursor-pointer"
                        data-testid={`mcp-tool-${tool.name}`}
                      >
                        <Checkbox
                          checked={selectedTools.has(tool.name)}
                          onCheckedChange={() => handleToolToggle(tool.name)}
                          className="mt-0.5"
                          data-testid={`mcp-tool-checkbox-${tool.name}`}
                        />
                        <div className="flex-1 min-w-0">
                          <div className="font-medium text-sm">{tool.name}</div>
                          {tool.description && (
                            <div className="text-xs text-muted-foreground mt-0.5">{tool.description}</div>
                          )}
                        </div>
                      </label>
                    ))}
                  </div>
                </div>
              )}

              {toolsFetched && fetchedTools.length === 0 && (
                <p className="text-sm text-muted-foreground" data-testid="mcp-no-tools">
                  No tools found on this MCP server.
                </p>
              )}

              <div className="flex gap-4 pt-4 border-t">
                <Button onClick={handleSaveAndFinish} data-testid="mcp-done-button">
                  Done
                </Button>
                <Button variant="outline" onClick={() => router.push('/ui/mcps')} data-testid="mcp-back-button">
                  Back to List
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      <Dialog open={enableDialogOpen} onOpenChange={setEnableDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Enable MCP Server URL</DialogTitle>
            <DialogDescription>
              This will add the URL to the allowlist, making it available for all users to create MCP connections.
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <p className="text-sm font-mono bg-muted p-2 rounded break-all">{form.getValues('url')}</p>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEnableDialogOpen(false)}>
              Cancel
            </Button>
            <Button
              onClick={handleEnableServer}
              disabled={enableServerMutation.isLoading}
              data-testid="mcp-confirm-enable-button"
            >
              {enableServerMutation.isLoading ? 'Enabling...' : 'Enable Server'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
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
