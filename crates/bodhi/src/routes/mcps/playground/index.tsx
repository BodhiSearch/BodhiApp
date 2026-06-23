import { useEffect, useMemo, useState } from 'react';

import { createFileRoute, Link, useSearch } from '@tanstack/react-router';
import { ArrowLeft } from 'lucide-react';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { useGetMcp } from '@/hooks/mcps';
import { useMcpClient } from '@/hooks/mcps/useMcpClient';
import { extractErrorMessage } from '@/lib/errorUtils';

import { ExecutionArea } from './-components/ExecutionArea';
import { ToolSidebar } from './-components/ToolSidebar';

export const Route = createFileRoute('/mcps/playground/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: McpPlaygroundPage,
});

function McpPlaygroundContent() {
  const search = useSearch({ from: '/mcps/playground/' });
  const id = search.id || '';
  const { data: mcp, isLoading, error } = useGetMcp(id, { enabled: !!id });
  const [selectedToolName, setSelectedToolName] = useState<string | null>(null);

  const mcpClient = useMcpClient(mcp?.path ?? null);

  useEffect(() => {
    if (mcp?.path) {
      mcpClient.connect();
    }
    return () => {
      mcpClient.disconnect();
    };
    // Connect/disconnect keyed on path only; mcpClient is recreated each render and would reconnect in a loop if listed.
  }, [mcp?.path]); // eslint-disable-line react-hooks/exhaustive-deps

  const tools = mcpClient.tools;

  const selectedTool = useMemo(() => tools.find((t) => t.name === selectedToolName) || null, [tools, selectedToolName]);

  const handleRefresh = () => {
    mcpClient.refreshTools();
  };

  if (!id) {
    return <ErrorPage message="No MCP ID provided" />;
  }

  if (error) {
    const errorMessage = extractErrorMessage(error, 'Failed to load MCP');
    return <ErrorPage message={errorMessage} />;
  }

  if (isLoading || !mcp) {
    return (
      <div className="container mx-auto p-4" data-testid="mcp-playground-loading">
        <div className="space-y-4">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-64 w-full" />
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-[calc(100vh-4rem)]" data-testid="mcp-playground-page">
      <div className="border-b px-4 py-3 flex items-center gap-3">
        <Button variant="ghost" size="sm" asChild data-testid="mcp-playground-back-button">
          <Link to="/mcps/">
            <ArrowLeft className="h-4 w-4 mr-1" />
            MCP Servers
          </Link>
        </Button>
        <span className="text-muted-foreground">/</span>
        <h1 className="font-semibold">{mcp.name} -- Playground</h1>
        {mcpClient.status === 'error' && mcpClient.error && (
          <span className="text-xs text-destructive ml-2">{mcpClient.error}</span>
        )}
      </div>

      <div className="flex flex-1 min-h-0">
        <ToolSidebar
          tools={tools}
          selectedTool={selectedToolName}
          onSelectTool={setSelectedToolName}
          onRefresh={handleRefresh}
          isRefreshing={mcpClient.status === 'connecting'}
          connectionStatus={mcpClient.status}
        />

        {selectedTool ? (
          <ExecutionArea key={selectedTool.name} tool={selectedTool} callTool={mcpClient.callTool} />
        ) : (
          <div className="flex-1 flex items-center justify-center text-muted-foreground">
            Select a tool from the sidebar to get started
          </div>
        )}
      </div>
    </div>
  );
}

export default function McpPlaygroundPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <McpPlaygroundContent />
    </AppInitializer>
  );
}
