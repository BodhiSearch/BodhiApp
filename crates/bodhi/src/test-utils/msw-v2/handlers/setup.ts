/**
 * Type-safe MSW v2 handlers for setup endpoint using patterns inspired by openapi-msw
 */
import { ENDPOINT_APP_SETUP } from '@/hooks/useQuery';
import { http, HttpResponse, type components } from '../setup';

/**
 * Create type-safe MSW v2 handlers for setup endpoint
 * Uses generated OpenAPI types directly
 */
export function mockSetup(config: Partial<components['schemas']['AppInfo']> = {}) {
  return [
    http.post(ENDPOINT_APP_SETUP, () => {
      const responseData: components['schemas']['AppInfo'] = {
        status: config.status || 'ready',
        version: config.version || '0.1.0',
      };
      return HttpResponse.json(responseData);
    }),
  ];
}

export function mockSetupSuccess() {
  return mockSetup({ status: 'ready' });
}

export function mockSetupSuccessWithDelay(delayMs: number = 1000) {
  return [
    http.post(ENDPOINT_APP_SETUP, async () => {
      await new Promise((resolve) => setTimeout(resolve, delayMs));
      const responseData: components['schemas']['AppInfo'] = {
        status: 'ready',
        version: '0.1.0',
      };
      return HttpResponse.json(responseData);
    }),
  ];
}

export function mockSetupResourceAdmin() {
  return mockSetup({ status: 'resource-admin' });
}

export function mockSetupError(
  config: {
    status?: 400 | 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    http.post(ENDPOINT_APP_SETUP, () => {
      return HttpResponse.json(
        {
          error: {
            message: config.message || 'Setup failed',
          },
        },
        { status: config.status || 400 }
      );
    }),
  ];
}
