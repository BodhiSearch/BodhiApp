/**
 * Type-safe MSW v2 handlers for API models endpoints using openapi-msw
 */
import {
  ENDPOINT_API_MODELS,
  ENDPOINT_API_MODELS_FETCH,
  ENDPOINT_API_MODELS_FORMATS,
  ENDPOINT_API_MODELS_TEST,
  ENDPOINT_API_MODEL_ID,
} from '@/hooks/models';

import { INTERNAL_SERVER_ERROR, typedHttp, http, HttpResponse, type components } from '../setup';

// =============================================================================
// CORE TYPED HTTP METHODS (Success cases + Error handlers)
// =============================================================================

/**
 * Mock handler for API models list endpoint with configurable responses
 */
/**
 * Note: The GET list endpoint for API models was removed.
 * Use mockModels from handlers/models.ts for the combined models list.
 * These handlers are kept for backward compatibility but use standard http.
 */
export function mockApiModels(
  {
    data = [],
    page = 1,
    page_size = 30,
    total = 0,
  }: { data?: components['schemas']['ApiAliasResponse'][]; page?: number; page_size?: number; total?: number } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.get(`*${ENDPOINT_API_MODELS}`, () => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      return HttpResponse.json({ data, page, page_size, total });
    }),
  ];
}

export function mockApiModelsError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
  }: Partial<components['schemas']['BodhiErrorBody']> & { status?: number } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.get(`*${ENDPOINT_API_MODELS}`, () => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      return HttpResponse.json({ error: { code, message, type } }, { status });
    }),
  ];
}

/**
 * Mock handler for API model creation endpoint with configurable responses
 *
 * Note: has_api_key semantics:
 * - true: API key exists (default)
 * - false: No API key stored
 */
export function mockCreateApiModel(
  {
    source = 'api',
    id = 'test-api-model-123',
    api_format = 'openai',
    base_url = 'https://api.openai.com/v1',
    has_api_key = true,
    models = [{ id: 'gpt-4', object: 'model', created: 0, owned_by: 'openai', provider: 'openai' }],
    prefix = null,
    forward_all_with_prefix = false,
    created_at = new Date().toISOString(),
    updated_at = new Date().toISOString(),
    ...rest
  }: Partial<components['schemas']['ApiAliasResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.post(ENDPOINT_API_MODELS, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      const responseData: components['schemas']['ApiAliasResponse'] = {
        source,
        id,
        api_format,
        base_url,
        has_api_key,
        models,
        prefix,
        forward_all_with_prefix,
        created_at,
        updated_at,
        ...rest,
      };
      return response(201 as const).json(responseData);
    }),
  ];
}

