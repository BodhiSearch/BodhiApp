/**
 * Type-safe MSW v2 handlers for API models endpoints using patterns inspired by openapi-msw
 */
import {
  ENDPOINT_API_MODELS,
  ENDPOINT_API_MODELS_TEST,
  ENDPOINT_API_MODELS_FETCH,
  ENDPOINT_API_MODELS_FORMATS,
} from '@/hooks/useApiModels';
import { http, HttpResponse, type components } from '../setup';

/**
 * Create type-safe MSW v2 handlers for API models list endpoint
 */
export function mockApiModels(config: Partial<components['schemas']['PaginatedApiModelResponse']> = {}) {
  return [
    http.get(ENDPOINT_API_MODELS, () => {
      const responseData: components['schemas']['PaginatedApiModelResponse'] = {
        data: config.data || [],
        page: config.page || 1,
        page_size: config.page_size || 30,
        total: config.total || 0,
      };
      return HttpResponse.json(responseData);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for API model creation endpoint
 */
export function mockCreateApiModel(
  config: {
    response?: components['schemas']['ApiModelResponse'];
    error?: { status?: 400 | 500; code?: string; message?: string };
  } = {}
) {
  return [
    http.post(ENDPOINT_API_MODELS, () => {
      if (config.error) {
        return HttpResponse.json(
          {
            error: {
              code: config.error.code || 'internal_error',
              message: config.error.message || 'Internal Server Error',
            },
          },
          { status: config.error.status || 500 }
        );
      }

      const responseData: components['schemas']['ApiModelResponse'] = config.response || {
        id: 'test-api-model-123',
        api_format: 'openai',
        base_url: 'https://api.openai.com/v1',
        api_key_masked: '****key',
        models: ['gpt-4'],
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      return HttpResponse.json(responseData, { status: 201 });
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for individual API model retrieval
 */
export function mockGetApiModel(
  config: {
    id: string;
    response?: components['schemas']['ApiModelResponse'];
    error?: { status?: 404 | 500; code?: string; message?: string };
  } = { id: 'test-api-model' }
) {
  return [
    http.get(`${ENDPOINT_API_MODELS}/:id`, ({ params }) => {
      const { id } = params;

      if (config.error) {
        return HttpResponse.json(
          {
            error: {
              code: config.error.code || 'not_found',
              message: config.error.message || `API model ${id} not found`,
            },
          },
          { status: config.error.status || 404 }
        );
      }

      const responseData: components['schemas']['ApiModelResponse'] = config.response || {
        id: id as string,
        api_format: 'openai',
        base_url: 'https://api.openai.com/v1',
        api_key_masked: '****123',
        models: ['gpt-3.5-turbo'],
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      return HttpResponse.json(responseData);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for API model update endpoint
 */
export function mockUpdateApiModel(
  config: {
    id: string;
    response?: components['schemas']['ApiModelResponse'];
    error?: { status?: 400 | 404 | 500; code?: string; message?: string };
  } = { id: 'test-api-model' }
) {
  return [
    http.put(`${ENDPOINT_API_MODELS}/:id`, ({ params }) => {
      const { id } = params;

      if (config.error) {
        return HttpResponse.json(
          {
            error: {
              code: config.error.code || 'internal_error',
              message: config.error.message || `Failed to update API model ${id}`,
            },
          },
          { status: config.error.status || 500 }
        );
      }

      const responseData: components['schemas']['ApiModelResponse'] = config.response || {
        id: id as string,
        api_format: 'openai',
        base_url: 'https://api.openai.com/v1',
        api_key_masked: '****key',
        models: ['gpt-4'],
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      return HttpResponse.json(responseData);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for API formats endpoint
 */
export function mockApiFormats(
  config: {
    data?: string[];
    error?: { status?: 500; code?: string; message?: string };
  } = {}
) {
  return [
    http.get(ENDPOINT_API_MODELS_FORMATS, () => {
      if (config.error) {
        return HttpResponse.json(
          {
            error: {
              code: config.error.code || 'internal_error',
              message: config.error.message || 'Failed to fetch API formats',
            },
          },
          { status: config.error.status || 500 }
        );
      }

      return HttpResponse.json({
        data: config.data || ['openai', 'openai-compatible'],
      });
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for API model test connection endpoint
 */
export function mockTestApiModel(
  config: {
    success?: boolean;
    response?: string;
    error?: string;
  } = {}
) {
  return [
    http.post(ENDPOINT_API_MODELS_TEST, () => {
      if (config.error) {
        return HttpResponse.json({
          success: false,
          error: config.error,
        });
      }

      return HttpResponse.json({
        success: config.success !== undefined ? config.success : true,
        response: config.response || 'Connection successful',
      });
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for API model fetch models endpoint
 */
export function mockFetchApiModels(
  config: {
    models?: string[];
    error?: { status?: 401 | 500; code?: string; message?: string };
  } = {}
) {
  return [
    http.post(ENDPOINT_API_MODELS_FETCH, () => {
      if (config.error) {
        return HttpResponse.json(
          {
            error: {
              code: config.error.code || 'authentication_error',
              message: config.error.message || 'Invalid API key',
            },
          },
          { status: config.error.status || 401 }
        );
      }

      return HttpResponse.json({
        models: config.models || ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview'],
      });
    }),
  ];
}

/**
 * Predefined mock configurations for common use cases
 */
export function mockApiModelsDefault() {
  return mockApiModels({
    data: [
      {
        id: 'test-api-model',
        api_format: 'openai',
        base_url: 'https://api.openai.com/v1',
        api_key_masked: '****123',
        models: ['gpt-4', 'gpt-3.5-turbo'],
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      },
    ],
    total: 1,
    page: 1,
    page_size: 30,
  });
}

export function mockApiModelsEmpty() {
  return mockApiModels({ data: [], total: 0 });
}

export function mockApiModelsError(
  config: {
    status?: 400 | 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    http.get(ENDPOINT_API_MODELS, () => {
      return HttpResponse.json(
        {
          error: {
            code: config.code || 'internal_error',
            message: config.message || 'Internal Server Error',
          },
        },
        { status: config.status || 500 }
      );
    }),
  ];
}

/**
 * Convenience methods for common test scenarios
 */
export function mockApiFormatsDefault() {
  return mockApiFormats({ data: ['openai', 'openai-compatible'] });
}

export function mockTestApiModelSuccess() {
  return mockTestApiModel({ success: true, response: 'Connection successful' });
}

export function mockTestApiModelError() {
  return mockTestApiModel({ error: 'Connection test failed' });
}

export function mockFetchApiModelsSuccess() {
  return mockFetchApiModels({ models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview'] });
}

export function mockFetchApiModelsAuthError() {
  return mockFetchApiModels({
    error: { status: 401, code: 'authentication_error', message: 'Invalid API key' },
  });
}

export function mockCreateApiModelSuccess() {
  return mockCreateApiModel({
    response: {
      id: 'test-model-123',
      api_format: 'openai',
      base_url: 'https://api.openai.com/v1',
      api_key_masked: '****key',
      models: ['gpt-4'],
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    },
  });
}

export function mockCreateApiModelError() {
  return mockCreateApiModel({
    error: { status: 400, code: 'invalid_request_error', message: 'Invalid API model data' },
  });
}
