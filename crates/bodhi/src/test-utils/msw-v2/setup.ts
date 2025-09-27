/**
 * MSW v2 server setup and configuration with type-safe patterns inspired by openapi-msw
 *
 * This setup provides:
 * - Dual MSW v1/v2 compatibility (uses MSW v2 via 'msw2' alias)
 * - Type-safe mocking using generated OpenAPI types
 * - Clean handler creation patterns inspired by openapi-msw library
 *
 * Key patterns:
 * 1. Use generated types from openapi-schema.ts as single source of truth
 * 2. Create handlers with explicit TypeScript types for responses
 * 3. Use createTypedResponse helper for consistent response creation
 *
 * Example usage in handlers:
 * ```typescript
 * import { http, type components } from '../setup';
 *
 * export function createTypedHandlers(config: Partial<components['schemas']['YourType']> = {}) {
 *   return [
 *     http.get('/your/endpoint', () => {
 *       const responseData: components['schemas']['YourType'] = {
 *         field: config.field || 'default'
 *       };
 *       return HttpResponse.json(responseData);
 *     })
 *   ];
 * }
 * ```
 */
import { setupServer } from 'msw2/node';
import { http, HttpResponse } from 'msw2';

// Export types from generated schema for use in tests
export type { paths, components } from '../generated/openapi-schema';

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
export function createTypedResponse<T extends Record<string, any>>(status: number, data: T) {
  return HttpResponse.json(data as any, { status });
}
