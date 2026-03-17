import express from 'express';
import { createMcpHandlers, cleanupSessions } from './mcp-server.js';

interface AuthParam {
  key: string;
  value: string;
}

function parseArgs(): { headers: AuthParam[]; queries: AuthParam[]; port: number } {
  const headers: AuthParam[] = [];
  const queries: AuthParam[] = [];
  let port = 55176;

  const args = process.argv.slice(2);
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--header' && i + 1 < args.length) {
      const param = args[++i];
      const eqIndex = param.indexOf('=');
      if (eqIndex > 0) {
        headers.push({
          key: param.substring(0, eqIndex),
          value: param.substring(eqIndex + 1),
        });
      }
    } else if (arg === '--query' && i + 1 < args.length) {
      const param = args[++i];
      const eqIndex = param.indexOf('=');
      if (eqIndex > 0) {
        queries.push({
          key: param.substring(0, eqIndex),
          value: param.substring(eqIndex + 1),
        });
      }
    } else if (arg === '--port' && i + 1 < args.length) {
      port = parseInt(args[++i], 10);
    }
  }

  return { headers, queries, port };
}

const config = parseArgs();

const app = express();

// CORS middleware
app.use((_req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  res.header(
    'Access-Control-Allow-Headers',
    'Origin, X-Requested-With, Content-Type, Accept, Authorization, Mcp-Session-Id, X-Auth-1, X-Auth-2'
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

// Health check endpoint (no auth required)
app.get('/ping', (_req, res) => {
  res.send('pong');
});

// MCP endpoints with auth validation
const { handleMcpPost, handleMcpGet, handleMcpDelete } = createMcpHandlers({
  headers: config.headers,
  queries: config.queries,
});

app.post('/mcp', handleMcpPost);
app.get('/mcp', handleMcpGet);
app.delete('/mcp', handleMcpDelete);

const server = app.listen(config.port, () => {
  console.log(`Test MCP Auth server listening on port ${config.port}`);
  console.log(`  MCP endpoint: http://localhost:${config.port}/mcp`);
  console.log(`  Health check: http://localhost:${config.port}/ping`);
  if (config.headers.length > 0) {
    console.log(
      `  Required headers: ${config.headers.map((h) => `${h.key}=${h.value}`).join(', ')}`
    );
  }
  if (config.queries.length > 0) {
    console.log(
      `  Required query params: ${config.queries.map((q) => `${q.key}=${q.value}`).join(', ')}`
    );
  }
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
