'use client';

import { useMemo, useState } from 'react';

import { ToolsetResponse, AppToolsetConfig } from '@bodhiapp/ts-client';
import { Wrench, ChevronRight, ChevronDown } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useToolsets, useToolsetTypes } from '@/hooks/useToolsets';
import { cn } from '@/lib/utils';

interface ToolsetsPopoverProps {
  enabledTools: Record<string, string[]>;
  onToggleTool: (toolsetId: string, toolName: string) => void;
  onToggleToolset: (toolsetId: string, allToolNames: string[]) => void;
  disabled?: boolean;
}

/**
 * Create a map from scope to enabled status from toolset_types array.
 */
function createScopeEnabledMap(toolsetTypes: AppToolsetConfig[]): Map<string, boolean> {
  const map = new Map<string, boolean>();
  toolsetTypes.forEach((config) => map.set(config.toolset_type, config.enabled));
  return map;
}

/**
 * Check if a toolset type is admin-enabled based on scope map.
 */
function isAdminEnabled(toolset: ToolsetResponse, scopeEnabledMap: Map<string, boolean>): boolean {
  return scopeEnabledMap.get(toolset.toolset_type) ?? true;
}

/**
 * Get the reason why a toolset is unavailable for use.
 */
function getUnavailableReason(toolset: ToolsetResponse, scopeEnabledMap: Map<string, boolean>): string | null {
  if (!isAdminEnabled(toolset, scopeEnabledMap)) {
    return 'Disabled by administrator';
  }
  if (!toolset.enabled) {
    return 'Disabled by user';
  }
  if (!toolset.has_api_key) {
    return 'API key not configured';
  }
  return null;
}

/**
 * Check if a toolset is available for use in chat.
 */
function isToolsetAvailable(toolset: ToolsetResponse, scopeEnabledMap: Map<string, boolean>): boolean {
  return isAdminEnabled(toolset, scopeEnabledMap) && toolset.enabled && toolset.has_api_key;
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
  toolset: ToolsetResponse;
  isExpanded: boolean;
  onToggleExpand: () => void;
  enabledTools: Record<string, string[]>;
  onToggleTool: (toolsetId: string, toolName: string) => void;
  onToggleToolset: (toolsetId: string, allToolNames: string[]) => void;
  scopeEnabledMap: Map<string, boolean>;
}

