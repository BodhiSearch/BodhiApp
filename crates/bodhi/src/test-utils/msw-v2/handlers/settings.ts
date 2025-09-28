/**
 * Type-safe MSW v2 handlers for settings endpoint using openapi-msw with full schema compliance
 */
import { ENDPOINT_SETTINGS } from '@/hooks/useQuery';
import { delay } from 'msw';
import { HttpResponse, INTERNAL_SERVER_ERROR, typedHttp, type components } from '../openapi-msw-setup';

// ============================================================================
// Settings List Endpoint (GET /bodhi/v1/settings)
// ============================================================================

// Success Handlers
/**
 * Create type-safe MSW v2 handlers for settings endpoint
 * Uses generated OpenAPI types directly
 */
export function mockSettings(response: components['schemas']['SettingInfo'][] = []) {
  return [
    typedHttp.get(ENDPOINT_SETTINGS, async ({ response: resp }) => {
      const responseData: components['schemas']['SettingInfo'][] = response;
      return resp(200 as const).json(responseData);
    }),
  ];
}

// Success Handler Variants
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

// Error Handlers
/**
 * Create type-safe MSW v2 handler for settings errors
 */
export function mockSettingsError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  return [
    typedHttp.get(ENDPOINT_SETTINGS, async ({ response: _response }) => {
      const errorBody = {
        code,
        message,
        type,
        ...rest,
      };
      return _response(status).json({ error: errorBody });
    }),
  ];
}

// Error Handler Variants
export function mockSettingsInternalError() {
  return mockSettingsError({
    code: 'internal_error',
    message: 'Test Error',
    type: 'internal_server_error',
    status: 500,
  });
}

// ============================================================================
// Update Setting Endpoint (PUT /bodhi/v1/settings/{key})
// ============================================================================

// Success Handlers
/**
 * Create type-safe MSW v2 handler for updating individual settings (Option 2 - Single Key)
 * Only responds to the specified key, returns 404 for others
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
  delayMs?: number
) {
  return [
    typedHttp.put('/bodhi/v1/settings/{key}', async ({ params, response }) => {
      // Only respond with success if key matches
      if (params.key === key) {
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
      }

      // Pass through to next handler for non-matching keys
      return;
    }),
  ];
}

// Error Handlers
/**
 * Create type-safe MSW v2 handler for setting update errors
 * Only returns error for the specified key
 */
export function mockUpdateSettingError(
  key: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = 'invalid_request_error',
    status = 400,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}
) {
  return [
    typedHttp.put('/bodhi/v1/settings/{key}', async ({ params, response: _response }) => {
      // Only return error for matching key
      if (params.key === key) {
        const errorBody = {
          code,
          message,
          type,
          ...rest,
        };
        return _response(status).json({ error: errorBody });
      }

      // Pass through to next handler for non-matching keys
      return;
    }),
  ];
}

// Error Handler Variants
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

/**
 * Create type-safe MSW v2 handler for setting update network errors
 * Only returns network error for the specified key
 */
export function mockUpdateSettingNetworkError(key: string) {
  return [
    typedHttp.put('/bodhi/v1/settings/{key}', async ({ params, response: _response }) => {
      // Only return network error for matching key
      if (params.key === key) {
        return HttpResponse.error();
      }

      // Pass through to next handler for non-matching keys
      return;
    }),
  ];
}

// ============================================================================
// Delete Setting Endpoint (DELETE /bodhi/v1/settings/{key})
// ============================================================================

// Success Handlers
/**
 * Create type-safe MSW v2 handler for deleting individual settings (Option 2 - Single Key)
 * Only responds to the specified key, returns 404 for others
 */
export function mockDeleteSetting(
  key: string,
  {
    current_value = 'default-value',
    default_value = 'default-value',
    source = 'default',
    metadata = { type: 'string' },
    ...rest
  }: Partial<Omit<components['schemas']['SettingInfo'], 'key'>> = {}
) {
  return [
    typedHttp.delete('/bodhi/v1/settings/{key}', async ({ params, response: _response }) => {
      // Only respond with success if key matches
      if (params.key === key) {
        const responseData: components['schemas']['SettingInfo'] = {
          key: params.key as string,
          current_value, // Reset to default after delete
          default_value,
          source,
          metadata,
          ...rest,
        };
        return _response(200 as const).json(responseData);
      }

      // Pass through to next handler for non-matching keys
      return;
    }),
  ];
}

// Error Handlers
/**
 * Create type-safe MSW v2 handler for setting delete errors
 * Only returns error for the specified key
 */
export function mockDeleteSettingError(
  key: string,
  {
    code = 'not_found',
    message = 'Setting not found',
    type = 'not_found_error',
    status = 404,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 404 | 500 } = {}
) {
  return [
    typedHttp.delete('/bodhi/v1/settings/{key}', async ({ params, response: _response }) => {
      // Only return error for matching key
      if (params.key === key) {
        const errorBody = {
          code,
          message,
          type,
          ...rest,
        };
        return _response(status).json({ error: errorBody });
      }

      // Pass through to next handler for non-matching keys
      return;
    }),
  ];
}

// Error Handler Variants
export function mockDeleteSettingNotFoundError(key: string) {
  return mockDeleteSettingError(key, {
    code: 'invalid_request_error',
    message: 'Cannot delete required setting',
    type: 'invalid_request_error',
    status: 404,
  });
}

// ============================================================================
// Catch-All 404 Handlers (Register LAST to catch unmatched requests)
// ============================================================================

/**
 * Catch-all handler for PUT requests to settings endpoints that weren't matched by any specific handler
 * This should be registered LAST to provide 404 responses for truly unmatched keys
 */
export function mockUpdateSettingNotFound() {
  return [
    typedHttp.put('/bodhi/v1/settings/{key}', async ({ params, response: _response }) => {
      return _response(404 as const).json({
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
 * Catch-all handler for DELETE requests to settings endpoints that weren't matched by any specific handler
 * This should be registered LAST to provide 404 responses for truly unmatched keys
 */
export function mockDeleteSettingNotFound() {
  return [
    typedHttp.delete('/bodhi/v1/settings/{key}', async ({ params, response: _response }) => {
      return _response(404 as const).json({
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
 * Convenience function to get both catch-all handlers
 * Register these LAST to catch any unmatched setting requests
 */
export function mockSettingsNotFound() {
  return [...mockUpdateSettingNotFound(), ...mockDeleteSettingNotFound()];
}
