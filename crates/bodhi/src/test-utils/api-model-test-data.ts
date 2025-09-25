import { HandlerOverrides } from './msw-handlers';
import { ApiModelResponse } from '@bodhiapp/ts-client';

// Common test scenarios
export const TEST_SCENARIOS = {
  OPENAI_HAPPY_PATH: {
    providerId: 'openai',
    apiKey: 'sk-test-key-123',
    expectedModels: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview'],
    selectedModels: ['gpt-4'],
    expectedBaseUrl: 'https://api.openai.com/v1',
  },
  OPENAI_COMPATIBLE_HAPPY_PATH: {
    providerId: 'openai-compatible',
    apiKey: 'test-api-key',
    baseUrl: 'https://api.custom-provider.com/v1',
    expectedModels: ['custom-model-1', 'custom-model-2'],
    selectedModels: ['custom-model-1'],
  },
  INVALID_API_KEY: {
    providerId: 'openai',
    apiKey: 'invalid-key',
    expectedError: 'Invalid API key',
  },
  CONNECTION_TIMEOUT: {
    providerId: 'openai',
    apiKey: 'sk-test-key',
    expectedError: 'Connection timeout',
  },
  LARGE_MODEL_LIST: {
    providerId: 'openai-compatible',
    apiKey: 'sk-test-key',
    baseUrl: 'https://api.custom-provider.com/v1',
    expectedModels: Array.from({ length: 50 }, (_, i) => `model-${i + 1}`),
    selectedModels: ['model-1', 'model-5', 'model-10'],
  },
};

// MSW handler configurations for different test scenarios
export const createTestHandlers = {
  openaiHappyPath: (): Partial<HandlerOverrides> => ({
    availableModels: TEST_SCENARIOS.OPENAI_HAPPY_PATH.expectedModels,
    createdModel: {
      id: 'test-openai-model',
      api_format: 'openai',
      base_url: 'https://api.openai.com/v1',
      api_key_masked: '****123',
      models: TEST_SCENARIOS.OPENAI_HAPPY_PATH.selectedModels,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    },
  }),

  openaiCompatibleHappyPath: (): Partial<HandlerOverrides> => ({
    availableModels: TEST_SCENARIOS.OPENAI_COMPATIBLE_HAPPY_PATH.expectedModels,
    createdModel: {
      id: 'test-compatible-model',
      api_format: 'openai-compatible',
      base_url: TEST_SCENARIOS.OPENAI_COMPATIBLE_HAPPY_PATH.baseUrl,
      api_key_masked: '****key',
      models: TEST_SCENARIOS.OPENAI_COMPATIBLE_HAPPY_PATH.selectedModels,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    },
  }),

  invalidApiKey: (): Partial<HandlerOverrides> => ({
    testConnectionError: 'Invalid API key',
    fetchModelsError: true,
  }),

  connectionTimeout: (): Partial<HandlerOverrides> => ({
    testConnectionError: 'Connection timeout',
  }),

  createError: (): Partial<HandlerOverrides> => ({
    availableModels: TEST_SCENARIOS.OPENAI_HAPPY_PATH.expectedModels,
    createError: 'API model with this configuration already exists',
  }),

  largeModelList: (): Partial<HandlerOverrides> => ({
    availableModels: TEST_SCENARIOS.LARGE_MODEL_LIST.expectedModels,
    createdModel: {
      id: 'test-large-model',
      api_format: 'openai-compatible',
      base_url: TEST_SCENARIOS.LARGE_MODEL_LIST.baseUrl,
      api_key_masked: '****key',
      models: TEST_SCENARIOS.LARGE_MODEL_LIST.selectedModels,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    },
  }),

  networkError: (): Partial<HandlerOverrides> => ({
    testConnectionError: 'Network error: Unable to reach API endpoint',
  }),

  serverError: (): Partial<HandlerOverrides> => ({
    createError: 'Internal server error',
  }),

  openaiEditFlow: (): Partial<HandlerOverrides> => ({
    existingModel: {
      id: 'test-model',
      api_format: 'openai',
      base_url: 'https://api.openai.com/v1',
      api_key_masked: '****123',
      models: ['gpt-3.5-turbo'],
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    },
    availableModels: TEST_SCENARIOS.OPENAI_HAPPY_PATH.expectedModels,
    updateApiModel: {
      id: 'test-model',
      api_format: 'openai',
      base_url: 'https://api.openai.com/v1',
      api_key_masked: '****456',
      models: ['gpt-4'],
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    },
  }),
};

