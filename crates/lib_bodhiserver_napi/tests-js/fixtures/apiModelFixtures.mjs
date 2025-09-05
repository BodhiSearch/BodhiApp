export class ApiModelFixtures {
  static createModelData(overrides = {}) {
    const timestamp = Date.now();
    return {
      modelId: `test-model-${timestamp}`,
      provider: 'OpenAI',
      baseUrl: 'https://api.openai.com/v1',
      models: ['gpt-4', 'gpt-3.5-turbo'],
      ...overrides,
    };
  }

  static createTestSuite(count = 3) {
    return Array.from({ length: count }, (_, i) =>
      this.createModelData({ modelId: `test-model-${Date.now()}-${i}` })
    );
  }

  static getRequiredEnvVars() {
    const apiKey = process.env.INTEG_TEST_OPENAI_API_KEY;
    if (!apiKey) {
      throw new Error('INTEG_TEST_OPENAI_API_KEY environment variable not set');
    }
    return { apiKey };
  }

  // Predefined test scenarios
  static createLifecycleTestData() {
    return this.createModelData({
      modelId: 'lifecycle-test-openai',
      models: ['gpt-4', 'gpt-3.5-turbo'],
    });
  }

  static createMobileTestData() {
    return this.createModelData({
      modelId: 'mobile-test-openai',
      models: ['gpt-4', 'gpt-3.5-turbo'],
    });
  }

  static createTabletTestData() {
    return this.createModelData({
      modelId: 'tablet-test-openai',
      models: ['gpt-4', 'gpt-3.5-turbo'],
    });
  }

  static createEditTestData() {
    return this.createModelData({
      modelId: 'edit-test-openai',
      models: ['gpt-4'],
    });
  }

  static createCustomProviderData(baseUrl, models) {
    return this.createModelData({
      modelId: `custom-provider-${Date.now()}`,
      provider: 'Custom',
      baseUrl,
      models,
    });
  }

  // Test data validation
  static validateModelData(data) {
    const required = ['modelId', 'provider', 'baseUrl', 'models'];
    const missing = required.filter((field) => !data[field]);

    if (missing.length > 0) {
      throw new Error(`Missing required fields: ${missing.join(', ')}`);
    }

    if (!Array.isArray(data.models) || data.models.length === 0) {
      throw new Error('models must be a non-empty array');
    }

    if (!data.baseUrl.startsWith('http')) {
      throw new Error('baseUrl must be a valid HTTP URL');
    }

    return true;
  }

  // Cleanup utilities
  static generateUniqueId(prefix = 'test') {
    return `${prefix}-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`;
  }

  static createTemporaryModel() {
    return this.createModelData({
      modelId: this.generateUniqueId('temp-model'),
    });
  }

  // Common test scenarios
  static scenarios = {
    BASIC_OPENAI: () =>
      this.createModelData({
        modelId: this.generateUniqueId('basic-openai'),
        models: ['gpt-3.5-turbo'],
      }),

    FULL_OPENAI: () =>
      this.createModelData({
        modelId: this.generateUniqueId('full-openai'),
        models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo'],
      }),

    MINIMAL_CONFIG: () =>
      this.createModelData({
        modelId: this.generateUniqueId('minimal'),
        models: ['gpt-3.5-turbo'],
      }),
  };

  // Environment setup helpers
  static checkEnvironment() {
    try {
      this.getRequiredEnvVars();
      return true;
    } catch (error) {
      console.warn(`Environment check failed: ${error.message}`);
      return false;
    }
  }

  static getTestEnvironment() {
    return {
      hasApiKey: !!process.env.INTEG_TEST_OPENAI_API_KEY,
      isCI: !!process.env.CI,
      testMode: process.env.NODE_ENV || 'test',
    };
  }
}
