import { randomUUID } from 'node:crypto';
import type { Request, Response } from 'express';
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StreamableHTTPServerTransport } from '@modelcontextprotocol/sdk/server/streamableHttp.js';
import { isInitializeRequest } from '@modelcontextprotocol/sdk/types.js';
import { z } from 'zod';
import { isValidAccessToken } from './oauth.js';

const SERVER_NAME = 'test-mcp-oauth-server';
const SERVER_VERSION = '1.0.0';

interface SessionEntry {
  transport: StreamableHTTPServerTransport;
  server: McpServer;
}

const sessions = new Map<string, SessionEntry>();

function createMcpServer(): McpServer {
  const server = new McpServer({
    name: SERVER_NAME,
    version: SERVER_VERSION,
  });

  server.tool('echo', { text: z.string() }, async ({ text }) => ({
    content: [{ type: 'text' as const, text: `echo: ${text}` }],
  }));

  server.tool('get_server_info', {}, async () => ({
    content: [
      {
        type: 'text' as const,
        text: JSON.stringify({
          name: SERVER_NAME,
          version: SERVER_VERSION,
          authenticated_user: 'test-user',
        }),
      },
    ],
  }));

  return server;
}

function extractBearerToken(req: Request): string | null {
  const auth = req.headers.authorization;
  if (!auth?.startsWith('Bearer ')) return null;
  return auth.slice(7);
}

export async function handleMcpPost(req: Request, res: Response): Promise<void> {
  const token = extractBearerToken(req);
  if (!token || !isValidAccessToken(token)) {
    res.status(401).json({ error: 'invalid_token' });
    return;
  }

  const sessionId = req.headers['mcp-session-id'] as string | undefined;

  try {
    if (sessionId && sessions.has(sessionId)) {
      const entry = sessions.get(sessionId)!;
      await entry.transport.handleRequest(req, res, req.body);
      return;
    }

    if (!sessionId && isInitializeRequest(req.body)) {
      const transport = new StreamableHTTPServerTransport({
        sessionIdGenerator: () => randomUUID(),
        onsessioninitialized: (sid: string) => {
          sessions.set(sid, { transport, server });
        },
      });

      transport.onclose = () => {
        const sid = transport.sessionId;
        if (sid) sessions.delete(sid);
      };

      const server = createMcpServer();
      await server.connect(transport);
      await transport.handleRequest(req, res, req.body);
      return;
    }

    res.status(400).json({
      jsonrpc: '2.0',
      error: { code: -32000, message: 'Bad Request: No valid session ID' },
      id: null,
    });
  } catch (error) {
    console.error('Error handling MCP request:', error);
    if (!res.headersSent) {
      res.status(500).json({
        jsonrpc: '2.0',
        error: { code: -32603, message: 'Internal server error' },
        id: null,
      });
    }
  }
}

export async function handleMcpGet(req: Request, res: Response): Promise<void> {
  const token = extractBearerToken(req);
  if (!token || !isValidAccessToken(token)) {
    res.status(401).json({ error: 'invalid_token' });
    return;
  }

  const sessionId = req.headers['mcp-session-id'] as string | undefined;
  if (!sessionId || !sessions.has(sessionId)) {
    res.status(400).send('Invalid or missing session ID');
    return;
  }

  const entry = sessions.get(sessionId)!;
  await entry.transport.handleRequest(req, res);
}

export async function handleMcpDelete(req: Request, res: Response): Promise<void> {
  const token = extractBearerToken(req);
  if (!token || !isValidAccessToken(token)) {
    res.status(401).json({ error: 'invalid_token' });
    return;
  }

  const sessionId = req.headers['mcp-session-id'] as string | undefined;
  if (!sessionId || !sessions.has(sessionId)) {
    res.status(400).send('Invalid or missing session ID');
    return;
  }

  const entry = sessions.get(sessionId)!;
  await entry.transport.handleRequest(req, res);
}

export function cleanupSessions(): void {
  for (const [sid, entry] of sessions) {
    entry.transport.close?.();
    sessions.delete(sid);
  }
}
