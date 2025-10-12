import { createMockOpenAIServer } from '@/utils/mock-openai-server.mjs';

export class MockOpenAIFixtures {
  static getMockModels() {
    return ['mock-gpt-4', 'mock-gpt-3.5-turbo'];
  }

  static getMockModelsResponse() {
    return {
      object: 'list',
      data: [
        {
          id: 'mock-gpt-4',
          object: 'model',
          created: 1687882411,
          owned_by: 'mock-openai',
        },
        {
          id: 'mock-gpt-3.5-turbo',
          object: 'model',
          created: 1687882410,
          owned_by: 'mock-openai',
        },
      ],
    };
  }

  static getMockChatCompletionResponse(userMessage) {
    return {
      id: 'chatcmpl-mock123',
      object: 'chat.completion',
      created: Date.now(),
      model: 'mock-gpt-4',
      choices: [
        {
          index: 0,
          message: {
            role: 'assistant',
            content: `Mock response to: ${userMessage}`,
          },
          finish_reason: 'stop',
        },
      ],
      usage: {
        prompt_tokens: 10,
        completion_tokens: 10,
        total_tokens: 20,
      },
    };
  }

  static getMockTestPromptResponse() {
    return {
      id: 'chatcmpl-mock-test',
      object: 'chat.completion',
      created: Date.now(),
      model: 'mock-gpt-4',
      choices: [
        {
          index: 0,
          message: {
            role: 'assistant',
            content: 'Test response',
          },
          finish_reason: 'stop',
        },
      ],
      usage: {
        prompt_tokens: 5,
        completion_tokens: 5,
        total_tokens: 10,
      },
    };
  }

  static get401ErrorResponse() {
    return {
      error: {
        message: 'Invalid authentication',
        type: 'invalid_request_error',
        code: 'invalid_api_key',
      },
    };
  }

  static createPublicMockServer(port) {
    return createMockOpenAIServer({
      requiresAuth: false,
      port,
    });
  }

  static createPrivateMockServer(port) {
    return createMockOpenAIServer({
      requiresAuth: true,
      port,
    });
  }

  static createMockApiModelData(mockServerUrl, overrides = {}) {
    return {
      api_format: 'openai',
      baseUrl: mockServerUrl,
      models: ['mock-gpt-4', 'mock-gpt-3.5-turbo'],
      prefix: null,
      ...overrides,
    };
  }

  static scenarios = {
    PUBLIC_MOCK_API: (mockServerUrl) =>
      MockOpenAIFixtures.createMockApiModelData(mockServerUrl, {
        models: ['mock-gpt-4'],
      }),

    PRIVATE_MOCK_API: (mockServerUrl) =>
      MockOpenAIFixtures.createMockApiModelData(mockServerUrl, {
        models: ['mock-gpt-4'],
      }),

    MULTI_MODEL_MOCK: (mockServerUrl) =>
      MockOpenAIFixtures.createMockApiModelData(mockServerUrl, {
        models: ['mock-gpt-4', 'mock-gpt-3.5-turbo'],
      }),
  };
}
