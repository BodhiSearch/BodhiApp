import { useCallback, useEffect, useMemo } from 'react';

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
  /** Add a server to the chat: marks it added and establishes its MCP connection. */
  addMcp: (mcp: Mcp) => void;
  /** Remove a server from the chat: drops it and tears down its MCP connection. */
  removeMcp: (mcpId: string) => void;
  mcpTools: Map<string, McpClientTool[]>;
  mcpConnectionStatus: Map<string, McpConnectionStatus>;
  agentTools: AgentTool[];
  /** Number of MCP servers added to this chat (rail-tab badge). */
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
  const addMcpToSelection = useMcpSelectionStore((s) => s.addMcp);
  const removeMcpFromSelection = useMcpSelectionStore((s) => s.removeMcp);
  const setEnabledMcpTools = useMcpSelectionStore((s) => s.setEnabledTools);

  const { data: mcpsResponse } = useListMcps();
  const mcps = useMemo(() => mcpsResponse?.mcps || [], [mcpsResponse?.mcps]);

  const mcpClients = useMcpClients();

  // The set of MCPs added to THIS chat (a key in the selection). Connections track this set — adding
  // a server connects it, removing it tears the connection down — via connectAll's diffing.
  const addedMcps = useMemo(
    () => mcps.filter((m) => enabledMcpTools[m.id] !== undefined && m.mcp_server.enabled && m.enabled && m.path),
    [mcps, enabledMcpTools]
  );

  // Reconcile connections to the added set on every change (connectAll diffs: connects new, drops
  // removed). NO cleanup here — tearing down on each change would disconnect/reconnect everything.
  useEffect(() => {
    mcpClients.connectAll(addedMcps);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- keyed on the added set; connectAll is stable
  }, [addedMcps]);

  // Disconnect everything only when the chat screen unmounts.
  useEffect(() => {
    return () => {
      mcpClients.disconnectAll();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- unmount-only; disconnectAll is stable
  }, []);

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

  // A freshly-added server is registered with an empty tool list (so it connects); once it connects
  // and its tools arrive, default to all tools enabled.
  useEffect(() => {
    for (const [id, tools] of Object.entries(enabledMcpTools)) {
      if (tools.length > 0) continue;
      const connected = mcpClients.allTools.get(id);
      if (connected && connected.length > 0) {
        addMcpToSelection(
          id,
          connected.map((t) => t.name)
        );
      }
    }
  }, [enabledMcpTools, mcpClients.allTools, addMcpToSelection]);

  // Drop selections whose MCP is no longer available (disabled by admin/user).
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

  // Add marks the server added (empty selection → connects → tools populate); remove drops it.
  const addMcp = useCallback((mcp: Mcp) => addMcpToSelection(mcp.id, []), [addMcpToSelection]);
  const removeMcp = useCallback((mcpId: string) => removeMcpFromSelection(mcpId), [removeMcpFromSelection]);

  // Badge = servers added to this chat.
  const mcpCount = useMemo(() => Object.keys(enabledMcpTools).length, [enabledMcpTools]);

  return {
    mcps,
    enabledMcpTools,
    toggleTool,
    addMcp,
    removeMcp,
    mcpTools: mcpClients.allTools,
    mcpConnectionStatus,
    agentTools,
    mcpCount,
  };
}
