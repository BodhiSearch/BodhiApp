import { config } from 'dotenv';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createConnection } from 'node:net';
import { createTestServer, waitForServer } from '../test-helpers.mjs';
import {
  getAuthServerConfig,
  getMultiTenantConfig,
  getPreConfiguredResourceClient,
} from '../utils/auth-server-client.mjs';
import { getDbConfig } from '../utils/db-config.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Parse CLI args
const args = process.argv.slice(2);
function getArg(name) {
  const idx = args.indexOf(`--${name}`);
  return idx >= 0 && idx + 1 < args.length ? args[idx + 1] : null;
}

const port = parseInt(getArg('port') || '51135', 10);
const dbType = getArg('db-type') || 'sqlite';
const deployment = getArg('deployment') || 'standalone';

function checkTcpPort(host, port, label) {
  return new Promise((resolve, reject) => {
    const socket = createConnection({ host, port }, () => {
      socket.destroy();
      console.log(`${label} reachable at ${host}:${port}`);
      resolve();
    });
    socket.setTimeout(5000);
    socket.on('timeout', () => {
      socket.destroy();
      reject(
        new Error(
          `${label} not reachable at ${host}:${port}. ` +
            `Start containers with: docker compose -f docker/docker-compose.test.yml up -d`
        )
      );
    });
    socket.on('error', (err) => {
      reject(
        new Error(
          `${label} not reachable at ${host}:${port} (${err.message}). ` +
            `Start containers with: docker compose -f docker/docker-compose.test.yml up -d`
        )
      );
    });
  });
}

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

  // Verify PostgreSQL containers are reachable before attempting to start
  if (dbType === 'postgres') {
    await checkTcpPort('localhost', 64320, 'PostgreSQL App DB');
    await checkTcpPort('localhost', 54320, 'PostgreSQL Session DB');
  }

  // Configure client credentials and env vars based on deployment mode
  let clientId, clientSecret, createdBy;
  const envVars = {};

  if (deployment === 'multi_tenant') {
    const mtConfig = getMultiTenantConfig();
    // Use tenant credentials for seeding via ensure_tenant
    clientId = mtConfig.tenantId;
    clientSecret = mtConfig.tenantSecret;
    createdBy = process.env.INTEG_TEST_USERNAME_ID;
    // Set multi-tenant settings as env vars (SettingService reads these via get_setting/get_env)
    envVars[bindings.BODHI_DEPLOYMENT] = 'multi_tenant';
    envVars[bindings.BODHI_MULTITENANT_CLIENT_ID] = mtConfig.dashboardClientId;
    envVars[bindings.BODHI_MULTITENANT_CLIENT_SECRET] = mtConfig.dashboardClientSecret;
  } else {
    const resourceClient = getPreConfiguredResourceClient();
    clientId = resourceClient.clientId;
    clientSecret = resourceClient.clientSecret;
    createdBy = process.env.INTEG_TEST_USERNAME_ID;
  }

  console.log(`Creating ${dbType} server (${deployment}) with configuration...`);
  const serverOptions = {
    port,
    host: 'localhost',
    appStatus: 'ready',
    authUrl: authServerConfig.authUrl,
    authRealm: authServerConfig.authRealm,
    clientId,
    clientSecret,
    createdBy,
    envVars,
  };

  // Add DB URLs for postgres (uses db-config.mjs as single source of truth)
  if (dbType === 'postgres') {
    const dbConfig = getDbConfig('multi_tenant');
    serverOptions.envVars = {
      ...serverOptions.envVars,
      [bindings.BODHI_APP_DB_URL]: dbConfig.appDbUrl,
      [bindings.BODHI_SESSION_DB_URL]: dbConfig.sessionDbUrl,
    };
  }

  const server = createTestServer(bindings, serverOptions);

  console.log(`Starting ${dbType} (${deployment}) server on port ${port}...`);
  await server.start();

  console.log('Waiting for server to be ready...');
  const ready = await waitForServer(server, 60, 1000);
  if (!ready) {
    throw new Error('Server failed to become ready within timeout');
  }

  console.log(`Shared ${dbType} (${deployment}) server ready on http://localhost:${port}`);

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
  console.error(`Failed to start shared ${dbType} (${deployment}) server:`, error);
  process.exit(1);
});
