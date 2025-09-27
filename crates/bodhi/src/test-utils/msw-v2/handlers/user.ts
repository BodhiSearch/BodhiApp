/**
 * Type-safe MSW v2 handlers for user endpoints using openapi-msw
 *
 * This handler provides fully migrated openapi-msw mock implementations for user management endpoints:
 * - GET /bodhi/v1/user - Current user information
 * - GET /bodhi/v1/users - List all users (paginated)
 * - PUT /bodhi/v1/users/{user_id}/role - Change user role
 * - DELETE /bodhi/v1/users/{user_id} - Remove user access
 *
 * All endpoints use typedHttp for full OpenAPI schema compliance and type safety.
 */
import { type components } from '../setup';
import { typedHttp } from '../openapi-msw-setup';
import {
  mockSimpleUsersResponse,
  mockMultipleAdminsResponse,
  mockMultipleManagersResponse,
  mockEmptyUsersResponse,
} from '@/test-fixtures/users';

/**
 * Mock handler for logged out user
 */
export function mockUserLoggedOut() {
  return [
    typedHttp.get('/bodhi/v1/user', ({ response }) => {
      return response(200 as const).json({
        auth_status: 'logged_out',
      });
    }),
  ];
}

/**
 * Mock handler for logged in user with configurable fields
 */
export function mockUserLoggedIn(config: Partial<components['schemas']['UserInfo']> & { delay?: number } = {}) {
  return [
    typedHttp.get('/bodhi/v1/user', ({ response }) => {
      const responseData = {
        auth_status: 'logged_in' as const,
        user_id: config.user_id || '550e8400-e29b-41d4-a716-446655440000',
        username: config.username || 'test@example.com',
        first_name: config.first_name !== undefined ? config.first_name : null,
        last_name: config.last_name !== undefined ? config.last_name : null,
        role: config.role !== undefined ? config.role : null,
      };
      const httpResponse = response(200 as const).json(responseData);

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(httpResponse), config.delay))
        : httpResponse;
    }),
  ];
}

export function mockUserError(
  config: {
    status?: 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    typedHttp.get('/bodhi/v1/user', ({ response }) => {
      return response((config.status || 500) as 500).json({
        error: {
          code: config.code || 'internal_error',
          message: config.message || 'Server error',
          type: 'internal_server_error',
        },
      });
    }),
  ];
}

/**
 * Mock handler for users list endpoint with configurable response data
 */
export function mockUsers(
  config: {
    users?: typeof mockSimpleUsersResponse;
    error?: boolean;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.get('/bodhi/v1/users', ({ response }) => {
      if (config.error) {
        const httpResponse = response(500).json({
          error: {
            code: 'internal_error',
            message: 'Failed to fetch users',
            type: 'internal_server_error',
          },
        });
        return config.delay
          ? new Promise((resolve) => setTimeout(() => resolve(httpResponse), config.delay))
          : httpResponse;
      }

      const responseData = {
        client_id: 'resource-test-client',
        users: config.users?.users || mockSimpleUsersResponse.users,
        page: config.users?.page || 1,
        page_size: config.users?.page_size || 10,
        total_pages: 1,
        total_users: config.users?.total || mockSimpleUsersResponse.total,
        has_next: false,
        has_previous: false,
      };

      const httpResponse = response(200 as const).json(responseData);
      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(httpResponse), config.delay))
        : httpResponse;
    }),
  ];
}

/**
 * Convenience methods for common user list scenarios
 */
export function mockUsersDefault() {
  return mockUsers({ users: mockSimpleUsersResponse });
}

export function mockUsersMultipleAdmins() {
  return mockUsers({ users: mockMultipleAdminsResponse });
}

export function mockUsersMultipleManagers() {
  return mockUsers({ users: mockMultipleManagersResponse });
}

export function mockUsersEmpty() {
  return mockUsers({ users: mockEmptyUsersResponse });
}

export function mockUsersError() {
  return mockUsers({ error: true });
}

/**
 * Mock handler for user role change endpoint
 */
export function mockUserRoleChange(
  config: {
    status?: 200 | 400 | 401 | 403 | 404 | 500;
    error?: boolean;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.put('/bodhi/v1/users/{user_id}/role', ({ response }) => {
      const status = config.status || 200;

      if (config.error || status !== 200) {
        let errorMessage = 'Server error';
        let errorCode = 'internal_error';
        let errorType = 'internal_server_error';

        if (status === 400) {
          errorMessage = 'Invalid role specified';
          errorCode = 'validation_error';
          errorType = 'invalid_request_error';
        } else if (status === 401) {
          errorMessage = 'Not authenticated';
          errorCode = 'authentication_error';
          errorType = 'unauthorized_error';
        } else if (status === 403) {
          errorMessage = 'Insufficient permissions';
          errorCode = 'authentication_error';
          errorType = 'permission_denied_error';
        } else if (status === 404) {
          errorMessage = 'User not found';
          errorCode = 'not_found_error';
          errorType = 'not_found_error';
        }

        const httpResponse = response(status as 400 | 401 | 403 | 404 | 500).json({
          error: {
            code: errorCode,
            message: errorMessage,
            type: errorType,
          },
        });
        return config.delay
          ? new Promise((resolve) => setTimeout(() => resolve(httpResponse), config.delay))
          : httpResponse;
      }

      const httpResponse = response(200).empty();
      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(httpResponse), config.delay))
        : httpResponse;
    }),
  ];
}

/**
 * Convenience methods for role change scenarios
 */
export function mockUserRoleChangeSuccess() {
  return mockUserRoleChange();
}

export function mockUserRoleChangeError(status: 400 | 401 | 403 | 404 | 500 = 500) {
  return mockUserRoleChange({ error: true, status });
}

/**
 * Mock handler for user removal endpoint
 */
export function mockUserRemove(
  config: {
    status?: 200 | 400 | 401 | 403 | 404 | 500;
    error?: boolean;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.delete('/bodhi/v1/users/{user_id}', ({ response }) => {
      const status = config.status || 200;

      if (config.error || status !== 200) {
        let errorMessage = 'Server error';
        let errorCode = 'internal_error';
        let errorType = 'internal_server_error';

        if (status === 400) {
          errorMessage = 'Invalid request';
          errorCode = 'validation_error';
          errorType = 'invalid_request_error';
        } else if (status === 401) {
          errorMessage = 'Not authenticated';
          errorCode = 'authentication_error';
          errorType = 'unauthorized_error';
        } else if (status === 403) {
          errorMessage = 'Insufficient permissions';
          errorCode = 'authentication_error';
          errorType = 'permission_denied_error';
        } else if (status === 404) {
          errorMessage = 'User not found';
          errorCode = 'not_found_error';
          errorType = 'not_found_error';
        }

        const httpResponse = response(status as 400 | 401 | 403 | 404 | 500).json({
          error: {
            code: errorCode,
            message: errorMessage,
            type: errorType,
          },
        });
        return config.delay
          ? new Promise((resolve) => setTimeout(() => resolve(httpResponse), config.delay))
          : httpResponse;
      }

      const httpResponse = response(200).empty();
      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(httpResponse), config.delay))
        : httpResponse;
    }),
  ];
}

/**
 * Convenience methods for user removal scenarios
 */
export function mockUserRemoveSuccess() {
  return mockUserRemove();
}

export function mockUserRemoveError(status: 400 | 401 | 403 | 404 | 500 = 500) {
  return mockUserRemove({ error: true, status });
}
