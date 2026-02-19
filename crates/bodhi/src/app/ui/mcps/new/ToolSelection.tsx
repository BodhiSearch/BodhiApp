'use client';

import { Loader2, RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Skeleton } from '@/components/ui/skeleton';
import type { McpServerResponse, McpTool } from '@/hooks/useMcps';

type ToolSelectionProps = {
  selectedServer: McpServerResponse | null;
  isFetchingTools: boolean;
  toolsFetched: boolean;
  fetchedTools: McpTool[];
  selectedTools: Set<string>;
  onToolToggle: (toolName: string) => void;
  onSelectAll: () => void;
  onDeselectAll: () => void;
  onFetchTools: () => void;
};

const ToolSelection = ({
  selectedServer,
  isFetchingTools,
  toolsFetched,
  fetchedTools,
  selectedTools,
  onToolToggle,
  onSelectAll,
  onDeselectAll,
  onFetchTools,
}: ToolSelectionProps) => {
  return (
    <div className="space-y-4" data-testid="mcp-tools-section">
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-semibold">Tools</h3>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={onFetchTools}
          disabled={!selectedServer || isFetchingTools}
          data-testid="mcp-fetch-tools-button"
        >
          {isFetchingTools ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <RefreshCw className="h-4 w-4 mr-2" />}
          {toolsFetched ? 'Refresh Tools' : 'Fetch Tools'}
        </Button>
      </div>

      {!toolsFetched && !isFetchingTools && (
        <p className="text-sm text-muted-foreground" data-testid="mcp-tools-empty-state">
          {selectedServer
            ? 'Click "Fetch Tools" to discover available tools from this MCP server.'
            : 'Select a server and fetch tools to see available tools.'}
        </p>
      )}

      {isFetchingTools && (
        <div className="space-y-2" data-testid="mcp-tools-loading">
          <Skeleton className="h-8 w-full" />
          <Skeleton className="h-8 w-full" />
          <Skeleton className="h-8 w-full" />
        </div>
      )}

      {toolsFetched && fetchedTools.length > 0 && (
        <div className="space-y-3">
          <div className="flex gap-2 text-sm">
            <Button type="button" variant="ghost" size="sm" onClick={onSelectAll} data-testid="mcp-select-all-tools">
              Select All
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={onDeselectAll}
              data-testid="mcp-deselect-all-tools"
            >
              Deselect All
            </Button>
            <span className="ml-auto text-muted-foreground flex items-center">
              {selectedTools.size}/{fetchedTools.length} selected
            </span>
          </div>
          <div className="border rounded-lg divide-y max-h-[400px] overflow-y-auto" data-testid="mcp-tools-list">
            {fetchedTools.map((tool) => (
              <label
                key={tool.name}
                className="flex items-start gap-3 p-3 hover:bg-muted/50 cursor-pointer"
                data-testid={`mcp-tool-${tool.name}`}
              >
                <Checkbox
                  checked={selectedTools.has(tool.name)}
                  onCheckedChange={() => onToolToggle(tool.name)}
                  className="mt-0.5"
                  data-testid={`mcp-tool-checkbox-${tool.name}`}
                />
                <div className="flex-1 min-w-0">
                  <div className="font-medium text-sm">{tool.name}</div>
                  {tool.description && <div className="text-xs text-muted-foreground mt-0.5">{tool.description}</div>}
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
  );
};

export default ToolSelection;
