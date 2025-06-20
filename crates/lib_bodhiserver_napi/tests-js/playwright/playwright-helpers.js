import { expect } from '@playwright/test';
import {
  loadBindings,
  randomPort,
  waitForServer,
  sleep,
  createTestServer,
} from '../test-helpers.js';

/**
 * Server manager for Playwright tests
 * Handles server lifecycle and provides URL for browser tests
 */
export class PlaywrightServerManager {
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
    const isRunning = await this.server.isRunning();
    if (isRunning) {
      await this.server.stop();
    }
    this.server = null;
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
 * @returns {PlaywrightServerManager} New server manager instance
 */
export function createServerManager(serverConfig = {}) {
  return new PlaywrightServerManager(serverConfig);
}

/**
 * Wait for SPA to be fully loaded and rendered
 * @param {import('@playwright/test').Page} page - Playwright page object
 */
export async function waitForSPAReady(page) {
  await page.waitForLoadState('networkidle');
  await page.waitForLoadState('domcontentloaded');
}

/**
 * Wait for page redirect to expected path
 * @param {import('@playwright/test').Page} page - Playwright page object
 * @param {string} expectedPath - Expected path to redirect to
 */
export async function waitForRedirect(page, expectedPath) {
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
export function getCurrentPath(page) {
  return new URL(page.url()).pathname;
}
