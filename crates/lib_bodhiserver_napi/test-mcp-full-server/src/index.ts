import { randomUUID } from 'node:crypto';
import express from 'express';
import type { Request, Response } from 'express';
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StreamableHTTPServerTransport } from '@modelcontextprotocol/sdk/server/streamableHttp.js';
import { isInitializeRequest } from '@modelcontextprotocol/sdk/types.js';
import { z } from 'zod';

const SERVER_NAME = 'test-mcp-full-server';
const SERVER_VERSION = '1.0.0';
const DEFAULT_PORT = 55179;

interface SessionEntry {
  transport: StreamableHTTPServerTransport;
  server: McpServer;
}

const sessions = new Map<string, SessionEntry>();

function parsePort(): number {
  const args = process.argv.slice(2);
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--port' && i + 1 < args.length) {
      return parseInt(args[i + 1], 10);
    }
  }
  return DEFAULT_PORT;
}

function createMcpServer(): McpServer {
  const server = new McpServer({
    name: SERVER_NAME,
    version: SERVER_VERSION,
  });

  // --- Tools ---

  server.tool('echo', { message: z.string() }, async ({ message }) => ({
    content: [{ type: 'text' as const, text: `echo: ${message}` }],
  }));

  server.tool('weather', { city: z.string() }, async ({ city }) => ({
    content: [
      {
        type: 'text' as const,
        text: JSON.stringify({
          city,
          temperature: '72°F',
          condition: 'Sunny',
          humidity: '45%',
          wind: '10 mph NW',
        }),
      },
    ],
  }));

  // --- Resources ---

  server.resource(
    'Test README',
    'file:///test-readme',
    { description: 'A test README file with text content', mimeType: 'text/plain' },
    async (uri) => ({
      contents: [
        {
          uri: uri.href,
          mimeType: 'text/plain',
          text: '# Test README\n\nThis is a test README file served by the MCP full test server.\n\nIt contains sample content for E2E testing of resource capabilities.',
        },
      ],
    })
  );

  server.resource(
    'Test Config',
    'file:///test-config',
    { description: 'A test configuration file with JSON content', mimeType: 'application/json' },
    async (uri) => ({
      contents: [
        {
          uri: uri.href,
          mimeType: 'application/json',
          text: JSON.stringify(
            {
              name: 'test-app',
              version: '1.0.0',
              settings: {
                debug: true,
                logLevel: 'info',
                maxRetries: 3,
              },
            },
            null,
            2
          ),
        },
      ],
    })
  );

  // --- Prompts ---

  server.prompt(
    'greeting',
    'Generate a greeting message for a given name',
    { name: z.string() },
    async ({ name }) => ({
      messages: [
        {
          role: 'user' as const,
          content: { type: 'text' as const, text: `Hello ${name}!` },
        },
      ],
    })
  );

  server.prompt(
    'summarize',
    'Request a summary of the given text',
    { text: z.string() },
    async ({ text }) => ({
      messages: [
        {
          role: 'user' as const,
          content: {
            type: 'text' as const,
            text: `Please summarize the following text:\n\n${text}`,
          },
        },
      ],
    })
  );

  return server;
}

async function handleMcpPost(req: Request, res: Response): Promise<void> {
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
  const sessionId = req.headers['mcp-session-id'] as string | undefined;
  if (!sessionId || !sessions.has(sessionId)) {
    res.status(400).send('Invalid or missing session ID');
    return;
  }

  const entry = sessions.get(sessionId)!;
  await entry.transport.handleRequest(req, res);
}

async function handleMcpDelete(req: Request, res: Response): Promise<void> {
  const sessionId = req.headers['mcp-session-id'] as string | undefined;
  if (!sessionId || !sessions.has(sessionId)) {
    res.status(400).send('Invalid or missing session ID');
    return;
  }

  const entry = sessions.get(sessionId)!;
  await entry.transport.handleRequest(req, res);
}

function cleanupSessions(): void {
  for (const [sid, entry] of sessions) {
    entry.transport.close?.();
    sessions.delete(sid);
  }
}

const port = parsePort();
const app = express();

// CORS middleware
app.use((_req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  res.header(
    'Access-Control-Allow-Headers',
    'Origin, X-Requested-With, Content-Type, Accept, Authorization, Mcp-Session-Id'
  );
  res.header('Access-Control-Allow-Methods', 'GET, POST, DELETE, OPTIONS');
  res.header('Access-Control-Expose-Headers', 'Mcp-Session-Id');
  if (_req.method === 'OPTIONS') {
    res.sendStatus(204);
    return;
  }
  next();
});

app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Health check endpoint
app.get('/ping', (_req, res) => {
  res.send('pong');
});

// MCP endpoints
app.post('/mcp', handleMcpPost);
app.get('/mcp', handleMcpGet);
app.delete('/mcp', handleMcpDelete);

const server = app.listen(port, () => {
  console.log(`Test MCP Full server listening on port ${port}`);
  console.log(`  MCP endpoint: http://localhost:${port}/mcp`);
  console.log(`  Health check: http://localhost:${port}/ping`);
  console.log(`  Tools: echo, weather`);
  console.log(`  Resources: file:///test-readme, file:///test-config`);
  console.log(`  Prompts: greeting, summarize`);
});

process.on('SIGINT', () => {
  console.log('Shutting down...');
  cleanupSessions();
  server.close();
  process.exit(0);
});

process.on('SIGTERM', () => {
  cleanupSessions();
  server.close();
  process.exit(0);
});
