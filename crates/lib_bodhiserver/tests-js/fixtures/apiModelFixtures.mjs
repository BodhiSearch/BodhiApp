export class ApiModelFixtures {
  // Single source of truth for live API model names.
  // Update these when models are deprecated.
  static OPENAI_MODEL = 'gpt-4.1-nano';
  static OPENROUTER_MODEL = 'openai/gpt-4.1-nano';
  static ANTHROPIC_MODEL = 'claude-haiku-4-5-20251001';
  static GEMINI_MODEL = 'gemini-2.5-flash';
  // Embedding models registered alongside chat models for SDK-compat tests.
  // Gemini's stable embeddings model is `gemini-embedding-001` (appears in
  // /v1beta/models default page). `text-embedding-004` is NOT listed.
  static OPENAI_EMBEDDING_MODEL = 'text-embedding-3-small';
  static GEMINI_EMBEDDING_MODEL = 'gemini-embedding-001';

  // Parameterized API format configs for multi-format E2E testing.
  // Add new formats here to automatically get test coverage.
  // Each entry must define:
  //   primaryEndpoints   – BodhiApp endpoint(s) that speak the format's native protocol
  //   buildPrimaryBody   – builds the request body for a primaryEndpoint call
  //   extractPrimaryResponse – extracts the text reply from a primaryEndpoint response
  static API_FORMATS = {
    openai: {
      format: 'openai',
      formatDisplayName: 'OpenAI - Completions',
      model: 'gpt-4.1-nano',
      baseUrl: 'https://api.openai.com/v1',
      envKey: 'INTEG_TEST_OPENAI_API_KEY',
      chatQuestion: 'What day comes after Monday?',
      chatExpected: 'tuesday',
      chatEndpoint: '/v1/chat/completions',
      mockResponse: 'David Smith is from Chicago',
      // Native endpoint(s) this format is served on
      primaryEndpoints: () => ['/v1/chat/completions'],
      buildPrimaryBody: (model, question) => ({
        model,
        messages: [{ role: 'user', content: question }],
      }),
      extractPrimaryResponse: (data) => data.choices?.[0]?.message?.content ?? '',
      // Prefix used when multiple formats are registered simultaneously to avoid
      // model-name collisions (e.g. both openai and openai_responses use gpt-4.1-nano).
      // The effective model ID becomes `${multiTestPrefix}${model}`.
      multiTestPrefix: 'oai/',
      // BodhiApp routes /v1/chat/completions to OpenAI | Anthropic aliases only.
      supportsUniversalChatCompletions: true,
      // Embedding model to register alongside the chat model for SDK-compat tests.
      embeddingModel: 'text-embedding-3-small',
    },
    openai_responses: {
      format: 'openai_responses',
      formatDisplayName: 'OpenAI - Responses',
      model: 'gpt-4.1-nano',
      baseUrl: 'https://api.openai.com/v1',
      envKey: 'INTEG_TEST_OPENAI_API_KEY',
      chatQuestion: 'What day comes after Monday?',
      chatExpected: 'tuesday',
      chatEndpoint: '/v1/responses',
      mockResponse: 'David Smith is from Chicago',
      primaryEndpoints: () => ['/v1/responses'],
      buildPrimaryBody: (model, question) => ({
        model,
        input: question,
      }),
      extractPrimaryResponse: (data) => {
        // Responses API: output[].content[].text
        if (Array.isArray(data.output)) {
          for (const item of data.output) {
            if (Array.isArray(item.content)) {
              for (const c of item.content) {
                if (c.text) return c.text;
              }
            }
          }
        }
        return '';
      },
      multiTestPrefix: 'res/',
      // BodhiApp rejects openai_responses aliases on /v1/chat/completions (format mismatch).
      // Use /v1/responses instead (the native endpoint for this format).
      supportsUniversalChatCompletions: false,
    },
    anthropic: {
      format: 'anthropic',
      formatDisplayName: 'Anthropic',
      model: ApiModelFixtures.ANTHROPIC_MODEL,
      baseUrl: 'https://api.anthropic.com/v1',
      envKey: 'INTEG_TEST_ANTHROPIC_API_KEY',
      chatQuestion: 'What day comes after Monday?',
      chatExpected: 'tuesday',
      chatEndpoint: '/v1/messages',
      mockResponse: 'David Smith is from Chicago',
      // BodhiApp exposes the Anthropic protocol on two paths:
      //   /v1/messages            – for clients that set base_url to http://bodhi-server/
      //   /anthropic/v1/messages  – for Anthropic SDK clients (base_url = http://bodhi-server/anthropic/)
      primaryEndpoints: () => ['/v1/messages', '/anthropic/v1/messages'],
      buildPrimaryBody: (model, question) => ({
        model,
        max_tokens: 50,
        messages: [{ role: 'user', content: question }],
      }),
      extractPrimaryResponse: (data) => data.content?.[0]?.text ?? '',
      // Claude model name is already unique; no prefix needed for disambiguation.
      multiTestPrefix: '',
      // Anthropic format IS supported by /v1/chat/completions (routes to api.anthropic.com/v1/chat/completions).
      supportsUniversalChatCompletions: true,
    },
    anthropic_oauth: {
      format: 'anthropic_oauth',
      formatDisplayName: 'Anthropic (Claude Code OAuth)',
      model: ApiModelFixtures.ANTHROPIC_MODEL,
      baseUrl: 'https://api.anthropic.com/v1',
      envKey: 'INTEG_TEST_ANTHROPIC_OAUTH_TOKEN',
      chatQuestion: 'What day comes after Monday?',
      chatExpected: 'tuesday',
      chatEndpoint: '/v1/messages',
      mockResponse: 'David Smith is from Chicago',
      primaryEndpoints: () => ['/v1/messages', '/anthropic/v1/messages'],
      buildPrimaryBody: (model, question) => ({
        model,
        max_tokens: 50,
        messages: [{ role: 'user', content: question }],
      }),
      extractPrimaryResponse: (data) => data.content?.[0]?.text ?? '',
      multiTestPrefix: 'oauth/',
      supportsUniversalChatCompletions: true,
      extraHeaders: {
        'anthropic-version': '2023-06-01',
        'anthropic-beta': 'claude-code-20250219,oauth-2025-04-20',
        'user-agent': 'claude-cli/2.1.80 (external, cli)',
      },
      extraBody: {
        max_tokens: 4096,
        system: [
          { type: 'text', text: "You are Claude Code, Anthropic's official CLI for Claude." },
        ],
      },
    },
    gemini: {
      format: 'gemini',
      formatDisplayName: 'Google Gemini',
      model: ApiModelFixtures.GEMINI_MODEL,
      baseUrl: 'https://generativelanguage.googleapis.com/v1beta',
      envKey: 'INTEG_TEST_GEMINI_API_KEY',
      chatQuestion: 'What day comes after Monday?',
      chatExpected: 'tuesday',
      chatEndpoint: `/v1beta/models/${ApiModelFixtures.GEMINI_MODEL}:generateContent`,
      mockResponse: 'David Smith is from Chicago',
      primaryEndpoints: (effectiveModel) => [
        `/v1beta/models/${effectiveModel}:generateContent`,
        `/v1beta/models/${effectiveModel}:streamGenerateContent`,
      ],
      // Streaming endpoints require SSE-aware fetch; map endpoint suffix to fetch strategy.
      streamingEndpoints: (effectiveModel) => [
        `/v1beta/models/${effectiveModel}:streamGenerateContent`,
      ],
      buildPrimaryBody: (model, question) => ({
        contents: [{ role: 'user', parts: [{ text: question }] }],
      }),
      extractPrimaryResponse: (data) => {
        // Streaming: concat text parts across all SSE chunks.
        if (Array.isArray(data.chunks) && data.chunks.length > 0) {
          return data.chunks
            .map((c) => c.candidates?.[0]?.content?.parts?.[0]?.text ?? '')
            .join('');
        }
        return data.candidates?.[0]?.content?.parts?.[0]?.text ?? '';
      },
      multiTestPrefix: 'gmn/',
      supportsUniversalChatCompletions: false,
      // Embedding model to register alongside the chat model for SDK-compat tests.
      embeddingModel: 'gemini-embedding-001',
      // Mock-specific fields for api-models-no-key.spec.mjs
      mockBaseUrlSuffix: '/v1beta',
      mockModel: 'mock-gemini-flash',
      mockSecondaryModel: 'mock-gemini-pro',
    },
  };

  static createModelDataForFormat(formatKey) {
    const config = ApiModelFixtures.API_FORMATS[formatKey];
    if (!config) throw new Error(`Unknown API format: ${formatKey}`);
    return ApiModelFixtures.createModelData({
      api_format: config.format,
      baseUrl: config.baseUrl,
      models: [config.model],
      ...(config.extraHeaders ? { extra_headers: config.extraHeaders } : {}),
      ...(config.extraBody ? { extra_body: config.extraBody } : {}),
    });
  }

  static createModelData(overrides = {}) {
    return {
      api_format: 'openai',
      baseUrl: 'https://api.openai.com/v1',
      models: [ApiModelFixtures.OPENAI_MODEL],
      prefix: null,
      ...overrides,
    };
  }

  static createTestSuite(count = 3) {
    return Array.from({ length: count }, () => ApiModelFixtures.createModelData());
  }

  static getRequiredEnvVars() {
    const apiKey = process.env.INTEG_TEST_OPENAI_API_KEY;
    if (!apiKey) {
      throw new Error('INTEG_TEST_OPENAI_API_KEY environment variable not set');
    }
    return { apiKey };
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

  // Common test scenarios
  static scenarios = {
    BASIC_OPENAI: () => ApiModelFixtures.createModelData(),

    FULL_OPENAI: () => ApiModelFixtures.createModelData(),

    MINIMAL_CONFIG: () => ApiModelFixtures.createModelData(),

    OPENAI_PREFIX: () =>
      ApiModelFixtures.createModelData({
        api_format: 'openai',
        baseUrl: 'https://api.openai.com/v1',
        prefix: 'openai:',
      }),

    CUSTOM_PREFIX: () =>
      ApiModelFixtures.createModelData({
        api_format: 'openai',
        baseUrl: 'https://api.openai.com/v1',
        prefix: 'custom-',
      }),

    NO_PREFIX: () =>
      ApiModelFixtures.createModelData({
        api_format: 'openai',
        baseUrl: 'https://api.openai.com/v1',
        prefix: null,
      }),

    EMPTY_PREFIX: () =>
      ApiModelFixtures.createModelData({
        api_format: 'openai',
        baseUrl: 'https://api.openai.com/v1',
        prefix: '',
      }),

    FORWARD_ALL_OPENAI: () =>
      ApiModelFixtures.createModelData({
        api_format: 'openai',
        baseUrl: 'https://api.openai.com/v1',
        prefix: 'fwd/',
        forward_all_with_prefix: true,
        models: [],
      }),
  };

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
      isCI: !!process.env.CI,
      testMode: process.env.NODE_ENV || 'test',
    };
  }
}
