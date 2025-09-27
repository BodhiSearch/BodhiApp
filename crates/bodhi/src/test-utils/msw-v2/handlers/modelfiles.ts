/**
 * Type-safe MSW v2 handlers for modelfiles endpoint using openapi-msw
 */
import { ENDPOINT_MODEL_FILES, ENDPOINT_MODEL_FILES_PULL } from '@/hooks/useQuery';
import { typedHttp } from '../openapi-msw-setup';
import type { components } from '../setup';

/**
 * Create type-safe MSW v2 handlers for modelfiles endpoint
 * Uses generated OpenAPI types directly
 */
export function mockModelFiles(config: Partial<components['schemas']['PaginatedLocalModelResponse']> = {}) {
  return [
    typedHttp.get(ENDPOINT_MODEL_FILES, ({ response }) => {
      const responseData: components['schemas']['PaginatedLocalModelResponse'] = {
        data: config.data || [],
        page: config.page || 1,
        page_size: config.page_size || 30,
        total: config.total || 0,
      };
      return response(200).json(responseData);
    }),
  ];
}

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

export function mockModelFilesEmpty() {
  return mockModelFiles({ data: [], total: 0 });
}

export function mockModelFilesError(
  config: {
    status?: 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    typedHttp.get(ENDPOINT_MODEL_FILES, ({ response }) => {
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

/**
 * Create type-safe MSW v2 handlers for model pull downloads endpoint
 * Uses generated OpenAPI types directly
 */
export function mockModelPullDownloads(config: Partial<components['schemas']['PaginatedDownloadResponse']> = {}) {
  return [
    typedHttp.get(ENDPOINT_MODEL_FILES_PULL, ({ response }) => {
      const responseData: components['schemas']['PaginatedDownloadResponse'] = {
        data: config.data || [],
        page: config.page || 1,
        page_size: config.page_size || 30,
        total: config.total || 0,
      };
      return response(200).json(responseData);
    }),
  ];
}

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

export function mockModelPullDownloadsEmpty() {
  return mockModelPullDownloads({ data: [], total: 0 });
}

export function mockModelPullDownloadsError(
  config: {
    status?: 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    typedHttp.get(ENDPOINT_MODEL_FILES_PULL, ({ response }) => {
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

/**
 * Create type-safe MSW v2 handlers for model pull POST endpoint
 * Uses generated OpenAPI types directly
 */
export function mockModelPull(config: Partial<components['schemas']['DownloadRequest']> & { delay?: number } = {}) {
  return [
    typedHttp.post(ENDPOINT_MODEL_FILES_PULL, ({ response }) => {
      const responseData: components['schemas']['DownloadRequest'] = {
        id: config.id || '123',
        repo: config.repo || 'test/repo1',
        filename: config.filename || 'model1.gguf',
        status: config.status || 'pending',
        error: config.error || null,
        created_at: config.created_at || new Date().toISOString(),
        updated_at: config.updated_at || new Date().toISOString(),
        total_bytes: config.total_bytes || null,
        downloaded_bytes: config.downloaded_bytes,
        started_at: config.started_at || new Date().toISOString(),
      };
      const responseResult = response(201).json(responseData);

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(responseResult), config.delay))
        : responseResult;
    }),
  ];
}

/**
 * Error handler for model pull POST endpoint
 */
export function mockModelPullError(
  config: {
    status?: 400 | 500;
    code?: string;
    message?: string;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.post(ENDPOINT_MODEL_FILES_PULL, ({ response }) => {
      const responseResult = response(config.status || 400).json({
        error: {
          code: config.code || 'pull_error-file_already_exists',
          message: config.message || 'file "model.gguf" already exists in repo "test/repo" with snapshot "main"',
          type: 'invalid_request_error',
        },
      });

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(responseResult), config.delay))
        : responseResult;
    }),
  ];
}
