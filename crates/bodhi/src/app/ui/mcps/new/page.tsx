'use client';

import { useCallback, useEffect, useMemo, useState } from 'react';

import { zodResolver } from '@hookform/resolvers/zod';
import { Check, ChevronsUpDown, Loader2, Plus, RefreshCw } from 'lucide-react';
import Link from 'next/link';
import { useRouter, useSearchParams } from 'next/navigation';
import { useForm } from 'react-hook-form';
import * as z from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from '@/components/ui/command';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
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
  type McpServerResponse,
  type McpTool,
} from '@/hooks/useMcps';
import { useUser } from '@/hooks/useUsers';
import { isAdminRole } from '@/lib/roles';
import { cn } from '@/lib/utils';

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
    },
  });

  useEffect(() => {
    if (existingMcp && editId) {
      form.reset({
        mcp_server_id: existingMcp.mcp_server.id,
        name: existingMcp.name,
        slug: existingMcp.slug,
        description: existingMcp.description || '',
        enabled: existingMcp.enabled,
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
      fetchToolsMutation.mutate({ mcp_server_id: serverId, auth: 'public' });
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
        tools_filter: Array.from(selectedTools),
        tools_cache: fetchedTools.length > 0 ? fetchedTools : undefined,
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
              <FormField
                control={form.control}
                name="mcp_server_id"
                render={({ field }) => (
                  <FormItem className="flex flex-col">
                    <FormLabel>MCP Server</FormLabel>
                    {editId ? (
                      <div className="space-y-2">
                        <Input value={selectedServer?.url || ''} disabled data-testid="mcp-server-url-readonly" />
                        <p className="text-xs text-muted-foreground">Server: {selectedServer?.name}</p>
                      </div>
                    ) : (
                      <Popover open={comboboxOpen} onOpenChange={setComboboxOpen}>
                        <PopoverTrigger asChild>
                          <FormControl>
                            <Button
                              variant="outline"
                              role="combobox"
                              aria-expanded={comboboxOpen}
                              className={cn('w-full justify-between', !field.value && 'text-muted-foreground')}
                              disabled={isSubmitting}
                              data-testid="mcp-server-combobox"
                            >
                              {selectedServer
                                ? `${selectedServer.name} — ${selectedServer.url}`
                                : 'Select an MCP server...'}
                              <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                            </Button>
                          </FormControl>
                        </PopoverTrigger>
                        <PopoverContent className="w-[--radix-popover-trigger-width] p-0" align="start">
                          <Command>
                            <CommandInput
                              placeholder="Search by name, URL, or description..."
                              data-testid="mcp-server-search"
                            />
                            <CommandList>
                              <CommandEmpty>
                                {loadingServers ? (
                                  <div className="flex items-center gap-2 py-2">
                                    <Loader2 className="h-4 w-4 animate-spin" />
                                    Loading servers...
                                  </div>
                                ) : (
                                  <div className="text-center py-4">
                                    <p className="text-sm text-muted-foreground mb-2">No servers found</p>
                                    {isAdmin && (
                                      <Button asChild variant="link" size="sm">
                                        <Link href="/ui/mcp-servers/new">Register a new server</Link>
                                      </Button>
                                    )}
                                  </div>
                                )}
                              </CommandEmpty>
                              <CommandGroup>
                                {enabledServers.map((server) => (
                                  <CommandItem
                                    key={server.id}
                                    value={`${server.name} ${server.url} ${server.description || ''}`}
                                    onSelect={() => handleServerSelect(server)}
                                    data-testid={`mcp-server-option-${server.id}`}
                                  >
                                    <Check
                                      className={cn(
                                        'mr-2 h-4 w-4',
                                        selectedServer?.id === server.id ? 'opacity-100' : 'opacity-0'
                                      )}
                                    />
                                    <div className="flex-1 min-w-0">
                                      <div className="font-medium">{server.name}</div>
                                      <div className="text-xs text-muted-foreground font-mono truncate">
                                        {server.url}
                                      </div>
                                      {server.description && (
                                        <div className="text-xs text-muted-foreground truncate">
                                          {server.description}
                                        </div>
                                      )}
                                    </div>
                                  </CommandItem>
                                ))}
                              </CommandGroup>
                              {isAdmin && (
                                <>
                                  <CommandSeparator />
                                  <CommandGroup>
                                    <CommandItem
                                      onSelect={() => router.push('/ui/mcp-servers/new')}
                                      data-testid="mcp-server-add-new"
                                    >
                                      <Plus className="mr-2 h-4 w-4" />
                                      Add New MCP Server
                                    </CommandItem>
                                  </CommandGroup>
                                </>
                              )}
                            </CommandList>
                          </Command>
                        </PopoverContent>
                      </Popover>
                    )}
                    {selectedServer && !editId && (
                      <p className="text-xs text-muted-foreground font-mono">{selectedServer.url}</p>
                    )}
                    <FormMessage />
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

              <div className="space-y-4" data-testid="mcp-tools-section">
                <div className="flex justify-between items-center">
                  <h3 className="text-lg font-semibold">Tools</h3>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={handleFetchTools}
                    disabled={!selectedServer || fetchToolsMutation.isLoading}
                    data-testid="mcp-fetch-tools-button"
                  >
                    {fetchToolsMutation.isLoading ? (
                      <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    ) : (
                      <RefreshCw className="h-4 w-4 mr-2" />
                    )}
                    {toolsFetched ? 'Refresh Tools' : 'Fetch Tools'}
                  </Button>
                </div>

                {!toolsFetched && !fetchToolsMutation.isLoading && (
                  <p className="text-sm text-muted-foreground" data-testid="mcp-tools-empty-state">
                    {selectedServer
                      ? 'Click "Fetch Tools" to discover available tools from this MCP server.'
                      : 'Select a server and fetch tools to see available tools.'}
                  </p>
                )}

                {fetchToolsMutation.isLoading && (
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
              </div>

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
