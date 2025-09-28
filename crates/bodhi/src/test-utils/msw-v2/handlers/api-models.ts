/**
 * Type-safe MSW v2 handlers for API models endpoints using openapi-msw
 */
import {
  ENDPOINT_API_MODELS,
  ENDPOINT_API_MODELS_FETCH,
  ENDPOINT_API_MODELS_FORMATS,
  ENDPOINT_API_MODELS_TEST,
  ENDPOINT_API_MODEL_ID,
} from '@/hooks/useApiModels';
import { INTERNAL_SERVER_ERROR, typedHttp, type components } from '../openapi-msw-setup';

// =============================================================================
// CORE TYPED HTTP METHODS (Success cases + Error handlers)
// =============================================================================

/**
 * Mock handler for API models list endpoint with configurable responses
 */
export function mockApiModels({
  data = [],
  page = 1,
  page_size = 30,
  total = 0,
  ...rest
}: Partial<components['schemas']['PaginatedApiModelResponse']> = {}) {
  return [
    typedHttp.get(ENDPOINT_API_MODELS, async ({ response: res }) => {
      const responseData: components['schemas']['PaginatedApiModelResponse'] = {
        data,
        page,
        page_size,
        total,
        ...rest,
      };
      return res(200 as const).json(responseData);
    }),
  ];
}

export function mockApiModelsError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  return [
    typedHttp.get(ENDPOINT_API_MODELS, async ({ response }) => {
      const errorData = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorData });
    }),
  ];
}

/**
 * Mock handler for API model creation endpoint with configurable responses
 */
export function mockCreateApiModel({
  id = 'test-api-model-123',
  api_format = 'openai',
  base_url = 'https://api.openai.com/v1',
  api_key_masked = '****key',
  models = ['gpt-4'],
  prefix = null,
  created_at = new Date().toISOString(),
  updated_at = new Date().toISOString(),
  ...rest
}: Partial<components['schemas']['ApiModelResponse']> = {}) {
  return [
    typedHttp.post(ENDPOINT_API_MODELS, async ({ response: res }) => {
      const responseData: components['schemas']['ApiModelResponse'] = {
        id,
        api_format,
        base_url,
        api_key_masked,
        models,
        prefix,
        created_at,
        updated_at,
        ...rest,
      };
      return res(201 as const).json(responseData);
    }),
  ];
}

export function mockCreateApiModelError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  return [
    typedHttp.post(ENDPOINT_API_MODELS, async ({ response }) => {
      const errorData = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorData });
    }),
  ];
}

/**
 * Mock handler for individual API model retrieval with configurable responses
 */
export function mockGetApiModel(
  expectedId: string,
  {
    id = '',
    api_format = 'openai',
    base_url = 'https://api.openai.com/v1',
    api_key_masked = '****123',
    models = ['gpt-3.5-turbo'],
    prefix = null,
    created_at = new Date().toISOString(),
    updated_at = new Date().toISOString(),
    ...rest
  }: Partial<components['schemas']['ApiModelResponse']> = {}
) {
  return [
    typedHttp.get(ENDPOINT_API_MODEL_ID, async ({ params, response: res }) => {
      const { id: paramId } = params;

      // Only respond if id matches
      if (paramId !== expectedId) {
        return; // Pass through to next handler
      }

      const responseData: components['schemas']['ApiModelResponse'] = {
        id: id || (paramId as string),
        api_format,
        base_url,
        api_key_masked,
        models,
        prefix,
        created_at,
        updated_at,
        ...rest,
      };
      return res(200 as const).json(responseData);
    }),
  ];
}

export function mockGetApiModelError(
  expectedId: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 404 | 500 } = {}
) {
  return [
    typedHttp.get(ENDPOINT_API_MODEL_ID, async ({ params, response }) => {
      const { id } = params;

      // Only respond if id matches
      if (id !== expectedId) {
        return; // Pass through to next handler
      }

      const errorData = {
        code,
        message: message || `API model ${id} not found`,
        type,
        ...rest,
      };
      return response(status).json({ error: errorData });
    }),
  ];
}

/**
 * Mock handler for API model update endpoint with configurable responses
 */
export function mockUpdateApiModel(
  expectedId: string,
  {
    id = '',
    api_format = 'openai',
    base_url = 'https://api.openai.com/v1',
    api_key_masked = '****key',
    models = ['gpt-4'],
    prefix = null,
    created_at = new Date().toISOString(),
    updated_at = new Date().toISOString(),
    ...rest
  }: Partial<components['schemas']['ApiModelResponse']> = {}
) {
  return [
    typedHttp.put(ENDPOINT_API_MODEL_ID, async ({ params, response: res }) => {
      const { id: paramId } = params;

      // Only respond if id matches
      if (paramId !== expectedId) {
        return; // Pass through to next handler
      }

      const responseData: components['schemas']['ApiModelResponse'] = {
        id: id || (paramId as string),
        api_format,
        base_url,
        api_key_masked,
        models,
        prefix,
        created_at,
        updated_at,
        ...rest,
      };
      return res(200 as const).json(responseData);
    }),
  ];
}

