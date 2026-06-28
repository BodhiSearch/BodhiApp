import { Plug } from 'lucide-react';

/**
 * Placeholder for the rail's MCP-servers tab. The real accordion (server list + per-tool
 * checkboxes, lifted out of the composer popover) lands in Phase 5; until then the MCP tool
 * picker stays in the composer.
 */
export function McpServersPane() {
  return (
    <div className="chat-rail-pane" data-testid="mcp-servers-pane">
      <div className="chat-mcp-empty">
        <Plug className="h-5 w-5" />
        <div className="t">MCP tools live in the composer</div>
        <div className="s">Use the plug button next to the message box to add servers and pick tools.</div>
      </div>
    </div>
  );
}
