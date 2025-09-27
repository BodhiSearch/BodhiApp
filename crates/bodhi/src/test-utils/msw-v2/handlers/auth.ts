/**
 * Type-safe MSW v2 handlers for authentication endpoints using openapi-msw
 */
import { ENDPOINT_AUTH_INITIATE, ENDPOINT_LOGOUT } from '@/hooks/useQuery';
import { ENDPOINT_AUTH_CALLBACK } from '@/hooks/useOAuth';
import { typedHttp } from '../openapi-msw-setup';
import { http, HttpResponse, type components } from '../setup';

/**
 * Unified handler for OAuth initiate endpoint with pure openapi-msw
 * For edge cases that don't conform to schema, use mockAuthInitiateInvalid
 *
 * @param config Configuration options
 * @param config.location - Redirect location URL (defaults to OAuth URL for status 201, chat for status 200)
 * @param config.status - HTTP status: 200 = already authenticated, 201 = OAuth redirect needed (default: 201)
 * @param config.delay - Add delay in milliseconds for testing loading states
 */
export function mockAuthInitiate(
  config: {
    location?: string;
    status?: 200 | 201;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.post(ENDPOINT_AUTH_INITIATE, ({ response }) => {
      const status = config.status || 201;

      let location = config.location;
      if (!location) {
        if (status === 200) {
          location = 'http://localhost:3000/ui/chat'; // Already authenticated
        } else {
          location = 'https://oauth.example.com/auth?client_id=test'; // OAuth redirect
        }
      }

      const responseData: components['schemas']['RedirectResponse'] = { location };
      const responseResult = response(status as 200 | 201 | 500).json(responseData);

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(responseResult), config.delay))
        : responseResult;
    }),
  ];
}

/**
 * Invalid handler for OAuth initiate endpoint edge cases using manual MSW
 * Handles cases that don't conform to OpenAPI schema requirements
 *
 * @param config Configuration options
 * @param config.status - HTTP status (default: depends on scenario)
 * @param config.delay - Add delay in milliseconds for testing loading states
 * @param config.noLocation - Return empty object without location field (successful response)
 * @param config.empty - Return empty response (error scenario with 500 status)
 * @param config.invalidUrl - Return invalid URL format for testing URL validation
 */
export function mockAuthInitiateInvalid(
  config: {
    status?: number;
    delay?: number;
    noLocation?: boolean;
    empty?: boolean;
    invalidUrl?: boolean;
  } = {}
) {
  return [
    http.post(ENDPOINT_AUTH_INITIATE, () => {
      let status: number;
      let responseData: any = {};

      if (config.empty) {
        // Error scenario - return 500 with empty response
        status = config.status || 500;
        responseData = {};
      } else if (config.noLocation) {
        // Success scenario but missing required location field
        status = config.status || 201;
        responseData = {};
      } else if (config.invalidUrl) {
        // Success scenario but with invalid URL format
        status = config.status || 201;
        responseData = { location: 'invalid-url-format' };
      } else {
        // Default case
        status = config.status || 201;
        responseData = {};
      }

      const response = HttpResponse.json(responseData, { status });
      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
    }),
  ];
}

/**
 * Error handler for OAuth initiate endpoint with pure openapi-msw
 * For edge cases that don't conform to schema, use mockAuthInitiateInvalid
 *
 * @param config Error configuration
 * @param config.status - HTTP status code (default: 500)
 * @param config.code - Error code (default: 'internal_error')
 * @param config.message - Error message (default: 'OAuth configuration error')
 * @param config.delay - Add delay in milliseconds for testing loading states
 */
export function mockAuthInitiateError(
  config: {
    status?: 500;
    code?: string;
    message?: string;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.post(ENDPOINT_AUTH_INITIATE, ({ response }) => {
      const responseData = response(config.status || 500).json({
        error: {
          code: config.code || 'internal_error',
          message: config.message || 'OAuth configuration error',
          type: 'internal_server_error',
        },
      });

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(responseData), config.delay))
        : responseData;
    }),
  ];
}

/**
 * Unified handler for logout endpoint with pure openapi-msw
 * For edge cases that don't conform to schema, use mockLogoutInvalid
 *
 * @param config Configuration options
 * @param config.location - Redirect location URL (default: 'http://localhost:1135/ui/login')
 * @param config.delay - Add delay in milliseconds for testing loading states
 */
export function mockLogout(
  config: {
    location?: string;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.post(ENDPOINT_LOGOUT, ({ response }) => {
      const responseData: components['schemas']['RedirectResponse'] = {
        location: config.location || 'http://localhost:1135/ui/login',
      };
      const responseResult = response(200 as 200 | 500).json(responseData);

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(responseResult), config.delay))
        : responseResult;
    }),
  ];
}