export function mockUpdateApiModelError(
  expectedId: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 404 | 500 } = {}
) {
  return [
    typedHttp.put(ENDPOINT_API_MODEL_ID, async ({ params, response }) => {
      const { id } = params;

      // Only respond if id matches
      if (id !== expectedId) {
        return; // Pass through to next handler
      }

      const errorData = {
        code,
        message: message || `Failed to update API model ${id}`,
        type,
        ...rest,
      };
      return response(status).json({ error: errorData });
    }),
  ];
}

/**
 * Mock handler for API model deletion endpoint with configurable responses
 */
export function mockDeleteApiModel(expectedId: string) {
  return [
    typedHttp.delete(ENDPOINT_API_MODEL_ID, async ({ params, response }) => {
      const { id } = params;

      // Only respond if id matches
      if (id !== expectedId) {
        return; // Pass through to next handler
      }

      return response(204 as const).empty();
    }),
  ];
}

export function mockDeleteApiModelError(
  expectedId: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 404 | 500 } = {}
) {
  return [
    typedHttp.delete(ENDPOINT_API_MODEL_ID, async ({ params, response }) => {
      const { id } = params;

      // Only respond if id matches
      if (id !== expectedId) {
        return; // Pass through to next handler
      }

      const errorData = {
        code,
        message: message || `API model ${id} not found`,
        type,
        ...rest,
      };
      return response(status).json({ error: errorData });
    }),
  ];
}

/**
 * Mock handler for API formats endpoint with configurable responses
 */
export function mockApiFormats({
  data = ['openai', 'placeholder'],
  ...rest
}: Partial<components['schemas']['ApiFormatsResponse']> = {}) {
  return [
    typedHttp.get(ENDPOINT_API_MODELS_FORMATS, async ({ response: res }) => {
      const responseData: components['schemas']['ApiFormatsResponse'] = {
        data,
        ...rest,
      };
      return res(200 as const).json(responseData);
    }),
  ];
}

export function mockApiFormatsError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  return [
    typedHttp.get(ENDPOINT_API_MODELS_FORMATS, async ({ response }) => {
      const errorData = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorData });
    }),
  ];
}

/**
 * Mock handler for API model test connection endpoint with configurable responses
 */
export function mockTestApiModel({
  success = true,
  response = 'Connection successful',
  ...rest
}: Partial<components['schemas']['TestPromptResponse']> = {}) {
  return [
    typedHttp.post(ENDPOINT_API_MODELS_TEST, async ({ response: res }) => {
      const responseData: components['schemas']['TestPromptResponse'] = {
        success,
        response,
        ...rest,
      };
      return res(200 as const).json(responseData);
    }),
  ];
}

export function mockTestApiModelError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  return [
    typedHttp.post(ENDPOINT_API_MODELS_TEST, async ({ response }) => {
      const errorData = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorData });
    }),
  ];
}

/**
 * Mock handler for API model fetch models endpoint with configurable responses
 */
export function mockFetchApiModels({
  models = ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview'],
  ...rest
}: Partial<components['schemas']['FetchModelsResponse']> = {}) {
  return [
    typedHttp.post(ENDPOINT_API_MODELS_FETCH, async ({ response: res }) => {
      const responseData: components['schemas']['FetchModelsResponse'] = {
        models,
        ...rest,
      };
      return res(200 as const).json(responseData);
    }),
  ];
}

export function mockFetchApiModelsError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  return [
    typedHttp.post(ENDPOINT_API_MODELS_FETCH, async ({ response }) => {
      const errorData = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorData });
    }),
  ];
}

// =============================================================================
// VARIANT METHODS (Using core methods above)
// =============================================================================

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

/**
 * Convenience methods for common test scenarios
 */
export function mockApiFormatsDefault() {
  return mockApiFormats({ data: ['openai', 'placeholder'] });
}

export function mockTestApiModelSuccess() {
  return mockTestApiModel({ success: true, response: 'Connection successful' });
}

export function mockFetchApiModelsSuccess() {
  return mockFetchApiModels({ models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview'] });
}

export function mockFetchApiModelsAuthError() {
  return mockFetchApiModelsError({
    code: 'authentication_error',
    message: 'Invalid API key',
    type: 'invalid_request_error',
  });
}

export function mockCreateApiModelSuccess() {
  return mockCreateApiModel({
    id: 'test-model-123',
    api_format: 'openai',
    base_url: 'https://api.openai.com/v1',
    api_key_masked: '****key',
    models: ['gpt-4'],
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  });
}

export function mockDeleteApiModelSuccess(expectedId: string) {
  return mockDeleteApiModel(expectedId);
}

export function mockDeleteApiModelNotFound(expectedId: string) {
  return mockDeleteApiModelError(expectedId, {
    code: 'entity_not_found',
    message: 'API model not found',
    type: 'not_found_error',
  });
}
