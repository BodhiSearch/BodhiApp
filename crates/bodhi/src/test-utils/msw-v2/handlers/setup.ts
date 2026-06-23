import { delay } from 'msw';

import { ENDPOINT_APP_SETUP } from '@/hooks/info';
import { typedHttp, type components, INTERNAL_SERVER_ERROR } from '@/test-utils/msw-v2/setup';

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
  return mockSetup({ status: 'resource_admin' });
}

export function mockSetupError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = 400,
  ...rest
}: Partial<components['schemas']['BodhiError']> & { status?: 400 | 500 } = {}) {
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
