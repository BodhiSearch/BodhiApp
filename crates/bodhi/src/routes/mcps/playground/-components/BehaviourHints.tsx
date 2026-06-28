import { ShellIcon } from '@/components/shell';
import type { McpClientTool } from '@/hooks/mcps/useMcpClient';

import { hintsForTool } from './behaviour-hints';

export function BehaviourHints({ tool }: { tool: Pick<McpClientTool, 'annotations'> }) {
  const hints = hintsForTool(tool);
  if (!hints.length) return null;
  return (
    <div className="pg-hints" data-testid="mcp-playground-hints">
      {hints.map((h) => (
        <span
          key={h.key}
          className={'pg-hint tone-' + h.tone}
          title={h.term + ' \u2014 ' + h.tip}
          data-testid={`mcp-playground-hint-${h.key}`}
        >
          <ShellIcon name={h.icon} size={11} />
          <span className="pg-hint-label">{h.label}</span>
        </span>
      ))}
    </div>
  );
}
