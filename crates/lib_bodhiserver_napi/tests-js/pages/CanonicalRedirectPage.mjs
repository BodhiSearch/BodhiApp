import { BasePage } from './BasePage.mjs';

export class CanonicalRedirectPage extends BasePage {
  constructor(page, port) {
    super(page, `http://localhost:${port}`);
    this.port = port;
  }

  /**
   * Navigate using a specific host (127.0.0.1 or localhost)
   */
  async navigateWithHost(host, path) {
    const url = `http://${host}:${this.port}${path}`;
    await this.page.goto(url);
  }

  /**
   * Wait for redirect to occur with timeout
   */
  async waitForRedirectTo(expectedHost) {
    await this.page.waitForURL((url) => url.origin === `http://${expectedHost}:${this.port}`);
  }

  /**
   * Get the current URL
   */
  getCurrentUrl() {
    return this.page.url();
  }
}
