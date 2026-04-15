import { mkdtempSync } from 'node:fs';
import { networkInterfaces, tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

/**
 * Shared static server URL for React OAuth test app (Playwright webServer).
 */
export const SHARED_STATIC_SERVER_URL = 'http://localhost:55173';

/**
 * Absolute path to the prebuilt bodhiserver_dev binary.
 * Built via `make build.dev-server` (see project Makefile).
 */
export const BODHISERVER_DEV_BIN = join(
  __dirname,
  '..',
  '..',
  '..',
  'target',
  'debug',
  'bodhiserver_dev'
);

function randomPort() {
  return Math.floor(Math.random() * (30000 - 20000) + 20000);
}

function createTempDir() {
  return mkdtempSync(join(tmpdir(), 'bodhi-test-'));
}

/**
 * Resolve HF_HOME with fallback logic:
 *   1. explicit hfHomePath, 2. HF_HOME env in CI, 3. project-root/hf-home.
 */
function getHfHomePath(explicitPath = null) {
  if (explicitPath) return explicitPath;
  if (process.env.CI === 'true' && process.env.HF_HOME) return process.env.HF_HOME;
  const projectRoot = join(__dirname, '..', '..', '..');
  return join(projectRoot, 'hf-home');
}

/**
 * Build the env-var map consumed by the bodhiserver_dev binary.
 *
 * Mirrors the previous NapiAppOptions configuration surface 1:1 but as a flat
 * env object that can be passed through `child_process.spawn`. Unknown keys
 * pass through verbatim so callers can still inject ad-hoc INTEG_TEST_* vars.
 */
export function buildEnvFromConfig(options = {}) {
  const {
    host = 'localhost',
    port = randomPort(),
    hfHomePath = null,
    bodhiHome = createTempDir(),
    execLookupPath = join(__dirname, '..', '..', 'llama_server_proc', 'bin'),
    logLevel = 'info',
    logStdout = true,
    envType = 'development',
    appType = 'container',
    appVersion = '1.0.0-test',
    authUrl = 'https://main-id.getbodhi.app',
    authRealm = 'bodhi',
    deployment = 'standalone',
    encryptionKey = 'dummy-key',
    appStatus = 'ready',
    clientId = null,
    clientSecret = null,
    createdBy = null,
    tenantName = null,
    envVars = {},
    systemSettings = {},
  } = options;

  const env = {
    ...envVars,
    BODHI_HOME: bodhiHome,
    HOME: bodhiHome,
    HF_HOME: getHfHomePath(hfHomePath),
    BODHI_HOST: host,
    BODHI_PORT: port.toString(),
    BODHI_LOG_LEVEL: logLevel,
    BODHI_LOG_STDOUT: logStdout.toString(),
    BODHI_EXEC_LOOKUP_PATH: execLookupPath,
    BODHI_ENV_TYPE: envType,
    BODHI_APP_TYPE: appType,
    BODHI_VERSION: appVersion,
    BODHI_AUTH_URL: authUrl,
    BODHI_AUTH_REALM: authRealm,
    BODHI_DEPLOYMENT: deployment,
    BODHI_ENCRYPTION_KEY: encryptionKey,
    BODHI_APP_STATUS: appStatus,
    ...systemSettings,
  };

  if (clientId && clientSecret) {
    env.BODHI_CLIENT_ID = clientId;
    env.BODHI_CLIENT_SECRET = clientSecret;
  }
  if (createdBy) env.BODHI_CREATED_BY = createdBy;
  if (tenantName) env.BODHI_TENANT_NAME = tenantName;

  // Mirror NAPI `server_url()`: prefer BODHI_PUBLIC_{SCHEME,HOST,PORT} when the
  // caller sets them (e.g. bind to 0.0.0.0 but advertise localhost). Tests use
  // baseUrl to compare against browser URLs, so it must match the browser-visible
  // origin — not the bind host.
  const publicHost = env.BODHI_PUBLIC_HOST || host;
  const publicPort = env.BODHI_PUBLIC_PORT ? Number(env.BODHI_PUBLIC_PORT) : Number(port);
  const publicScheme = env.BODHI_PUBLIC_SCHEME || 'http';

  return { env, host, port, bodhiHome, publicHost, publicPort, publicScheme };
}

async function pingUntilReady(baseUrl, maxAttempts = 60, intervalMs = 1000) {
  for (let i = 0; i < maxAttempts; i++) {
    try {
      const res = await fetch(`${baseUrl}/ping`);
      if (res.ok) return true;
    } catch (_) {
      // not ready yet
    }
    await sleep(intervalMs);
  }
  return false;
}

/**
 * Resolve once `bodhiserver_dev` prints its `listening on <url>` line on
 * stdout. Rejects on timeout, early process exit, or spawn error.
 */
function waitForListening(child, timeoutMs = 60000) {
  return new Promise((resolve, reject) => {
    let buffer = '';
    const onData = (chunk) => {
      const text = chunk.toString();
      buffer += text;
      process.stdout.write(text);
      const match = buffer.match(/bodhiserver_dev: listening on (\S+)/);
      if (match) {
        cleanup();
        resolve(match[1]);
      }
    };
    const onExit = (code, signal) => {
      cleanup();
      reject(new Error(`bodhiserver_dev exited before ready (code=${code} signal=${signal})`));
    };
    const onError = (err) => {
      cleanup();
      reject(err);
    };
    const timer = setTimeout(() => {
      cleanup();
      reject(new Error(`bodhiserver_dev did not become ready within ${timeoutMs}ms`));
    }, timeoutMs);
    function cleanup() {
      clearTimeout(timer);
      child.stdout?.off('data', onData);
      child.off('exit', onExit);
      child.off('error', onError);
    }
    child.stdout?.on('data', onData);
    child.on('exit', onExit);
    child.on('error', onError);
  });
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForSPAReady(page) {
  await page.waitForLoadState('networkidle');
  await page.waitForLoadState('domcontentloaded');
}

async function waitForRedirect(page, expectedPath) {
  await page.waitForURL((url) => new URL(url).pathname === expectedPath);
}

function getCurrentPath(page) {
  return new URL(page.url()).pathname;
}

function getLocalNetworkIP() {
  const interfaces = networkInterfaces();
  for (const name in interfaces) {
    for (const iface of interfaces[name]) {
      if (!iface.internal && iface.family === 'IPv4') return iface.address;
    }
  }
  return null;
}

async function resetDatabase(baseUrl) {
  const response = await fetch(`${baseUrl}/dev/db-reset`, { method: 'POST' });
  if (!response.ok) {
    const body = await response.text();
    throw new Error(`db-reset failed: ${response.status} - ${body}`);
  }
  return response.json();
}

export {
  createTempDir,
  getCurrentPath,
  getHfHomePath,
  getLocalNetworkIP,
  pingUntilReady,
  randomPort,
  resetDatabase,
  sleep,
  waitForListening,
  waitForRedirect,
  waitForSPAReady,
};
