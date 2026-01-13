/**
 * MSW v2 server setup and configuration with type-safe patterns using openapi-msw
 *
 * This setup provides:
 * - OpenAPI MSW typed HTTP handlers for full type safety
 * - MSW v2 server setup and configuration
 * - Type-safe mocking using generated OpenAPI types
 * - Clean handler creation patterns for both typed and standard handlers
 *
 * Key patterns:
 * 1. Use typedHttp for type-safe OpenAPI-based handlers
 * 2. Use standard http for general MSW handlers
 * 3. Use generated types from @bodhiapp/ts-client as single source of truth
 * 4. Use createTypedResponse helper for consistent response creation
 *
 * Example usage in handlers:
 * ```typescript
 * import { typedHttp, http, type components } from '../setup';
 *
 * // OpenAPI typed handler (preferred)
 * export function createTypedHandlers() {
 *   return [
 *     typedHttp.get('/api/endpoint', ({ response }) => {
 *       const responseData: components['schemas']['YourType'] = { ... };
 *       return response(200).json(responseData);
 *     })
 *   ];
 * }
 *
 * // Standard MSW handler
 * export function createStandardHandlers() {
 *   return [
 *     http.get('/api/endpoint', () => {
 *       return HttpResponse.json(responseData);
 *     })
 *   ];
 * }
 * ```
 */
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { createOpenApiHttp } from 'openapi-msw';

// Export types from ts-client for use in tests
export type { components, paths } from '@bodhiapp/ts-client';

// Default internal server error values for error handlers
export const INTERNAL_SERVER_ERROR = {
  code: 'internal_error',
  message: 'Internal server error',
  type: 'internal_server_error',
  status: 500,
} as const;

// Create typed HTTP handler using OpenAPI schema
export const typedHttp = createOpenApiHttp<import('@bodhiapp/ts-client').paths>();

// Re-export MSW v2 http and HttpResponse for convenience
export { http, HttpResponse };

// Create MSW v2 server instance
export const server = setupServer();

// Standard setup functions for tests
export function setupMswV2() {
  beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }));
  afterEach(() => server.resetHandlers());
  afterAll(() => server.close());
}

// Type-safe response helper inspired by openapi-msw patterns
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function createTypedResponse<T extends Record<string, any>>(status: number, data: T) {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return HttpResponse.json(data as any, { status });
}
