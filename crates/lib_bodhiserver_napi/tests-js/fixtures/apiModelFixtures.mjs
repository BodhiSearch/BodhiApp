export class ApiModelFixtures {
  // Single source of truth for live API model names.
  // Update these when models are deprecated.
  static OPENAI_MODEL = 'gpt-4.1-nano';
  static OPENROUTER_MODEL = 'openai/gpt-4.1-nano';

  static createModelData(overrides = {}) {
    return {
      api_format: 'openai',
      baseUrl: 'https://api.openai.com/v1',
      models: [ApiModelFixtures.OPENAI_MODEL],
      prefix: null, // Default no prefix
      ...overrides,
    };
  }

  static createTestSuite(count = 3) {
    return Array.from({ length: count }, (_, i) => ApiModelFixtures.createModelData());
  }

  static getRequiredEnvVars() {
    const apiKey = process.env.INTEG_TEST_OPENAI_API_KEY;
    const openrouterApiKey = process.env.INTEG_TEST_OPENROUTER_API_KEY;
    if (!apiKey) {
      throw new Error('INTEG_TEST_OPENAI_API_KEY environment variable not set');
    }
    if (!openrouterApiKey) {
      throw new Error('INTEG_TEST_OPENROUTER_API_KEY environment variable not set');
    }
    return { apiKey, openrouterApiKey };
  }

  // Predefined test scenarios
  static createLifecycleTestData() {
    return ApiModelFixtures.createModelData();
  }

  static createMobileTestData() {
    return ApiModelFixtures.createModelData();
  }

  static createTabletTestData() {
    return ApiModelFixtures.createModelData();
  }

  static createEditTestData() {
    return ApiModelFixtures.createModelData();
  }

  static createCustomAliasData(baseUrl, models) {
    return ApiModelFixtures.createModelData({
      api_format: 'openai',
      baseUrl,
      models,
    });
  }

  // Test data validation
  static validateModelData(data) {
    const required = ['api_format', 'baseUrl', 'models'];
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
  static createTemporaryModel() {
    return ApiModelFixtures.createModelData();
  }

  // Common test scenarios
  static scenarios = {
    BASIC_OPENAI: () => this.createModelData(),

    FULL_OPENAI: () => this.createModelData(),

    MINIMAL_CONFIG: () => this.createModelData(),

    // Prefix-specific scenarios with separators
    OPENROUTER: () =>
      this.createModelData({
        api_format: 'openai',
        baseUrl: 'https://openrouter.ai/api/v1',
        models: [this.OPENROUTER_MODEL],
        prefix: 'openrouter/',
      }),

    OPENAI_PREFIX: () =>
      this.createModelData({
        api_format: 'openai',
        baseUrl: 'https://api.openai.com/v1',
        prefix: 'openai:',
      }),

    CUSTOM_PREFIX: () =>
      this.createModelData({
        api_format: 'openai',
        baseUrl: 'https://openrouter.ai/api/v1',
        models: [this.OPENROUTER_MODEL],
        prefix: 'custom-',
      }),

    NO_PREFIX: () =>
      this.createModelData({
        api_format: 'openai',
        baseUrl: 'https://api.openai.com/v1',
        prefix: null,
      }),

    EMPTY_PREFIX: () =>
      this.createModelData({
        api_format: 'openai',
        baseUrl: 'https://api.openai.com/v1',
        prefix: '',
      }),

    FORWARD_ALL_OPENAI: () =>
      this.createModelData({
        api_format: 'openai',
        baseUrl: 'https://api.openai.com/v1',
        prefix: 'fwd/',
        forward_all_with_prefix: true,
        models: [], // Empty models list for forward_all mode
      }),
  };

  // Environment setup helpers
  static checkEnvironment() {
    try {
      ApiModelFixtures.getRequiredEnvVars();
      return true;
    } catch (error) {
      console.warn(`Environment check failed: ${error.message}`);
      return false;
    }
  }

  static getTestEnvironment() {
    return {
      hasApiKey: !!process.env.INTEG_TEST_OPENAI_API_KEY,
      hasOpenRouterApiKey: !!process.env.INTEG_TEST_OPENROUTER_API_KEY,
      isCI: !!process.env.CI,
      testMode: process.env.NODE_ENV || 'test',
    };
  }
}
