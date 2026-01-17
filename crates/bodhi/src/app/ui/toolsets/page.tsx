'use client';

import { Pencil, Wrench } from 'lucide-react';
import { useRouter } from 'next/navigation';

import AppInitializer from '@/components/AppInitializer';
import { DataTable } from '@/components/DataTable';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { TableCell } from '@/components/ui/table';
import { UserOnboarding } from '@/components/UserOnboarding';
import { useAvailableToolsets } from '@/hooks/useToolsets';

import type { ToolsetWithTools } from '@bodhiapp/ts-client';

// Toolset metadata - hardcoded for now
const TOOLSET_METADATA: Record<string, { name: string; description: string }> = {
  'builtin-exa-web-search': {
    name: 'Exa Web Search',
    description: 'Search the web using Exa AI for real-time information',
  },
};

const columns = [
  { id: 'name', name: 'Name', sorted: false },
  { id: 'description', name: 'Description', sorted: false, className: 'hidden md:table-cell' },
  { id: 'status', name: 'Status', sorted: false },
  { id: 'actions', name: '', sorted: false },
];

function getToolsetStatus(toolset: ToolsetWithTools): {
  label: string;
  variant: 'default' | 'secondary' | 'destructive' | 'outline';
} {
  const appEnabled = toolset.app_enabled;
  const userConfig = toolset.user_config;

  if (!appEnabled) {
    return { label: 'App Disabled', variant: 'destructive' };
  }

  if (!userConfig) {
    return { label: 'Not Configured', variant: 'secondary' };
  }

  if (!userConfig.enabled) {
    return { label: 'Configured', variant: 'outline' };
  }

  return { label: 'Enabled', variant: 'default' };
}

function ToolsetsPageContent() {
  const router = useRouter();
  const { data, isLoading, error } = useAvailableToolsets();

  const handleEdit = (toolsetId: string) => {
    router.push(`/ui/toolsets/edit?toolset_id=${toolsetId}`);
  };

  const renderRow = (toolset: ToolsetWithTools) => {
    const toolsetId = toolset.toolset_id;
    const meta = TOOLSET_METADATA[toolsetId] || {
      name: toolset.name,
      description: toolset.description,
    };
    const status = getToolsetStatus(toolset);

    return [
      <TableCell key="name" data-testid={`toolset-name-${toolsetId}`}>
        <div className="flex items-center gap-2">
          <Wrench className="h-4 w-4 text-muted-foreground" />
          <span className="font-medium">{meta.name}</span>
        </div>
      </TableCell>,
      <TableCell
        key="description"
        className="hidden md:table-cell max-w-md truncate"
        data-testid={`toolset-description-${toolsetId}`}
      >
        <span className="text-muted-foreground">{meta.description}</span>
      </TableCell>,
      <TableCell key="status" data-testid={`toolset-status-${toolsetId}`}>
        <Badge variant={status.variant}>{status.label}</Badge>
      </TableCell>,
      <TableCell key="actions" data-testid={`toolset-actions-${toolsetId}`}>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => handleEdit(toolsetId)}
          title={`Configure ${meta.name}`}
          className="h-8 w-8 p-0"
          data-testid={`toolset-edit-button-${toolsetId}`}
        >
          <Pencil className="h-4 w-4" />
        </Button>
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

      <div className="my-4" data-testid="toolsets-table-container">
        <DataTable
          data={toolsets}
          columns={columns}
          loading={isLoading}
          renderRow={renderRow}
          getItemId={(toolset) => toolset.toolset_id}
          sort={{ column: 'name', direction: 'asc' }}
          onSortChange={() => {}}
          data-testid="toolsets-table"
        />
      </div>

      {toolsets.length === 0 && <div className="text-center py-8 text-muted-foreground">No toolsets available</div>}
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
