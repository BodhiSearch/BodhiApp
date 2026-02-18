import { config } from 'dotenv';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createTestServer, waitForServer } from '../test-helpers.mjs';
import {
  getAuthServerConfig,
  getPreConfiguredResourceClient,
} from '../utils/auth-server-client.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function main() {
  // Load .env.test defensively (won't exist in CI)
  const envPath = join(__dirname, '..', '.env.test');
  config({ path: envPath });

  console.log('Loading NAPI bindings...');
  // Load bindings directly from root index.js (bypassing path alias)
  const bindingsPath = join(__dirname, '..', '..', 'index.js');
  const appBindingsModule = await import(bindingsPath);
  const bindings = appBindingsModule.default;

  console.log('Loading configuration from environment...');
  const authServerConfig = getAuthServerConfig();
  const resourceClient = getPreConfiguredResourceClient();

  console.log('Creating server with configuration...');
  const server = createTestServer(bindings, {
    port: 51135,
    host: 'localhost',
    appStatus: 'ready',
    authUrl: authServerConfig.authUrl,
    authRealm: authServerConfig.authRealm,
    clientId: resourceClient.clientId,
    clientSecret: resourceClient.clientSecret,
  });

  console.log('Starting server on port 51135...');
  await server.start();

  console.log('Waiting for server to be ready...');
  const ready = await waitForServer(server, 60, 1000);
  if (!ready) {
    throw new Error('Server failed to become ready within timeout');
  }

  console.log('Shared server ready on http://localhost:51135');

  // Setup signal handlers for graceful shutdown
  const shutdown = async () => {
    console.log('Shutting down server...');
    await server.stop();
    process.exit(0);
  };

  process.on('SIGTERM', shutdown);
  process.on('SIGINT', shutdown);

  // Keep process alive with explicit interval
  // Using setInterval prevents Node from exiting
  setInterval(() => {
    // Empty interval just keeps event loop active
  }, 1000000);
}

main().catch((error) => {
  console.error('Failed to start shared server:', error);
  process.exit(1);
});
