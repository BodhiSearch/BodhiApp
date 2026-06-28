import { useState } from 'react';

import type { Mcp } from '@bodhiapp/ts-client';
import { Link } from '@tanstack/react-router';
import { ChevronDown, ChevronRight, Loader2, Plug } from 'lucide-react';

import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import type { McpClientTool, McpConnectionStatus } from '@/hooks/mcps/useMcpClient';
import { cn } from '@/lib/utils';

interface McpServersPaneProps {
  mcps: Mcp[];
  enabledMcpTools: Record<string, string[]>;
  onToggleTool: (mcpId: string, toolName: string) => void;
  onToggleMcp: (mcpId: string, allToolNames: string[]) => void;
  mcpTools: Map<string, McpClientTool[]>;
  mcpConnectionStatus: Map<string, McpConnectionStatus>;
}

function isMcpAvailable(mcp: Mcp): boolean {
  return mcp.mcp_server.enabled && mcp.enabled;
}

function getUnavailableReason(mcp: Mcp): string | null {
  if (!mcp.mcp_server.enabled) return 'Disabled by administrator';
  if (!mcp.enabled) return 'Disabled by user';
  return null;
}

function checkboxState(
  mcpId: string,
  totalTools: number,
  enabledMcpTools: Record<string, string[]>
): 'checked' | 'unchecked' | 'indeterminate' {
  const count = enabledMcpTools[mcpId]?.length || 0;
  if (count === 0) return 'unchecked';
  if (count === totalTools) return 'checked';
  return 'indeterminate';
}

interface McpRowProps {
  mcp: Mcp;
  expanded: boolean;
  onToggleExpand: () => void;
  enabledMcpTools: Record<string, string[]>;
  onToggleTool: (mcpId: string, toolName: string) => void;
  onToggleMcp: (mcpId: string, allToolNames: string[]) => void;
  tools: McpClientTool[];
  connectionStatus: McpConnectionStatus | undefined;
}

function McpRow({
  mcp,
  expanded,
  onToggleExpand,
  enabledMcpTools,
  onToggleTool,
  onToggleMcp,
  tools,
  connectionStatus,
}: McpRowProps) {
  const unavailableReason = getUnavailableReason(mcp);
  const isAvailable = isMcpAvailable(mcp);
  const isConnecting = connectionStatus === 'connecting';
  const allToolNames = tools.map((t) => t.name);
  const enabledCount = enabledMcpTools[mcp.id]?.length || 0;
  const state = checkboxState(mcp.id, tools.length, enabledMcpTools);

  const header = (
    <div
      className={cn('chat-mcp-srv-head', !isAvailable && 'opacity-50')}
      data-testid={`mcp-row-${mcp.id}`}
      onClick={() => isAvailable && onToggleExpand()}
    >
      <button
        type="button"
        className="chat-mcp-chev"
        disabled={!isAvailable}
        data-testid={`mcp-expand-${mcp.id}`}
        onClick={(e) => {
          e.stopPropagation();
          onToggleExpand();
        }}
      >
        {expanded ? <ChevronDown className="h-3.5 w-3.5" /> : <ChevronRight className="h-3.5 w-3.5" />}
      </button>

      <Checkbox
        id={`mcp-${mcp.id}`}
        data-testid={`mcp-checkbox-${mcp.id}`}
        checked={state === 'checked'}
        data-state={state}
        disabled={!isAvailable}
        onClick={(e) => e.stopPropagation()}
        onCheckedChange={() => {
          if (isAvailable) onToggleMcp(mcp.id, allToolNames);
        }}
      />

      <Label htmlFor={`mcp-${mcp.id}`} className="chat-mcp-name" onClick={(e) => e.stopPropagation()}>
        {mcp.slug}
      </Label>

      <span className="chat-mcp-count">
        {isConnecting ? (
          <span className="flex items-center gap-1">
            <Loader2 className="h-3 w-3 animate-spin" />
            Connecting…
          </span>
        ) : (
          `${enabledCount}/${tools.length}`
        )}
      </span>
    </div>
  );

  const headerWithTooltip = unavailableReason ? (
    <TooltipProvider>
      <Tooltip delayDuration={300}>
        <TooltipTrigger asChild>{header}</TooltipTrigger>
        <TooltipContent>
          <p>{unavailableReason}</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  ) : (
    header
  );

  return (
    <div className="chat-mcp-srv" data-testid={`mcp-item-${mcp.id}`}>
      {headerWithTooltip}
      {expanded && isAvailable && (
        <div className="chat-mcp-tools">
          {tools.map((tool) => {
            const on = enabledMcpTools[mcp.id]?.includes(tool.name) || false;
            return (
              <div key={tool.name} className="chat-mcp-tool" data-testid={`mcp-tool-row-${mcp.id}-${tool.name}`}>
                <Checkbox
                  id={`mcp-tool-${mcp.id}-${tool.name}`}
                  data-testid={`mcp-tool-checkbox-${mcp.id}-${tool.name}`}
                  checked={on}
                  onCheckedChange={() => onToggleTool(mcp.id, tool.name)}
                />
                <Label htmlFor={`mcp-tool-${mcp.id}-${tool.name}`} className="chat-mcp-tool-name">
                  {tool.name}
                </Label>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

/** The rail's MCP-servers tab: an accordion of configured servers with per-tool checkboxes. Lifted
 *  out of the old composer popover; shares the chat's single MCP connection manager via the route. */
export function McpServersPane({
  mcps,
  enabledMcpTools,
  onToggleTool,
  onToggleMcp,
  mcpTools,
  mcpConnectionStatus,
}: McpServersPaneProps) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  const toggleExpand = (mcpId: string) =>
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(mcpId)) next.delete(mcpId);
      else next.add(mcpId);
      return next;
    });

  return (
    <div className="chat-rail-pane" data-testid="mcp-servers-pane">
      {mcps.length === 0 ? (
        <div className="chat-mcp-empty" data-testid="mcps-empty-state">
          <Plug className="h-5 w-5" />
          <div className="t">No MCP servers configured</div>
          <div className="s">
            <Link to="/mcps/" className="text-primary hover:underline" data-testid="mcps-settings-link">
              Configure servers
            </Link>{' '}
            to let the model call their tools.
          </div>
        </div>
      ) : (
        <div className="chat-mcp-list">
          {mcps.map((mcp) => (
            <McpRow
              key={mcp.id}
              mcp={mcp}
              expanded={expanded.has(mcp.id)}
              onToggleExpand={() => toggleExpand(mcp.id)}
              enabledMcpTools={enabledMcpTools}
              onToggleTool={onToggleTool}
              onToggleMcp={onToggleMcp}
              tools={mcpTools.get(mcp.id) || []}
              connectionStatus={mcpConnectionStatus.get(mcp.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
