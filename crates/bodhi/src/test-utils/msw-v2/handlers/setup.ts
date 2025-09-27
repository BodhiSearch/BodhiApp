/**
 * Type-safe MSW v2 handlers for setup endpoint using openapi-msw
 */
import { ENDPOINT_APP_SETUP } from '@/hooks/useQuery';
import { typedHttp } from '../openapi-msw-setup';
import type { components } from '../setup';

/**
 * Create type-safe MSW v2 handlers for setup endpoint
 * Uses openapi-msw for full type safety with OpenAPI schema enforcement
 */
export function mockSetup(config: Partial<components['schemas']['SetupResponse']> = {}) {
  return [
    typedHttp.post(ENDPOINT_APP_SETUP, ({ response }) => {
      return response(200).json({
        status: config.status || 'ready',
      });
    }),
  ];
}

export function mockSetupSuccess() {
  return mockSetup({ status: 'ready' });
}

export function mockSetupSuccessWithDelay(delayMs: number = 1000) {
  return [
    typedHttp.post(ENDPOINT_APP_SETUP, async ({ response }) => {
      await new Promise((resolve) => setTimeout(resolve, delayMs));
      return response(200).json({
        status: 'ready',
      });
    }),
  ];
}

export function mockSetupResourceAdmin() {
  return mockSetup({ status: 'resource-admin' });
}

/**
 * Create error handler for setup endpoint
 * Supports status codes defined in OpenAPI schema (400, 500)
 */
export function mockSetupError(
  config: {
    status?: 400 | 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    typedHttp.post(ENDPOINT_APP_SETUP, ({ response }) => {
      return response(config.status || 400).json({
        error: {
          code: config.code || 'validation_error',
          message: config.message || 'Setup failed',
          type: config.status === 500 ? 'internal_server_error' : 'invalid_request_error',
        },
      });
    }),
  ];
}