export function mockCreateApiModelError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.post(ENDPOINT_API_MODELS, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
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
 *
 * Note: has_api_key semantics:
 * - true: API key exists (default)
 * - false: No API key stored
 */
export function mockGetApiModel(
  expectedId: string,
  {
    source = 'api',
    id = '',
    api_format = 'openai',
    base_url = 'https://api.openai.com/v1',
    has_api_key = true,
    models = [{ id: 'gpt-3.5-turbo', object: 'model', created: 0, owned_by: 'openai', provider: 'openai' }],
    prefix = null,
    forward_all_with_prefix = false,
    created_at = new Date().toISOString(),
    updated_at = new Date().toISOString(),
    ...rest
  }: Partial<components['schemas']['ApiAliasResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_API_MODEL_ID, async ({ params, response }) => {
      const { id: paramId } = params;

      // Only respond if id matches
      if (paramId !== expectedId) {
        return; // Pass through to next handler
      }

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['ApiAliasResponse'] = {
        source,
        id: id || (paramId as string),
        api_format,
        base_url,
        has_api_key,
        models,
        prefix,
        forward_all_with_prefix,
        created_at,
        updated_at,
        ...rest,
      };
      return response(200 as const).json(responseData);
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
  }: Partial<components['schemas']['BodhiErrorBody']> & { status?: 400 | 401 | 403 | 404 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_API_MODEL_ID, async ({ params, response }) => {
      const { id } = params;

      // Only respond if id matches
      if (id !== expectedId) {
        return; // Pass through to next handler
      }

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

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
 *
 * Note: has_api_key semantics:
 * - true: API key exists (default)
 * - false: No API key stored
 */
export function mockUpdateApiModel(
  expectedId: string,
  {
    source = 'api',
    id = '',
    api_format = 'openai',
    base_url = 'https://api.openai.com/v1',
    has_api_key = true,
    models = [{ id: 'gpt-4', object: 'model', created: 0, owned_by: 'openai', provider: 'openai' }],
    prefix = null,
    forward_all_with_prefix = false,
    created_at = new Date().toISOString(),
    updated_at = new Date().toISOString(),
    ...rest
  }: Partial<components['schemas']['ApiAliasResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.put(ENDPOINT_API_MODEL_ID, async ({ params, response }) => {
      const { id: paramId } = params;

      // Only respond if id matches
      if (paramId !== expectedId) {
        return; // Pass through to next handler
      }

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['ApiAliasResponse'] = {
        source,
        id: id || (paramId as string),
        api_format,
        base_url,
        has_api_key,
        models,
        prefix,
        forward_all_with_prefix,
        created_at,
        updated_at,
        ...rest,
      };
      return response(200 as const).json(responseData);
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
  }: Partial<components['schemas']['BodhiErrorBody']> & { status?: 400 | 401 | 403 | 404 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.put(ENDPOINT_API_MODEL_ID, async ({ params, response }) => {
      const { id } = params;

      // Only respond if id matches
      if (id !== expectedId) {
        return; // Pass through to next handler
      }

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

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
export function mockDeleteApiModel(expectedId: string, { stub }: { delayMs?: number; stub?: boolean } = {}) {
  let hasBeenCalled = false;
  return [
    typedHttp.delete(ENDPOINT_API_MODEL_ID, async ({ params, response }) => {
      const { id } = params;

      // Only respond if id matches
      if (id !== expectedId) {
        return; // Pass through to next handler
      }

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

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
  }: Partial<components['schemas']['BodhiErrorBody']> & { status?: 400 | 401 | 403 | 404 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.delete(ENDPOINT_API_MODEL_ID, async ({ params, response }) => {
      const { id } = params;

      // Only respond if id matches
      if (id !== expectedId) {
        return; // Pass through to next handler
      }

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

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
export function mockApiFormats(
  { data = ['openai', 'placeholder'], ...rest }: Partial<components['schemas']['ApiFormatsResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_API_MODELS_FORMATS, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      const responseData: components['schemas']['ApiFormatsResponse'] = {
        data,
        ...rest,
      };
      return response(200 as const).json(responseData);
    }),
  ];
}

export function mockApiFormatsError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_API_MODELS_FORMATS, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
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
export function mockTestApiModel(
  {
    success = true,
    response: responseMessage = 'Connection successful',
    ...rest
  }: Partial<components['schemas']['TestPromptResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.post(ENDPOINT_API_MODELS_TEST, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      const responseData: components['schemas']['TestPromptResponse'] = {
        success,
        response: responseMessage,
        ...rest,
      };
      return response(200 as const).json(responseData);
    }),
  ];
}

export function mockTestApiModelError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.post(ENDPOINT_API_MODELS_TEST, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
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
export function mockFetchApiModels(
  {
    models = ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview'],
    ...rest
  }: Partial<components['schemas']['FetchModelsResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.post(ENDPOINT_API_MODELS_FETCH, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      const responseData: components['schemas']['FetchModelsResponse'] = {
        models,
        ...rest,
      };
      return response(200 as const).json(responseData);
    }),
  ];
}

export function mockFetchApiModelsError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.post(ENDPOINT_API_MODELS_FETCH, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
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
        source: 'api',
        id: 'test-api-model',
        api_format: 'openai',
        base_url: 'https://api.openai.com/v1',
        has_api_key: true, // Has API key
        models: [
          { id: 'gpt-4', object: 'model', created: 0, owned_by: 'openai', provider: 'openai' },
          { id: 'gpt-3.5-turbo', object: 'model', created: 0, owned_by: 'openai', provider: 'openai' },
        ],
        prefix: null,
        forward_all_with_prefix: false,
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
  return mockApiModels({
    data: [],
    total: 0,
    page: 1,
    page_size: 30,
  });
}

/**
 * Convenience methods for common test scenarios
 */
export function mockApiFormatsDefault() {
  return mockApiFormats({
    data: ['openai', 'openai_responses', 'anthropic', 'placeholder'],
  });
}

export function mockTestApiModelSuccess() {
  return mockTestApiModel({
    success: true,
    response: 'Connection successful',
  });
}

export function mockFetchApiModelsSuccess() {
  return mockFetchApiModels({
    models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview'],
  });
}

export function mockFetchApiModelsAuthError() {
  return mockFetchApiModelsError({
    code: 'authentication_error',
    message: 'Invalid API key',
    type: 'invalid_request_error',
    status: 500,
  });
}

export function mockCreateApiModelSuccess() {
  return mockCreateApiModel({
    id: 'test-model-123',
    api_format: 'openai',
    base_url: 'https://api.openai.com/v1',
    has_api_key: true, // Has API key
    models: [{ id: 'gpt-4', object: 'model', created: 0, owned_by: 'openai', provider: 'openai' }],
    prefix: null,
    forward_all_with_prefix: false,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  });
}

/**
 * Mock API model with API key (has_api_key: true)
 * Use this when testing scenarios where an API key exists
 */
export function mockApiModelWithKey(overrides?: Partial<components['schemas']['ApiAliasResponse']>) {
  return mockCreateApiModel({
    has_api_key: true,
    ...overrides,
  });
}

/**
 * Mock API model without API key (null)
 * Use this when testing public API scenarios or models without authentication
 */
export function mockApiModelWithoutKey(overrides?: Partial<components['schemas']['ApiAliasResponse']>) {
  return mockCreateApiModel({
    has_api_key: false,
    ...overrides,
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
    status: 500,
  });
}