function ToolsetItem({
  toolset,
  isExpanded,
  onToggleExpand,
  enabledTools,
  onToggleTool,
  onToggleToolset,
  scopeEnabledMap,
}: ToolsetItemProps) {
  const unavailableReason = getUnavailableReason(toolset, scopeEnabledMap);
  const isAvailable = isToolsetAvailable(toolset, scopeEnabledMap);
  const allToolNames = toolset.tools.map((tool) => tool.function.name);
  const enabledCount = enabledTools[toolset.id]?.length || 0;
  const checkboxState = getCheckboxState(toolset.id, toolset.tools.length, enabledTools);

  const parentRow = (
    <div
      className={cn(
        'flex items-center space-x-2 rounded-md p-2 hover:bg-accent',
        !isAvailable && 'opacity-50 cursor-not-allowed'
      )}
      data-testid={`toolset-row-${toolset.id}`}
      data-test-toolset-name={toolset.slug}
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
        data-testid={`toolset-expand-${toolset.id}`}
      >
        {isExpanded ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
      </Button>

      {/* Parent checkbox (tri-state) */}
      <Checkbox
        id={`toolset-${toolset.id}`}
        data-testid={`toolset-checkbox-${toolset.id}`}
        data-test-toolset-name={`toolset-checkbox-name-${toolset.slug}`}
        checked={checkboxState === 'checked'}
        data-state={checkboxState}
        disabled={!isAvailable}
        onCheckedChange={() => {
          if (isAvailable) {
            onToggleToolset(toolset.id, allToolNames);
          }
        }}
      />

      {/* Toolset name and count */}
      <div className="flex-1 min-w-0">
        <Label
          htmlFor={`toolset-${toolset.id}`}
          className={cn('text-sm font-medium cursor-pointer', !isAvailable && 'cursor-not-allowed')}
          data-test-toolset-name={toolset.slug}
        >
          {toolset.slug}
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
    <div
      data-testid={`toolset-item-${toolset.id}`}
      data-testid-scope={toolset.toolset_type}
      data-test-toolset-name={toolset.slug}
    >
      {rowWithTooltip}
      {/* Expanded child tools */}
      {isExpanded && isAvailable && (
        <div className="ml-6 space-y-1 mt-1">
          {toolset.tools.map((tool) => {
            const isToolEnabled = enabledTools[toolset.id]?.includes(tool.function.name) || false;
            return (
              <div
                key={tool.function.name}
                className="flex items-center space-x-2 p-2 rounded-md hover:bg-accent"
                data-testid={`tool-row-${toolset.id}-${tool.function.name}`}
              >
                <Checkbox
                  id={`tool-${toolset.id}-${tool.function.name}`}
                  data-testid={`tool-checkbox-${toolset.id}-${tool.function.name}`}
                  checked={isToolEnabled}
                  onCheckedChange={() => onToggleTool(toolset.id, tool.function.name)}
                />
                <Label htmlFor={`tool-${toolset.id}-${tool.function.name}`} className="text-sm cursor-pointer flex-1">
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
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set());
  const { data: toolsetsResponse, isLoading: toolsetsLoading } = useToolsets();
  const { data: typesResponse, isLoading: typesLoading } = useToolsetTypes();

  const toolsets = useMemo(() => toolsetsResponse?.toolsets || [], [toolsetsResponse?.toolsets]);
  const toolsetTypes = useMemo(() => toolsetsResponse?.toolset_types || [], [toolsetsResponse?.toolset_types]);
  const types = useMemo(() => typesResponse?.types || [], [typesResponse?.types]);
  const isLoading = toolsetsLoading || typesLoading;

  // Create scope enabled map from toolset_types
  const scopeEnabledMap = useMemo(() => createScopeEnabledMap(toolsetTypes), [toolsetTypes]);

  // Create a map from scope UUID to display name
  const typeDisplayNames = useMemo(() => {
    const map = new Map<string, string>();
    types.forEach((type) => map.set(type.toolset_type, type.name));
    return map;
  }, [types]);

  // Group toolsets by scope UUID
  const groupedToolsets = useMemo(() => {
    const groups: Record<string, ToolsetResponse[]> = {};
    toolsets.forEach((toolset) => {
      const toolsetType = toolset.toolset_type;
      if (!groups[toolsetType]) {
        groups[toolsetType] = [];
      }
      groups[toolsetType].push(toolset);
    });
    return groups;
  }, [toolsets]);

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

  const toggleGroup = (typeId: string) => {
    setExpandedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(typeId)) {
        next.delete(typeId);
      } else {
        next.add(typeId);
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
            <div className="px-2 py-4 text-sm text-muted-foreground text-center" data-testid="toolsets-empty-state">
              {toolsetTypes.length > 0 ? (
                <>
                  <p>No toolsets configured.</p>
                  <p className="mt-1">
                    {toolsetTypes.length} toolset type{toolsetTypes.length !== 1 ? 's' : ''} available.
                  </p>
                  <a
                    href="/ui/toolsets"
                    className="text-primary hover:underline mt-2 inline-block"
                    data-testid="toolsets-settings-link"
                  >
                    Configure in settings
                  </a>
                </>
              ) : (
                <p>No toolsets available</p>
              )}
            </div>
          ) : (
            <div className="space-y-2">
              {Object.entries(groupedToolsets).map(([toolsetType, tools]) => {
                const displayName = typeDisplayNames.get(toolsetType) || toolsetType;
                const isSingleToolset = tools.length === 1;
                const isGroupExpanded = expandedGroups.has(toolsetType);

                // For single toolset, just show the toolset without group header
                if (isSingleToolset) {
                  const toolset = tools[0];
                  return (
                    <ToolsetItem
                      key={toolset.id}
                      toolset={toolset}
                      isExpanded={expandedToolsets.has(toolset.id)}
                      onToggleExpand={() => toggleExpand(toolset.id)}
                      enabledTools={enabledTools}
                      onToggleTool={onToggleTool}
                      onToggleToolset={onToggleToolset}
                      scopeEnabledMap={scopeEnabledMap}
                    />
                  );
                }

                // Multiple toolsets: show group header with collapsible section
                return (
                  <div
                    key={toolsetType}
                    data-testid={`toolset-group-${toolsetType}`}
                    data-test-toolset-name={displayName}
                  >
                    <Button
                      type="button"
                      variant="ghost"
                      className="w-full justify-start p-2 h-auto font-medium text-sm"
                      onClick={() => toggleGroup(toolsetType)}
                      data-testid={`toolset-group-toggle-${toolsetType}`}
                    >
                      {isGroupExpanded ? (
                        <ChevronDown className="h-4 w-4 mr-2" />
                      ) : (
                        <ChevronRight className="h-4 w-4 mr-2" />
                      )}
                      {displayName}
                      <span className="ml-2 text-muted-foreground">({tools.length})</span>
                    </Button>
                    {isGroupExpanded && (
                      <div className="ml-2 space-y-1 mt-1">
                        {tools.map((toolset) => (
                          <ToolsetItem
                            key={toolset.id}
                            toolset={toolset}
                            isExpanded={expandedToolsets.has(toolset.id)}
                            onToggleExpand={() => toggleExpand(toolset.id)}
                            enabledTools={enabledTools}
                            onToggleTool={onToggleTool}
                            onToggleToolset={onToggleToolset}
                            scopeEnabledMap={scopeEnabledMap}
                          />
                        ))}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}
