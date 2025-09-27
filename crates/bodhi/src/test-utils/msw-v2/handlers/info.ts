/**
 * Type-safe MSW v2 handlers for app info endpoint using patterns inspired by openapi-msw
 */
import { ENDPOINT_APP_INFO } from '@/hooks/useQuery';
import { http, HttpResponse, type components } from '../setup';

/**
 * Create type-safe MSW v2 handlers for app info endpoint
 * Uses generated OpenAPI types directly
 */
export function mockAppInfo(config: Partial<components['schemas']['AppInfo']> = {}) {
  return [
    http.get(ENDPOINT_APP_INFO, () => {
      const responseData: components['schemas']['AppInfo'] = {
        status: config.status || 'ready',
        version: config.version || '0.1.0',
      };
      return HttpResponse.json(responseData);
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
 */
export function mockAppInfoError(
  config: {
    status?: 401 | 403 | 500;
    code?: string;
    message?: string;
    delay?: number;
  } = {}
) {
  return [
    http.get(ENDPOINT_APP_INFO, () => {
      const response = HttpResponse.json(
        {
          error: {
            code: config.code || 'internal_error',
            message: config.message || 'Server error',
          },
        },
        { status: config.status || 500 }
      );

      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
    }),
  ];
}

/**
 * Create app info handler with delay support for loading state testing
 */
export function mockAppInfoWithDelay(config: Partial<components['schemas']['AppInfo']> & { delay?: number } = {}) {
  return [
    http.get(ENDPOINT_APP_INFO, () => {
      const responseData: components['schemas']['AppInfo'] = {
        status: config.status || 'ready',
        version: config.version || '0.1.0',
      };
      const response = HttpResponse.json(responseData);

      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
    }),
  ];
}
