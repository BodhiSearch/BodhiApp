'use client';

import { useState } from 'react';

import { Pencil, Play, Plug, Plus, Trash2 } from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';

import AppInitializer from '@/components/AppInitializer';
import { DataTable } from '@/components/DataTable';
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
import { TableCell } from '@/components/ui/table';
import { UserOnboarding } from '@/components/UserOnboarding';
import { toast } from '@/hooks/use-toast';
import { useDeleteMcp, useMcps, type McpResponse } from '@/hooks/useMcps';

const columns = [
  { id: 'name', name: 'Name', sorted: false },
  { id: 'url', name: 'URL', sorted: false, className: 'hidden md:table-cell' },
  { id: 'tools', name: 'Tools', sorted: false, className: 'hidden lg:table-cell' },
  { id: 'status', name: 'Status', sorted: false },
  { id: 'actions', name: '', sorted: false },
];

function McpStatus({ mcp }: { mcp: McpResponse }) {
  if (!mcp.enabled) {
    return <Badge variant="secondary">Disabled</Badge>;
  }
  const toolCount = mcp.tools_filter?.length ?? 0;
  if (toolCount === 0) {
    return <Badge variant="outline">No Tools</Badge>;
  }
  return <Badge variant="default">Active</Badge>;
}

function McpsPageContent() {
  const router = useRouter();
  const { data, isLoading, error } = useMcps();
  const deleteMutation = useDeleteMcp({
    onSuccess: () => {
      toast({ title: 'MCP deleted successfully' });
      setDeleteDialogOpen(false);
      setMcpToDelete(null);
    },
    onError: (message) => {
      toast({ title: 'Failed to delete MCP', description: message, variant: 'destructive' });
    },
  });

  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [mcpToDelete, setMcpToDelete] = useState<McpResponse | null>(null);

  const handleDeleteClick = (mcp: McpResponse) => {
    setMcpToDelete(mcp);
    setDeleteDialogOpen(true);
  };

  const handleDeleteConfirm = () => {
    if (mcpToDelete) {
      deleteMutation.mutate({ id: mcpToDelete.id });
    }
  };

  const renderRow = (mcp: McpResponse) => {
    const toolCount = mcp.tools_filter?.length ?? 0;

    return [
      <TableCell key="name">
        <div className="flex items-center gap-2">
          <Plug className="h-4 w-4 text-muted-foreground" />
          <div>
            <span className="font-medium">{mcp.name}</span>
            {mcp.description && (
              <p className="text-xs text-muted-foreground truncate max-w-[200px]">{mcp.description}</p>
            )}
          </div>
        </div>
      </TableCell>,
      <TableCell key="url" className="hidden md:table-cell" data-testid={`mcp-url-${mcp.id}`}>
        <span className="text-muted-foreground text-sm font-mono truncate max-w-[300px] inline-block">{mcp.url}</span>
      </TableCell>,
      <TableCell key="tools" className="hidden lg:table-cell" data-testid={`mcp-tools-${mcp.id}`}>
        <span className="text-muted-foreground">
          {toolCount} tool{toolCount !== 1 ? 's' : ''}
        </span>
      </TableCell>,
      <TableCell key="status" data-testid={`mcp-status-${mcp.id}`}>
        <McpStatus mcp={mcp} />
      </TableCell>,
      <TableCell key="actions" data-testid={`mcp-actions-${mcp.id}`}>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => router.push(`/ui/mcps/playground?id=${mcp.id}`)}
            title={`Playground ${mcp.name}`}
            className="h-8 w-8 p-0"
            data-testid={`mcp-playground-button-${mcp.id}`}
          >
            <Play className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => router.push(`/ui/mcps/new?id=${mcp.id}`)}
            title={`Edit ${mcp.name}`}
            className="h-8 w-8 p-0"
            data-testid={`mcp-edit-button-${mcp.id}`}
          >
            <Pencil className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleDeleteClick(mcp)}
            title={`Delete ${mcp.name}`}
            className="h-8 w-8 p-0 text-destructive hover:text-destructive"
            data-testid={`mcp-delete-button-${mcp.id}`}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      </TableCell>,
    ];
  };

  if (error) {
    const errorMessage = error.response?.data?.error?.message || error.message || 'Failed to load MCPs';
    return <ErrorPage message={errorMessage} />;
  }

  if (isLoading) {
    return (
      <div className="container mx-auto p-4" data-testid="mcps-page-loading">
        <div className="space-y-4">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
        </div>
      </div>
    );
  }

  const mcps = data?.mcps || [];

  return (
    <div className="container mx-auto p-4" data-testid="mcps-page">
      <UserOnboarding storageKey="mcps-banner-dismissed">
        Connect MCP (Model Context Protocol) servers to extend your AI with external tools. MCP servers provide
        capabilities like web search, code analysis, and more.
      </UserOnboarding>

      <div className="flex justify-between items-center mb-4">
        <h1 className="text-2xl font-bold">MCP Servers</h1>
        <Button asChild data-testid="mcp-new-button">
          <Link href="/ui/mcps/new">
            <Plus className="h-4 w-4 mr-2" />
            Add MCP Server
          </Link>
        </Button>
      </div>

      <div className="my-4" data-testid="mcps-table-container">
        <DataTable
          data={mcps}
          columns={columns}
          loading={isLoading}
          renderRow={renderRow}
          getItemId={(mcp) => mcp.id}
          sort={{ column: 'name', direction: 'asc' }}
          onSortChange={() => {}}
          data-testid="mcps-table"
          getRowProps={(mcp) => ({
            'data-test-uuid': mcp.id,
            'data-testid': `mcp-row-${mcp.id}`,
            'data-test-mcp-name': mcp.name,
          })}
        />
      </div>

      {mcps.length === 0 && (
        <div className="text-center py-8 text-muted-foreground">
          <p>No MCP servers configured</p>
          <Button asChild variant="link" className="mt-2">
            <Link href="/ui/mcps/new">Add your first MCP server</Link>
          </Button>
        </div>
      )}

      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete MCP Server</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete &quot;{mcpToDelete?.name}&quot;? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteConfirm} disabled={deleteMutation.isLoading}>
              {deleteMutation.isLoading ? 'Deleting...' : 'Delete'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

export default function McpsPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <McpsPageContent />
    </AppInitializer>
  );
}
