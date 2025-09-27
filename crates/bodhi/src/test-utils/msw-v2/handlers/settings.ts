/**
 * Type-safe MSW v2 handlers for settings endpoint using patterns inspired by openapi-msw
 */
import { ENDPOINT_SETTINGS } from '@/hooks/useQuery';
import { http, HttpResponse, type components } from '../setup';

/**
 * Create type-safe MSW v2 handlers for settings endpoint
 * Uses generated OpenAPI types directly
 */
export function mockSettings(config: components['schemas']['SettingInfo'][] = []) {
  return [
    http.get(ENDPOINT_SETTINGS, () => {
      const responseData: components['schemas']['SettingInfo'][] = config;
      return HttpResponse.json(responseData);
    }),
  ];
}

export function mockSettingsDefault() {
  return mockSettings([
    {
      key: 'BODHI_HOME',
      current_value: '/home/user/.bodhi',
      default_value: '/home/user/.cache/bodhi',
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
  config: {
    status?: 400 | 401 | 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    http.get(ENDPOINT_SETTINGS, () => {
      return HttpResponse.json(
        {
          error: {
            code: config.code || 'internal_error',
            message: config.message || 'Server error',
          },
        },
        { status: config.status || 500 }
      );
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for updating individual settings
 */
export function mockUpdateSetting(
  settingKey: string,
  config: components['schemas']['SettingInfo'] = {
    key: settingKey,
    current_value: 'updated-value',
    default_value: 'default-value',
    source: 'settings_file',
    metadata: { type: 'string' },
  }
) {
  return [
    http.put(`${ENDPOINT_SETTINGS}/${settingKey}`, () => {
      return HttpResponse.json(config);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for setting update errors
 */
export function mockUpdateSettingError(
  settingKey: string,
  config: {
    status?: 400 | 401 | 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    http.put(`${ENDPOINT_SETTINGS}/${settingKey}`, () => {
      return HttpResponse.json(
        {
          error: {
            code: config.code || 'internal_error',
            message: config.message || 'Server error',
          },
        },
        { status: config.status || 500 }
      );
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for network errors
 */
export function mockUpdateSettingNetworkError(settingKey: string) {
  return [
    http.put(`${ENDPOINT_SETTINGS}/${settingKey}`, () => {
      return HttpResponse.error();
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for updating individual settings with delay
 */
export function mockUpdateSettingWithDelay(
  settingKey: string,
  config: components['schemas']['SettingInfo'] = {
    key: settingKey,
    current_value: 'updated-value',
    default_value: 'default-value',
    source: 'settings_file',
    metadata: { type: 'string' },
  },
  delayMs: number = 100
) {
  return [
    http.put(`${ENDPOINT_SETTINGS}/${settingKey}`, async () => {
      await new Promise((resolve) => setTimeout(resolve, delayMs));
      return HttpResponse.json(config);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for deleting individual settings
 */
export function mockDeleteSetting(
  settingKey: string,
  config: components['schemas']['SettingInfo'] = {
    key: settingKey,
    current_value: 'default-value', // Reset to default after delete
    default_value: 'default-value',
    source: 'default',
    metadata: { type: 'string' },
  }
) {
  return [
    http.delete(`${ENDPOINT_SETTINGS}/${settingKey}`, () => {
      return HttpResponse.json(config);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for setting delete errors
 */
export function mockDeleteSettingError(
  settingKey: string,
  config: {
    status?: 400 | 401 | 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    http.delete(`${ENDPOINT_SETTINGS}/${settingKey}`, () => {
      return HttpResponse.json(
        {
          error: {
            code: config.code || 'internal_error',
            message: config.message || 'Server error',
          },
        },
        { status: config.status || 500 }
      );
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for network errors on delete
 */
export function mockDeleteSettingNetworkError(settingKey: string) {
  return [
    http.delete(`${ENDPOINT_SETTINGS}/${settingKey}`, () => {
      return HttpResponse.error();
    }),
  ];
}
