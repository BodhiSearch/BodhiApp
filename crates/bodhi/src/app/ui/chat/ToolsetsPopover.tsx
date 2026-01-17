'use client';

import { ToolsetListItem } from '@bodhiapp/ts-client';
import { Wrench } from 'lucide-react';
import { useState } from 'react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useAvailableToolsets } from '@/hooks/useToolsets';
import { cn } from '@/lib/utils';

import { parseToolsetId, parseToolName } from '../../../hooks/use-chat';

interface ToolsetsPopoverProps {
  enabledToolsets: string[];
  onToggleToolset: (toolsetId: string) => void;
  disabled?: boolean;
}

/**
 * Get the reason why a toolset is unavailable for use.
 */
function getUnavailableReason(toolset: ToolsetListItem): string | null {
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
function isToolsetAvailable(toolset: ToolsetListItem): boolean {
  return (
    toolset.app_enabled && toolset.user_config != null && toolset.user_config.enabled && toolset.user_config.has_api_key
  );
}

/**
 * Extract unique toolset IDs from available toolsets.
 * Each toolset may have multiple tools, but we group by toolset ID.
 */
function groupToolsetsByToolsetId(
  toolsets: ToolsetListItem[]
): Map<string, { name: string; tools: ToolsetListItem[] }> {
  const grouped = new Map<string, { name: string; tools: ToolsetListItem[] }>();

  for (const toolset of toolsets) {
    const toolsetId = parseToolsetId(toolset.function.name);
    if (!toolsetId) continue;

    const existing = grouped.get(toolsetId);
    if (existing) {
      existing.tools.push(toolset);
    } else {
      // Use the toolset ID as display name (formatted)
      const displayName = toolsetId
        .replace(/^builtin-/, '')
        .split('-')
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ');

      grouped.set(toolsetId, {
        name: displayName,
        tools: [toolset],
      });
    }
  }

  return grouped;
}

interface ToolsetCheckboxItemProps {
  toolsetId: string;
  displayName: string;
  tools: ToolsetListItem[];
  isEnabled: boolean;
  onToggle: (toolsetId: string) => void;
}

function ToolsetCheckboxItem({ toolsetId, displayName, tools, isEnabled, onToggle }: ToolsetCheckboxItemProps) {
  // Use the first tool to determine availability (all tools in a toolset share the same status)
  const firstTool = tools[0];
  const unavailableReason = getUnavailableReason(firstTool);
  const isAvailable = isToolsetAvailable(firstTool);

  const checkbox = (
    <div
      className={cn('flex items-center space-x-3 rounded-md p-2', !isAvailable && 'opacity-50 cursor-not-allowed')}
      data-testid={`toolset-item-${toolsetId}`}
    >
      <Checkbox
        id={`toolset-${toolsetId}`}
        data-testid={`toolset-checkbox-${toolsetId}`}
        checked={isEnabled && isAvailable}
        disabled={!isAvailable}
        onCheckedChange={() => {
          if (isAvailable) {
            onToggle(toolsetId);
          }
        }}
      />
      <div className="flex-1 min-w-0">
        <Label
          htmlFor={`toolset-${toolsetId}`}
          className={cn('text-sm font-medium cursor-pointer', !isAvailable && 'cursor-not-allowed')}
        >
          {displayName}
        </Label>
        <p className="text-xs text-muted-foreground">{tools.length} tool(s)</p>
      </div>
    </div>
  );

  if (unavailableReason) {
    return (
      <TooltipProvider>
        <Tooltip delayDuration={300}>
          <TooltipTrigger asChild>{checkbox}</TooltipTrigger>
          <TooltipContent>
            <p>{unavailableReason}</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  return checkbox;
}

export function ToolsetsPopover({ enabledToolsets, onToggleToolset, disabled = false }: ToolsetsPopoverProps) {
  const [open, setOpen] = useState(false);
  const { data: toolsetsResponse, isLoading } = useAvailableToolsets();

  const toolsets = toolsetsResponse?.toolsets || [];
  const groupedToolsets = groupToolsetsByToolsetId(toolsets);

  // Count only toolsets that are both enabled AND available
  const enabledCount = enabledToolsets.filter((id) => {
    const group = groupedToolsets.get(id);
    return group && group.tools.length > 0 && isToolsetAvailable(group.tools[0]);
  }).length;

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
      <PopoverContent className="w-64 p-2" align="start" side="top" data-testid="toolsets-popover-content">
        <div className="space-y-1">
          <h4 className="font-medium text-sm px-2 py-1">Toolsets</h4>
          {isLoading ? (
            <div className="px-2 py-4 text-sm text-muted-foreground text-center">Loading...</div>
          ) : groupedToolsets.size === 0 ? (
            <div className="px-2 py-4 text-sm text-muted-foreground text-center">No toolsets available</div>
          ) : (
            <div className="space-y-1">
              {Array.from(groupedToolsets.entries()).map(([toolsetId, { name, tools }]) => (
                <ToolsetCheckboxItem
                  key={toolsetId}
                  toolsetId={toolsetId}
                  displayName={name}
                  tools={tools}
                  isEnabled={enabledToolsets.includes(toolsetId)}
                  onToggle={onToggleToolset}
                />
              ))}
            </div>
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}
