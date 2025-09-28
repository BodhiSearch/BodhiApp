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
import { delay } from 'msw';
import { typedHttp, type components, INTERNAL_SERVER_ERROR } from '../openapi-msw-setup';
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
export function mockUserLoggedIn(
  {
    user_id = '550e8400-e29b-41d4-a716-446655440000',
    username = 'test@example.com',
    first_name = null,
    last_name = null,
    role = null,
    ...rest
  }: Partial<components['schemas']['UserInfo']> = {},
  delayMs?: number
) {
  return [
    typedHttp.get('/bodhi/v1/user', async ({ response: httpResponse }) => {
      if (delayMs) {
        await delay(delayMs);
      }
      const responseData = {
        auth_status: 'logged_in' as const,
        user_id,
        username,
        first_name,
        last_name,
        role,
        ...rest,
      };
      return httpResponse(200 as const).json(responseData);
    }),
  ];
}

export function mockUserError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 500 } = {}) {
  return [
    typedHttp.get('/bodhi/v1/user', async ({ response }) => {
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
 * Mock handler for users list endpoint with configurable response data
 */
export function mockUsers({
  client_id = 'resource-test-client',
  users = mockSimpleUsersResponse.users,
  page = 1,
  page_size = 10,
  total_pages = 1,
  total_users = mockSimpleUsersResponse.total,
  has_next = false,
  has_previous = false,
  ...rest
}: Partial<components['schemas']['UserListResponse']> = {}) {
  return [
    typedHttp.get('/bodhi/v1/users', async ({ response: httpResponse }) => {
      const responseData = {
        client_id,
        users,
        page,
        page_size,
        total_pages,
        total_users,
        has_next,
        has_previous,
        ...rest,
      };
      return httpResponse(200 as const).json(responseData);
    }),
  ];
}

/**
 * Mock handler for users list endpoint error responses
 */
export function mockUsersError({
  code = INTERNAL_SERVER_ERROR.code,
  message = 'Failed to fetch users',
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 500 } = {}) {
  return [
    typedHttp.get('/bodhi/v1/users', async ({ response }) => {
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
 * Convenience methods for common user list scenarios
 */
export function mockUsersDefault() {
  return mockUsers({ users: mockSimpleUsersResponse.users, total_users: mockSimpleUsersResponse.total });
}

export function mockUsersMultipleAdmins() {
  return mockUsers({ users: mockMultipleAdminsResponse.users, total_users: mockMultipleAdminsResponse.total });
}

export function mockUsersMultipleManagers() {
  return mockUsers({ users: mockMultipleManagersResponse.users, total_users: mockMultipleManagersResponse.total });
}

export function mockUsersEmpty() {
  return mockUsers({ users: mockEmptyUsersResponse.users, total_users: mockEmptyUsersResponse.total });
}

/**
 * Mock handler for user role change endpoint
 */
export function mockUserRoleChange(user_id: string) {
  return [
    typedHttp.put('/bodhi/v1/users/{user_id}/role', async ({ params, response }) => {
      // Only respond if user_id matches
      if (params.user_id !== user_id) {
        return; // Pass through to next handler
      }
      return response(200).empty();
    }),
  ];
}

/**
 * Mock handler for user role change endpoint error responses
 */
export function mockUserRoleChangeError(
  user_id: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 500 } = {}
) {
  return [
    typedHttp.put('/bodhi/v1/users/{user_id}/role', async ({ params, response }) => {
      // Only respond if user_id matches
      if (params.user_id !== user_id) {
        return; // Pass through to next handler
      }
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
 * Convenience methods for role change scenarios
 */
export function mockUserRoleChangeSuccess(user_id: string) {
  return mockUserRoleChange(user_id);
}

/**
 * Mock handler for user removal endpoint
 */
export function mockUserRemove(user_id: string) {
  return [
    typedHttp.delete('/bodhi/v1/users/{user_id}', async ({ params, response }) => {
      // Only respond if user_id matches
      if (params.user_id !== user_id) {
        return; // Pass through to next handler
      }
      return response(200).empty();
    }),
  ];
}

/**
 * Mock handler for user removal endpoint error responses
 */
export function mockUserRemoveError(
  user_id: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 500 } = {}
) {
  return [
    typedHttp.delete('/bodhi/v1/users/{user_id}', async ({ params, response }) => {
      // Only respond if user_id matches
      if (params.user_id !== user_id) {
        return; // Pass through to next handler
      }
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
 * Convenience methods for user removal scenarios
 */
export function mockUserRemoveSuccess(user_id: string) {
  return mockUserRemove(user_id);
}
