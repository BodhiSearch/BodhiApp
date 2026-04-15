import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { createConnection } from 'node:net';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { config } from 'dotenv';
import { BODHISERVER_DEV_BIN, buildEnvFromConfig, waitForListening } from '../test-helpers.mjs';
import {
  getAuthServerConfig,
  getMultiTenantConfig,
  getPreConfiguredResourceClient,
} from '../utils/auth-server-client.mjs';
import { getDbConfig } from '../utils/db-config.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const args = process.argv.slice(2);
function getArg(name) {
  const idx = args.indexOf(`--${name}`);
  return idx >= 0 && idx + 1 < args.length ? args[idx + 1] : null;
}

const port = parseInt(getArg('port') || '51135', 10);
const dbType = getArg('db-type') || 'sqlite';
const deployment = getArg('deployment') || 'standalone';

function checkTcpPort(host, tcpPort, label) {
  return new Promise((resolve, reject) => {
    const socket = createConnection({ host, port: tcpPort }, () => {
      socket.destroy();
      console.log(`${label} reachable at ${host}:${tcpPort}`);
      resolve();
    });
    socket.setTimeout(5000);
    socket.on('timeout', () => {
      socket.destroy();
      reject(
        new Error(
          `${label} not reachable at ${host}:${tcpPort}. ` +
            `Start containers with: docker compose -f docker/docker-compose.test.yml up -d`
        )
      );
    });
    socket.on('error', (err) => {
      reject(
        new Error(
          `${label} not reachable at ${host}:${tcpPort} (${err.message}). ` +
            `Start containers with: docker compose -f docker/docker-compose.test.yml up -d`
        )
      );
    });
  });
}

async function main() {
  const envPath = join(__dirname, '..', '.env.test');
  config({ path: envPath });

  if (!existsSync(BODHISERVER_DEV_BIN)) {
    throw new Error(
      `bodhiserver_dev binary not found at ${BODHISERVER_DEV_BIN}. ` +
        'Build it with `make build.dev-server`.'
    );
  }

  const authServerConfig = getAuthServerConfig();

  if (dbType === 'postgres') {
    await checkTcpPort('localhost', 64320, 'PostgreSQL App DB');
    await checkTcpPort('localhost', 54320, 'PostgreSQL Session DB');
  }

  let clientId;
  let clientSecret;
  let createdBy;
  const envVars = {};

  if (deployment === 'multi_tenant') {
    const mtConfig = getMultiTenantConfig();
    clientId = mtConfig.tenantId;
    clientSecret = mtConfig.tenantSecret;
    createdBy = process.env.INTEG_TEST_USERNAME_ID;
    envVars.BODHI_MULTITENANT_CLIENT_ID = mtConfig.dashboardClientId;
    envVars.BODHI_MULTITENANT_CLIENT_SECRET = mtConfig.dashboardClientSecret;
  } else {
    const resourceClient = getPreConfiguredResourceClient();
    clientId = resourceClient.clientId;
    clientSecret = resourceClient.clientSecret;
    createdBy = process.env.INTEG_TEST_USERNAME_ID;
  }

  const tenantName =
    deployment === 'multi_tenant'
      ? `[do-not-delete] Test ${process.env.INTEG_TEST_USERNAME || 'user@email.com'} tenant`
      : null;

  if (dbType === 'postgres') {
    const dbConfig = getDbConfig('multi_tenant');
    envVars.BODHI_APP_DB_URL = dbConfig.appDbUrl;
    envVars.BODHI_SESSION_DB_URL = dbConfig.sessionDbUrl;
  }

  const { env } = buildEnvFromConfig({
    host: 'localhost',
    port,
    appStatus: 'ready',
    authUrl: authServerConfig.authUrl,
    authRealm: authServerConfig.authRealm,
    deployment,
    clientId,
    clientSecret,
    createdBy,
    tenantName,
    envVars,
  });

  console.log(`Starting ${dbType} (${deployment}) server on port ${port}...`);
  const child = spawn(BODHISERVER_DEV_BIN, [], {
    env: { ...process.env, ...env },
    stdio: ['ignore', 'pipe', 'inherit'],
  });

  await waitForListening(child, 60000);
  console.log(`Shared ${dbType} (${deployment}) server ready on http://localhost:${port}`);

  const shutdown = () => {
    console.log('Shutting down server...');
    if (child.exitCode === null) child.kill('SIGTERM');
  };
  process.on('SIGTERM', shutdown);
  process.on('SIGINT', shutdown);

  child.on('exit', (code, signal) => {
    console.log(`bodhiserver_dev exited (code=${code} signal=${signal})`);
    process.exit(code ?? (signal ? 128 : 0));
  });
}

main().catch((error) => {
  console.error(`Failed to start shared ${dbType} (${deployment}) server:`, error);
  process.exit(1);
});
