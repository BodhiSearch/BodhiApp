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
        title: 'Recommended Models',
        description: 'Model download selection',
      },
      {
        step: 4,
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
}
