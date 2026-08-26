// Prefer typedHttp (type-checked against ts-client paths) over plain http; use createTypedResponse for arbitrary-status responses.
// Example: typedHttp.get('/api/endpoint', ({ response }) => response(200).json(data))
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { createOpenApiHttp } from 'openapi-msw';

export type { components, paths } from '@bodhiapp/ts-client';

export const INTERNAL_SERVER_ERROR = {
  code: 'internal_error',
  message: 'Internal server error',
  type: 'internal_server_error',
  status: 500,
} as const;

export const typedHttp = createOpenApiHttp<import('@bodhiapp/ts-client').paths>();

export { http, HttpResponse };

export const server = setupServer();

export function setupMswV2() {
  beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }));
  afterEach(() => server.resetHandlers());
  afterAll(() => server.close());
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function createTypedResponse<T extends Record<string, any>>(status: number, data: T) {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return HttpResponse.json(data as any, { status });
}
