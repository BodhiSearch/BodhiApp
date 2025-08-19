import { mkdtempSync } from 'fs';
import { tmpdir } from 'os';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

// Get the current directory for ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

/**
 * Load the NAPI bindings using dynamic import
 * @returns {Object} The NAPI bindings object
 */
async function loadBindings() {
  // Dynamic import of the local NAPI bindings during development
  const appBindingsModule = await import('../index.js');
  // CommonJS modules are wrapped in a default export when dynamically imported
  return appBindingsModule.default;
}

/**
 * Generate a random port between 20000 and 30000
 * @returns {number} Random port number
 */
function randomPort() {
  return Math.floor(Math.random() * (30000 - 20000) + 20000);
}

/**
 * Create a temporary directory for testing
 * @returns {string} Path to the temporary directory
 */
function createTempDir() {
  return mkdtempSync(join(tmpdir(), 'bodhi-test-'));
}

/**
 * Create a test server with a temporary directory
 * @param {Object} bindings - The NAPI bindings
 * @param {Object} options - Configuration options
 * @returns {Object} BodhiServer instance
 */
function createTestServer(bindings, options = {}) {
  const config = createFullTestConfig(bindings, options);
  const server = new bindings.BodhiServer(config);
  return server;
}

/**
 * Create a full test configuration with all options using new API
 * @param {Object} bindings - The NAPI bindings
 * @param {Object} options - Configuration options
 * @returns {Object} NapiAppOptions object
 */
function createFullTestConfig(bindings, options = {}) {
  const appHome = createTempDir();
  const {
    host = 'localhost',
    port = randomPort(),
    execLookupPath = join(__dirname, '..', '..', 'llama_server_proc', 'bin'),
    logLevel = 'debug',
    logStdout = true,
    envVars = {},
    authUrl = 'https://main-id.getbodhi.app',
    authRealm = 'bodhi',
    clientId = null,
    clientSecret = null,
    appStatus = 'ready',
  } = options;

  let config = bindings.createNapiAppOptions();

  // Set any additional environment variables
  for (const [key, value] of Object.entries(envVars)) {
    config = bindings.setEnvVar(config, key, value);
  }

  // Set basic environment variables
  config = bindings.setEnvVar(config, 'HOME', appHome);
  config = bindings.setEnvVar(config, bindings.BODHI_HOST, host);
  config = bindings.setEnvVar(config, bindings.BODHI_PORT, port.toString());
  // Set app settings
  config = bindings.setAppSetting(config, bindings.BODHI_EXEC_LOOKUP_PATH, execLookupPath);
  config = bindings.setAppSetting(config, bindings.BODHI_LOG_LEVEL, logLevel);
  config = bindings.setAppSetting(config, bindings.BODHI_LOG_STDOUT, logStdout.toString());

  // Set basic system settings
  config = bindings.setSystemSetting(config, bindings.BODHI_ENV_TYPE, 'development');
  config = bindings.setSystemSetting(config, bindings.BODHI_APP_TYPE, 'container');
  config = bindings.setSystemSetting(config, bindings.BODHI_VERSION, '1.0.0-test');
  config = bindings.setSystemSetting(config, bindings.BODHI_AUTH_URL, authUrl);
  config = bindings.setSystemSetting(config, bindings.BODHI_AUTH_REALM, authRealm);

  if (appStatus) {
    config = bindings.setAppStatus(config, appStatus);
  }

  // Set client credentials if provided
  if (clientId && clientSecret) {
    config = bindings.setClientCredentials(config, clientId, clientSecret);
  }

  return config;
}

/**
 * Wait for a specified amount of time
 * @param {number} ms - Milliseconds to wait
 * @returns {Promise} Promise that resolves after the specified time
 */
function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Wait for a server to be ready by attempting to ping it
 * @param {Object} server - The BodhiServer instance
 * @param {number} maxAttempts - Maximum number of ping attempts
 * @param {number} interval - Interval between attempts in ms
 * @returns {Promise<boolean>} True if server responds, false if timeout
 */
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

/**
 * Wait for SPA to be fully loaded and rendered
 * @param {import('@playwright/test').Page} page - Playwright page object
 */
async function waitForSPAReady(page) {
  await page.waitForLoadState('networkidle');
  await page.waitForLoadState('domcontentloaded');
}

/**
 * Wait for page redirect to expected path
 * @param {import('@playwright/test').Page} page - Playwright page object
 * @param {string} expectedPath - Expected path to redirect to
 */
async function waitForRedirect(page, expectedPath) {
  await page.waitForURL((url) => {
    const pathname = new URL(url).pathname;
    return pathname === expectedPath;
  });
}

/**
 * Get current page path
 * @param {import('@playwright/test').Page} page - Playwright page object
 * @returns {string} Current page path
 */
function getCurrentPath(page) {
  return new URL(page.url()).pathname;
}

export {
  createFullTestConfig, createTempDir,
  createTestServer, getCurrentPath, loadBindings,
  randomPort, sleep, waitForRedirect, waitForServer,
  waitForSPAReady
};

