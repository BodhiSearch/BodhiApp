import { useEffect, useMemo } from 'react';

import type { Mcp } from '@bodhiapp/ts-client';
import type { AgentTool } from '@mariozechner/pi-agent-core';

import { useMcpAgentTools } from '@/hooks/chat/useMcpAgentTools';
import { useListMcps } from '@/hooks/mcps';
import type { McpClientTool, McpConnectionStatus } from '@/hooks/mcps/useMcpClient';
import { useMcpClients } from '@/hooks/mcps/useMcpClients';
import { useMcpSelectionStore } from '@/stores/mcpSelectionStore';

export interface ChatMcp {
  mcps: Mcp[];
  enabledMcpTools: Record<string, string[]>;
  toggleTool: (mcpId: string, toolName: string) => void;
  toggleMcp: (mcpId: string, allToolNames: string[]) => void;
  mcpTools: Map<string, McpClientTool[]>;
  mcpConnectionStatus: Map<string, McpConnectionStatus>;
  agentTools: AgentTool[];
  /** Number of MCP servers with at least one tool enabled (rail-tab badge). */
  mcpCount: number;
}

/**
 * The chat's MCP wiring, lifted out of ChatUI so the composer (agent tool execution) and the rail's
 * MCP-servers picker share a single connection manager + selection. Call this ONCE per chat screen
 * (in the route) and pass the result to both ChatUI and the rail pane.
 */
export function useChatMcp(): ChatMcp {
  const enabledMcpTools = useMcpSelectionStore((s) => s.enabledTools);
  const toggleTool = useMcpSelectionStore((s) => s.toggleTool);
  const toggleMcp = useMcpSelectionStore((s) => s.toggleMcp);
  const setEnabledMcpTools = useMcpSelectionStore((s) => s.setEnabledTools);

  const { data: mcpsResponse } = useListMcps();
  const mcps = useMemo(() => mcpsResponse?.mcps || [], [mcpsResponse?.mcps]);

  const mcpClients = useMcpClients();

  useEffect(() => {
    const enabledMcps = mcps.filter((m) => m.mcp_server.enabled && m.enabled && m.path);
    mcpClients.connectAll(enabledMcps);
    return () => {
      mcpClients.disconnectAll();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- keyed on mcps; mcpClients.connectAll/disconnectAll are stable and intentionally excluded
  }, [mcps]);

  const mcpSlugs = useMemo(() => {
    const map = new Map<string, string>();
    mcps.forEach((m) => map.set(m.id, m.slug));
    return map;
  }, [mcps]);

  const mcpConnectionStatus = useMemo(() => {
    const map = new Map<string, McpConnectionStatus>();
    for (const [id, state] of mcpClients.clients) {
      map.set(id, state.status);
    }
    return map;
  }, [mcpClients.clients]);

  // Drop tools whose MCP is no longer available (disabled by admin/user).
  useEffect(() => {
    if (mcps.length === 0) return;

    const availableIds = new Set(mcps.filter((m) => m.mcp_server.enabled && m.enabled).map((m) => m.id));

    const filtered: Record<string, string[]> = {};
    let hasUnavailable = false;
    for (const [id, tools] of Object.entries(enabledMcpTools)) {
      if (availableIds.has(id)) {
        filtered[id] = tools;
      } else {
        hasUnavailable = true;
      }
    }

    if (hasUnavailable) {
      setEnabledMcpTools(filtered);
    }
  }, [mcps, enabledMcpTools, setEnabledMcpTools]);

  const agentTools = useMcpAgentTools({
    enabledMcpTools,
    allTools: mcpClients.allTools,
    slugs: mcpSlugs,
    callTool: mcpClients.callTool,
  });

  const mcpCount = useMemo(
    () => Object.values(enabledMcpTools).filter((tools) => tools.length > 0).length,
    [enabledMcpTools]
  );

  return {
    mcps,
    enabledMcpTools,
    toggleTool,
    toggleMcp,
    mcpTools: mcpClients.allTools,
    mcpConnectionStatus,
    agentTools,
    mcpCount,
  };
}
