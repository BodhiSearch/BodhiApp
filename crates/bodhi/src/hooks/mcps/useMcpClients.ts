import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { Mcp } from '@bodhiapp/ts-client';
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StreamableHTTPClientTransport } from '@modelcontextprotocol/sdk/client/streamableHttp.js';

import apiClient from '@/lib/apiClient';
import type { McpConnectionStatus, McpClientTool, McpToolCallResult } from './useMcpClient';

export interface McpClientState {
  status: McpConnectionStatus;
  tools: McpClientTool[];
  error: string | null;
}

interface ClientEntry {
  client: Client;
  transport: StreamableHTTPClientTransport;
}

export interface UseMcpClientsReturn {
  clients: Map<string, McpClientState>;
  allTools: Map<string, McpClientTool[]>;
  connectAll: (mcps: Mcp[]) => Promise<void>;
  disconnectAll: () => Promise<void>;
  callTool: (mcpId: string, toolName: string, args: Record<string, unknown>) => Promise<McpToolCallResult>;
  isConnecting: boolean;
}

export function useMcpClients(): UseMcpClientsReturn {
  const connectionsRef = useRef<Map<string, ClientEntry>>(new Map());
  const connectedEndpointsRef = useRef<Map<string, string>>(new Map());
  const [clientStates, setClientStates] = useState<Map<string, McpClientState>>(new Map());
  const [isConnecting, setIsConnecting] = useState(false);

  const disconnectAll = useCallback(async () => {
    const connections = connectionsRef.current;
    const closePromises: Promise<void>[] = [];

    for (const [, entry] of connections) {
      closePromises.push(
        (async () => {
          try {
            await entry.transport.close();
            await entry.client.close();
          } catch {
            // Best effort cleanup
          }
        })()
      );
    }

    await Promise.allSettled(closePromises);
    connectionsRef.current = new Map();
    connectedEndpointsRef.current = new Map();
    setClientStates(new Map());
  }, []);

  const connectAll = useCallback(async (mcps: Mcp[]) => {
    const eligibleMcps = mcps.filter((m) => m.mcp_server.enabled && m.enabled && m.mcp_endpoint);

    // Build incoming set: id -> endpoint
    const incomingEndpoints = new Map<string, string>();
    for (const mcp of eligibleMcps) {
      incomingEndpoints.set(mcp.id, mcp.mcp_endpoint!);
    }

    const currentEndpoints = connectedEndpointsRef.current;

    // Determine MCPs to disconnect (in current but not in incoming, or endpoint changed)
    const toDisconnect: string[] = [];
    for (const [id, endpoint] of currentEndpoints) {
      if (!incomingEndpoints.has(id) || incomingEndpoints.get(id) !== endpoint) {
        toDisconnect.push(id);
      }
    }

    // Determine MCPs to connect (in incoming but not in current, or endpoint changed)
    const toConnect: Mcp[] = [];
    for (const mcp of eligibleMcps) {
      const currentEndpoint = currentEndpoints.get(mcp.id);
      if (currentEndpoint === undefined || currentEndpoint !== mcp.mcp_endpoint) {
        toConnect.push(mcp);
      }
    }

    // Nothing to do if no changes
    if (toDisconnect.length === 0 && toConnect.length === 0) return;

    // Disconnect removed/changed MCPs
    const disconnectPromises: Promise<void>[] = [];
    for (const id of toDisconnect) {
      const entry = connectionsRef.current.get(id);
      if (entry) {
        disconnectPromises.push(
          (async () => {
            try {
              await entry.transport.close();
              await entry.client.close();
            } catch {
              // Best effort cleanup
            }
          })()
        );
        connectionsRef.current.delete(id);
      }
      connectedEndpointsRef.current.delete(id);
    }
    await Promise.allSettled(disconnectPromises);

    // Remove disconnected states, keep unchanged states
    if (toDisconnect.length > 0) {
      setClientStates((prev) => {
        const next = new Map(prev);
        for (const id of toDisconnect) {
          next.delete(id);
        }
        return next;
      });
    }

    if (toConnect.length === 0) return;

    setIsConnecting(true);

    // Set connecting status for new MCPs
    setClientStates((prev) => {
      const next = new Map(prev);
      for (const mcp of toConnect) {
        next.set(mcp.id, { status: 'connecting', tools: [], error: null });
      }
      return next;
    });

    const baseUrl =
      apiClient.defaults.baseURL || (typeof window !== 'undefined' ? window.location.origin : 'http://localhost');

    const results = await Promise.allSettled(
      toConnect.map(async (mcp) => {
        const fullUrl = baseUrl + mcp.mcp_endpoint;
        const client = new Client({ name: 'bodhi-app', version: '1.0.0' }, { capabilities: {} });
        const credentialFetch: typeof fetch = (url, init) => fetch(url, { ...init, credentials: 'include' });
        const transport = new StreamableHTTPClientTransport(new URL(fullUrl), {
          fetch: credentialFetch,
        });

        await client.connect(transport);

        const toolsResult = await client.listTools();
        const mappedTools: McpClientTool[] = (toolsResult.tools || []).map((t) => ({
          name: t.name,
          description: t.description,
          inputSchema: (t.inputSchema ?? {}) as Record<string, unknown>,
        }));

        return { mcpId: mcp.id, client, transport, tools: mappedTools };
      })
    );

    setClientStates((prev) => {
      const next = new Map(prev);
      for (let i = 0; i < results.length; i++) {
        const result = results[i];
        const mcp = toConnect[i];

        if (result.status === 'fulfilled') {
          const { client, transport, tools } = result.value;
          connectionsRef.current.set(mcp.id, { client, transport });
          connectedEndpointsRef.current.set(mcp.id, mcp.mcp_endpoint!);
          next.set(mcp.id, { status: 'connected', tools, error: null });
        } else {
          const errorMsg = result.reason instanceof Error ? result.reason.message : 'Connection failed';
          next.set(mcp.id, { status: 'error', tools: [], error: errorMsg });
        }
      }
      return next;
    });

    setIsConnecting(false);
  }, []);

  const callTool = useCallback(
    async (mcpId: string, toolName: string, args: Record<string, unknown>): Promise<McpToolCallResult> => {
      const entry = connectionsRef.current.get(mcpId);
      if (!entry) {
        return { content: `Not connected to MCP: ${mcpId}`, isError: true };
      }

      try {
        const result = await entry.client.callTool({ name: toolName, arguments: args });
        return { content: result.content, isError: result.isError };
      } catch (err) {
        return {
          content: err instanceof Error ? err.message : 'MCP tool execution failed',
          isError: true,
        };
      }
    },
    []
  );

  const allTools = useMemo(() => {
    const map = new Map<string, McpClientTool[]>();
    for (const [id, state] of clientStates) {
      map.set(id, state.tools);
    }
    return map;
  }, [clientStates]);

  useEffect(() => {
    return () => {
      const connections = connectionsRef.current;
      for (const [, entry] of connections) {
        entry.transport.close().catch(() => {});
        entry.client.close().catch(() => {});
      }
    };
  }, []);

  return {
    clients: clientStates,
    allTools,
    connectAll,
    disconnectAll,
    callTool,
    isConnecting,
  };
}
