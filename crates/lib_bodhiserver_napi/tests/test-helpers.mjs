import { mkdtempSync } from 'fs';
import { tmpdir, networkInterfaces } from 'os';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function loadBindings() {
  const appBindingsModule = await import('../index.js');
  // CommonJS modules are wrapped in a default export when dynamically imported
  return appBindingsModule.default;
}

function randomPort() {
  return Math.floor(Math.random() * (30000 - 20000) + 20000);
}

function createTempDir() {
  return mkdtempSync(join(tmpdir(), 'bodhi-test-'));
}

function getHfHomePath(explicitPath = null) {
  if (explicitPath) {
    return explicitPath;
  }

  if (process.env.CI === 'true' && process.env.HF_HOME) {
    return process.env.HF_HOME;
  }

  const projectRoot = join(__dirname, '..', '..', '..');
  const defaultPath = join(projectRoot, 'hf-home');
  return defaultPath;
}

function createTestServer(bindings, options = {}) {
  const hfHomePath = getHfHomePath(options.hfHomePath);
  console.log(`Using HF_HOME: ${hfHomePath}`);

  const envVars = {
    ...options.envVars,
    HF_HOME: hfHomePath,
  };

  const config = createFullTestConfig(bindings, {
    ...options,
    envVars,
  });

  const server = new bindings.BodhiServer(config);
  return server;
}

function createFullTestConfig(bindings, options = {}) {
  const appHome = createTempDir();
  const {
    host = 'localhost',
    port = randomPort(),
    execLookupPath = join(__dirname, '..', '..', 'llama_server_proc', 'bin'),
    logLevel = 'info',
    logStdout = true,
    envVars = {},
    authUrl = 'https://main-id.getbodhi.app',
    authRealm = 'bodhi',
    clientId = null,
    clientSecret = null,
    appStatus = 'ready',
    createdBy = null,
  } = options;

  let config = bindings.createNapiAppOptions();

  for (const [key, value] of Object.entries(envVars)) {
    config = bindings.setEnvVar(config, key, value);
  }

  config = bindings.setEnvVar(config, 'HOME', appHome);
  config = bindings.setEnvVar(config, bindings.BODHI_HOST, host);
  config = bindings.setEnvVar(config, bindings.BODHI_PORT, port.toString());
  config = bindings.setAppSetting(config, bindings.BODHI_EXEC_LOOKUP_PATH, execLookupPath);
  config = bindings.setAppSetting(config, bindings.BODHI_LOG_LEVEL, logLevel);
  config = bindings.setAppSetting(config, bindings.BODHI_LOG_STDOUT, logStdout.toString());

  config = bindings.setSystemSetting(config, bindings.BODHI_ENV_TYPE, 'development');
  config = bindings.setSystemSetting(config, bindings.BODHI_APP_TYPE, 'container');
  config = bindings.setSystemSetting(config, bindings.BODHI_VERSION, '1.0.0-test');
  config = bindings.setSystemSetting(config, bindings.BODHI_AUTH_URL, authUrl);
  config = bindings.setSystemSetting(config, bindings.BODHI_AUTH_REALM, authRealm);
  config = bindings.setSystemSetting(config, bindings.BODHI_DEPLOYMENT, 'standalone');

  if (appStatus) {
    config = bindings.setAppStatus(config, appStatus);
  }

  if (clientId && clientSecret) {
    config = bindings.setClientCredentials(config, clientId, clientSecret);
  }

  if (createdBy) {
    config = bindings.setCreatedBy(config, createdBy);
  }

  return config;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForServer(server, maxAttempts = 30, interval = 1000) {
  for (let i = 0; i < maxAttempts; i++) {
    try {
      if (server.isRunning()) {
        await server.ping();
        return true;
      }
    } catch (error) {
      // Server not ready yet, continue waiting
    }
    await sleep(interval);
  }
  return false;
}

async function waitForSPAReady(page) {
  await page.waitForLoadState('networkidle');
  await page.waitForLoadState('domcontentloaded');
}

async function waitForRedirect(page, expectedPath) {
  await page.waitForURL((url) => {
    const pathname = new URL(url).pathname;
    return pathname === expectedPath;
  });
}

function getCurrentPath(page) {
  return new URL(page.url()).pathname;
}

function getLocalNetworkIP() {
  const interfaces = networkInterfaces();

  for (const name in interfaces) {
    for (const iface of interfaces[name]) {
      if (!iface.internal && iface.family === 'IPv4') {
        return iface.address;
      }
    }
  }
  return null;
}

export {
  createFullTestConfig,
  createTempDir,
  createTestServer,
  getCurrentPath,
  getHfHomePath,
  getLocalNetworkIP,
  loadBindings,
  randomPort,
  sleep,
  waitForRedirect,
  waitForServer,
  waitForSPAReady,
};
