/**
 * Type-safe MSW v2 handlers for user endpoints using patterns inspired by openapi-msw
 */
import { ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { http, HttpResponse, type components } from '../setup';
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
    http.get(ENDPOINT_USER_INFO, () => {
      const responseData: components['schemas']['UserResponse'] = {
        auth_status: 'logged_out',
      };
      return HttpResponse.json(responseData);
    }),
  ];
}

/**
 * Mock handler for logged in user with configurable fields
 */
export function mockUserLoggedIn(config: Partial<components['schemas']['UserInfo']> & { delay?: number } = {}) {
  return [
    http.get(ENDPOINT_USER_INFO, () => {
      const responseData: components['schemas']['UserResponse'] = {
        auth_status: 'logged_in',
        user_id: config.user_id || '550e8400-e29b-41d4-a716-446655440000',
        username: config.username || 'test@example.com',
        first_name: config.first_name !== undefined ? config.first_name : null,
        last_name: config.last_name !== undefined ? config.last_name : null,
        role: config.role !== undefined ? config.role : null,
      };
      const response = HttpResponse.json(responseData);

      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
    }),
  ];
}

export function mockUserError(
  config: {
    status?: 401 | 403 | 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    http.get(ENDPOINT_USER_INFO, () => {
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
    http.get('/bodhi/v1/users', ({ request }) => {
      if (config.error) {
        const response = HttpResponse.json(
          {
            error: {
              type: 'internal_error',
              message: 'Failed to fetch users',
            },
          },
          { status: 500 }
        );
        return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
      }

      const responseData: components['schemas']['UserListResponse'] = {
        client_id: 'resource-test-client',
        users: config.users?.users || mockSimpleUsersResponse.users,
        page: config.users?.page || 1,
        page_size: config.users?.page_size || 10,
        total_pages: 1,
        total_users: config.users?.total || mockSimpleUsersResponse.total,
        has_next: false,
        has_previous: false,
      };

      const response = HttpResponse.json(responseData);
      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
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
    status?: 200 | 400 | 403 | 404 | 500;
    error?: boolean;
    delay?: number;
  } = {}
) {
  return [
    http.put('/bodhi/v1/users/:userId/role', () => {
      const status = config.status || 200;

      if (config.error || status !== 200) {
        let errorMessage = 'Server error';
        let errorCode = 'internal_error';

        if (status === 400) {
          errorMessage = 'Invalid role specified';
          errorCode = 'validation_error';
        } else if (status === 403) {
          errorMessage = 'Insufficient permissions';
          errorCode = 'authentication_error';
        } else if (status === 404) {
          errorMessage = 'User not found';
          errorCode = 'not_found_error';
        }

        const response = HttpResponse.json(
          {
            error: {
              code: errorCode,
              message: errorMessage,
            },
          },
          { status }
        );
        return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
      }

      const response = HttpResponse.json({});
      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
    }),
  ];
}

/**
 * Convenience methods for role change scenarios
 */
export function mockUserRoleChangeSuccess() {
  return mockUserRoleChange();
}

export function mockUserRoleChangeError(status: 400 | 403 | 404 | 500 = 500) {
  return mockUserRoleChange({ error: true, status });
}

/**
 * Mock handler for user removal endpoint
 */
export function mockUserRemove(
  config: {
    status?: 200 | 403 | 404 | 500;
    error?: boolean;
    delay?: number;
  } = {}
) {
  return [
    http.delete('/bodhi/v1/users/:userId', () => {
      const status = config.status || 200;

      if (config.error || status !== 200) {
        let errorMessage = 'Server error';
        let errorCode = 'internal_error';

        if (status === 403) {
          errorMessage = 'Insufficient permissions';
          errorCode = 'authentication_error';
        } else if (status === 404) {
          errorMessage = 'User not found';
          errorCode = 'not_found_error';
        }

        const response = HttpResponse.json(
          {
            error: {
              code: errorCode,
              message: errorMessage,
            },
          },
          { status }
        );
        return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
      }

      const response = HttpResponse.json({});
      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
    }),
  ];
}

/**
 * Convenience methods for user removal scenarios
 */
export function mockUserRemoveSuccess() {
  return mockUserRemove();
}

export function mockUserRemoveError(status: 403 | 404 | 500 = 500) {
  return mockUserRemove({ error: true, status });
}
