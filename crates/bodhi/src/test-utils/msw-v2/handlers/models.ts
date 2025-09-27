/**
 * Type-safe MSW v2 handlers for models endpoint using openapi-msw
 */
import { ENDPOINT_MODELS } from '@/hooks/useQuery';
import { typedHttp } from '../openapi-msw-setup';
import { type components } from '../setup';

/**
 * Create type-safe MSW v2 handlers for models list endpoint
 * Uses generated OpenAPI types directly
 */
export function mockModels(config: Partial<components['schemas']['PaginatedAliasResponse']> = {}) {
  return [
    typedHttp.get(ENDPOINT_MODELS, ({ response }) => {
      const responseData: components['schemas']['PaginatedAliasResponse'] = {
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
 * Create type-safe MSW v2 handler for model creation endpoint
 */
export function mockCreateModel(
  config: {
    response?: components['schemas']['UserAliasResponse'];
    error?: { status?: 400 | 500; code?: string; message?: string };
  } = {}
) {
  return [
    typedHttp.post(ENDPOINT_MODELS, ({ response }) => {
      if (config.error) {
        return response(config.error.status || 500).json({
          error: {
            code: config.error.code || 'internal_error',
            message: config.error.message || 'Internal Server Error',
            type: config.error.status === 400 ? 'invalid_request_error' : 'internal_server_error',
          },
        });
      }

      const responseData: components['schemas']['UserAliasResponse'] = config.response || {
        alias: 'new-model',
        repo: 'test-repo',
        filename: 'test-file.bin',
        snapshot: 'abc123',
        request_params: {},
        context_params: [],
        model_params: {},
        source: 'user',
      };
      return response(201).json(responseData);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for individual model retrieval
 */
export function mockGetModel(
  config: {
    alias: string;
    response?: components['schemas']['UserAliasResponse'];
    error?: { status?: 404 | 500; code?: string; message?: string };
  } = { alias: 'test-model' }
) {
  return [
    typedHttp.get('/bodhi/v1/models/{alias}', ({ response, params }) => {
      const { alias } = params;

      if (config.error) {
        return response(config.error.status || 404).json({
          error: {
            code: config.error.code || 'not_found',
            message: config.error.message || `Model ${alias} not found`,
            type: config.error.status === 404 ? 'not_found_error' : 'internal_server_error',
          },
        });
      }

      const responseData: components['schemas']['UserAliasResponse'] = config.response || {
        alias: alias as string,
        repo: 'test-repo',
        filename: 'test-file.bin',
        snapshot: 'abc123',
        request_params: {},
        context_params: [],
        model_params: {},
        source: 'user',
      };
      return response(200).json(responseData);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for model update endpoint
 */
export function mockUpdateModel(
  config: {
    id: string;
    response?: components['schemas']['UserAliasResponse'];
    error?: { status?: 400 | 500; code?: string; message?: string };
  } = { id: 'test-model' }
) {
  return [
    typedHttp.put('/bodhi/v1/models/{id}', ({ response, params }) => {
      const { id } = params;

      if (config.error) {
        return response(config.error.status || 500).json({
          error: {
            code: config.error.code || 'internal_error',
            message: config.error.message || `Failed to update model ${id}`,
            type:
              config.error.status === 400
                ? 'invalid_request_error'
                : 'internal_server_error',
          },
        });
      }

      const responseData: components['schemas']['UserAliasResponse'] = config.response || {
        alias: id as string,
        repo: 'test-repo',
        filename: 'test-file.bin',
        snapshot: 'abc123',
        request_params: {},
        context_params: [],
        model_params: {},
        source: 'user',
      };
      return response(200).json(responseData);
    }),
  ];
}

/**
 * Predefined mock configurations for common use cases
 */
export function mockModelsDefault() {
  return mockModels({
    data: [
      {
        source: 'user',
        alias: 'test-model',
        repo: 'test-repo',
        filename: 'test-file.bin',
        snapshot: 'abc123',
        request_params: {},
        context_params: [],
      },
    ],
    total: 1,
    page: 1,
    page_size: 30,
  });
}

export function mockModelsWithApiModel() {
  return mockModels({
    data: [
      {
        source: 'api',
        id: 'test-api-model',
        api_format: 'openai',
        base_url: 'https://api.openai.com/v1',
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

export function mockModelsWithSourceModel() {
  return mockModels({
    data: [
      {
        source: 'model',
        alias: 'test-model',
        repo: 'test-repo',
        filename: 'test-file.bin',
        snapshot: 'abc123',
      },
    ],
    total: 1,
    page: 1,
    page_size: 30,
  });
}

export function mockModelsEmpty() {
  return mockModels({ data: [], total: 0 });
}

export function mockModelsError(
  config: {
    status?: 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    typedHttp.get(ENDPOINT_MODELS, ({ response }) => {
      return response(config.status || 500).json({
        error: {
          code: config.code || 'internal_error',
          message: config.message || 'Internal Server Error',
          type: 'internal_server_error',
        },
      });
    }),
  ];
}
