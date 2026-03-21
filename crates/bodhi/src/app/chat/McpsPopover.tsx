'use client';

import { useMemo, useState } from 'react';

import { Mcp } from '@bodhiapp/ts-client';
import { Plug, ChevronRight, ChevronDown } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useListMcps } from '@/hooks/mcps';
import { cn } from '@/lib/utils';

interface McpsPopoverProps {
  enabledMcpTools: Record<string, string[]>;
  onToggleTool: (mcpId: string, toolName: string) => void;
  onToggleMcp: (mcpId: string, allToolNames: string[]) => void;
  disabled?: boolean;
}

/**
 * Check if an MCP instance is available for use in chat.
 */
function isMcpAvailable(mcp: Mcp): boolean {
  return (
    mcp.mcp_server.enabled &&
    mcp.enabled &&
    mcp.tools_cache != null &&
    mcp.tools_cache.length > 0 &&
    (mcp.tools_filter == null || mcp.tools_filter.length > 0)
  );
}

/**
 * Get the reason why an MCP is unavailable.
 */
function getUnavailableReason(mcp: Mcp): string | null {
  if (!mcp.mcp_server.enabled) return 'Disabled by administrator';
  if (!mcp.enabled) return 'Disabled by user';
  if (!mcp.tools_cache || mcp.tools_cache.length === 0) return 'Tools not yet discovered';
  if (mcp.tools_filter && mcp.tools_filter.length === 0) return 'All tools blocked by filter';
  return null;
}

/**
 * Get the visible tools for an MCP, applying tools_filter ceiling.
 */
function getVisibleTools(mcp: Mcp) {
  if (!mcp.tools_cache) return [];
  if (mcp.tools_filter == null) return mcp.tools_cache;
  return mcp.tools_cache.filter((t) => mcp.tools_filter!.includes(t.name));
}

function getCheckboxState(
  mcpId: string,
  totalTools: number,
  enabledMcpTools: Record<string, string[]>
): 'checked' | 'unchecked' | 'indeterminate' {
  const enabledCount = enabledMcpTools[mcpId]?.length || 0;
  if (enabledCount === 0) return 'unchecked';
  if (enabledCount === totalTools) return 'checked';
  return 'indeterminate';
}

interface McpItemProps {
  mcp: Mcp;
  isExpanded: boolean;
  onToggleExpand: () => void;
  enabledMcpTools: Record<string, string[]>;
  onToggleTool: (mcpId: string, toolName: string) => void;
  onToggleMcp: (mcpId: string, allToolNames: string[]) => void;
}

