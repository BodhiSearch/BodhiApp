import {
  loadBindings,
  randomPort,
  waitForServer,
  sleep,
  createTestServer,
} from './test-helpers.js';

/**
 * Server manager for Playwright tests
 * Handles server lifecycle and provides URL for browser tests
 */
export class PlaywrightServerManager {
  constructor() {
    this.server = null;
    this.bindings = null;
    this.baseUrl = null;
  }

  /**
   * Initialize the server manager with NAPI bindings
   */
  async initialize() {
    this.bindings = await loadBindings();
  }

  /**
   * Check if server is responding to HTTP requests
   * @param {string} url - URL to check
   * @param {number} maxAttempts - Maximum number of attempts
   * @returns {Promise<boolean>} True if server responds
   */
  async waitForHttpResponse(url, maxAttempts = 30) {
    for (let i = 0; i < maxAttempts; i++) {
      try {
        const response = await fetch(url, {
          method: 'GET',
          signal: AbortSignal.timeout(5000), // 5 second timeout
        });

        // Accept any response (even errors) as long as the server is responding
        console.log(`Server health check: ${response.status} ${response.statusText}`);
        return true;
      } catch (error) {
        console.log(`Server health check attempt ${i + 1}/${maxAttempts}: ${error.message}`);
        await sleep(1000);
      }
    }
    return false;
  }

  /**
   * Start a server for testing
   * @param {Object} options - Server configuration options
   * @returns {Promise<string>} The server URL
   */
  async startServer(options = {}) {
    if (!this.bindings) {
      await this.initialize();
    }

    // Create server with temp directory and random port
    const host = options.host || '127.0.0.1';
    const port = options.port || randomPort();

    this.server = createTestServer(this.bindings, { host, port });

    try {
      console.log(`Starting server on ${host}:${port}...`);

      // Start the server
      await this.server.start();

      // Wait for server to report as running
      const isRunning = await waitForServer(this.server, 30, 1000);

      if (!isRunning) {
        throw new Error('Server did not start within timeout');
      }

      this.baseUrl = this.server.serverUrl();
      console.log(`Server started on ${this.baseUrl}`);

      // Wait for HTTP responses to be available
      const isResponding = await this.waitForHttpResponse(this.baseUrl, 15);

      if (!isResponding) {
        console.warn(
          'Server started but not responding to HTTP requests - this may be expected in dev environment'
        );
        // Don't throw error, let the test handle it
      }

      return this.baseUrl;
    } catch (error) {
      // If server startup fails, it's likely due to missing llama-server binary
      // This is expected in development environments
      throw new Error(`Server startup failed: ${error.message}`);
    }
  }

  /**
   * Stop the server
   */
  async stopServer() {
    if (this.server) {
      try {
        if (await this.server.isRunning()) {
          console.log('Stopping server...');
          await this.server.stop();
          console.log('Server stopped');
        }
      } catch (error) {
        console.warn('Failed to stop server:', error.message);
      }
      this.server = null;
      this.baseUrl = null;
    }
  }

  /**
   * Get the server URL
   * @returns {string|null} The server URL or null if not started
   */
  getBaseUrl() {
    return this.baseUrl;
  }

  /**
   * Check if server is running
   * @returns {Promise<boolean>} True if server is running
   */
  async isRunning() {
    if (!this.server) return false;
    try {
      return await this.server.isRunning();
    } catch {
      return false;
    }
  }

  /**
   * Get server configuration
   * @returns {Object|null} Server configuration or null if not started
   */
  getConfig() {
    return this.server ? this.server.config : null;
  }
}

/**
 * Create a server manager for Playwright tests
 * @returns {PlaywrightServerManager} New server manager instance
 */
function createServerManager() {
  return new PlaywrightServerManager();
}

/**
 * Wait for a page to load and check for specific elements
 * @param {import('@playwright/test').Page} page - Playwright page object
 * @param {string} url - URL to navigate to
 * @param {Object} options - Wait options
 * @returns {Promise<void>}
 */
async function waitForPageLoad(page, url, options = {}) {
  const { timeout = 30000, waitUntil = 'domcontentloaded' } = options;

  try {
    await page.goto(url, {
      waitUntil,
      timeout,
    });

    // Wait for the page to be fully loaded
    await page.waitForLoadState('domcontentloaded');
  } catch (error) {
    console.log(`Page load failed for ${url}: ${error.message}`);
    throw error;
  }
}

/**
 * Check if the current page URL matches expected pattern
 * @param {import('@playwright/test').Page} page - Playwright page object
 * @param {string|RegExp} expectedPath - Expected path or pattern
 * @returns {Promise<boolean>} True if URL matches
 */
async function checkCurrentPath(page, expectedPath) {
  const currentUrl = page.url();
  const url = new URL(currentUrl);

  if (typeof expectedPath === 'string') {
    return url.pathname === expectedPath;
  } else if (expectedPath instanceof RegExp) {
    return expectedPath.test(url.pathname);
  }

  return false;
}

/**
 * Wait for a redirect to occur
 * @param {import('@playwright/test').Page} page - Playwright page object
 * @param {string|RegExp} expectedPath - Expected redirect path
 * @param {number} timeout - Timeout in milliseconds
 * @returns {Promise<string>} The final URL after redirect
 */
async function waitForRedirect(page, expectedPath, timeout = 10000) {
  const startTime = Date.now();

  while (Date.now() - startTime < timeout) {
    if (await checkCurrentPath(page, expectedPath)) {
      return page.url();
    }
    await sleep(100);
  }

  throw new Error(
    `Redirect to ${expectedPath} did not occur within ${timeout}ms. Current URL: ${page.url()}`
  );
}

export { createServerManager, waitForPageLoad, checkCurrentPath, waitForRedirect };
