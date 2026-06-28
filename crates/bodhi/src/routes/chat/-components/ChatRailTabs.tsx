import { Plug, SlidersHorizontal } from 'lucide-react';

import { cn } from '@/lib/utils';

export type ChatRailTab = 'parameters' | 'mcp';

interface ChatRailTabsProps {
  value: ChatRailTab;
  onChange: (tab: ChatRailTab) => void;
  mcpCount: number;
}

/**
 * The chat rail's segmented header (Parameters · MCP servers). Published into the shell's
 * railHeader slot; the active pane is published as the rail body by the route.
 */
export function ChatRailTabs({ value, onChange, mcpCount }: ChatRailTabsProps) {
  return (
    <div className="chat-rail-tabs">
      <button
        type="button"
        className={cn('chat-rail-tab', value === 'parameters' && 'active')}
        data-testid="chat-rail-tab-parameters"
        onClick={() => onChange('parameters')}
      >
        <SlidersHorizontal className="h-3.5 w-3.5" />
        Parameters
      </button>
      <button
        type="button"
        className={cn('chat-rail-tab', value === 'mcp' && 'active')}
        data-testid="chat-rail-tab-mcp"
        onClick={() => onChange('mcp')}
      >
        <Plug className="h-3.5 w-3.5" />
        MCP servers
        {/* Distinct from the composer popover's `mcps-badge` (still present until Phase 5). */}
        <span className="chat-rail-badge" data-testid="chat-rail-mcp-count">
          {mcpCount}
        </span>
      </button>
    </div>
  );
}
