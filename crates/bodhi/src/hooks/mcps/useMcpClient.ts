import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StreamableHTTPClientTransport } from '@modelcontextprotocol/sdk/client/streamableHttp.js';

import apiClient from '@/lib/apiClient';

import { mapSdkToolsToClient } from './toolMapping';

export type McpConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'refreshing' | 'error';

export interface McpToolAnnotations {
  title?: string;
  readOnlyHint?: boolean;
  destructiveHint?: boolean;
  idempotentHint?: boolean;
  openWorldHint?: boolean;
}

export interface McpClientTool {
  name: string;
  description?: string;
  title?: string;
  inputSchema: Record<string, unknown>;
  annotations?: McpToolAnnotations;
}

export interface McpClientPromptArg {
  name: string;
  description?: string;
  required?: boolean;
}

export interface McpClientPrompt {
  name: string;
  description?: string;
  title?: string;
  arguments?: McpClientPromptArg[];
}

export interface McpClientResource {
  uri: string;
  name: string;
  title?: string;
  description?: string;
  mimeType?: string;
}

export interface McpClientResourceTemplate {
  uriTemplate: string;
  name: string;
  title?: string;
  description?: string;
  mimeType?: string;
}

export interface McpToolCallResult {
  content: unknown;
  isError?: boolean;
  structuredContent?: unknown;
}

export interface McpPromptMessage {
  role: 'user' | 'assistant';
  content: unknown;
}

export interface McpPromptGetResult {
  description?: string;
  messages: McpPromptMessage[];
  isError?: boolean;
  errorMessage?: string;
}

export interface McpResourceContent {
  uri: string;
  mimeType?: string;
  text?: string;
  blob?: string;
}

export interface McpResourceReadResult {
  contents: McpResourceContent[];
  isError?: boolean;
  errorMessage?: string;
}

export interface McpCapabilityCounts {
  tools: number;
  prompts: number;
  resources: number;
  resourceTemplates: number;
}

export interface UseMcpClientReturn {
  status: McpConnectionStatus;
  tools: McpClientTool[];
  prompts: McpClientPrompt[];
  resources: McpClientResource[];
  resourceTemplates: McpClientResourceTemplate[];
  counts: McpCapabilityCounts;
  error: string | null;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  callTool: (name: string, args: Record<string, unknown>) => Promise<McpToolCallResult>;
  getPrompt: (name: string, args?: Record<string, string>) => Promise<McpPromptGetResult>;
  readResource: (uri: string) => Promise<McpResourceReadResult>;
  refresh: () => Promise<void>;
}

async function listGuarded<T>(fn: () => Promise<T[]>): Promise<T[]> {
  try {
    return await fn();
  } catch {
    // Server doesn't advertise the capability, or the method is unknown.
    // Capability lists are best-effort: degrade to empty instead of failing connect/refresh.
    return [];
  }
}

export function useMcpClient(endpoint: string | null): UseMcpClientReturn {
  const clientRef = useRef<Client | null>(null);
  const transportRef = useRef<StreamableHTTPClientTransport | null>(null);
  const [status, setStatus] = useState<McpConnectionStatus>('disconnected');
  const [tools, setTools] = useState<McpClientTool[]>([]);
  const [prompts, setPrompts] = useState<McpClientPrompt[]>([]);
  const [resources, setResources] = useState<McpClientResource[]>([]);
  const [resourceTemplates, setResourceTemplates] = useState<McpClientResourceTemplate[]>([]);
  const [error, setError] = useState<string | null>(null);

  const clearAll = useCallback(() => {
    setTools([]);
    setPrompts([]);
    setResources([]);
    setResourceTemplates([]);
  }, []);

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
    clearAll();
    setError(null);
  }, [clearAll]);

  const listAll = useCallback(async (client: Client) => {
    const [nextTools, nextPrompts, nextResources, nextTemplates] = await Promise.all([
      listGuarded(async () => mapSdkToolsToClient((await client.listTools()).tools)),
      listGuarded(async () => {
        const r = await client.listPrompts();
        return (r.prompts || []).map<McpClientPrompt>((p) => ({
          name: p.name,
          title: p.title,
          description: p.description,
          arguments: p.arguments,
        }));
      }),
      listGuarded(async () => {
        const r = await client.listResources();
        return (r.resources || []).map<McpClientResource>((res) => ({
          uri: res.uri,
          name: res.name,
          title: res.title,
          description: res.description,
          mimeType: res.mimeType,
        }));
      }),
      listGuarded(async () => {
        const r = await client.listResourceTemplates();
        return (r.resourceTemplates || []).map<McpClientResourceTemplate>((t) => ({
          uriTemplate: t.uriTemplate,
          name: t.name,
          title: t.title,
          description: t.description,
          mimeType: t.mimeType,
        }));
      }),
    ]);
    setTools(nextTools);
    setPrompts(nextPrompts);
    setResources(nextResources);
    setResourceTemplates(nextTemplates);
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

      await listAll(client);
      setStatus('connected');
    } catch (err) {
      setStatus('error');
      setError(err instanceof Error ? err.message : 'Failed to connect to MCP server');
      clientRef.current = null;
      transportRef.current = null;
    }
  }, [endpoint, disconnect, listAll]);

  const callTool = useCallback(async (name: string, args: Record<string, unknown>): Promise<McpToolCallResult> => {
    if (!clientRef.current) {
      return { content: 'Not connected to MCP server', isError: true };
    }

    try {
      const result = await clientRef.current.callTool({ name, arguments: args });
      return {
        content: result.content,
        isError: result.isError as boolean,
        structuredContent: result.structuredContent,
      };
    } catch (err) {
      return {
        content: err instanceof Error ? err.message : 'MCP tool execution failed',
        isError: true,
      };
    }
  }, []);

  const getPrompt = useCallback(async (name: string, args?: Record<string, string>): Promise<McpPromptGetResult> => {
    if (!clientRef.current) {
      return { messages: [], isError: true, errorMessage: 'Not connected to MCP server' };
    }
    try {
      const result = await clientRef.current.getPrompt({ name, arguments: args ?? {} });
      return {
        description: result.description,
        messages: (result.messages || []) as McpPromptMessage[],
      };
    } catch (err) {
      return {
        messages: [],
        isError: true,
        errorMessage: err instanceof Error ? err.message : 'Failed to get prompt',
      };
    }
  }, []);

  const readResource = useCallback(async (uri: string): Promise<McpResourceReadResult> => {
    if (!clientRef.current) {
      return { contents: [], isError: true, errorMessage: 'Not connected to MCP server' };
    }
    try {
      const result = await clientRef.current.readResource({ uri });
      return { contents: (result.contents || []) as McpResourceContent[] };
    } catch (err) {
      return {
        contents: [],
        isError: true,
        errorMessage: err instanceof Error ? err.message : 'Failed to read resource',
      };
    }
  }, []);

  const refresh = useCallback(async () => {
    if (!clientRef.current) return;
    setStatus('refreshing');
    try {
      await listAll(clientRef.current);
      setStatus('connected');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to refresh capabilities');
      setStatus('connected');
    }
  }, [listAll]);

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

  const counts: McpCapabilityCounts = useMemo(
    () => ({
      tools: tools.length,
      prompts: prompts.length,
      resources: resources.length,
      resourceTemplates: resourceTemplates.length,
    }),
    [tools.length, prompts.length, resources.length, resourceTemplates.length]
  );

  return {
    status,
    tools,
    prompts,
    resources,
    resourceTemplates,
    counts,
    error,
    connect,
    disconnect,
    callTool,
    getPrompt,
    readResource,
    refresh,
  };
}
