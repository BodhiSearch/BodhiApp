/**
 * Token Fixtures
 *
 * Provides test data and utilities for API token testing
 */

export class TokenFixtures {
  /**
   * Generate test token names
   */
  static getTestTokenNames() {
    return {
      basic: 'Test API Token',
      admin1: 'Admin Token 1',
      admin2: 'Admin Token 2',
      user: 'User Token',
      chat: 'Chat Integration Token',
    };
  }

  /**
   * Get invalid token formats for error testing
   */
  static getInvalidTokens() {
    return {
      invalidFormat: 'invalid_token',
      nonExistent: 'bodhiapp_nonexistent123',
      empty: '',
      malformed: 'bodhiapp_',
      wrongPrefix: 'wrong_prefix_abc123',
    };
  }

  /**
   * Mock clipboard API for testing copy functionality
   * @param {Page} page - Playwright page object
   */
  static async mockClipboard(page) {
    let clipboardContent = '';

    await page.evaluate(() => {
      window.clipboardData = '';
      Object.defineProperty(navigator, 'clipboard', {
        value: {
          writeText: (text) => {
            window.clipboardData = text;
            return Promise.resolve();
          },
          readText: () => {
            return Promise.resolve(window.clipboardData);
          },
        },
        writable: true,
      });
    });

    return {
      getContent: async () => {
        return await page.evaluate(() => window.clipboardData);
      },
      clear: async () => {
        await page.evaluate(() => {
          window.clipboardData = '';
        });
      },
    };
  }

  /**
   * Expected error messages
   */
  static getErrorMessages() {
    return {
      missingToken: /token|authorization|authentication/i,
      invalidToken: /invalid|unauthorized|authentication failed/i,
      networkError: /error|failed|network/i,
    };
  }
}
