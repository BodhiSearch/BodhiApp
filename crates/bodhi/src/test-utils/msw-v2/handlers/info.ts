/**
 * Type-safe MSW v2 handlers for app info endpoint using openapi-msw
 */
import { ENDPOINT_APP_INFO } from '@/hooks/useQuery';
import { typedHttp } from '../openapi-msw-setup';
import type { components } from '../setup';

/**
 * Create type-safe MSW v2 handlers for app info endpoint
 * Uses openapi-msw for full type safety with OpenAPI schema enforcement
 */
export function mockAppInfo(config: Partial<components['schemas']['AppInfo']> = {}) {
  return [
    typedHttp.get(ENDPOINT_APP_INFO, ({ response }) => {
      return response(200).json({
        status: config.status || 'ready',
        version: config.version || '0.1.0',
      });
    }),
  ];
}

export function mockAppInfoReady() {
  return mockAppInfo({ status: 'ready' });
}

export function mockAppInfoSetup() {
  return mockAppInfo({ status: 'setup' });
}

export function mockAppInfoResourceAdmin() {
  return mockAppInfo({ status: 'resource-admin' });
}

/**
 * Create error handler for app info endpoint
 * Only supports status codes defined in OpenAPI schema (500)
 */
export function mockAppInfoError(
  config: {
    status?: 500; // Only 500 is defined in OpenAPI schema for this endpoint
    code?: string;
    message?: string;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.get(ENDPOINT_APP_INFO, ({ response }) => {
      const errorResponse = response(config.status || 500).json({
        error: {
          code: config.code || 'internal_error',
          message: config.message || 'Server error',
          type: 'internal_server_error',
        },
      });

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(errorResponse), config.delay))
        : errorResponse;
    }),
  ];
}

/**
 * Create app info handler with delay support for loading state testing
 */
export function mockAppInfoWithDelay(config: Partial<components['schemas']['AppInfo']> & { delay?: number } = {}) {
  return [
    typedHttp.get(ENDPOINT_APP_INFO, ({ response }) => {
      const successResponse = response(200).json({
        status: config.status || 'ready',
        version: config.version || '0.1.0',
      });

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(successResponse), config.delay))
        : successResponse;
    }),
  ];
}