/**
 * Invalid handler for logout endpoint edge cases using manual MSW
 * Handles cases that don't conform to OpenAPI schema requirements
 *
 * @param config Configuration options
 * @param config.delay - Add delay in milliseconds for testing loading states
 * @param config.noLocation - Return empty object without location field
 * @param config.empty - Return empty response
 */
export function mockLogoutInvalid(
  config: {
    delay?: number;
    noLocation?: boolean;
    empty?: boolean;
  } = {}
) {
  return [
    http.post(ENDPOINT_LOGOUT, () => {
      let responseData: any = {};

      if (config.empty || config.noLocation) {
        responseData = {};
      }

      const response = HttpResponse.json(responseData, { status: 200 });
      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
    }),
  ];
}

/**
 * Error handler for logout endpoint
 *
 * @param config Error configuration
 * @param config.status - HTTP status code (default: 500)
 * @param config.code - Error code (default: 'session_error')
 * @param config.message - Error message (default: 'Session deletion failed')
 */
export function mockLogoutError(
  config: {
    status?: 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    typedHttp.post(ENDPOINT_LOGOUT, ({ response }) => {
      return response(config.status || 500).json({
        error: {
          code: config.code || 'session_error',
          message: config.message || 'Session deletion failed',
          type: 'internal_server_error',
        },
      });
    }),
  ];
}

/**
 * Unified handler for OAuth callback endpoint with pure openapi-msw
 * For edge cases that don't conform to schema, use mockAuthCallbackInvalid
 *
 * @param config Configuration options
 * @param config.location - Redirect location URL (default: 'http://localhost:3000/ui/chat')
 * @param config.status - HTTP status code (default: 200)
 * @param config.delay - Add delay in milliseconds for testing loading states
 */
export function mockAuthCallback(
  config: {
    location?: string;
    status?: number;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.post(ENDPOINT_AUTH_CALLBACK, ({ response }) => {
      const status = config.status || 200;

      const location = config.location || 'http://localhost:3000/ui/chat';

      const responseData: components['schemas']['RedirectResponse'] = { location };
      const responseResult = response(status as 200 | 422 | 500).json(responseData);

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(responseResult), config.delay))
        : responseResult;
    }),
  ];
}

/**
 * Invalid handler for OAuth callback endpoint edge cases using manual MSW
 * Handles cases that don't conform to OpenAPI schema requirements
 *
 * @param config Configuration options
 * @param config.status - HTTP status code (default: depends on scenario)
 * @param config.delay - Add delay in milliseconds for testing loading states
 * @param config.noLocation - Return empty object without location field (successful response)
 * @param config.empty - Return empty response (error scenario)
 * @param config.invalidUrl - Return invalid URL format for testing URL validation
 */
export function mockAuthCallbackInvalid(
  config: {
    status?: number;
    delay?: number;
    noLocation?: boolean;
    empty?: boolean;
    invalidUrl?: boolean;
  } = {}
) {
  return [
    http.post(ENDPOINT_AUTH_CALLBACK, () => {
      let status: number;
      let responseData: any = {};

      if (config.empty) {
        // Error scenario - return error status with empty response
        status = config.status || 500;
        responseData = {};
      } else if (config.noLocation) {
        // Success scenario but missing required location field
        status = config.status || 200;
        responseData = {};
      } else if (config.invalidUrl) {
        // Success scenario but with invalid URL format
        status = config.status || 200;
        responseData = { location: 'invalid-url-format' };
      } else {
        // Default case
        status = config.status || 200;
        responseData = {};
      }

      const response = HttpResponse.json(responseData, { status });
      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
    }),
  ];
}

/**
 * Error handler for OAuth callback endpoint
 *
 * @param config Error configuration
 * @param config.status - HTTP status code (default: 400)
 * @param config.code - Error code (default: 'invalid_state')
 * @param config.message - Error message (default: 'Invalid state parameter')
 * @param config.type - Error type (default: 'invalid_request' for 400, 'internal_server_error' for 500)
 * @param config.delay - Add delay in milliseconds for testing loading states
 */
export function mockAuthCallbackError(
  config: {
    status?: 422 | 500;
    code?: string;
    message?: string;
    type?: string;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.post(ENDPOINT_AUTH_CALLBACK, ({ response }) => {
      const responseData = response(config.status || 422).json({
        error: {
          code: config.code || 'invalid_state',
          message: config.message || 'Invalid state parameter',
          type: config.type || (config.status === 422 ? 'invalid_request_error' : 'internal_server_error'),
        },
      });

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(responseData), config.delay))
        : responseData;
    }),
  ];
}
