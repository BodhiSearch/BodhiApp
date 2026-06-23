import { RefreshCw } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { type McpClientTool } from '@/hooks/mcps/useMcpClient';
import { cn } from '@/lib/utils';

export function ToolSidebar({
  tools,
  selectedTool,
  onSelectTool,
  onRefresh,
  isRefreshing,
  connectionStatus,
}: {
  tools: McpClientTool[];
  selectedTool: string | null;
  onSelectTool: (name: string) => void;
  onRefresh: () => void;
  isRefreshing: boolean;
  connectionStatus: string;
}) {
  return (
    <div className="w-64 shrink-0 border-r flex flex-col h-full" data-testid="mcp-playground-tool-sidebar">
      <div className="p-3 border-b flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={onRefresh}
            disabled={isRefreshing || connectionStatus !== 'connected'}
            data-testid="mcp-playground-refresh-button"
          >
            <RefreshCw className={cn('h-4 w-4', isRefreshing && 'animate-spin')} />
          </Button>
          <span className="text-xs text-muted-foreground" data-testid="mcp-playground-connection-status">
            {connectionStatus}
          </span>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto" data-testid="mcp-playground-tool-list">
        {tools.map((tool) => {
          const isSelected = selectedTool === tool.name;
          return (
            <button
              key={tool.name}
              onClick={() => onSelectTool(tool.name)}
              className={cn('w-full text-left px-3 py-2 text-sm border-b transition-colors', isSelected && 'bg-accent')}
              data-testid={`mcp-playground-tool-${tool.name}`}
            >
              <div className="font-medium truncate">{tool.name}</div>
              {tool.description && <div className="text-xs text-muted-foreground truncate">{tool.description}</div>}
            </button>
          );
        })}
        {tools.length === 0 && <div className="p-4 text-sm text-muted-foreground text-center">No tools available</div>}
      </div>
    </div>
  );
}
