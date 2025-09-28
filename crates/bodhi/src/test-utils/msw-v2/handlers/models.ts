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
  return [
    typedHttp.get(ENDPOINT_MODELS, async ({ response: httpResponse }) => {
      const responseData: components['schemas']['PaginatedAliasResponse'] = {
        data,
        page,
        page_size,
        total,
        ...rest,
      };
      return httpResponse(200 as const).json(responseData);
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

// Error Handlers

export function mockModelsError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  return [
    typedHttp.get(ENDPOINT_MODELS, async ({ response }) => {
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
  return [
    typedHttp.post(ENDPOINT_MODELS, async ({ response: httpResponse }) => {
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
      return httpResponse(201 as const).json(responseData);
    }),
  ];
}

// Error Handlers

export function mockCreateModelError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  return [
    typedHttp.post(ENDPOINT_MODELS, async ({ response }) => {
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

export function mockCreateModelInternalError(config: {} = {}) {
  return mockCreateModelError({
    code: 'internal_server_error',
    message: 'Internal Server Error',
    type: 'internal_server_error',
    status: 500,
  });
}

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
  return [
    typedHttp.get(ENDPOINT_MODEL_ALIAS, async ({ response: httpResponse, params }) => {
      const { alias: paramAlias } = params;

      // Only respond if alias matches
      if (paramAlias !== alias) {
        return; // Pass through to next handler
      }

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
      return httpResponse(200 as const).json(responseData);
    }),
  ];
}

// Error Handlers

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
  return [
    typedHttp.get(ENDPOINT_MODEL_ALIAS, async ({ response, params }) => {
      const { alias: paramAlias } = params;

      // Only respond if alias matches
      if (paramAlias !== alias) {
        return; // Pass through to next handler
      }

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

export function mockGetModelNotFoundError(alias: string, config: {} = {}) {
  return mockGetModelError(alias, {
    code: 'not_found',
    type: 'not_found_error',
    status: 404,
  });
}

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
  return [
    typedHttp.put(ENDPOINT_MODEL_ID, async ({ response: httpResponse, params }) => {
      const { id: paramId } = params;

      // Only respond if id matches
      if (paramId !== id) {
        return; // Pass through to next handler
      }

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
      return httpResponse(200 as const).json(responseData);
    }),
  ];
}

// Error Handlers

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
  return [
    typedHttp.put(ENDPOINT_MODEL_ID, async ({ response, params }) => {
      const { id: paramId } = params;

      // Only respond if id matches
      if (paramId !== id) {
        return; // Pass through to next handler
      }

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

export function mockUpdateModelInternalError(id: string, config: {} = {}) {
  return mockUpdateModelError(id, {
    code: 'internal_server_error',
    message: 'Internal Server Error',
    type: 'internal_server_error',
    status: 500,
  });
}

export function mockUpdateModelBadRequestError(id: string, config: { message?: string } = {}) {
  const { message = 'Invalid request data' } = config;
  return mockUpdateModelError(id, {
    code: 'invalid_request',
    message,
    type: 'invalid_request_error',
    status: 400,
  });
}
