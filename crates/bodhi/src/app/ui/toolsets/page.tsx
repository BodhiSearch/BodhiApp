'use client';

import { useState } from 'react';

import { Pencil, Plus, Trash2, Wrench } from 'lucide-react';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';

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
import { useDeleteToolset, useToolsets, type ToolsetResponse } from '@/hooks/useToolsets';
import { useUser } from '@/hooks/useUsers';
import { isAdminRole } from '@/lib/roles';
import { cn } from '@/lib/utils';

const columns = [
  { id: 'name', name: 'Name', sorted: false },
  { id: 'type', name: 'Type', sorted: false, className: 'hidden md:table-cell' },
  { id: 'status', name: 'Status', sorted: false },
  { id: 'actions', name: '', sorted: false },
];

function getToolsetStatus(toolset: ToolsetResponse): {
  label: string;
  variant: 'default' | 'secondary' | 'destructive' | 'outline';
} {
  if (!toolset.app_enabled) {
    return { label: 'App Disabled', variant: 'destructive' };
  }

  if (!toolset.enabled) {
    return { label: 'Disabled', variant: 'secondary' };
  }

  if (!toolset.has_api_key) {
    return { label: 'No API Key', variant: 'outline' };
  }

  return { label: 'Enabled', variant: 'default' };
}

function ToolsetsPageContent() {
  const router = useRouter();
  const pathname = usePathname();
  const { data: userInfo } = useUser();
  const { data, isLoading, error } = useToolsets();
  const deleteMutation = useDeleteToolset({
    onSuccess: () => {
      toast({ title: 'Toolset deleted successfully' });
      setDeleteDialogOpen(false);
      setToolsetToDelete(null);
    },
    onError: (message) => {
      toast({ title: 'Failed to delete toolset', description: message, variant: 'destructive' });
    },
  });

  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [toolsetToDelete, setToolsetToDelete] = useState<ToolsetResponse | null>(null);

  const isAdmin = userInfo?.auth_status === 'logged_in' && userInfo.role ? isAdminRole(userInfo.role) : false;

  const handleEdit = (toolset: ToolsetResponse) => {
    router.push(`/ui/toolsets/edit?id=${toolset.id}`);
  };

  const handleDeleteClick = (toolset: ToolsetResponse) => {
    setToolsetToDelete(toolset);
    setDeleteDialogOpen(true);
  };

  const handleDeleteConfirm = () => {
    if (toolsetToDelete) {
      deleteMutation.mutate({ id: toolsetToDelete.id });
    }
  };

  const renderRow = (toolset: ToolsetResponse) => {
    const status = getToolsetStatus(toolset);
    const canEdit = toolset.app_enabled;

    return [
      <TableCell key="name" data-testid={`toolset-name-${toolset.id}`}>
        <div className="flex items-center gap-2">
          <Wrench className="h-4 w-4 text-muted-foreground" />
          <span className="font-medium">{toolset.name}</span>
        </div>
      </TableCell>,
      <TableCell key="type" className="hidden md:table-cell" data-testid={`toolset-type-${toolset.id}`}>
        <span className="text-muted-foreground">{toolset.toolset_type}</span>
      </TableCell>,
      <TableCell key="status" data-testid={`toolset-status-${toolset.id}`}>
        <Badge variant={status.variant}>{status.label}</Badge>
      </TableCell>,
      <TableCell key="actions" data-testid={`toolset-actions-${toolset.id}`}>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleEdit(toolset)}
            disabled={!canEdit}
            title={canEdit ? `Edit ${toolset.name}` : 'Disabled by administrator'}
            className="h-8 w-8 p-0"
            data-testid={`toolset-edit-button-${toolset.id}`}
          >
            <Pencil className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleDeleteClick(toolset)}
            title={`Delete ${toolset.name}`}
            className="h-8 w-8 p-0 text-destructive hover:text-destructive"
            data-testid={`toolset-delete-button-${toolset.id}`}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      </TableCell>,
    ];
  };

  if (error) {
    const errorMessage = error.response?.data?.error?.message || error.message || 'Failed to load toolsets';
    return <ErrorPage message={errorMessage} />;
  }

  if (isLoading) {
    return (
      <div className="container mx-auto p-4" data-testid="toolsets-page-loading">
        <div className="space-y-4">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
        </div>
      </div>
    );
  }

  const toolsets = data?.toolsets || [];

  return (
    <div className="container mx-auto p-4" data-testid="toolsets-page">
      <UserOnboarding storageKey="toolsets-banner-dismissed">
        Configure AI toolsets to enhance your chat experience. Toolsets like web search allow the AI to access real-time
        information from the internet.
      </UserOnboarding>

      {isAdmin && (
        <div className="bg-muted/50 p-1 rounded-lg mb-6">
          <nav className="flex space-x-1" aria-label="Toolsets Navigation">
            <Link
              href="/ui/toolsets"
              className={cn(
                'px-3 py-2 text-sm font-medium rounded-md transition-all',
                pathname === '/ui/toolsets'
                  ? 'bg-background text-foreground shadow-sm'
                  : 'text-muted-foreground hover:text-foreground hover:bg-background/50'
              )}
            >
              My Toolsets
            </Link>
            <Link
              href="/ui/toolsets/admin"
              className={cn(
                'px-3 py-2 text-sm font-medium rounded-md transition-all',
                pathname === '/ui/toolsets/admin'
                  ? 'bg-background text-foreground shadow-sm'
                  : 'text-muted-foreground hover:text-foreground hover:bg-background/50'
              )}
            >
              Admin
            </Link>
          </nav>
        </div>
      )}

      <div className="flex justify-between items-center mb-4">
        <h1 className="text-2xl font-bold">Toolsets</h1>
        <Button asChild data-testid="toolset-new-button">
          <Link href="/ui/toolsets/new">
            <Plus className="h-4 w-4 mr-2" />
            New Toolset
          </Link>
        </Button>
      </div>

      <div className="my-4" data-testid="toolsets-table-container">
        <DataTable
          data={toolsets}
          columns={columns}
          loading={isLoading}
          renderRow={renderRow}
          getItemId={(toolset) => toolset.id}
          sort={{ column: 'name', direction: 'asc' }}
          onSortChange={() => {}}
          data-testid="toolsets-table"
        />
      </div>

      {toolsets.length === 0 && (
        <div className="text-center py-8 text-muted-foreground">
          <p>No toolsets configured</p>
          <Button asChild variant="link" className="mt-2">
            <Link href="/ui/toolsets/new">Create your first toolset</Link>
          </Button>
        </div>
      )}

      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Toolset</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete &quot;{toolsetToDelete?.name}&quot;? This action cannot be undone.
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

export default function ToolsetsPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <ToolsetsPageContent />
    </AppInitializer>
  );
}
