/**
 * Type-safe MSW v2 handlers for authentication endpoints using patterns inspired by openapi-msw
 */
import { ENDPOINT_AUTH_INITIATE, ENDPOINT_LOGOUT } from '@/hooks/useQuery';
import { ENDPOINT_AUTH_CALLBACK } from '@/hooks/useOAuth';
import { http, HttpResponse, type components } from '../setup';

/**
 * Unified handler for OAuth initiate endpoint with flexible configuration
 *
 * @param config Configuration options
 * @param config.location - Redirect location URL (defaults to OAuth URL for status 201, chat for status 200)
 * @param config.status - HTTP status: 200 = already authenticated, 201 = OAuth redirect needed (default: 201)
 * @param config.delay - Add delay in milliseconds for testing loading states
 * @param config.noLocation - Return empty object without location field
 * @param config.invalidUrl - Return invalid URL format for testing URL validation
 */
export function mockAuthInitiate(
  config: {
    location?: string;
    status?: 200 | 201;
    delay?: number;
    noLocation?: boolean;
    invalidUrl?: boolean;
  } = {}
) {
  return [
    http.post(ENDPOINT_AUTH_INITIATE, () => {
      const status = config.status || 201;

      // Handle edge cases
      if (config.noLocation) {
        const response = HttpResponse.json({}, { status });
        return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
      }

      let location = config.location;
      if (!location) {
        if (config.invalidUrl) {
          location = 'invalid-url-format';
        } else if (status === 200) {
          location = 'http://localhost:3000/ui/chat'; // Already authenticated
        } else {
          location = 'https://oauth.example.com/auth?client_id=test'; // OAuth redirect
        }
      }

      const responseData: components['schemas']['RedirectResponse'] = { location };
      const response = HttpResponse.json(responseData, { status });

      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
    }),
  ];
}

/**
 * Error handler for OAuth initiate endpoint
 *
 * @param config Error configuration
 * @param config.status - HTTP status code (default: 500)
 * @param config.code - Error code (default: 'internal_error')
 * @param config.message - Error message (default: 'OAuth configuration error')
 * @param config.delay - Add delay in milliseconds for testing loading states
 * @param config.empty - Return empty response (for generic 500 errors)
 */
export function mockAuthInitiateError(
  config: {
    status?: 422 | 500;
    code?: string;
    message?: string;
    delay?: number;
    empty?: boolean;
  } = {}
) {
  return [
    http.post(ENDPOINT_AUTH_INITIATE, () => {
      // For generic 500 errors with no specific message, return empty response
      if (config.empty || (config.status === 500 && !config.message && !config.code)) {
        const response = HttpResponse.json({}, { status: config.status || 500 });
        return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
      }

      const response = HttpResponse.json(
        {
          error: {
            code: config.code || 'internal_error',
            message: config.message || 'OAuth configuration error',
            type: config.status === 422 ? 'validation_error' : 'internal_server_error',
          },
        },
        { status: config.status || 500 }
      );

      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
    }),
  ];
}

/**
 * Unified handler for logout endpoint with flexible configuration
 *
 * @param config Configuration options
 * @param config.location - Redirect location URL (default: 'http://localhost:1135/ui/login')
 * @param config.delay - Add delay in milliseconds for testing loading states
 * @param config.noLocation - Return empty object without location field
 */
export function mockLogout(
  config: {
    location?: string;
    delay?: number;
    noLocation?: boolean;
  } = {}
) {
  return [
    http.post(ENDPOINT_LOGOUT, () => {
      // Handle missing location field case
      if (config.noLocation) {
        const response = HttpResponse.json({}, { status: 200 });
        return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
      }

      const responseData: components['schemas']['RedirectResponse'] = {
        location: config.location || 'http://localhost:1135/ui/login',
      };
      const response = HttpResponse.json(responseData);

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
    http.post(ENDPOINT_LOGOUT, () => {
      return HttpResponse.json(
        {
          error: {
            code: config.code || 'session_error',
            message: config.message || 'Session deletion failed',
          },
        },
        { status: config.status || 500 }
      );
    }),
  ];
}

/**
 * Unified handler for OAuth callback endpoint with flexible configuration
 *
 * @param config Configuration options
 * @param config.location - Redirect location URL (default: 'http://localhost:3000/ui/chat')
 * @param config.status - HTTP status code (default: 200)
 * @param config.delay - Add delay in milliseconds for testing loading states
 * @param config.noLocation - Return empty object without location field
 * @param config.invalidUrl - Return invalid URL format for testing URL validation
 */
export function mockAuthCallback(
  config: {
    location?: string;
    status?: number;
    delay?: number;
    noLocation?: boolean;
    invalidUrl?: boolean;
  } = {}
) {
  return [
    http.post(ENDPOINT_AUTH_CALLBACK, () => {
      const status = config.status || 200;

      // Handle edge cases
      if (config.noLocation) {
        const response = HttpResponse.json({}, { status });
        return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
      }

      let location = config.location;
      if (!location) {
        if (config.invalidUrl) {
          location = 'invalid-url-format';
        } else {
          location = 'http://localhost:3000/ui/chat';
        }
      }

      const responseData: components['schemas']['RedirectResponse'] = { location };
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
    status?: 400 | 500;
    code?: string;
    message?: string;
    type?: string;
    delay?: number;
  } = {}
) {
  return [
    http.post(ENDPOINT_AUTH_CALLBACK, () => {
      const response = HttpResponse.json(
        {
          error: {
            code: config.code || 'invalid_state',
            message: config.message || 'Invalid state parameter',
            type: config.type || (config.status === 400 ? 'invalid_request' : 'internal_server_error'),
          },
        },
        { status: config.status || 400 }
      );

      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
    }),
  ];
}
