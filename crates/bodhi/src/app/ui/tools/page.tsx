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
import { useAvailableTools, ToolListItem } from '@/hooks/useTools';

// Tool metadata - hardcoded for now
const TOOL_METADATA: Record<string, { name: string; description: string }> = {
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

function getToolStatus(tool: ToolListItem): {
  label: string;
  variant: 'default' | 'secondary' | 'destructive' | 'outline';
} {
  const appEnabled = tool.app_enabled;
  const userConfig = tool.user_config;

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

function ToolsPageContent() {
  const router = useRouter();
  const { data, isLoading, error } = useAvailableTools();

  const handleEdit = (toolId: string) => {
    router.push(`/ui/tools/edit?toolid=${toolId}`);
  };

  const renderRow = (tool: ToolListItem) => {
    const toolId = tool.function.name;
    const meta = TOOL_METADATA[toolId] || {
      name: toolId,
      description: tool.function.description,
    };
    const status = getToolStatus(tool);

    return [
      <TableCell key="name" data-testid={`tool-name-${toolId}`}>
        <div className="flex items-center gap-2">
          <Wrench className="h-4 w-4 text-muted-foreground" />
          <span className="font-medium">{meta.name}</span>
        </div>
      </TableCell>,
      <TableCell
        key="description"
        className="hidden md:table-cell max-w-md truncate"
        data-testid={`tool-description-${toolId}`}
      >
        <span className="text-muted-foreground">{meta.description}</span>
      </TableCell>,
      <TableCell key="status" data-testid={`tool-status-${toolId}`}>
        <Badge variant={status.variant}>{status.label}</Badge>
      </TableCell>,
      <TableCell key="actions" data-testid={`tool-actions-${toolId}`}>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => handleEdit(toolId)}
          title={`Configure ${meta.name}`}
          className="h-8 w-8 p-0"
          data-testid={`tool-edit-button-${toolId}`}
        >
          <Pencil className="h-4 w-4" />
        </Button>
      </TableCell>,
    ];
  };

  if (error) {
    const errorMessage = error.response?.data?.error?.message || error.message || 'Failed to load tools';
    return <ErrorPage message={errorMessage} />;
  }

  if (isLoading) {
    return (
      <div className="container mx-auto p-4" data-testid="tools-page-loading">
        <div className="space-y-4">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
        </div>
      </div>
    );
  }

  const tools = data?.tools || [];

  return (
    <div className="container mx-auto p-4" data-testid="tools-page">
      <UserOnboarding storageKey="tools-banner-dismissed">
        Configure AI tools to enhance your chat experience. Tools like web search allow the AI to access real-time
        information from the internet.
      </UserOnboarding>

      <div className="my-4" data-testid="tools-table-container">
        <DataTable
          data={tools}
          columns={columns}
          loading={isLoading}
          renderRow={renderRow}
          getItemId={(tool) => tool.function.name}
          sort={{ column: 'name', direction: 'asc' }}
          onSortChange={() => {}}
          data-testid="tools-table"
        />
      </div>

      {tools.length === 0 && <div className="text-center py-8 text-muted-foreground">No tools available</div>}
    </div>
  );
}

export default function ToolsPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <ToolsPageContent />
    </AppInitializer>
  );
}
