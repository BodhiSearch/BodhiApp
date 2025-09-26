import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { existsSync } from 'fs';
import { chromium } from '@playwright/test';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

/**
 * Browser manager for Chrome extension testing
 * Launches Chromium with the Bodhi browser extension loaded
 */
export class BrowserWithExtension {
  constructor(options = {}) {
    this.options = {
      headless: options.headless ?? (process.env.CI ? true : false),
      timeout: options.timeout ?? 30000,
      extensionPath: options.extensionPath ?? join(__dirname, '..', 'extension', 'bodhi-browser-ext'),
      ...options
    };
    this.context = null;
    this.browser = null;
  }

  /**
   * Launch browser with extension loaded
   */
  async launch() {
    // Validate extension exists
    if (!existsSync(this.options.extensionPath)) {
      throw new Error(`Extension not found at: ${this.options.extensionPath}`);
    }

    if (!existsSync(join(this.options.extensionPath, 'manifest.json'))) {
      throw new Error(`Extension manifest not found: ${this.options.extensionPath}/manifest.json`);
    }

    console.log(`[BrowserWithExtension] Loading extension from: ${this.options.extensionPath}`);

    // Configure Chrome args for extension loading
    const args = [
      `--disable-extensions-except=${this.options.extensionPath}`,
      `--load-extension=${this.options.extensionPath}`,
      '--no-sandbox',
      '--disable-setuid-sandbox',
      '--disable-dev-shm-usage',
      '--disable-web-security',
      '--disable-features=TranslateUI',
      '--disable-backgrounding-occluded-windows',
      '--disable-renderer-backgrounding',
      '--disable-background-timer-throttling',
      '--mute-audio'
    ];

    // Platform-specific args for CI
    if (process.env.CI) {
      args.push(
        '--disable-gpu',
        '--disable-dev-shm-usage',
        '--disable-software-rasterizer'
      );
    }

    const launchOptions = {
      headless: this.options.headless,
      args,
      timeout: this.options.timeout
    };

    try {
      // Use persistent context to maintain extension state
      this.context = await chromium.launchPersistentContext('', launchOptions);
      this.browser = this.context.browser();

      // Set default timeout for all operations
      this.context.setDefaultTimeout(this.options.timeout);

      console.log('[BrowserWithExtension] Browser launched successfully with extension');
      return this.context;
    } catch (error) {
      const errorMsg = error.message;
      console.error(`[BrowserWithExtension] Failed to launch browser: ${errorMsg}`);
      throw new Error(`Browser launch failed: ${errorMsg}`);
    }
  }

  /**
   * Create a new page in the browser context
   */
  async createPage() {
    if (!this.context) {
      throw new Error('[BrowserWithExtension] Browser not launched. Call launch() first.');
    }

    const page = await this.context.newPage();

    // Add error and console logging
    page.on('console', msg => {
      if (!process.env.CI || msg.type() === 'error') {
        console.log(`[BROWSER-${msg.type().toUpperCase()}]:`, msg.text());
      }
    });

    page.on('pageerror', error => {
      console.error('[BROWSER-PAGE-ERROR]:', error.message);
    });

    return page;
  }


  /**
   * Close the browser and clean up resources
   */
  async close() {
    if (this.context) {
      await this.context.close();
      this.context = null;
    }
    if (this.browser) {
      await this.browser.close();
      this.browser = null;
    }
    console.log('[BrowserWithExtension] Browser closed');
  }
}