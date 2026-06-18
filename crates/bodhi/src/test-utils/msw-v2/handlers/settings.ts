import { delay } from 'msw';

import { ENDPOINT_SETTINGS, ENDPOINT_SETTING_KEY } from '@/hooks/settings';

import { HttpResponse, INTERNAL_SERVER_ERROR, typedHttp, type components } from '../setup';

export function mockSettings(
  settings: components['schemas']['SettingInfo'][] = [],
  { delayMs, stub }: { delayMs?: number; stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_SETTINGS, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      if (delayMs) {
        await delay(delayMs);
      }
      const responseData: components['schemas']['SettingInfo'][] = settings;
      return response(200 as const).json(responseData);
    }),
  ];
}

export function mockSettingsDefault() {
  return mockSettings([
    {
      key: 'BODHI_HOME',
      current_value: '/home/user/.bodhi',
      default_value: '/home/user/.bodhi',
      source: 'default',
      metadata: {
        type: 'string',
      },
    },
    {
      key: 'BODHI_LOG_LEVEL',
      current_value: 'info',
      default_value: 'warn',
      source: 'settings_file',
      metadata: {
        type: 'option',
        options: ['error', 'warn', 'info', 'debug', 'trace'],
      },
    },
    {
      key: 'BODHI_PORT',
      current_value: 1135,
      default_value: 1135,
      source: 'default',
      metadata: {
        type: 'number',
        min: 1025,
        max: 65535,
      },
    },
    {
      key: 'BODHI_EXEC_VARIANT',
      current_value: 'cpu',
      default_value: 'metal',
      source: 'settings_file',
      metadata: {
        type: 'string',
      },
    },
  ]);
}

export function mockSettingsEmpty() {
  return mockSettings([]);
}

export function mockSettingsError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_SETTINGS, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      const errorBody = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorBody });
    }),
  ];
}

export function mockSettingsInternalError() {
  return mockSettingsError({
    code: 'internal_error',
    message: 'Test Error',
    type: 'internal_server_error',
    status: 500,
  });
}

/**
 * Responds only to the specified key; unmatched keys 404 via the catch-all.
 */
export function mockUpdateSetting(
  key: string,
  {
    current_value = 'updated-value',
    default_value = 'default-value',
    source = 'settings_file',
    metadata = { type: 'string' },
    ...rest
  }: Partial<Omit<components['schemas']['SettingInfo'], 'key'>> = {},
  { delayMs, stub }: { delayMs?: number; stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.put(ENDPOINT_SETTING_KEY, async ({ params, response }) => {
      // match key before consuming the one-shot guard, or non-matching requests exhaust it
      if (params.key !== key) return;

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      if (delayMs) {
        await delay(delayMs);
      }
      const responseData: components['schemas']['SettingInfo'] = {
        key: params.key as string,
        current_value,
        default_value,
        source,
        metadata,
        ...rest,
      };
      return response(200 as const).json(responseData);
    }),
  ];
}

export function mockUpdateSettingError(
  key: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = 'invalid_request_error',
    status = 400,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.put(ENDPOINT_SETTING_KEY, async ({ params, response }) => {
      // match key before consuming the one-shot guard, or non-matching requests exhaust it
      if (params.key !== key) return;

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const errorBody = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorBody });
    }),
  ];
}

export function mockUpdateSettingInvalidError(key: string) {
  return mockUpdateSettingError(key, {
    code: 'invalid_request_error',
    message: 'Invalid setting value',
    type: 'invalid_request_error',
    status: 400,
  });
}

export function mockUpdateSettingServerError(key: string) {
  return mockUpdateSettingError(key, {
    code: 'internal_server_error',
    message: 'Server error',
    type: 'internal_server_error',
    status: 500,
  });
}

export function mockUpdateSettingNetworkError(key: string, { stub }: { stub?: boolean } = {}) {
  let hasBeenCalled = false;
  return [
    typedHttp.put(ENDPOINT_SETTING_KEY, async ({ params, response: _response }) => {
      // match key before consuming the one-shot guard, or non-matching requests exhaust it
      if (params.key !== key) return;

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      return HttpResponse.error();
    }),
  ];
}

/**
 * Responds only to the specified key; unmatched keys 404 via the catch-all.
 */
export function mockDeleteSetting(
  key: string,
  {
    current_value = 'default-value',
    default_value = 'default-value',
    source = 'default',
    metadata = { type: 'string' },
    ...rest
  }: Partial<Omit<components['schemas']['SettingInfo'], 'key'>> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.delete(ENDPOINT_SETTING_KEY, async ({ params, response }) => {
      // match key before consuming the one-shot guard, or non-matching requests exhaust it
      if (params.key !== key) return;

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['SettingInfo'] = {
        key: params.key as string,
        current_value,
        default_value,
        source,
        metadata,
        ...rest,
      };
      return response(200 as const).json(responseData);
    }),
  ];
}

export function mockDeleteSettingError(
  key: string,
  {
    code = 'not_found',
    message = 'Setting not found',
    type = 'not_found_error',
    status = 404,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 404 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.delete(ENDPOINT_SETTING_KEY, async ({ params, response }) => {
      // match key before consuming the one-shot guard, or non-matching requests exhaust it
      if (params.key !== key) return;

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const errorBody = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorBody });
    }),
  ];
}

export function mockDeleteSettingNotFoundError(key: string) {
  return mockDeleteSettingError(key, {
    code: 'invalid_request_error',
    message: 'Cannot delete required setting',
    type: 'invalid_request_error',
    status: 404,
  });
}

/**
 * Catch-all for PUT settings; register LAST so it only 404s truly unmatched keys.
 */
export function mockUpdateSettingNotFound() {
  return [
    typedHttp.put('/bodhi/v1/settings/{key}', async ({ params, response }) => {
      return response(404 as const).json({
        error: {
          code: 'not_found',
          message: `Setting ${params.key} not found`,
          type: 'not_found_error',
        },
      });
    }),
  ];
}

/**
 * Catch-all for DELETE settings; register LAST so it only 404s truly unmatched keys.
 */
export function mockDeleteSettingNotFound() {
  return [
    typedHttp.delete('/bodhi/v1/settings/{key}', async ({ params, response }) => {
      return response(404 as const).json({
        error: {
          code: 'not_found',
          message: `Setting ${params.key} not found`,
          type: 'not_found_error',
        },
      });
    }),
  ];
}

/**
 * Both catch-all handlers; register LAST.
 */
export function mockSettingsNotFound() {
  return [...mockUpdateSettingNotFound(), ...mockDeleteSettingNotFound()];
}
