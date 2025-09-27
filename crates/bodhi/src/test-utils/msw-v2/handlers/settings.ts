/**
 * Type-safe MSW v2 handlers for settings endpoint using openapi-msw with full schema compliance
 */
import { ENDPOINT_SETTINGS } from '@/hooks/useQuery';
import { typedHttp } from '../openapi-msw-setup';
import type { components } from '../setup';

/**
 * Create type-safe MSW v2 handlers for settings endpoint
 * Uses generated OpenAPI types directly
 */
export function mockSettings(config: components['schemas']['SettingInfo'][] = []) {
  return [
    typedHttp.get(ENDPOINT_SETTINGS, ({ response }) => {
      const responseData: components['schemas']['SettingInfo'][] = config;
      return response(200).json(responseData);
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
    status?: 401 | 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    typedHttp.get(ENDPOINT_SETTINGS, ({ response }) => {
      return response(config.status || 500).json({
        error: {
          code: config.code || 'internal_error',
          message: config.message || 'Server error',
          type: config.status === 401 ? 'unauthorized_error' : 'internal_server_error',
        },
      });
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
    typedHttp.put('/bodhi/v1/settings/{key}', ({ response }) => {
      return response(200).json(config);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for setting update errors
 */
export function mockUpdateSettingError(
  settingKey: string,
  config: {
    status?: 400 | 404;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    typedHttp.put('/bodhi/v1/settings/{key}', ({ response }) => {
      const statusCode = config.status || 400;
      const errorType = statusCode === 404 ? 'not_found_error' : 'invalid_request_error';
      return response(statusCode).json({
        error: {
          code: config.code || 'internal_error',
          message: config.message || 'Server error',
          type: errorType,
        },
      });
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for network errors
 */
export function mockUpdateSettingNetworkError(settingKey: string) {
  return [
    typedHttp.put('/bodhi/v1/settings/{key}', ({ response }) => {
      return response(400).json({
        error: {
          code: 'network_error',
          message: 'Failed to update setting',
          type: 'invalid_request_error',
        },
      });
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
    typedHttp.put('/bodhi/v1/settings/{key}', async ({ response }) => {
      await new Promise((resolve) => setTimeout(resolve, delayMs));
      return response(200).json(config);
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
    typedHttp.delete('/bodhi/v1/settings/{key}', ({ response }) => {
      return response(200).json(config);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for setting delete errors
 */
export function mockDeleteSettingError(
  settingKey: string,
  config: {
    status?: 404;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    typedHttp.delete('/bodhi/v1/settings/{key}', ({ response }) => {
      return response(config.status || 404).json({
        error: {
          code: config.code || 'not_found',
          message: config.message || 'Setting not found',
          type: 'not_found_error',
        },
      });
    }),
  ];
}

/**
 * Create type-safe MSW v2 handler for network errors on delete
 */
export function mockDeleteSettingNetworkError(settingKey: string) {
  return [
    typedHttp.delete('/bodhi/v1/settings/{key}', ({ response }) => {
      return response(404).json({
        error: {
          code: 'not_found',
          message: 'Setting not found',
          type: 'not_found_error',
        },
      });
    }),
  ];
}
