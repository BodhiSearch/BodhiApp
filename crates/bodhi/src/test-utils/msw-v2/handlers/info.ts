import { delay } from 'msw';

import { ENDPOINT_APP_INFO } from '@/hooks/info';
import { typedHttp, type components, INTERNAL_SERVER_ERROR } from '@/test-utils/msw-v2/setup';

export function mockAppInfo(
  {
    status = 'ready',
    version = '0.1.0',
    deployment = 'standalone',
    reference_api_url = 'https://api.getbodhi.app/',
    ...rest
  }: Partial<components['schemas']['AppInfo']> = {},
  { delayMs, stub }: { delayMs?: number; stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_APP_INFO, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      if (delayMs) {
        await delay(delayMs);
      }

      const responseData: components['schemas']['AppInfo'] = {
        commit_sha: 'not-set',
        status,
        version,
        deployment,
        url: 'http://localhost:1135',
        reference_api_url,
        ...rest,
      };

      return response(200 as const).json(responseData);
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
  return mockAppInfo({ status: 'resource_admin' });
}

export function mockAppInfoError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_APP_INFO, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
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

export function mockAppInfoInternalError() {
  return mockAppInfoError({
    code: 'internal_server_error',
    message: 'API Error',
    type: 'internal_server_error',
    status: 500,
  });
}