// Mock API model responses for different scenarios
export const mockApiModelResponses: Record<string, ApiModelResponse> = {
  openai: {
    id: 'openai-test-model',
    api_format: 'openai',
    base_url: 'https://api.openai.com/v1',
    api_key_masked: '****123',
    models: ['gpt-4', 'gpt-3.5-turbo'],
    prefix: null,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  openaiCompatible: {
    id: 'compatible-test-model',
    api_format: 'openai',
    base_url: 'https://api.custom-provider.com/v1',
    api_key_masked: '****key',
    models: ['custom-model-1'],
    prefix: 'custom-',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
};

// Test data for forms
export const formTestData = {
  validOpenaiForm: {
    providerId: 'openai',
    apiKey: 'sk-valid-key-123',
    models: ['gpt-4'],
  },
  validCompatibleForm: {
    providerId: 'openai-compatible',
    apiKey: 'valid-custom-key',
    baseUrl: 'https://api.custom-provider.com/v1',
    models: ['custom-model-1'],
  },
  invalidForms: {
    noApiKey: {
      providerId: 'openai',
      apiKey: '',
      models: ['gpt-4'],
    },
    noBaseUrl: {
      providerId: 'openai-compatible',
      apiKey: 'valid-key',
      baseUrl: '',
      models: ['custom-model-1'],
    },
    noModels: {
      providerId: 'openai',
      apiKey: 'sk-valid-key',
      models: [],
    },
  },
};

// Common model lists for testing
export const modelLists = {
  openai: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview'],
  openaiCompatible: ['custom-model-1', 'custom-model-2', 'custom-model-3'],
  large: Array.from({ length: 50 }, (_, i) => `model-${i + 1}`),
  empty: [],
};

// Provider configurations
export const providerConfigs = {
  openai: {
    id: 'openai',
    name: 'OpenAI',
    requiresBaseUrl: false,
    defaultBaseUrl: 'https://api.openai.com/v1',
  },
  openaiCompatible: {
    id: 'openai-compatible',
    name: 'OpenAI-Compatible',
    requiresBaseUrl: true,
    defaultBaseUrl: '',
  },
};

// Error messages for testing
export const errorMessages = {
  invalidApiKey: 'Invalid API key',
  connectionTimeout: 'Connection timeout',
  networkError: 'Network error: Unable to reach API endpoint',
  serverError: 'Internal server error',
  validationErrors: {
    apiKeyRequired: 'API key is required',
    baseUrlRequired: 'Base URL is required',
    modelsRequired: 'At least one model must be selected',
  },
};

// Toast messages for testing
export const toastMessages = {
  success: {
    created: 'API model created successfully',
    updated: 'API model updated successfully',
    connectionTest: 'Connection test successful',
    modelsFetched: 'Models fetched successfully',
  },
  error: {
    createFailed: 'Failed to create API model',
    updateFailed: 'Failed to update API model',
    connectionFailed: 'Connection test failed',
    fetchModelsFailed: 'Failed to fetch models',
  },
};

// Loading states text
export const loadingStates = {
  testingConnection: 'Testing connection...',
  fetchingModels: 'Fetching models...',
  creatingModel: 'Creating API model...',
  updatingModel: 'Updating API model...',
  loading: 'Loading...',
};

// Navigation paths
export const navigationPaths = {
  modelsPage: '/ui/models',
  editModel: (id: string) => `/ui/models/edit?id=${id}`,
  setupComplete: '/ui/setup/complete',
  setupNext: '/ui/setup/extension',
};

// Accessibility test data
export const a11yTestData = {
  requiredFields: ['api-key-input'],
  conditionalFields: ['base-url-input'],
  buttons: ['test-connection-button', 'fetch-models-button', 'create-api-model-button'],
  interactiveElements: ['provider-openai', 'provider-openai-compatible'],
};
