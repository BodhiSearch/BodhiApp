'use client';

import { useState } from 'react';

import { Eye, Pencil, Plus } from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';

import AppInitializer from '@/components/AppInitializer';
import { DataTable } from '@/components/DataTable';
import { McpManagementTabs } from '@/components/McpManagementTabs';
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
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { TableCell } from '@/components/ui/table';
import { toast } from '@/hooks/use-toast';
import {
  useListAuthConfigs,
  useMcpServers,
  useUpdateMcpServer,
  type McpAuthConfigResponse,
  type McpServerResponse,
} from '@/hooks/useMcps';
import { useUser } from '@/hooks/useUsers';
import { isAdminRole } from '@/lib/roles';
import { authConfigTypeBadge } from '@/lib/mcpUtils';

function ServerAuthConfigsSummary({ serverId }: { serverId: string }) {
  const { data: authConfigsData } = useListAuthConfigs(serverId);

  const items = authConfigsData?.auth_configs ?? [];

  if (items.length === 0) return null;

  return (
    <div
      className="pl-6 py-1 space-y-1 bg-muted/30 border-l-2 border-muted"
      data-testid={`server-auth-configs-${serverId}`}
    >
      {items.map((config) => (
        <div key={config.id} className="flex items-center gap-2 text-sm py-0.5">
          <span className="text-muted-foreground">{config.name}</span>
          <Badge variant="outline" className="text-xs">
            {authConfigTypeBadge(config)}
          </Badge>
        </div>
      ))}
    </div>
  );
}

const columns = [
  { id: 'name', name: 'Name', sorted: false },
  { id: 'url', name: 'URL', sorted: false, className: 'hidden md:table-cell' },
  { id: 'enabled', name: 'Status', sorted: false },
  { id: 'mcps', name: 'MCPs', sorted: false, className: 'hidden lg:table-cell' },
  { id: 'actions', name: '', sorted: false },
];

