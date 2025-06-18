import { join, dirname } from 'path';
import { mkdtempSync } from 'fs';
import { tmpdir } from 'os';
import { fileURLToPath } from 'url';
import { expect } from '@playwright/test';

// Get the current directory for ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Get the path to the NAPI bindings
const napiBindingsPath = join(__dirname, '..', 'index.js');

/**
 * Load the NAPI bindings
 * @returns {Object} The NAPI bindings object
 */
async function loadBindings() {
  const bindings = await import(napiBindingsPath);
  return bindings.default || bindings;
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
 * Create a basic test configuration using new API
 * @param {Object} bindings - The NAPI bindings
 * @param {Object} options - Configuration options
 * @returns {Object} NapiAppOptions object
 */
function createTestConfig(bindings, options = {}) {
  const {
    bodhiHome = createTempDir(),
    host = '127.0.0.1',
    port = randomPort(),
    execLookupPath = join(__dirname, '..', '..', 'llama_server_proc', 'bin'),
  } = options;

  let config = bindings.createNapiAppOptions();

  // Set basic environment variables
  config = bindings.setEnvVar(config, bindings.BODHI_HOME, bodhiHome);
  config = bindings.setEnvVar(config, bindings.BODHI_HOST, host);
  config = bindings.setEnvVar(config, bindings.BODHI_PORT, port.toString());
  config = bindings.setEnvVar(config, bindings.BODHI_EXEC_LOOKUP_PATH, execLookupPath);

  // Set basic system settings
  config = bindings.setSystemSetting(config, bindings.BODHI_ENV_TYPE, 'development');
  config = bindings.setSystemSetting(config, bindings.BODHI_APP_TYPE, 'container');
  config = bindings.setSystemSetting(config, bindings.BODHI_VERSION, '1.0.0-test');
  config = bindings.setSystemSetting(config, bindings.BODHI_AUTH_URL, 'http://localhost:8080');
  config = bindings.setSystemSetting(config, bindings.BODHI_AUTH_REALM, 'bodhi');

  return config;
}

/**
 * Create a test server with a temporary directory
 * @param {Object} bindings - The NAPI bindings
 * @param {Object} options - Configuration options
 * @returns {Object} BodhiServer instance
 */
function createTestServer(bindings, options = {}) {
  const { host = '127.0.0.1', port = randomPort(), appStatus = null } = options;

  let config = createFullTestConfig(bindings, { host, port });

  if (appStatus) {
    config = bindings.setAppStatus(config, appStatus);
    expect(config.appStatus).toBe(appStatus);
  }

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
  const {
    bodhiHome = createTempDir(),
    host = '127.0.0.1',
    port = randomPort(),
    execLookupPath = join(__dirname, '..', '..', 'llama_server_proc', 'bin'),
    logLevel = 'info',
    logStdout = true,
    envVars = {},
  } = options;

  let config = bindings.createNapiAppOptions();

  // Set basic environment variables
  config = bindings.setEnvVar(config, bindings.BODHI_HOME, bodhiHome);
  config = bindings.setEnvVar(config, bindings.BODHI_HOST, host);
  config = bindings.setEnvVar(config, bindings.BODHI_PORT, port.toString());
  config = bindings.setEnvVar(config, bindings.BODHI_EXEC_LOOKUP_PATH, execLookupPath);
  config = bindings.setEnvVar(config, bindings.BODHI_LOG_LEVEL, logLevel);
  config = bindings.setEnvVar(config, bindings.BODHI_LOG_STDOUT, logStdout.toString());

  // Set any additional environment variables
  for (const [key, value] of Object.entries(envVars)) {
    config = bindings.setEnvVar(config, key, value);
  }

  // Set basic system settings
  config = bindings.setSystemSetting(config, bindings.BODHI_ENV_TYPE, 'development');
  config = bindings.setSystemSetting(config, bindings.BODHI_APP_TYPE, 'container');
  config = bindings.setSystemSetting(config, bindings.BODHI_VERSION, '1.0.0-test');
  config = bindings.setSystemSetting(config, bindings.BODHI_AUTH_URL, 'http://localhost:8080');
  config = bindings.setSystemSetting(config, bindings.BODHI_AUTH_REALM, 'bodhi');

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

export {
  loadBindings,
  randomPort,
  createTempDir,
  createTestConfig,
  createTestServer,
  createFullTestConfig,
  sleep,
  waitForServer,
};
