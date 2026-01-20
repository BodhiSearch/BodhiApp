'use client';

import { useState } from 'react';

import Link from 'next/link';
import { usePathname } from 'next/navigation';

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
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { TableCell } from '@/components/ui/table';
import { toast } from '@/hooks/use-toast';
import {
  useDisableToolsetType,
  useEnableToolsetType,
  useToolsetTypes,
  type ToolsetTypeResponse,
} from '@/hooks/useToolsets';
import { cn } from '@/lib/utils';

const columns = [
  { id: 'name', name: 'Type', sorted: false },
  { id: 'description', name: 'Description', sorted: false },
  { id: 'status', name: 'App Status', sorted: false },
  { id: 'actions', name: 'Action', sorted: false },
];

function AdminToolsetsPageContent() {
  const pathname = usePathname();
  const { data, isLoading, error } = useToolsetTypes();

  const [enableDialogOpen, setEnableDialogOpen] = useState(false);
  const [disableDialogOpen, setDisableDialogOpen] = useState(false);
  const [selectedType, setSelectedType] = useState<ToolsetTypeResponse | null>(null);

  const enableMutation = useEnableToolsetType({
    onSuccess: () => {
      toast({ title: 'Toolset type enabled', description: `Enabled ${selectedType?.name}` });
      setEnableDialogOpen(false);
      setSelectedType(null);
    },
    onError: (message) => {
      toast({ title: 'Failed to enable toolset type', description: message, variant: 'destructive' });
    },
  });

  const disableMutation = useDisableToolsetType({
    onSuccess: () => {
      toast({ title: 'Toolset type disabled', description: `Disabled ${selectedType?.name}` });
      setDisableDialogOpen(false);
      setSelectedType(null);
    },
    onError: (message) => {
      toast({ title: 'Failed to disable toolset type', description: message, variant: 'destructive' });
    },
  });

  const handleToggle = (type: ToolsetTypeResponse, enabled: boolean) => {
    setSelectedType(type);
    if (enabled) {
      setEnableDialogOpen(true);
    } else {
      setDisableDialogOpen(true);
    }
  };

  const handleEnableConfirm = () => {
    if (selectedType) {
      enableMutation.mutate({ typeId: selectedType.toolset_id });
    }
  };

  const handleDisableConfirm = () => {
    if (selectedType) {
      disableMutation.mutate({ typeId: selectedType.toolset_id });
    }
  };

  const renderRow = (type: ToolsetTypeResponse) => {
    const isToggling =
      selectedType?.toolset_id === type.toolset_id && (enableMutation.isLoading || disableMutation.isLoading);

    return [
      <TableCell key="name" data-testid={`type-name-${type.toolset_id}`}>
        <span className="font-medium">{type.name}</span>
      </TableCell>,
      <TableCell key="description" data-testid={`type-description-${type.toolset_id}`}>
        <span className="text-muted-foreground">{type.description}</span>
      </TableCell>,
      <TableCell key="status" data-testid={`type-status-${type.toolset_id}`}>
        <Badge variant={type.app_enabled ? 'default' : 'secondary'}>{type.app_enabled ? 'Enabled' : 'Disabled'}</Badge>
      </TableCell>,
      <TableCell key="actions" data-testid={`type-actions-${type.toolset_id}`}>
        <div className="flex items-center gap-2">
          <Switch
            checked={type.app_enabled}
            onCheckedChange={(checked) => handleToggle(type, checked)}
            disabled={isToggling}
            data-testid={`type-toggle-${type.toolset_id}`}
          />
          <span className="text-sm text-muted-foreground">{type.app_enabled ? 'Enabled' : 'Disabled'}</span>
        </div>
      </TableCell>,
    ];
  };

  if (error) {
    const errorMessage = error.response?.data?.error?.message || error.message || 'Failed to load toolset types';
    return <ErrorPage message={errorMessage} />;
  }

  if (isLoading) {
    return (
      <div className="container mx-auto p-4" data-testid="admin-toolsets-loading">
        <div className="space-y-4">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
        </div>
      </div>
    );
  }

  const types = data?.types || [];

  return (
    <div className="container mx-auto p-4" data-testid="admin-toolsets-page">
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

      <div className="mb-4">
        <h1 className="text-2xl font-bold">Manage Toolset Types</h1>
        <p className="text-muted-foreground">Enable or disable toolset types for all users on this server.</p>
      </div>

      <div className="my-4" data-testid="types-table-container">
        <DataTable
          data={types}
          columns={columns}
          loading={isLoading}
          renderRow={renderRow}
          getItemId={(type) => type.toolset_id}
          getRowProps={(type) => ({
            'data-testid': `type-row-${type.toolset_id}`,
            'data-test-state': type.app_enabled ? 'enabled' : 'disabled',
          })}
          sort={{ column: 'name', direction: 'asc' }}
          onSortChange={() => {}}
          data-testid="types-table"
        />
      </div>

      {types.length === 0 && <div className="text-center py-8 text-muted-foreground">No toolset types available</div>}

      <AlertDialog open={enableDialogOpen} onOpenChange={setEnableDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Enable Toolset Type</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to enable &quot;{selectedType?.name}&quot; for all users? Users will be able to
              create and configure instances of this toolset type.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleEnableConfirm} disabled={enableMutation.isLoading}>
              {enableMutation.isLoading ? 'Enabling...' : 'Enable'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog open={disableDialogOpen} onOpenChange={setDisableDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Disable Toolset Type</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to disable &quot;{selectedType?.name}&quot;? Users will not be able to create new
              instances of this toolset type, but existing instances will remain accessible (read-only).
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleDisableConfirm} disabled={disableMutation.isLoading}>
              {disableMutation.isLoading ? 'Disabling...' : 'Disable'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

export default function AdminToolsetsPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready" minRole="admin">
      <AdminToolsetsPageContent />
    </AppInitializer>
  );
}
