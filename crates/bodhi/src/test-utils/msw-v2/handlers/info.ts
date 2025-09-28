/**
 * Type-safe MSW v2 handlers for app info endpoint using openapi-msw
 */
import { ENDPOINT_APP_INFO } from '@/hooks/useQuery';
import { delay } from 'msw';
import { typedHttp, type components, INTERNAL_SERVER_ERROR } from '../openapi-msw-setup';

// ============================================================================
// Stub Handlers (No closure state - for tests requiring multiple responses)
// ============================================================================

/**
 * Core response logic for app info endpoint (shared by mock and stub handlers)
 * @private
 */
async function createAppInfoResponse(
  { status = 'ready', version = '0.1.0', ...rest }: Partial<components['schemas']['AppInfo']> = {},
  delayMs: number | undefined,
  response: any
) {
  if (delayMs) {
    await delay(delayMs);
  }
  const responseData: components['schemas']['AppInfo'] = {
    status,
    version,
    ...rest,
  };

  return response(200 as const).json(responseData);
}

/**
 * Stub handler for app info endpoint that responds every time (no closure state)
 * Use this for tests that require multiple API calls to the same endpoint
 */
export function stubAppInfo(
  { status = 'ready', version = '0.1.0', ...rest }: Partial<components['schemas']['AppInfo']> = {},
  delayMs?: number
) {
  return [
    typedHttp.get(ENDPOINT_APP_INFO, async ({ response }) => {
      return createAppInfoResponse({ status, version, ...rest }, delayMs, response);
    }),
  ];
}

// ============================================================================
// Success Handlers
// ============================================================================

/**
 * Create type-safe MSW v2 handlers for app info endpoint
 * Uses openapi-msw for full type safety with OpenAPI schema enforcement
 */
export function mockAppInfo(
  { status = 'ready', version = '0.1.0', ...rest }: Partial<components['schemas']['AppInfo']> = {},
  delayMs?: number
) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_APP_INFO, async ({ response }) => {
      if (hasBeenCalled) return;
      hasBeenCalled = true;
      return createAppInfoResponse({ status, version, ...rest }, delayMs, response);
    }),
  ];
}

// ============================================================================
// Success Handler Variants
// ============================================================================

/**
 * Mock handler for app info endpoint with ready status
 * Uses generated OpenAPI types directly
 */
export function mockAppInfoReady() {
  return mockAppInfo({ status: 'ready' });
}

/**
 * Mock handler for app info endpoint with setup status
 * Uses generated OpenAPI types directly
 */
export function mockAppInfoSetup() {
  return mockAppInfo({ status: 'setup' });
}

/**
 * Mock handler for app info endpoint with resource-admin status
 * Uses generated OpenAPI types directly
 */
export function mockAppInfoResourceAdmin() {
  return mockAppInfo({ status: 'resource-admin' });
}

// ============================================================================
// Error Handlers
// ============================================================================

/**
 * Create error handler for app info endpoint
 * Supports common HTTP status codes: 400, 401, 403, 500
 */
export function mockAppInfoError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 500 } = {}) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_APP_INFO, async ({ response }) => {
      if (hasBeenCalled) return;
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

// ============================================================================
// Error Handler Variants
// ============================================================================

/**
 * Mock handler for app info endpoint internal server error
 * Uses generated OpenAPI types directly
 */
export function mockAppInfoInternalError() {
  return mockAppInfoError({
    code: 'internal_server_error',
    message: 'API Error',
    type: 'internal_server_error',
    status: 500,
  });
}
