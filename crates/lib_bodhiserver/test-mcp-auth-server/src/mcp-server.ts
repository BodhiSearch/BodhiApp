import { randomUUID } from 'node:crypto';
import type { Request, Response } from 'express';
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StreamableHTTPServerTransport } from '@modelcontextprotocol/sdk/server/streamableHttp.js';
import { isInitializeRequest } from '@modelcontextprotocol/sdk/types.js';
import { registerTools } from './tools.js';

const SERVER_NAME = 'test-mcp-auth-server';
const SERVER_VERSION = '1.0.0';

interface AuthParam {
  key: string;
  value: string;
}

interface AuthConfig {
  headers: AuthParam[];
  queries: AuthParam[];
}

interface SessionEntry {
  transport: StreamableHTTPServerTransport;
  server: McpServer;
}

const sessions = new Map<string, SessionEntry>();

let lastRequest: Request | null = null;

function getLastRequest(): Request | null {
  return lastRequest;
}

function createMcpServer(): McpServer {
  const server = new McpServer({
    name: SERVER_NAME,
    version: SERVER_VERSION,
  });

  registerTools(server, getLastRequest);

  return server;
}

function validateAuth(req: Request, authConfig: AuthConfig): boolean {
  for (const { key, value } of authConfig.headers) {
    const headerValue = req.headers[key.toLowerCase()];
    if (headerValue !== value) {
      return false;
    }
  }

  for (const { key, value } of authConfig.queries) {
    if (req.query[key] !== value) {
      return false;
    }
  }

  return true;
}

export function createMcpHandlers(authConfig: AuthConfig) {
  async function handleMcpPost(req: Request, res: Response): Promise<void> {
    if (!validateAuth(req, authConfig)) {
      res.status(401).json({ error: 'unauthorized', message: 'Invalid auth parameters' });
      return;
    }

    lastRequest = req;

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

  async function handleMcpGet(req: Request, res: Response): Promise<void> {
    if (!validateAuth(req, authConfig)) {
      res.status(401).json({ error: 'unauthorized', message: 'Invalid auth parameters' });
      return;
    }

    lastRequest = req;

    const sessionId = req.headers['mcp-session-id'] as string | undefined;
    if (!sessionId || !sessions.has(sessionId)) {
      res.status(400).send('Invalid or missing session ID');
      return;
    }

    const entry = sessions.get(sessionId)!;
    await entry.transport.handleRequest(req, res);
  }

  async function handleMcpDelete(req: Request, res: Response): Promise<void> {
    if (!validateAuth(req, authConfig)) {
      res.status(401).json({ error: 'unauthorized', message: 'Invalid auth parameters' });
      return;
    }

    lastRequest = req;

    const sessionId = req.headers['mcp-session-id'] as string | undefined;
    if (!sessionId || !sessions.has(sessionId)) {
      res.status(400).send('Invalid or missing session ID');
      return;
    }

    const entry = sessions.get(sessionId)!;
    await entry.transport.handleRequest(req, res);
  }

  return { handleMcpPost, handleMcpGet, handleMcpDelete };
}

export function cleanupSessions(): void {
  for (const [sid, entry] of sessions) {
    entry.transport.close?.();
    sessions.delete(sid);
  }
}
