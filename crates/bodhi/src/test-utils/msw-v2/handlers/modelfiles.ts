/**
 * Type-safe MSW v2 handlers for modelfiles endpoint using openapi-msw
 */
import { ENDPOINT_MODEL_FILES, ENDPOINT_MODEL_FILES_PULL } from '@/hooks/useModels';

import { typedHttp, type components, INTERNAL_SERVER_ERROR } from '../setup';

// ============================================================================
// Model Files Endpoint (/bodhi/v1/modelfiles)
// ============================================================================

// Success Handlers

/**
 * Create type-safe MSW v2 handlers for modelfiles endpoint
 * Uses generated OpenAPI types directly
 */
export function mockModelFiles(
  {
    data = [],
    page = 1,
    page_size = 30,
    total = 0,
    ...rest
  }: Partial<components['schemas']['PaginatedLocalModelResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_MODEL_FILES, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['PaginatedLocalModelResponse'] = {
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

/**
 * Mock handler for model files error endpoint
 * Uses generated OpenAPI types directly
 */
export function mockModelFilesError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_MODEL_FILES, async ({ response }) => {
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

// Success Handler Variants

/**
 * Mock handler for model files default data
 * Uses generated OpenAPI types directly
 */
export function mockModelFilesDefault() {
  return mockModelFiles({
    data: [
      {
        repo: 'test-repo',
        filename: 'test-file.txt',
        size: 1073741824, // 1 GB
        snapshot: 'abc123',
        model_params: {},
      },
    ],
    total: 1,
    page: 1,
    page_size: 30,
  });
}

/**
 * Mock handler for model files empty data
 * Uses generated OpenAPI types directly
 */
export function mockModelFilesEmpty() {
  return mockModelFiles({
    data: [],
    page: 1,
    page_size: 30,
    total: 0,
  });
}

// ============================================================================
// Model Pull Downloads Endpoint (/bodhi/v1/modelfiles/pull GET)
// ============================================================================

// Success Handlers

/**
 * Create type-safe MSW v2 handlers for model pull downloads endpoint
 * Uses generated OpenAPI types directly
 */
export function mockModelPullDownloads(
  {
    data = [],
    page = 1,
    page_size = 30,
    total = 0,
    ...rest
  }: Partial<components['schemas']['PaginatedDownloadResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_MODEL_FILES_PULL, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['PaginatedDownloadResponse'] = {
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

/**
 * Mock handler for model pull downloads error endpoint
 * Uses generated OpenAPI types directly
 */
export function mockModelPullDownloadsError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_MODEL_FILES_PULL, async ({ response }) => {
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

// Success Handler Variants

/**
 * Mock handler for model pull downloads default data
 * Uses generated OpenAPI types directly
 */
export function mockModelPullDownloadsDefault() {
  return mockModelPullDownloads({
    data: [
      {
        id: '1',
        repo: 'test/repo1',
        filename: 'model1.gguf',
        status: 'pending',
        error: null,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
        total_bytes: 1000000,
        downloaded_bytes: 500000,
        started_at: '2024-01-01T00:00:00Z',
      },
      {
        id: '2',
        repo: 'test/repo2',
        filename: 'model2.gguf',
        status: 'completed',
        error: null,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
        total_bytes: 2000000,
        downloaded_bytes: 2000000,
        started_at: '2024-01-01T00:00:00Z',
      },
      {
        id: '3',
        repo: 'test/repo3',
        filename: 'model3.gguf',
        status: 'error',
        error: 'Download failed',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
        total_bytes: 1500000,
        downloaded_bytes: 750000,
        started_at: '2024-01-01T00:00:00Z',
      },
    ],
    total: 3,
    page: 1,
    page_size: 30,
  });
}

/**
 * Mock handler for model pull downloads empty data
 * Uses generated OpenAPI types directly
 */
export function mockModelPullDownloadsEmpty() {
  return mockModelPullDownloads({
    data: [],
    page: 1,
    page_size: 30,
    total: 0,
  });
}

// Error Handler Variants

/**
 * Mock handler for model pull downloads internal server error
 */
export function mockModelPullDownloadsInternalError() {
  return mockModelPullDownloadsError({
    code: 'internal_server_error',
    message: 'Internal Server Error',
    type: 'internal_server_error',
    status: 500,
  });
}

// ============================================================================
// Model Pull POST Endpoint (/bodhi/v1/modelfiles/pull POST)
// ============================================================================

// Success Handlers

/**
 * Create type-safe MSW v2 handlers for model pull POST endpoint
 * Uses generated OpenAPI types directly
 */
/**
 * Mock handler for model pull POST endpoint
 * Uses generated OpenAPI types directly
 */
export function mockModelPull(
  {
    id = '123',
    repo = 'test/repo1',
    filename = 'model1.gguf',
    status = 'pending',
    error = null,
    created_at = new Date().toISOString(),
    updated_at = new Date().toISOString(),
    total_bytes = null,
    downloaded_bytes,
    started_at = new Date().toISOString(),
    ...rest
  }: Partial<components['schemas']['DownloadRequest']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.post(ENDPOINT_MODEL_FILES_PULL, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['DownloadRequest'] = {
        id,
        repo,
        filename,
        status,
        error,
        created_at,
        updated_at,
        total_bytes,
        downloaded_bytes,
        started_at,
        ...rest,
      };

      return response(201 as const).json(responseData);
    }),
  ];
}

// Error Handlers

/**
 * Mock handler for model pull error endpoint
 * Uses generated OpenAPI types directly
 */
export function mockModelPullError(
  {
    code = 'pull_error-file_already_exists',
    message = 'file "model.gguf" already exists in repo "test/repo" with snapshot "main"',
    type = 'invalid_request_error',
    status = 400,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.post(ENDPOINT_MODEL_FILES_PULL, async ({ response }) => {
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

// Error Handler Variants

/**
 * Mock handler for model pull file already exists error
 */
export function mockModelPullFileExistsError(config: { repo?: string; filename?: string } = {}) {
  const { repo = 'test/repo', filename = 'model.gguf' } = config;
  return mockModelPullError({
    code: 'pull_error-file_already_exists',
    message: `file "${filename}" already exists in repo "${repo}" with snapshot "main"`,
    type: 'invalid_request_error',
    status: 400,
  });
}

/**
 * Mock handler for model pull internal server error
 */
export function mockModelPullInternalError() {
  return mockModelPullError({
    code: 'internal_server_error',
    message: 'Internal Server Error',
    type: 'internal_server_error',
    status: 500,
  });
}