function McpServersPageContent() {
  const router = useRouter();
  const { data: userInfo } = useUser();
  const isAdmin = userInfo?.auth_status === 'logged_in' && userInfo.role ? isAdminRole(userInfo.role) : false;
  const { data, isLoading, error } = useMcpServers({});
  const updateMutation = useUpdateMcpServer({
    onSuccess: () => {
      toast({ title: 'MCP server updated' });
      setToggleDialog(null);
    },
    onError: (message) => {
      toast({ title: 'Failed to update MCP server', description: message, variant: 'destructive' });
      setToggleDialog(null);
    },
  });

  const [toggleDialog, setToggleDialog] = useState<McpServerResponse | null>(null);

  const handleToggle = (server: McpServerResponse) => {
    setToggleDialog(server);
  };

  const handleToggleConfirm = () => {
    if (!toggleDialog) return;
    updateMutation.mutate({
      id: toggleDialog.id,
      url: toggleDialog.url,
      name: toggleDialog.name,
      description: toggleDialog.description,
      enabled: !toggleDialog.enabled,
    });
  };

  const renderRow = (server: McpServerResponse) => {
    const totalMcps = server.enabled_mcp_count + server.disabled_mcp_count;

    return [
      <TableCell key="name">
        <div>
          <span className="font-medium">{server.name}</span>
          {server.description && (
            <p className="text-xs text-muted-foreground truncate max-w-[200px]">{server.description}</p>
          )}
        </div>
      </TableCell>,
      <TableCell key="url" className="hidden md:table-cell">
        <span className="text-muted-foreground text-sm font-mono truncate max-w-[300px] inline-block">
          {server.url}
        </span>
      </TableCell>,
      <TableCell key="enabled" data-testid={`server-status-${server.id}`}>
        {isAdmin ? (
          <Switch
            checked={server.enabled}
            onCheckedChange={() => handleToggle(server)}
            data-testid={`server-toggle-${server.id}`}
          />
        ) : (
          <Badge variant={server.enabled ? 'default' : 'secondary'}>{server.enabled ? 'Enabled' : 'Disabled'}</Badge>
        )}
      </TableCell>,
      <TableCell key="mcps" className="hidden lg:table-cell" data-testid={`server-mcp-count-${server.id}`}>
        <span className="text-muted-foreground">{totalMcps}</span>
      </TableCell>,
      <TableCell key="actions" data-testid={`server-actions-${server.id}`}>
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="sm"
            asChild
            className="h-8 w-8 p-0"
            data-testid={`server-view-button-${server.id}`}
          >
            <Link href={`/ui/mcp-servers/view?id=${server.id}`} title={`View ${server.name}`}>
              <Eye className="h-4 w-4" />
            </Link>
          </Button>
          {isAdmin && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => router.push(`/ui/mcp-servers/edit?id=${server.id}`)}
              title={`Edit ${server.name}`}
              className="h-8 w-8 p-0"
              data-testid={`server-edit-button-${server.id}`}
            >
              <Pencil className="h-4 w-4" />
            </Button>
          )}
        </div>
      </TableCell>,
    ];
  };

  const renderExpandedRow = (server: McpServerResponse) => {
    return <ServerAuthConfigsSummary serverId={server.id} />;
  };

  if (error) {
    const errorMessage = error.response?.data?.error?.message || error.message || 'Failed to load MCP servers';
    return <ErrorPage message={errorMessage} />;
  }

  if (isLoading) {
    return (
      <div className="container mx-auto p-4" data-testid="mcp-servers-page-loading">
        <McpManagementTabs />
        <div className="space-y-4">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
        </div>
      </div>
    );
  }

  const servers = data?.mcp_servers || [];

  return (
    <div className="container mx-auto p-4" data-testid="mcp-servers-page">
      <McpManagementTabs />

      <div className="flex justify-between items-center mb-4">
        <h1 className="text-2xl font-bold">MCP Servers</h1>
        {isAdmin && (
          <Button asChild data-testid="mcp-server-new-button">
            <Link href="/ui/mcp-servers/new">
              <Plus className="h-4 w-4 mr-2" />
              New MCP Server
            </Link>
          </Button>
        )}
      </div>

      <div className="my-4" data-testid="mcp-servers-table-container">
        <DataTable
          data={servers}
          columns={columns}
          loading={isLoading}
          renderRow={renderRow}
          renderExpandedRow={renderExpandedRow}
          getItemId={(s) => s.id}
          sort={{ column: 'name', direction: 'asc' }}
          onSortChange={() => {}}
          getRowProps={(s) => ({
            'data-testid': `server-row-${s.id}`,
            'data-test-server-name': s.name,
          })}
        />
      </div>

      {servers.length === 0 && (
        <div className="text-center py-8 text-muted-foreground">
          <p>No MCP servers registered yet</p>
          {isAdmin && (
            <Button asChild variant="link" className="mt-2">
              <Link href="/ui/mcp-servers/new">Register the first MCP server</Link>
            </Button>
          )}
        </div>
      )}

      <AlertDialog open={!!toggleDialog} onOpenChange={() => setToggleDialog(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {toggleDialog?.enabled ? 'Disable' : 'Enable'} &quot;{toggleDialog?.name}&quot;?
            </AlertDialogTitle>
            <AlertDialogDescription>
              {toggleDialog?.enabled
                ? `Disabling this server will prevent new MCP instances from being created. Existing ${(toggleDialog?.enabled_mcp_count ?? 0) + (toggleDialog?.disabled_mcp_count ?? 0)} MCPs will be affected.`
                : 'Enabling this server will allow users to create MCP instances using it.'}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleToggleConfirm} disabled={updateMutation.isLoading}>
              {updateMutation.isLoading ? 'Updating...' : toggleDialog?.enabled ? 'Disable' : 'Enable'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

export default function McpServersPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <McpServersPageContent />
    </AppInitializer>
  );
}
