import {
  ENDPOINT_MODELS_REFRESH,
  ENDPOINT_QUEUE,
  ENDPOINT_MODELS,
  ENDPOINT_MODEL_ID,
  ENDPOINT_ALIAS,
  ENDPOINT_ALIAS_ID,
} from '@/hooks/models';
import { createMockUserAlias, createMockApiAlias, createMockModelAlias } from '@/test-fixtures/models';

import { typedHttp, type components, INTERNAL_SERVER_ERROR } from '../setup';

export function mockModels(
  {
    data = [],
    page = 1,
    page_size = 30,
    total = 0,
    ...rest
  }: Partial<components['schemas']['PaginatedAliasResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_MODELS, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
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

export function mockModelsDefault() {
  return mockModels({
    data: [createMockUserAlias()],
    total: 1,
    page: 1,
    page_size: 30,
  });
}

export function mockModelsWithApiModel() {
  return mockModels({
    data: [createMockApiAlias()],
    total: 1,
    page: 1,
    page_size: 30,
  });
}

export function mockModelsWithSourceModel() {
  return mockModels({
    data: [createMockModelAlias()],
    total: 1,
    page: 1,
    page_size: 30,
  });
}

export function mockModelsEmpty() {
  return mockModels({
    data: [],
    total: 0,
    page: 1,
    page_size: 30,
  });
}

export function mockModelsError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_MODELS, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
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

export function mockModelsInternalError(config: { message?: string } = {}) {
  const { message = 'Internal server error' } = config;
  return mockModelsError({
    code: 'internal_error',
    message,
    type: 'internal_server_error',
    status: 500,
  });
}

export function mockCreateModel(
  {
    id = 'test-uuid-create',
    alias = 'new-model',
    repo = createMockUserAlias().repo,
    filename = createMockUserAlias().filename,
    snapshot = createMockUserAlias().snapshot,
    request_params = {},
    context_params = [],
    model_params = {},
    source = 'user',
    created_at = new Date().toISOString(),
    updated_at = new Date().toISOString(),
    ...rest
  }: Partial<components['schemas']['UserAliasResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.post(ENDPOINT_ALIAS, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['UserAliasResponse'] = {
        id,
        alias,
        repo,
        filename,
        snapshot,
        request_params,
        context_params,
        model_params,
        source,
        created_at,
        updated_at,
        ...rest,
      };
      return response(201 as const).json(responseData);
    }),
  ];
}

export function mockCreateModelError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.post(ENDPOINT_ALIAS, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
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

export function mockCreateModelInternalError(_config: Record<string, never> = {}) {
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

export function mockGetModel(
  id: string,
  {
    alias = createMockUserAlias().alias,
    repo = createMockUserAlias().repo,
    filename = createMockUserAlias().filename,
    snapshot = createMockUserAlias().snapshot,
    request_params = {},
    context_params = [],
    model_params = {},
    source = 'user',
    ...rest
  }: Partial<Omit<components['schemas']['UserAliasResponse'], 'id'>> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_MODEL_ID, async ({ response, params }) => {
      const { id: paramId } = params;

      if (paramId !== id) {
        return; // undefined falls through to the next handler
      }

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['UserAliasResponse'] = {
        id: paramId as string,
        alias,
        repo,
        filename,
        snapshot,
        request_params,
        context_params,
        model_params,
        source,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        ...rest,
      };
      return response(200 as const).json(responseData);
    }),
  ];
}

export function mockGetModelError(
  id: string,
  {
    code = 'not_found',
    message,
    type = 'not_found_error',
    status = 404,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 404 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_MODEL_ID, async ({ response, params }) => {
      const { id: paramId } = params;

      if (paramId !== id) {
        return; // undefined falls through to the next handler
      }

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const errorBody = {
        code,
        message: message || `Model ${paramId} not found`,
        type,
        ...rest,
      };
      return response(status).json({ error: errorBody });
    }),
  ];
}

export function mockGetModelNotFoundError(id: string, _config: Record<string, never> = {}) {
  return mockGetModelError(id, {
    code: 'not_found',
    message: `Model ${id} not found`,
    type: 'not_found_error',
    status: 404,
  });
}

export function mockGetModelInternalError(id: string, _config: Record<string, never> = {}) {
  return mockGetModelError(id, {
    code: 'internal_error',
    message: 'Internal server error',
    type: 'internal_server_error',
    status: 500,
  });
}

export function mockUpdateModel(
  id: string,
  {
    alias = createMockUserAlias().alias,
    repo = createMockUserAlias().repo,
    filename = createMockUserAlias().filename,
    snapshot = createMockUserAlias().snapshot,
    request_params = {},
    context_params = [],
    model_params = {},
    source = 'user',
    ...rest
  }: Partial<Omit<components['schemas']['UserAliasResponse'], 'id'>> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.put(ENDPOINT_ALIAS_ID, async ({ response, params }) => {
      const { id: paramId } = params;

      if (paramId !== id) {
        return; // undefined falls through to the next handler
      }

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['UserAliasResponse'] = {
        id: paramId as string,
        alias,
        repo,
        filename,
        snapshot,
        request_params,
        context_params,
        model_params,
        source,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        ...rest,
      };
      return response(200 as const).json(responseData);
    }),
  ];
}

export function mockUpdateModelError(
  id: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.put(ENDPOINT_ALIAS_ID, async ({ response, params }) => {
      const { id: paramId } = params;

      if (paramId !== id) {
        return; // undefined falls through to the next handler
      }

      if (hasBeenCalled && !stub) return;
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

export function mockUpdateModelInternalError(id: string, _config: Record<string, never> = {}) {
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

export function mockRefreshAllMetadata(
  { num_queued = 'all', alias = null }: Partial<components['schemas']['RefreshResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.post(ENDPOINT_MODELS_REFRESH, async ({ request, response }) => {
      const body = await request.json();

      if (body?.source !== 'all') {
        return; // undefined falls through to the next handler
      }

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['RefreshResponse'] = {
        num_queued,
        alias,
      };
      return response(202 as const).json(responseData);
    }),
  ];
}

export function mockQueueStatus(status: 'idle' | 'processing' = 'idle', { stub }: { stub?: boolean } = {}) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_QUEUE, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['QueueStatusResponse'] = {
        status,
      };
      return response(200 as const).json(responseData);
    }),
  ];
}

export function mockRefreshSingleMetadata(
  {
    repo = createMockModelAlias().repo,
    filename = 'test-file.gguf',
    snapshot = createMockModelAlias().snapshot,
    alias = `${repo}/${filename}`,
    source: _source = 'model',
    metadata = null,
    ...rest
  }: Partial<components['schemas']['ModelAliasResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.post(ENDPOINT_MODELS_REFRESH, async ({ request, response }) => {
      const body = await request.json();

      if (
        body?.source !== 'model' ||
        body?.repo !== repo ||
        body?.filename !== filename ||
        body?.snapshot !== snapshot
      ) {
        return; // undefined falls through to the next handler
      }

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['ModelAliasResponse'] = {
        alias,
        repo,
        filename,
        snapshot,
        source: 'model',
        metadata,
        ...rest,
      };
      return response(200 as const).json(responseData);
    }),
  ];
}

export function mockRefreshAllMetadataError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = 'Failed to refresh metadata',
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.post(ENDPOINT_MODELS_REFRESH, async ({ request, response }) => {
      const body = await request.json();

      if (body?.source !== 'all') {
        return; // undefined falls through to the next handler
      }

      if (hasBeenCalled && !stub) return;
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
