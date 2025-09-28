/**
 * Type-safe MSW v2 handlers for models endpoint using openapi-msw
 */
import { ENDPOINT_MODELS, ENDPOINT_MODEL_ALIAS, ENDPOINT_MODEL_ID } from '@/hooks/useQuery';
import { typedHttp, type components, INTERNAL_SERVER_ERROR } from '../openapi-msw-setup';

// ============================================================================
// Models List Endpoint (/bodhi/v1/models GET)
// ============================================================================

// Success Handlers

/**
 * Create type-safe MSW v2 handlers for models list endpoint
 * Uses generated OpenAPI types directly
 */
export function mockModels({
  data = [],
  page = 1,
  page_size = 30,
  total = 0,
  ...rest
}: Partial<components['schemas']['PaginatedAliasResponse']> = {}) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_MODELS, async ({ response }) => {
      if (hasBeenCalled) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['PaginatedAliasResponse'] = {
        data,
        page,
        page_size,
        total,
        ...rest,
      };
      return response(200 as const).json(responseData);
    }),
  ];
}

// Success Handler Variants

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

/**
 * Mock handler for models list with API model data
 * Uses delegation pattern
 */
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

/**
 * Mock handler for models list with source model data
 * Uses delegation pattern
 */
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

/**
 * Mock handler for empty models list
 * Uses delegation pattern
 */
export function mockModelsEmpty() {
  return mockModels({
    data: [],
    total: 0,
    page: 1,
    page_size: 30,
  });
}

// Error Handlers

/**
 * Mock handler for models list error endpoint
 * Uses generated OpenAPI types directly
 */
export function mockModelsError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_MODELS, async ({ response }) => {
      if (hasBeenCalled) return;
      hasBeenCalled = true;

      const errorBody = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorBody });
    }),
  ];
}

// Error Handler Variants

/**
 * Mock handler for models list internal server error
 * Uses delegation pattern
 */
export function mockModelsInternalError(config: { message?: string } = {}) {
  const { message = 'Internal server error' } = config;
  return mockModelsError({
    code: 'internal_error',
    message,
    type: 'internal_server_error',
    status: 500,
  });
}

// ============================================================================
// Create Model Endpoint (/bodhi/v1/models POST)
// ============================================================================

// Success Handlers

/**
 * Create type-safe MSW v2 handler for model creation endpoint
 */
export function mockCreateModel({
  alias = 'new-model',
  repo = 'test-repo',
  filename = 'test-file.bin',
  snapshot = 'abc123',
  request_params = {},
  context_params = [],
  model_params = {},
  source = 'user',
  ...rest
}: Partial<components['schemas']['UserAliasResponse']> = {}) {
  let hasBeenCalled = false;

  return [
    typedHttp.post(ENDPOINT_MODELS, async ({ response }) => {
      if (hasBeenCalled) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['UserAliasResponse'] = {
        alias,
        repo,
        filename,
        snapshot,
        request_params,
        context_params,
        model_params,
        source,
        ...rest,
      };
      return response(201 as const).json(responseData);
    }),
  ];
}

// Error Handlers

/**
 * Mock handler for model creation error endpoint
 * Uses generated OpenAPI types directly
 */
export function mockCreateModelError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  let hasBeenCalled = false;

  return [
    typedHttp.post(ENDPOINT_MODELS, async ({ response }) => {
      if (hasBeenCalled) return;
      hasBeenCalled = true;

      const errorBody = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorBody });
    }),
  ];
}

// Error Handler Variants

/**
 * Mock handler for model creation internal server error
 * Uses delegation pattern
 */
export function mockCreateModelInternalError(config: {} = {}) {
  return mockCreateModelError({
    code: 'internal_server_error',
    message: 'Internal Server Error',
    type: 'internal_server_error',
    status: 500,
  });
}

/**
 * Mock handler for model creation bad request error
 * Uses delegation pattern
 */
export function mockCreateModelBadRequestError(config: { message?: string } = {}) {
  const { message = 'Invalid request data' } = config;
  return mockCreateModelError({
    code: 'invalid_request',
    message,
    type: 'invalid_request_error',
    status: 400,
  });
}

// ============================================================================
// Get Model Endpoint (/bodhi/v1/models/{alias} GET)
// ============================================================================

// Success Handlers

/**
 * Create type-safe MSW v2 handler for individual model retrieval
 */
