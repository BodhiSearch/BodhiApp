import { expect } from '@playwright/test';
import { createTestServer, loadBindings, waitForServer } from '@/test-helpers.mjs';

/**
 * Server manager for Playwright tests
 * Handles server lifecycle and provides URL for browser tests
 */
export class BodhiAppServer {
  constructor(serverConfig = {}) {
    this.server = null;
    this.bindings = null;
    this.baseUrl = null;
    this.serverConfig = serverConfig;
  }

  /**
   * Initialize the server manager with NAPI bindings
   */
  async initialize() {
    this.bindings = await loadBindings();
  }

  /**
   * Start a server for testing using constructor configuration
   * @returns {Promise<string>} The server URL
   */
  async startServer() {
    await this.initialize();
    this.server = createTestServer(this.bindings, this.serverConfig);

    await this.server.start();
    const isRunning = await waitForServer(this.server, 30, 1000);

    // Server must be running - fail if not
    expect(isRunning).toBe(true);

    this.baseUrl = this.server.serverUrl();
    return this.baseUrl;
  }

  /**
   * Stop the server
   */
  async stopServer() {
    if (this.server) {
      const isRunning = await this.server.isRunning();
      if (isRunning) {
        await this.server.stop();
      }
      this.server = null;
    }
    this.baseUrl = null;
  }

  /**
   * Get the server URL
   * @returns {string} The server URL
   */
  getBaseUrl() {
    return this.baseUrl;
  }
}

/**
 * Create a server manager for Playwright tests
 * @param {Object} serverConfig - Server configuration options
 * @returns {BodhiAppServer} New server manager instance
 */
export function createServerManager(serverConfig = {}) {
  return new BodhiAppServer(serverConfig);
}
