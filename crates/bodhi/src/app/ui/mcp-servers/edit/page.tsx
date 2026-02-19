'use client';

import { useEffect, useState } from 'react';

import { Trash2 } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';

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
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { toast } from '@/hooks/use-toast';
import {
  useMcpServer,
  useUpdateMcpServer,
  useListAuthConfigs,
  useDeleteAuthConfig,
  type McpAuthConfigResponse,
} from '@/hooks/useMcps';
import { authConfigTypeBadge, authConfigBadgeVariant, authConfigDetail } from '@/lib/mcpUtils';

function EditMcpServerContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const serverId = searchParams.get('id') || '';
  const { data: server, isLoading, error } = useMcpServer(serverId, { enabled: !!serverId });
  const { data: authConfigsData, isLoading: configsLoading } = useListAuthConfigs(serverId);

  const [url, setUrl] = useState('');
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [enabled, setEnabled] = useState(true);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [urlWarning, setUrlWarning] = useState(false);
  const [originalUrl, setOriginalUrl] = useState('');
  const [deleteTarget, setDeleteTarget] = useState<McpAuthConfigResponse | null>(null);

  const authConfigs = authConfigsData?.auth_configs ?? [];

  const updateMutation = useUpdateMcpServer({
    onSuccess: () => {
      toast({ title: 'MCP server updated' });
      router.push(`/ui/mcp-servers/view?id=${serverId}`);
    },
    onError: (message) => {
      toast({ title: 'Failed to update MCP server', description: message, variant: 'destructive' });
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

  useEffect(() => {
    if (server) {
      setUrl(server.url);
      setName(server.name);
      setDescription(server.description || '');
      setEnabled(server.enabled);
      setOriginalUrl(server.url);
    }
  }, [server]);

  const validate = () => {
    const newErrors: Record<string, string> = {};
    if (!name.trim()) newErrors.name = 'Name is required';
    if (name.length > 100) newErrors.name = 'Name cannot exceed 100 characters';
    if (!url.trim()) newErrors.url = 'URL is required';
    if (url.length > 2048) newErrors.url = 'URL cannot exceed 2048 characters';
    try {
      if (url.trim()) new URL(url.trim());
    } catch {
      newErrors.url = 'URL is not valid';
    }
    if (description.length > 255) newErrors.description = 'Description cannot exceed 255 characters';
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) return;

    if (url.trim() !== originalUrl && !urlWarning) {
      setUrlWarning(true);
      return;
    }

    submitUpdate();
  };

  const submitUpdate = () => {
    updateMutation.mutate({
      id: serverId,
      url: url.trim(),
      name: name.trim(),
      description: description.trim() || undefined,
      enabled,
    });
    setUrlWarning(false);
  };

  const handleDeleteConfirm = () => {
    if (!deleteTarget) return;
    deleteAuthConfig.mutate({ configId: deleteTarget.id });
  };

  if (!serverId) {
    return <ErrorPage message="No server ID provided" />;
  }

  if (error) {
    const errorMessage = error.response?.data?.error?.message || 'Failed to load MCP server';
    return <ErrorPage message={errorMessage} />;
  }

  if (isLoading) {
    return (
      <div className="container mx-auto p-4 max-w-2xl" data-testid="edit-mcp-server-loading">
        <Card>
          <CardHeader>
            <Skeleton className="h-6 w-48" />
          </CardHeader>
          <CardContent className="space-y-4">
            <Skeleton className="h-10 w-full" />
            <Skeleton className="h-10 w-full" />
            <Skeleton className="h-20 w-full" />
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="container mx-auto p-4 max-w-2xl" data-testid="edit-mcp-server-page">
      <Card>
        <CardHeader>
          <CardTitle>Edit MCP Server</CardTitle>
          <CardDescription>Update MCP server configuration.</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="url">URL *</Label>
              <Input
                id="url"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="https://mcp.example.com/mcp"
                data-testid="mcp-server-url-input"
              />
              {errors.url && <p className="text-sm text-destructive">{errors.url}</p>}
            </div>

            <div className="space-y-2">
              <Label htmlFor="name">Name *</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="My MCP Server"
                data-testid="mcp-server-name-input"
              />
              {errors.name && <p className="text-sm text-destructive">{errors.name}</p>}
            </div>

            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Optional description"
                data-testid="mcp-server-description-input"
              />
              {errors.description && <p className="text-sm text-destructive">{errors.description}</p>}
            </div>

            <div className="flex items-center space-x-2">
              <Switch
                id="enabled"
                checked={enabled}
                onCheckedChange={setEnabled}
                data-testid="mcp-server-enabled-switch"
              />
              <Label htmlFor="enabled">Enabled</Label>
            </div>

            {/* Auth Configurations Section (read-only) */}
            <div className="border-t pt-4 mt-6" data-testid="auth-configs-section">
              <h3 className="text-lg font-semibold mb-3">Auth Configurations</h3>

              {configsLoading ? (
                <div className="space-y-2">
                  <Skeleton className="h-12 w-full" />
                </div>
              ) : authConfigs.length === 0 ? (
                <Card>
                  <CardContent className="py-8 text-center text-muted-foreground">
                    No auth configurations yet.
                  </CardContent>
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
                            <Badge
                              variant={authConfigBadgeVariant(config)}
                              data-testid={`auth-config-type-badge-${id}`}
                            >
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

            <div className="flex gap-2 justify-end">
              <Button type="button" variant="outline" onClick={() => router.push('/ui/mcp-servers')}>
                Cancel
              </Button>
              <Button type="submit" disabled={updateMutation.isLoading} data-testid="mcp-server-save-button">
                {updateMutation.isLoading ? 'Saving...' : 'Save'}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>

      {/* Delete Auth Config Confirmation Dialog */}
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

      <AlertDialog open={urlWarning} onOpenChange={setUrlWarning}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>URL Changed</AlertDialogTitle>
            <AlertDialogDescription>
              Changing the URL will clear cached tools and tool filters on all linked MCP instances. Are you sure you
              want to continue?
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={submitUpdate} data-testid="url-change-confirm">
              Continue
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

export default function EditMcpServerPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <EditMcpServerContent />
    </AppInitializer>
  );
}
