/**
 * Type-safe MSW v2 handlers for authentication endpoints using openapi-msw
 */
import { ENDPOINT_AUTH_CALLBACK } from '@/hooks/useOAuth';
import { ENDPOINT_AUTH_INITIATE, ENDPOINT_LOGOUT } from '@/hooks/useQuery';
import { delay } from 'msw';
import { typedHttp, type components, INTERNAL_SERVER_ERROR } from '../openapi-msw-setup';

// =============================================================================
// CORE TYPED HTTP METHODS (Success cases + Error handlers)
// =============================================================================

/**
 * Mock handler for OAuth initiate endpoint with configurable responses
 *
 * @param response - Partial RedirectResponse data to override defaults
 */
export function mockAuthInitiate(
  {
    location = 'https://oauth.example.com/auth?client_id=test',
    ...rest
  }: Partial<components['schemas']['RedirectResponse']> = {},
  delayMs?: number
) {
  return [
    typedHttp.post(ENDPOINT_AUTH_INITIATE, async ({ response: httpResponse }) => {
      if (delayMs) {
        await delay(delayMs);
      }
      const responseData: components['schemas']['RedirectResponse'] = {
        location,
        ...rest,
      };
      return httpResponse(201).json(responseData);
    }),
  ];
}

export function mockAuthInitiateError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  return [
    typedHttp.post(ENDPOINT_AUTH_INITIATE, async ({ response }) => {
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

/**
 * Mock handler for OAuth callback endpoint with configurable responses
 *
 * @param response - Partial RedirectResponse data to override defaults
 * @param body - Partial AuthCallbackRequest to match against request body
 */
export function mockAuthCallback(
  { location = 'http://localhost:3000/ui/chat', ...rest }: Partial<components['schemas']['RedirectResponse']> = {},
  body?: Partial<components['schemas']['AuthCallbackRequest']>,
  delayMs?: number
) {
  return [
    typedHttp.post(ENDPOINT_AUTH_CALLBACK, async ({ request, response: httpResponse }) => {
      // If body is provided, validate it
      if (body) {
        const requestBody = (await request.json()) as components['schemas']['AuthCallbackRequest'];
        for (const [key, expectedValue] of Object.entries(body)) {
          if (requestBody[key] !== expectedValue) {
            // Body fields don't match, don't handle this request
            return;
          }
        }
      }

      if (delayMs) {
        await delay(delayMs);
      }
      const responseData: components['schemas']['RedirectResponse'] = {
        location,
        ...rest,
      };
      return httpResponse(200).json(responseData);
    }),
  ];
}

export function mockAuthCallbackError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 422 | 500 } = {}) {
  return [
    typedHttp.post(ENDPOINT_AUTH_CALLBACK, async ({ response }) => {
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

/**
 * Mock handler for logout endpoint with configurable responses
 *
 * @param response - Partial RedirectResponse data to override defaults
 */
export function mockLogout(
  { location = 'http://localhost:1135/ui/login', ...rest }: Partial<components['schemas']['RedirectResponse']> = {},
  delayMs?: number
) {
  return [
    typedHttp.post(ENDPOINT_LOGOUT, async ({ response: httpResponse }) => {
      if (delayMs) {
        await delay(delayMs);
      }
      const responseData: components['schemas']['RedirectResponse'] = {
        location,
        ...rest,
      };
      return httpResponse(200).json(responseData);
    }),
  ];
}

export function mockLogoutError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  return [
    typedHttp.post(ENDPOINT_LOGOUT, async ({ response }) => {
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

// =============================================================================
// VARIANT METHODS (Using core methods above)
// =============================================================================

/**
 * Mock handler for OAuth initiate when user is already authenticated
 * Returns 200 with home page URL
 */
export function mockAuthInitiateAlreadyAuthenticated(config: { location?: string } = {}) {
  return mockAuthInitiate({
    location: config.location || 'http://localhost:3000/ui/chat',
  });
}

/**
 * Mock handler for OAuth initiate when user is not authenticated
 * Returns 201 with OAuth authorization URL
 */
export function mockAuthInitiateUnauthenticated(config: { location?: string } = {}) {
  return mockAuthInitiate({
    location: config.location || 'https://oauth.example.com/auth?client_id=test',
  });
}

/**
 * Mock handler for OAuth configuration error during initiate
 */
export function mockAuthInitiateConfigError() {
  return mockAuthInitiateError({
    code: 'oauth_config_error',
    message: 'OAuth configuration error',
    type: 'invalid_request_error',
    status: 500,
  });
}

/**
 * Mock handler for successful OAuth callback completion
 */
export function mockAuthCallbackSuccess(config: { location?: string } = {}) {
  return mockAuthCallback({
    location: config.location || 'http://localhost:3000/ui/chat',
  });
}

/**
 * Mock handler for OAuth callback state mismatch error
 */
export function mockAuthCallbackStateError() {
  return mockAuthCallbackError({
    code: 'oauth_state_mismatch',
    message: 'Invalid state parameter',
    type: 'invalid_request_error',
    status: 422,
  });
}

/**
 * Mock handler for invalid authorization code during callback
 */
export function mockAuthCallbackInvalidCode() {
  return mockAuthCallbackError({
    code: 'invalid_auth_code',
    message: 'Invalid authorization code',
    type: 'invalid_request_error',
    status: 422,
  });
}

/**
 * Mock handler for successful logout
 */
export function mockLogoutSuccess(config: { location?: string } = {}) {
  return mockLogout({
    location: config.location || 'http://localhost:1135/ui/login',
  });
}

/**
 * Mock handler for logout session deletion failure
 */
export function mockLogoutSessionError() {
  return mockLogoutError({
    code: 'session_error',
    message: 'Session deletion failed',
    type: 'internal_server_error',
    status: 500,
  });
}