export function mockGetModel(
  alias: string,
  {
    repo = 'test-repo',
    filename = 'test-file.bin',
    snapshot = 'abc123',
    request_params = {},
    context_params = [],
    model_params = {},
    source = 'user',
    ...rest
  }: Partial<Omit<components['schemas']['UserAliasResponse'], 'alias'>> = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_MODEL_ALIAS, async ({ response, params }) => {
      const { alias: paramAlias } = params;

      // Only respond if alias matches
      if (paramAlias !== alias) {
        return; // Pass through to next handler
      }

      if (hasBeenCalled) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['UserAliasResponse'] = {
        alias: paramAlias as string,
        repo,
        filename,
        snapshot,
        request_params,
        context_params,
        model_params,
        source,
        ...rest,
      };
      return response(200 as const).json(responseData);
    }),
  ];
}

// Error Handlers

/**
 * Mock handler for individual model retrieval error
 * Uses generated OpenAPI types directly
 */
export function mockGetModelError(
  alias: string,
  {
    code = 'not_found',
    message,
    type = 'not_found_error',
    status = 404,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 404 | 500 } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_MODEL_ALIAS, async ({ response, params }) => {
      const { alias: paramAlias } = params;

      // Only respond if alias matches
      if (paramAlias !== alias) {
        return; // Pass through to next handler
      }

      if (hasBeenCalled) return;
      hasBeenCalled = true;

      const errorBody = {
        code,
        message: message || `Model ${paramAlias} not found`,
        type,
        ...rest,
      };
      return response(status).json({ error: errorBody });
    }),
  ];
}

// Error Handler Variants

/**
 * Mock handler for model not found error
 * Uses delegation pattern
 */
export function mockGetModelNotFoundError(alias: string, config: {} = {}) {
  return mockGetModelError(alias, {
    code: 'not_found',
    message: `Model ${alias} not found`,
    type: 'not_found_error',
    status: 404,
  });
}

/**
 * Mock handler for individual model retrieval internal server error
 * Uses delegation pattern
 */
export function mockGetModelInternalError(alias: string, config: {} = {}) {
  return mockGetModelError(alias, {
    code: 'internal_error',
    message: 'Internal server error',
    type: 'internal_server_error',
    status: 500,
  });
}

// ============================================================================
// Update Model Endpoint (/bodhi/v1/models/{id} PUT)
// ============================================================================

// Success Handlers

/**
 * Create type-safe MSW v2 handler for model update endpoint
 */
export function mockUpdateModel(
  id: string,
  {
    repo = 'test-repo',
    filename = 'test-file.bin',
    snapshot = 'abc123',
    request_params = {},
    context_params = [],
    model_params = {},
    source = 'user',
    ...rest
  }: Partial<Omit<components['schemas']['UserAliasResponse'], 'alias'>> = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.put(ENDPOINT_MODEL_ID, async ({ response, params }) => {
      const { id: paramId } = params;

      // Only respond if id matches
      if (paramId !== id) {
        return; // Pass through to next handler
      }

      if (hasBeenCalled) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['UserAliasResponse'] = {
        alias: paramId as string,
        repo,
        filename,
        snapshot,
        request_params,
        context_params,
        model_params,
        source,
        ...rest,
      };
      return response(200 as const).json(responseData);
    }),
  ];
}

// Error Handlers

/**
 * Mock handler for model update error endpoint
 * Uses generated OpenAPI types directly
 */
export function mockUpdateModelError(
  id: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.put(ENDPOINT_MODEL_ID, async ({ response, params }) => {
      const { id: paramId } = params;

      // Only respond if id matches
      if (paramId !== id) {
        return; // Pass through to next handler
      }

      if (hasBeenCalled) return;
      hasBeenCalled = true;

      const errorBody = {
        code,
        message: message || `Failed to update model ${paramId}`,
        type,
        ...rest,
      };
      return response(status).json({ error: errorBody });
    }),
  ];
}

// Error Handler Variants

/**
 * Mock handler for model update internal server error
 * Uses delegation pattern
 */
export function mockUpdateModelInternalError(id: string, config: {} = {}) {
  return mockUpdateModelError(id, {
    code: 'internal_server_error',
    message: 'Internal Server Error',
    type: 'internal_server_error',
    status: 500,
  });
}

/**
 * Mock handler for model update bad request error
 * Uses delegation pattern
 */
export function mockUpdateModelBadRequestError(id: string, config: { message?: string } = {}) {
  const { message = 'Invalid request data' } = config;
  return mockUpdateModelError(id, {
    code: 'invalid_request',
    message,
    type: 'invalid_request_error',
    status: 400,
  });
}
