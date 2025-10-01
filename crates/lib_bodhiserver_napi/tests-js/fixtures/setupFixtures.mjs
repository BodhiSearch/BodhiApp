export class SetupFixtures {
  static createServerConfig(overrides = {}) {
    const timestamp = Date.now();
    return {
      serverName: `Test Bodhi Server ${timestamp}`,
      description: 'A test server for integration testing',
      ...overrides,
    };
  }

  static createSetupFlowData() {
    return {
      serverName: this.createServerConfig().serverName,
      skipDownloads: true, // Skip downloads for faster tests
      completeFlow: true,
    };
  }

  static getServerManagerConfig(authServerConfig, port, host = 'localhost') {
    return {
      appStatus: 'setup',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      host,
      port,
      logLevel: 'debug',
    };
  }

  static getNetworkIPServerConfig(authServerConfig, port, host = '0.0.0.0') {
    return {
      appStatus: 'setup',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      host, // Bind to all interfaces for network IP access
      port,
      logLevel: 'debug',
    };
  }

  // Test scenarios for different setup paths
  static scenarios = {
    FULL_SETUP: () => ({
      ...this.createSetupFlowData(),
      skipDownloads: false,
      downloadModels: ['llama2:7b', 'mistral:7b'],
    }),

    QUICK_SETUP: () => ({
      ...this.createSetupFlowData(),
      skipDownloads: true,
    }),

    SETUP_WITH_CUSTOM_NAME: (name) => ({
      ...this.createSetupFlowData(),
      serverName: name,
    }),

    MINIMAL_SETUP: () => ({
      serverName: `Minimal Test ${Date.now()}`,
      skipDownloads: true,
      completeFlow: true,
    }),

    NETWORK_IP_SETUP: () => ({
      ...this.createSetupFlowData(),
      serverName: `Network IP Test ${Date.now()}`,
      skipDownloads: true,
      useNetworkIP: true,
    }),

    // New scenarios for API Models and Browser Extension
    SETUP_WITH_API_MODELS: () => ({
      ...this.createSetupFlowData(),
      skipDownloads: true,
      apiModels: {
        provider: 'openai',
        apiKey: 'sk-test-key-123',
        models: ['gpt-4'],
      },
      skipBrowserExtension: true,
    }),

    SETUP_WITH_BROWSER_EXTENSION: () => ({
      ...this.createSetupFlowData(),
      skipDownloads: true,
      skipApiModels: true,
      browserExtension: {
        browser: 'chrome',
        extensionInstalled: true,
      },
    }),

    COMPLETE_SETUP_WITH_ALL_FEATURES: () => ({
      ...this.createSetupFlowData(),
      skipDownloads: false,
      downloadModels: ['llama2:7b'],
      apiModels: {
        provider: 'openai',
        apiKey: 'sk-test-key-123',
        models: ['gpt-4'],
      },
      browserExtension: {
        browser: 'chrome',
        extensionInstalled: false, // Will skip extension
      },
    }),

    SKIP_ALL_OPTIONAL_STEPS: () => ({
      ...this.createSetupFlowData(),
      skipDownloads: true,
      skipApiModels: true,
      skipBrowserExtension: true,
    }),
  };

  // Validation helpers
  static validateSetupConfig(config) {
    const required = ['serverName'];
    const missing = required.filter((field) => !config[field]);

    if (missing.length > 0) {
      throw new Error(`Missing required setup fields: ${missing.join(', ')}`);
    }

    if (config.serverName.length < 3) {
      throw new Error('Server name must be at least 3 characters long');
    }

    return true;
  }

  // Environment helpers
  static getTestEnvironment() {
    return {
      hasAuthServer: !!process.env.INTEG_TEST_AUTH_URL,
      isCI: !!process.env.CI,
      testMode: process.env.NODE_ENV || 'test',
      skipDownloads: process.env.SKIP_MODEL_DOWNLOADS === 'true',
    };
  }

  static checkRequiredEnvironment() {
    const env = this.getTestEnvironment();

    if (!env.hasAuthServer && !process.env.INTEG_TEST_AUTH_URL) {
      console.warn('Auth server not configured for setup tests');
    }

    return env;
  }

  // Setup flow validation
  static getExpectedSetupSteps() {
    return [
      {
        step: 1,
        path: '/ui/setup/',
        title: 'Welcome to Bodhi App',
        description: 'Initial server setup',
      },
      {
        step: 2,
        path: '/ui/setup/resource-admin/',
        title: 'Admin Setup',
        description: 'Authentication configuration',
      },
      {
        step: 3,
        path: '/ui/setup/download-models/',
        title: 'Chat Models',
        description: 'Model download selection',
      },
      {
        step: 4,
        path: '/ui/setup/api-models/',
        title: 'API Models Setup',
        description: 'Cloud AI models configuration',
      },
      {
        step: 5,
        path: '/ui/setup/browser-extension/',
        title: 'Browser Extension Setup',
        description: 'Browser extension installation',
      },
      {
        step: 6,
        path: '/ui/setup/complete/',
        title: 'Setup Complete',
        description: 'Setup completion',
      },
    ];
  }

  static getStepByNumber(stepNumber) {
    const steps = this.getExpectedSetupSteps();
    return steps.find((step) => step.step === stepNumber);
  }

  static getStepByPath(path) {
    const steps = this.getExpectedSetupSteps();
    return steps.find((step) => step.path === path);
  }

  // Test data cleanup
  static generateUniqueServerName(prefix = 'Test Server') {
    return `${prefix} ${Date.now()} ${Math.random().toString(36).substring(2, 7)}`;
  }

  static createTemporarySetup() {
    return this.scenarios.MINIMAL_SETUP();
  }

  // Model download fixtures
  static getRecommendedModels() {
    return [
      {
        name: 'Llama 2 7B',
        id: 'llama2:7b',
        size: '3.8GB',
        recommended: true,
      },
      {
        name: 'Mistral 7B',
        id: 'mistral:7b',
        size: '4.1GB',
        recommended: true,
      },
      {
        name: 'Gemma 7B',
        id: 'gemma:7b',
        size: '4.8GB',
        recommended: false,
      },
    ];
  }

  static getTestModel() {
    return this.getRecommendedModels()[0]; // Return first recommended model
  }

  // API Models test data
  static getApiProviders() {
    return [
      {
        name: 'OpenAI',
        id: 'openai',
        baseUrl: 'https://api.openai.com/v1',
        models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview'],
        testApiKey: 'sk-test-key-123',
      },
      {
        name: 'Anthropic',
        id: 'anthropic',
        baseUrl: 'https://api.anthropic.com',
        models: ['claude-3-opus', 'claude-3-sonnet', 'claude-3-haiku'],
        testApiKey: 'sk-ant-test-key-123',
      },
      {
        name: 'Google',
        id: 'google',
        baseUrl: 'https://generativelanguage.googleapis.com/v1beta',
        models: ['gemini-pro', 'gemini-pro-vision'],
        testApiKey: 'AIza-test-key-123',
      },
    ];
  }

  static getTestApiProvider(providerId = 'openai') {
    return this.getApiProviders().find((p) => p.id === providerId);
  }

  // Browser Extension test data
  static getBrowserTypes() {
    return [
      {
        name: 'Google Chrome',
        id: 'chrome',
        supported: true,
        extensionUrl: 'https://chrome.google.com/webstore/detail/bodhi-browser/[EXTENSION_ID]',
        statusMessage: 'Extension available in Chrome Web Store',
      },
      {
        name: 'Microsoft Edge',
        id: 'edge',
        supported: true,
        extensionUrl: 'https://chrome.google.com/webstore/detail/bodhi-browser/[EXTENSION_ID]',
        statusMessage: 'Extension available in Chrome Web Store (Edge uses Chrome extensions)',
      },
      {
        name: 'Mozilla Firefox',
        id: 'firefox',
        supported: false,
        extensionUrl: null,
        statusMessage: 'Firefox extension coming soon',
      },
      {
        name: 'Safari',
        id: 'safari',
        supported: false,
        extensionUrl: null,
        statusMessage: 'Safari extension coming soon',
      },
    ];
  }

  static getTestBrowser(browserId = 'chrome') {
    return this.getBrowserTypes().find((b) => b.id === browserId);
  }

  static getSupportedBrowsers() {
    return this.getBrowserTypes().filter((b) => b.supported);
  }

  static getUnsupportedBrowsers() {
    return this.getBrowserTypes().filter((b) => !b.supported);
  }

  // Extension detection states
  static getExtensionStates() {
    return {
      DETECTING: 'detecting',
      INSTALLED: 'installed',
      NOT_INSTALLED: 'not-installed',
    };
  }

  static createExtensionTestData(state = 'not-installed', extensionId = null) {
    return {
      status: state,
      extensionId: extensionId,
      refresh: () => {}, // Mock function
      redetect: () => {}, // Mock function
    };
  }
}
