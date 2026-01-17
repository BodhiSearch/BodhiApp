'use client';

import { useState } from 'react';

import { ToolsetWithTools } from '@bodhiapp/ts-client';
import { Wrench, ChevronRight, ChevronDown } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useAvailableToolsets } from '@/hooks/useToolsets';
import { cn } from '@/lib/utils';

interface ToolsetsPopoverProps {
  enabledTools: Record<string, string[]>;
  onToggleTool: (toolsetId: string, toolName: string) => void;
  onToggleToolset: (toolsetId: string, allToolNames: string[]) => void;
  disabled?: boolean;
}

/**
 * Get the reason why a toolset is unavailable for use.
 */
function getUnavailableReason(toolset: ToolsetWithTools): string | null {
  if (!toolset.app_enabled) {
    return 'Disabled by administrator';
  }
  if (!toolset.user_config) {
    return 'Configure in Toolsets settings';
  }
  if (!toolset.user_config.enabled) {
    return 'Disabled in settings';
  }
  if (!toolset.user_config.has_api_key) {
    return 'API key not configured';
  }
  return null;
}

/**
 * Check if a toolset is available for use in chat.
 */
function isToolsetAvailable(toolset: ToolsetWithTools): boolean {
  return (
    toolset.app_enabled && toolset.user_config != null && toolset.user_config.enabled && toolset.user_config.has_api_key
  );
}

/**
 * Calculate tri-state checkbox state for toolset.
 */
function getCheckboxState(
  toolsetId: string,
  totalTools: number,
  enabledTools: Record<string, string[]>
): 'checked' | 'unchecked' | 'indeterminate' {
  const enabledCount = enabledTools[toolsetId]?.length || 0;
  if (enabledCount === 0) return 'unchecked';
  if (enabledCount === totalTools) return 'checked';
  return 'indeterminate';
}

interface ToolsetItemProps {
  toolset: ToolsetWithTools;
  isExpanded: boolean;
  onToggleExpand: () => void;
  enabledTools: Record<string, string[]>;
  onToggleTool: (toolsetId: string, toolName: string) => void;
  onToggleToolset: (toolsetId: string, allToolNames: string[]) => void;
}

function ToolsetItem({
  toolset,
  isExpanded,
  onToggleExpand,
  enabledTools,
  onToggleTool,
  onToggleToolset,
}: ToolsetItemProps) {
  const unavailableReason = getUnavailableReason(toolset);
  const isAvailable = isToolsetAvailable(toolset);
  const allToolNames = toolset.tools.map((tool) => tool.function.name);
  const enabledCount = enabledTools[toolset.toolset_id]?.length || 0;
  const checkboxState = getCheckboxState(toolset.toolset_id, toolset.tools.length, enabledTools);

  const parentRow = (
    <div
      className={cn(
        'flex items-center space-x-2 rounded-md p-2 hover:bg-accent',
        !isAvailable && 'opacity-50 cursor-not-allowed'
      )}
      data-testid={`toolset-row-${toolset.toolset_id}`}
    >
      {/* Expand/collapse chevron */}
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
        data-testid={`toolset-expand-${toolset.toolset_id}`}
      >
        {isExpanded ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
      </Button>

      {/* Parent checkbox (tri-state) */}
      <Checkbox
        id={`toolset-${toolset.toolset_id}`}
        data-testid={`toolset-checkbox-${toolset.toolset_id}`}
        checked={checkboxState === 'checked'}
        data-state={checkboxState}
        disabled={!isAvailable}
        onCheckedChange={() => {
          if (isAvailable) {
            onToggleToolset(toolset.toolset_id, allToolNames);
          }
        }}
      />

      {/* Toolset name and count */}
      <div className="flex-1 min-w-0">
        <Label
          htmlFor={`toolset-${toolset.toolset_id}`}
          className={cn('text-sm font-medium cursor-pointer', !isAvailable && 'cursor-not-allowed')}
        >
          {toolset.name}
        </Label>
        <p className="text-xs text-muted-foreground">
          ({enabledCount}/{toolset.tools.length})
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
    <div data-testid={`toolset-item-${toolset.toolset_id}`}>
      {rowWithTooltip}
      {/* Expanded child tools */}
      {isExpanded && isAvailable && (
        <div className="ml-6 space-y-1 mt-1">
          {toolset.tools.map((tool) => {
            const isToolEnabled = enabledTools[toolset.toolset_id]?.includes(tool.function.name) || false;
            return (
              <div
                key={tool.function.name}
                className="flex items-center space-x-2 p-2 rounded-md hover:bg-accent"
                data-testid={`tool-row-${toolset.toolset_id}-${tool.function.name}`}
              >
                <Checkbox
                  id={`tool-${toolset.toolset_id}-${tool.function.name}`}
                  data-testid={`tool-checkbox-${toolset.toolset_id}-${tool.function.name}`}
                  checked={isToolEnabled}
                  onCheckedChange={() => onToggleTool(toolset.toolset_id, tool.function.name)}
                />
                <Label
                  htmlFor={`tool-${toolset.toolset_id}-${tool.function.name}`}
                  className="text-sm cursor-pointer flex-1"
                >
                  {tool.function.name}
                </Label>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

export function ToolsetsPopover({
  enabledTools,
  onToggleTool,
  onToggleToolset,
  disabled = false,
}: ToolsetsPopoverProps) {
  const [open, setOpen] = useState(false);
  const [expandedToolsets, setExpandedToolsets] = useState<Set<string>>(new Set());
  const { data: toolsetsResponse, isLoading } = useAvailableToolsets();

  const toolsets = toolsetsResponse?.toolsets || [];

  // Count total enabled tools across all toolsets
  const enabledCount = Object.values(enabledTools).reduce((sum, tools) => sum + tools.length, 0);

  const toggleExpand = (toolsetId: string) => {
    setExpandedToolsets((prev) => {
      const next = new Set(prev);
      if (next.has(toolsetId)) {
        next.delete(toolsetId);
      } else {
        next.add(toolsetId);
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
          data-testid="toolsets-popover-trigger"
        >
          <Wrench className="h-4 w-4" />
          {enabledCount > 0 && (
            <Badge
              variant="default"
              className="absolute -top-1 -right-1 h-4 min-w-4 px-1 text-[10px] flex items-center justify-center"
              data-testid="toolsets-badge"
            >
              {enabledCount}
            </Badge>
          )}
          <span className="sr-only">Configure toolsets</span>
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-80 p-2" align="start" side="top" data-testid="toolsets-popover-content">
        <div className="space-y-1">
          <h4 className="font-medium text-sm px-2 py-1">Toolsets</h4>
          {isLoading ? (
            <div className="px-2 py-4 text-sm text-muted-foreground text-center">Loading...</div>
          ) : toolsets.length === 0 ? (
            <div className="px-2 py-4 text-sm text-muted-foreground text-center">No toolsets available</div>
          ) : (
            <div className="space-y-1">
              {toolsets.map((toolset) => (
                <ToolsetItem
                  key={toolset.toolset_id}
                  toolset={toolset}
                  isExpanded={expandedToolsets.has(toolset.toolset_id)}
                  onToggleExpand={() => toggleExpand(toolset.toolset_id)}
                  enabledTools={enabledTools}
                  onToggleTool={onToggleTool}
                  onToggleToolset={onToggleToolset}
                />
              ))}
            </div>
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}
