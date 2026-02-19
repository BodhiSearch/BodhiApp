import express from 'express';
import { createOAuthRouter, DCR_MODE } from './oauth.js';
import { handleMcpPost, handleMcpGet, handleMcpDelete, cleanupSessions } from './mcp-server.js';

const PORT = parseInt(process.env.TEST_MCP_OAUTH_PORT ?? '55174', 10);

const app = express();

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

app.use(createOAuthRouter());

app.post('/mcp', handleMcpPost);
app.get('/mcp', handleMcpGet);
app.delete('/mcp', handleMcpDelete);

const server = app.listen(PORT, () => {
  const mode = DCR_MODE ? 'DCR' : 'pre-registered';
  console.log(`Test MCP OAuth server listening on port ${PORT} (mode: ${mode})`);
  console.log(`  OAuth metadata: http://localhost:${PORT}/.well-known/oauth-authorization-server`);
  console.log(`  MCP endpoint:   http://localhost:${PORT}/mcp`);
  if (DCR_MODE) {
    console.log(`  Register:       http://localhost:${PORT}/register`);
    console.log(`  Protected res:  http://localhost:${PORT}/.well-known/oauth-protected-resource`);
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
