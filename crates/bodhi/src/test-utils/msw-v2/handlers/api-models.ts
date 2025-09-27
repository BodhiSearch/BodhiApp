/**
 * Type-safe MSW v2 handlers for API models endpoints using openapi-msw
 */
import {
  ENDPOINT_API_MODELS,
  ENDPOINT_API_MODELS_TEST,
  ENDPOINT_API_MODELS_FETCH,
  ENDPOINT_API_MODELS_FORMATS,
} from '@/hooks/useApiModels';
import { typedHttp } from '../openapi-msw-setup';
import { type components } from '../setup';

/**
 * Create type-safe MSW v2 handlers for API models list endpoint
 */
export function mockApiModels(config: Partial<components['schemas']['PaginatedApiModelResponse']> = {}) {
  return [
    typedHttp.get(ENDPOINT_API_MODELS, ({ response }) => {
      const responseData: components['schemas']['PaginatedApiModelResponse'] = {
        data: config.data || [],
        page: config.page || 1,
        page_size: config.page_size || 30,
        total: config.total || 0,
      };
      return response(200).json(responseData);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for API model creation endpoint
 */
export function mockCreateApiModel(
  config: {
    response?: components['schemas']['ApiModelResponse'];
    error?: { status?: 201 | 400 | 409 | 500; code?: string; message?: string; type?: string };
  } = {}
) {
  return [
    typedHttp.post(ENDPOINT_API_MODELS, ({ response }) => {
      if (config.error) {
        const errorType =
          config.error.status === 400
            ? 'invalid_request_error'
            : config.error.status === 409
              ? 'invalid_request_error'
              : 'internal_server_error';

        return response(config.error.status || 500).json({
          error: {
            code: config.error.code || 'internal_error',
            message: config.error.message || 'Internal Server Error',
            type: config.error.type || errorType,
          },
        });
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
      return response(201).json(responseData);
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
    error?: { status?: 404 | 500; code?: string; message?: string; type?: string };
  } = { id: 'test-api-model' }
) {
  return [
    typedHttp.get('/bodhi/v1/api-models/{id}', ({ params, response }) => {
      const { id } = params;

      if (config.error) {
        const errorType = config.error.status === 404 ? 'not_found_error' : 'internal_server_error';

        return response(config.error.status || 404).json({
          error: {
            code: config.error.code || 'entity_not_found',
            message: config.error.message || `API model ${id} not found`,
            type: config.error.type || errorType,
          },
        });
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
      return response(200).json(responseData);
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
    error?: { status?: 400 | 404 | 500; code?: string; message?: string; type?: string };
  } = { id: 'test-api-model' }
) {
  return [
    typedHttp.put('/bodhi/v1/api-models/{alias}', ({ params, response }) => {
      const { alias } = params;

      if (config.error) {
        const errorType =
          config.error.status === 400
            ? 'invalid_request_error'
            : config.error.status === 404
              ? 'not_found_error'
              : 'internal_server_error';

        return response(config.error.status || 500).json({
          error: {
            code: config.error.code || 'internal_error',
            message: config.error.message || `Failed to update API model ${alias}`,
            type: config.error.type || errorType,
          },
        });
      }

      const responseData: components['schemas']['ApiModelResponse'] = config.response || {
        id: alias as string,
        api_format: 'openai',
        base_url: 'https://api.openai.com/v1',
        api_key_masked: '****key',
        models: ['gpt-4'],
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      return response(200).json(responseData);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for API model deletion endpoint
 */
export function mockDeleteApiModel(
  config: {
    id: string;
    error?: { status?: 404 | 500; code?: string; message?: string; type?: string };
  } = { id: 'test-api-model' }
) {
  return [
    typedHttp.delete('/bodhi/v1/api-models/{alias}', ({ params, response }) => {
      const { alias } = params;

      if (config.error) {
        const errorType = config.error.status === 404 ? 'not_found_error' : 'internal_server_error';

        return response(config.error.status || 404).json({
          error: {
            code: config.error.code || 'entity_not_found',
            message: config.error.message || `API model ${alias} not found`,
            type: config.error.type || errorType,
          },
        });
      }

      return response(204).empty();
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for API formats endpoint
 */
export function mockApiFormats(
  config: {
    data?: components['schemas']['ApiFormat'][];
    error?: { status?: 500; code?: string; message?: string; type?: string };
  } = {}
) {
  return [
    typedHttp.get(ENDPOINT_API_MODELS_FORMATS, ({ response }) => {
      if (config.error) {
        return response(config.error.status || 500).json({
          error: {
            code: config.error.code || 'internal_error',
            message: config.error.message || 'Failed to fetch API formats',
            type: config.error.type || 'internal_server_error',
          },
        });
      }

      const responseData: components['schemas']['ApiFormatsResponse'] = {
        data: config.data || ['openai', 'placeholder'],
      };
      return response(200).json(responseData);
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
    error?: { status?: 400 | 500; code?: string; message?: string; type?: string };
  } = {}
) {
  return [
    typedHttp.post(ENDPOINT_API_MODELS_TEST, ({ response }) => {
      if (config.error) {
        const errorType = config.error.status === 400 ? 'invalid_request_error' : 'internal_server_error';

        return response(config.error.status || 500).json({
          error: {
            code: config.error.code || 'internal_error',
            message: config.error.message || 'Test failed',
            type: config.error.type || errorType,
          },
        });
      }

      return response(200).json({
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
    error?: { status?: 400 | 500; code?: string; message?: string; type?: string };
  } = {}
) {
  return [
    typedHttp.post(ENDPOINT_API_MODELS_FETCH, ({ response }) => {
      if (config.error) {
        const errorType = config.error.status === 400 ? 'invalid_request_error' : 'internal_server_error';

        return response(config.error.status || 500).json({
          error: {
            code: config.error.code || 'authentication_error',
            message: config.error.message || 'Invalid API key',
            type: config.error.type || errorType,
          },
        });
      }

      return response(200).json({
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
    status?: 500;
    code?: string;
    message?: string;
    type?: string;
  } = {}
) {
  return [
    typedHttp.get(ENDPOINT_API_MODELS, ({ response }) => {
      return response(config.status || 500).json({
        error: {
          code: config.code || 'internal_error',
          message: config.message || 'Internal Server Error',
          type: config.type || 'internal_server_error',
        },
      });
    }),
  ];
}

/**
 * Convenience methods for common test scenarios
 */
export function mockApiFormatsDefault() {
  return mockApiFormats({ data: ['openai', 'placeholder'] });
}

export function mockTestApiModelSuccess() {
  return mockTestApiModel({ success: true, response: 'Connection successful' });
}

export function mockTestApiModelError() {
  return mockTestApiModel({
    error: { status: 400, code: 'connection_error', message: 'Connection test failed', type: 'invalid_request_error' },
  });
}

export function mockFetchApiModelsSuccess() {
  return mockFetchApiModels({ models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview'] });
}

export function mockFetchApiModelsAuthError() {
  return mockFetchApiModels({
    error: { status: 400, code: 'authentication_error', message: 'Invalid API key', type: 'invalid_request_error' },
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
    error: {
      status: 400,
      code: 'invalid_request_error',
      message: 'Invalid API model data',
      type: 'invalid_request_error',
    },
  });
}

export function mockDeleteApiModelSuccess() {
  return mockDeleteApiModel({ id: 'test-api-model' });
}

export function mockDeleteApiModelNotFound() {
  return mockDeleteApiModel({
    id: 'missing-model',
    error: { status: 404, code: 'entity_not_found', message: 'API model not found', type: 'not_found_error' },
  });
}
