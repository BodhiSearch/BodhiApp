import { useState } from 'react';

import type { Mcp } from '@bodhiapp/ts-client';
import { Link } from '@tanstack/react-router';
import { ChevronDown, ChevronRight, Loader2, Plug, Trash2 } from 'lucide-react';

import { Checkbox } from '@/components/ui/checkbox';
import { Command, CommandEmpty, CommandInput, CommandItem, CommandList } from '@/components/ui/command';
import { Label } from '@/components/ui/label';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import type { McpClientTool, McpConnectionStatus } from '@/hooks/mcps/useMcpClient';

interface McpServersPaneProps {
  mcps: Mcp[];
  enabledMcpTools: Record<string, string[]>;
  onToggleTool: (mcpId: string, toolName: string) => void;
  /** Add a server to the chat (enables all its tools) or remove it (when already added). */
  onToggleMcp: (mcpId: string, allToolNames: string[]) => void;
  mcpTools: Map<string, McpClientTool[]>;
  mcpConnectionStatus: Map<string, McpConnectionStatus>;
}

function isMcpAvailable(mcp: Mcp): boolean {
  return mcp.mcp_server.enabled && mcp.enabled;
}

interface AddedRowProps {
  mcp: Mcp;
  expanded: boolean;
  onToggleExpand: () => void;
  enabledMcpTools: Record<string, string[]>;
  onToggleTool: (mcpId: string, toolName: string) => void;
  onRemove: () => void;
  tools: McpClientTool[];
  connectionStatus: McpConnectionStatus | undefined;
}

function AddedServerRow({
  mcp,
  expanded,
  onToggleExpand,
  enabledMcpTools,
  onToggleTool,
  onRemove,
  tools,
  connectionStatus,
}: AddedRowProps) {
  const isConnecting = connectionStatus === 'connecting';
  const enabledCount = enabledMcpTools[mcp.id]?.length || 0;

  return (
    <div className="chat-mcp-srv" data-testid={`mcp-item-${mcp.id}`}>
      <div className="chat-mcp-srv-head" data-testid={`mcp-row-${mcp.id}`} onClick={onToggleExpand}>
        <button
          type="button"
          className="chat-mcp-chev"
          data-testid={`mcp-expand-${mcp.id}`}
          onClick={(e) => {
            e.stopPropagation();
            onToggleExpand();
          }}
        >
          {expanded ? <ChevronDown className="h-3.5 w-3.5" /> : <ChevronRight className="h-3.5 w-3.5" />}
        </button>
        <span className="chat-mcp-name">{mcp.slug}</span>
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
        <button
          type="button"
          className="chat-mcp-remove"
          aria-label="Remove from chat"
          data-testid={`mcp-remove-${mcp.id}`}
          onClick={(e) => {
            e.stopPropagation();
            onRemove();
          }}
        >
          <Trash2 className="h-3.5 w-3.5" />
        </button>
      </div>
      {expanded && (
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

interface AddComboProps {
  available: Mcp[];
  onAdd: (mcp: Mcp) => void;
}

/**
 * "Add an MCP server" picker on shadcn Command + Popover (cmdk). cmdk gives real listbox/option
 * semantics, keyboard nav (↑/↓ + Enter), search filtering and the app's popover theming for free.
 */
function AddServerCombo({ available, onAdd }: AddComboProps) {
  const [open, setOpen] = useState(false);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button
          type="button"
          role="combobox"
          aria-expanded={open}
          className="chat-mcp-add-trigger"
          data-testid="mcp-add-trigger"
        >
          <Plug className="h-3.5 w-3.5" />
          <span className="lbl">Add an MCP server…</span>
          <ChevronDown className="h-3.5 w-3.5" />
        </button>
      </PopoverTrigger>
      <PopoverContent className="chat-mcp-add-pop" align="start">
        <Command>
          <CommandInput placeholder="Search servers…" data-testid="mcp-add-search" />
          <CommandList>
            <CommandEmpty>No servers found</CommandEmpty>
            {available.map((mcp) => (
              <CommandItem
                key={mcp.id}
                value={mcp.slug}
                aria-label={mcp.slug}
                data-testid={`mcp-add-option-${mcp.id}`}
                onSelect={() => {
                  onAdd(mcp);
                  setOpen(false);
                }}
                className="chat-mcp-add-opt"
              >
                <span className="name">{mcp.slug}</span>
              </CommandItem>
            ))}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}

/** The rail's MCP-servers tab. Users ADD servers (enabling all their tools) from a combobox, then
 *  toggle individual tools off or remove the server. "Added" = the chat's enabledMcpTools has an
 *  entry for that MCP. */
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

  const isAdded = (mcp: Mcp) => Boolean(enabledMcpTools[mcp.id]);
  const added = mcps.filter(isAdded);
  // Offer only configured, available, not-yet-added servers in the combobox.
  const available = mcps.filter((m) => !isAdded(m) && isMcpAvailable(m));

  if (mcps.length === 0) {
    return (
      <div className="chat-rail-pane" data-testid="mcp-servers-pane">
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
      </div>
    );
  }

  return (
    <div className="chat-rail-pane" data-testid="mcp-servers-pane">
      {available.length > 0 ? (
        <AddServerCombo
          available={available}
          onAdd={(mcp) =>
            onToggleMcp(
              mcp.id,
              (mcpTools.get(mcp.id) || []).map((t) => t.name)
            )
          }
        />
      ) : (
        added.length > 0 && <div className="chat-mcp-add-done">All configured servers added</div>
      )}

      {added.length === 0 ? (
        <div className="chat-mcp-empty" data-testid="mcps-none-added">
          <Plug className="h-5 w-5" />
          <div className="t">No MCP servers in this chat</div>
          <div className="s">Add one above to let the model call its tools.</div>
        </div>
      ) : (
        <div className="chat-mcp-list">
          {added.map((mcp) => (
            <AddedServerRow
              key={mcp.id}
              mcp={mcp}
              expanded={expanded.has(mcp.id)}
              onToggleExpand={() => toggleExpand(mcp.id)}
              enabledMcpTools={enabledMcpTools}
              onToggleTool={onToggleTool}
              onRemove={() => onToggleMcp(mcp.id, enabledMcpTools[mcp.id] || [])}
              tools={mcpTools.get(mcp.id) || []}
              connectionStatus={mcpConnectionStatus.get(mcp.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
