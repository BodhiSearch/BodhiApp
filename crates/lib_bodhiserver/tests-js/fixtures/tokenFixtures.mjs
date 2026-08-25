export class TokenFixtures {
  static getTestTokenNames() {
    return {
      basic: 'Test API Token',
      admin1: 'Admin Token 1',
      admin2: 'Admin Token 2',
      user: 'User Token',
      chat: 'Chat Integration Token',
      scoped: 'Scoped Grants Token',
    };
  }

  static getInvalidTokens() {
    return {
      invalidFormat: 'invalid_token',
      nonExistent: 'sk-bodhiapp_nonexistent123',
      empty: '',
      malformed: 'sk-bodhiapp_',
      wrongPrefix: 'wrong_prefix_abc123',
    };
  }

  static async mockClipboard(page) {
    const clipboardContent = '';

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

  static getErrorMessages() {
    return {
      missingToken: /token|authorization|authentication/i,
      invalidToken: /invalid|unauthorized|authentication failed/i,
      networkError: /error|failed|network/i,
    };
  }
}
