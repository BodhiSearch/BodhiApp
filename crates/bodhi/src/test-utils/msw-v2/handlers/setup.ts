/**
 * Type-safe MSW v2 handlers for setup endpoint using openapi-msw
 */
import { ENDPOINT_APP_SETUP } from '@/hooks/useInfo';
import { delay } from 'msw';
import { typedHttp, type components, INTERNAL_SERVER_ERROR } from '../setup';

/**
 * Create type-safe MSW v2 handlers for setup endpoint
 * Uses openapi-msw for full type safety with OpenAPI schema enforcement
 */
export function mockSetup(
  { status = 'ready', ...rest }: Partial<components['schemas']['SetupResponse']> = {},
  delayMs?: number
) {
  return [
    typedHttp.post(ENDPOINT_APP_SETUP, async ({ response: responseFn }) => {
      if (delayMs) {
        await delay(delayMs);
      }
      const responseData: components['schemas']['SetupResponse'] = {
        status,
        ...rest,
      };
      return responseFn(200 as const).json(responseData);
    }),
  ];
}

export function mockSetupSuccess() {
  return mockSetup({ status: 'ready' });
}

export function mockSetupSuccessWithDelay(delayMs: number = 1000) {
  return mockSetup({ status: 'ready' }, delayMs);
}

export function mockSetupResourceAdmin() {
  return mockSetup({ status: 'resource-admin' });
}

/**
 * Create error handler for setup endpoint
 * Supports status codes defined in OpenAPI schema (400, 500)
 */
export function mockSetupError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = 400,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 500 } = {}) {
  return [
    typedHttp.post(ENDPOINT_APP_SETUP, async ({ response }) => {
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
