import { useCallback, useEffect, useRef, useState } from 'react';

import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StreamableHTTPClientTransport } from '@modelcontextprotocol/sdk/client/streamableHttp.js';

import apiClient from '@/lib/apiClient';

export type McpConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'refreshing' | 'error';

export interface McpClientTool {
  name: string;
  description?: string;
  inputSchema: Record<string, unknown>;
}

export interface McpToolCallResult {
  content: unknown;
  isError?: boolean;
}

export interface UseMcpClientReturn {
  status: McpConnectionStatus;
  tools: McpClientTool[];
  error: string | null;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  callTool: (name: string, args: Record<string, unknown>) => Promise<McpToolCallResult>;
  refreshTools: () => Promise<void>;
}

export function useMcpClient(endpoint: string | null): UseMcpClientReturn {
  const clientRef = useRef<Client | null>(null);
  const transportRef = useRef<StreamableHTTPClientTransport | null>(null);
  const [status, setStatus] = useState<McpConnectionStatus>('disconnected');
  const [tools, setTools] = useState<McpClientTool[]>([]);
  const [error, setError] = useState<string | null>(null);

  const disconnect = useCallback(async () => {
    try {
      if (transportRef.current) {
        await transportRef.current.close();
      }
      if (clientRef.current) {
        await clientRef.current.close();
      }
    } catch {
      // Best effort cleanup
    }
    clientRef.current = null;
    transportRef.current = null;
    setStatus('disconnected');
    setTools([]);
    setError(null);
  }, []);

  const connect = useCallback(async () => {
    if (!endpoint) return;

    await disconnect();

    setStatus('connecting');
    setError(null);

    try {
      const baseUrl =
        apiClient.defaults.baseURL || (typeof window !== 'undefined' ? window.location.origin : 'http://localhost');
      const fullUrl = baseUrl + endpoint;

      const client = new Client({ name: 'bodhi-app', version: '1.0.0' }, { capabilities: {} });
      const credentialFetch: typeof fetch = (url, init) => fetch(url, { ...init, credentials: 'include' });
      const transport = new StreamableHTTPClientTransport(new URL(fullUrl), {
        fetch: credentialFetch,
      });

      await client.connect(transport);

      clientRef.current = client;
      transportRef.current = transport;

      const result = await client.listTools();
      const mappedTools: McpClientTool[] = (result.tools || []).map((t) => ({
        name: t.name,
        description: t.description,
        inputSchema: (t.inputSchema ?? {}) as Record<string, unknown>,
      }));
      setTools(mappedTools);
      setStatus('connected');
    } catch (err) {
      setStatus('error');
      setError(err instanceof Error ? err.message : 'Failed to connect to MCP server');
      clientRef.current = null;
      transportRef.current = null;
    }
  }, [endpoint, disconnect]);

  const callTool = useCallback(async (name: string, args: Record<string, unknown>): Promise<McpToolCallResult> => {
    if (!clientRef.current) {
      return { content: 'Not connected to MCP server', isError: true };
    }

    try {
      const result = await clientRef.current.callTool({ name, arguments: args });
      return { content: result.content, isError: result.isError as boolean };
    } catch (err) {
      return {
        content: err instanceof Error ? err.message : 'MCP tool execution failed',
        isError: true,
      };
    }
  }, []);

  const refreshTools = useCallback(async () => {
    if (!clientRef.current) return;

    setStatus('refreshing');
    try {
      const result = await clientRef.current.listTools();
      const mappedTools: McpClientTool[] = (result.tools || []).map((t) => ({
        name: t.name,
        description: t.description,
        inputSchema: (t.inputSchema ?? {}) as Record<string, unknown>,
      }));
      setTools(mappedTools);
      setStatus('connected');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to refresh tools');
      setStatus('connected');
    }
  }, []);

  useEffect(() => {
    return () => {
      if (transportRef.current) {
        transportRef.current.close().catch(() => {});
      }
      if (clientRef.current) {
        clientRef.current.close().catch(() => {});
      }
    };
  }, []);

  return {
    status,
    tools,
    error,
    connect,
    disconnect,
    callTool,
    refreshTools,
  };
}