function McpItem({ mcp, isExpanded, onToggleExpand, enabledMcpTools, onToggleTool, onToggleMcp }: McpItemProps) {
  const unavailableReason = getUnavailableReason(mcp);
  const isAvailable = isMcpAvailable(mcp);
  const visibleTools = getVisibleTools(mcp);
  const allToolNames = visibleTools.map((t) => t.name);
  const enabledCount = enabledMcpTools[mcp.id]?.length || 0;
  const checkboxState = getCheckboxState(mcp.id, visibleTools.length, enabledMcpTools);

  const parentRow = (
    <div
      className={cn(
        'flex items-center space-x-2 rounded-md p-2 hover:bg-accent',
        !isAvailable && 'opacity-50 cursor-not-allowed'
      )}
      data-testid={`mcp-row-${mcp.id}`}
    >
      <Button
        type="button"
        variant="ghost"
        size="icon"
        className="h-4 w-4 p-0 hover:bg-transparent"
        onClick={(e) => {
          e.stopPropagation();
          onToggleExpand();
        }}
        disabled={!isAvailable}
        data-testid={`mcp-expand-${mcp.id}`}
      >
        {isExpanded ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
      </Button>

      <Checkbox
        id={`mcp-${mcp.id}`}
        data-testid={`mcp-checkbox-${mcp.id}`}
        checked={checkboxState === 'checked'}
        data-state={checkboxState}
        disabled={!isAvailable}
        onCheckedChange={() => {
          if (isAvailable) {
            onToggleMcp(mcp.id, allToolNames);
          }
        }}
      />

      <div className="flex-1 min-w-0">
        <Label
          htmlFor={`mcp-${mcp.id}`}
          className={cn('text-sm font-medium cursor-pointer', !isAvailable && 'cursor-not-allowed')}
        >
          {mcp.slug}
        </Label>
        <p className="text-xs text-muted-foreground">
          ({enabledCount}/{visibleTools.length})
        </p>
      </div>
    </div>
  );

  const rowWithTooltip = unavailableReason ? (
    <TooltipProvider>
      <Tooltip delayDuration={300}>
        <TooltipTrigger asChild>{parentRow}</TooltipTrigger>
        <TooltipContent>
          <p>{unavailableReason}</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  ) : (
    parentRow
  );

  return (
    <div data-testid={`mcp-item-${mcp.id}`}>
      {rowWithTooltip}
      {isExpanded && isAvailable && (
        <div className="ml-6 space-y-1 mt-1">
          {visibleTools.map((tool) => {
            const isToolEnabled = enabledMcpTools[mcp.id]?.includes(tool.name) || false;
            return (
              <div
                key={tool.name}
                className="flex items-center space-x-2 p-2 rounded-md hover:bg-accent"
                data-testid={`mcp-tool-row-${mcp.id}-${tool.name}`}
              >
                <Checkbox
                  id={`mcp-tool-${mcp.id}-${tool.name}`}
                  data-testid={`mcp-tool-checkbox-${mcp.id}-${tool.name}`}
                  checked={isToolEnabled}
                  onCheckedChange={() => onToggleTool(mcp.id, tool.name)}
                />
                <Label htmlFor={`mcp-tool-${mcp.id}-${tool.name}`} className="text-sm cursor-pointer flex-1">
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

export function McpsPopover({ enabledMcpTools, onToggleTool, onToggleMcp, disabled = false }: McpsPopoverProps) {
  const [open, setOpen] = useState(false);
  const [expandedMcps, setExpandedMcps] = useState<Set<string>>(new Set());
  const { data: mcpsResponse, isLoading } = useListMcps();

  const mcps = useMemo(() => mcpsResponse?.mcps || [], [mcpsResponse?.mcps]);

  const enabledCount = Object.values(enabledMcpTools).reduce((sum, tools) => sum + tools.length, 0);

  const toggleExpand = (mcpId: string) => {
    setExpandedMcps((prev) => {
      const next = new Set(prev);
      if (next.has(mcpId)) {
        next.delete(mcpId);
      } else {
        next.add(mcpId);
      }
      return next;
    });
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="h-8 w-8 relative"
          disabled={disabled || isLoading}
          data-testid="mcps-popover-trigger"
        >
          <Plug className="h-4 w-4" />
          {enabledCount > 0 && (
            <Badge
              variant="default"
              className="absolute -top-1 -right-1 h-4 min-w-4 px-1 text-[10px] flex items-center justify-center"
              data-testid="mcps-badge"
            >
              {enabledCount}
            </Badge>
          )}
          <span className="sr-only">Configure MCPs</span>
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-80 p-2" align="start" side="top" data-testid="mcps-popover-content">
        <div className="space-y-1">
          <h4 className="font-medium text-sm px-2 py-1">MCPs</h4>
          {isLoading ? (
            <div className="px-2 py-4 text-sm text-muted-foreground text-center">Loading...</div>
          ) : mcps.length === 0 ? (
            <div className="px-2 py-4 text-sm text-muted-foreground text-center" data-testid="mcps-empty-state">
              <p>No MCPs configured.</p>
              <a
                href="/mcps"
                className="text-primary hover:underline mt-2 inline-block"
                data-testid="mcps-settings-link"
              >
                Configure in settings
              </a>
            </div>
          ) : (
            <div className="space-y-2">
              {mcps.map((mcp) => (
                <McpItem
                  key={mcp.id}
                  mcp={mcp}
                  isExpanded={expandedMcps.has(mcp.id)}
                  onToggleExpand={() => toggleExpand(mcp.id)}
                  enabledMcpTools={enabledMcpTools}
                  onToggleTool={onToggleTool}
                  onToggleMcp={onToggleMcp}
                />
              ))}
            </div>
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}
