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
import { ENDPOINT_USER_INFO, ENDPOINT_USERS, ENDPOINT_USER_ROLE, ENDPOINT_USER_ID } from '@/hooks/useQuery';
import {
  mockSimpleUsersResponse,
  mockMultipleAdminsResponse,
  mockMultipleManagersResponse,
  mockEmptyUsersResponse,
} from '@/test-fixtures/users';

// ============================================================================
// Stub Handlers (No closure state - for tests requiring multiple responses)
// ============================================================================

/**
 * Core response logic for user logged in endpoint (shared by mock and stub handlers)
 * @private
 */
async function createUserLoggedInResponse(
  {
    user_id = '550e8400-e29b-41d4-a716-446655440000',
    username = 'test@example.com',
    first_name = null,
    last_name = null,
    role = null,
    ...rest
  }: Partial<components['schemas']['UserInfo']> = {},
  delayMs: number | undefined,
  response: any
) {
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
  return response(200 as const).json(responseData);
}

/**
 * Stub handler for user info endpoint that responds every time (no closure state)
 * Use this for tests that require multiple API calls to the same endpoint
 */
export function stubUserLoggedIn(
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
    typedHttp.get(ENDPOINT_USER_INFO, async ({ response }) => {
      return createUserLoggedInResponse({ user_id, username, first_name, last_name, role, ...rest }, delayMs, response);
    }),
  ];
}

// ============================================================================
// User Info Endpoint (GET /bodhi/v1/user)
// ============================================================================

/**
 * Mock handler for user info endpoint - logged out state
 * Uses generated OpenAPI types directly
 */
export function mockUserLoggedOut() {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_USER_INFO, ({ response }) => {
      if (hasBeenCalled) return;
      hasBeenCalled = true;
      return response(200 as const).json({
        auth_status: 'logged_out',
      });
    }),
  ];
}

/**
 * Mock handler for user info endpoint - logged in state with configurable fields
 * Uses generated OpenAPI types directly
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
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_USER_INFO, async ({ response }) => {
      if (hasBeenCalled) return;
      hasBeenCalled = true;
      return createUserLoggedInResponse({ user_id, username, first_name, last_name, role, ...rest }, delayMs, response);
    }),
  ];
}

/**
 * Mock handler for user info endpoint error responses
 * Uses generated OpenAPI types directly
 */
export function mockUserInfoError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 500 } = {}) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_USER_INFO, async ({ response }) => {
      if (hasBeenCalled) return;
      hasBeenCalled = true;
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

// ============================================================================
// Users List Endpoint (GET /bodhi/v1/users)
// ============================================================================

/**
 * Mock handler for users list endpoint with configurable response data
 * Uses generated OpenAPI types directly
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
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_USERS, async ({ response }) => {
      if (hasBeenCalled) return;
      hasBeenCalled = true;
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
      return response(200 as const).json(responseData);
    }),
  ];
}

/**
 * Mock handler for users list endpoint error responses
 * Uses generated OpenAPI types directly
 */
export function mockUsersError({
  code = INTERNAL_SERVER_ERROR.code,
  message = 'Failed to fetch users',
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 500 } = {}) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_USERS, async ({ response }) => {
      if (hasBeenCalled) return;
      hasBeenCalled = true;
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

// ============================================================================
// Convenience Methods for Users List
// ============================================================================

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

// ============================================================================
// User Role Change Endpoint (PUT /bodhi/v1/users/{user_id}/role)
// ============================================================================

/**
 * Mock handler for user role change endpoint
 * Uses generated OpenAPI types directly
 */
export function mockUserRoleChange(user_id: string) {
  let hasBeenCalled = false;
  return [
    typedHttp.put(ENDPOINT_USER_ROLE, async ({ params, response }) => {
      // Only respond if user_id matches
      if (params.user_id !== user_id) {
        return; // Pass through to next handler
      }
      if (hasBeenCalled) return;
      hasBeenCalled = true;
      return response(200).empty();
    }),
  ];
}

/**
 * Mock handler for user role change endpoint error responses
 * Uses generated OpenAPI types directly
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
  let hasBeenCalled = false;
  return [
    typedHttp.put(ENDPOINT_USER_ROLE, async ({ params, response }) => {
      // Only respond if user_id matches
      if (params.user_id !== user_id) {
        return; // Pass through to next handler
      }
      if (hasBeenCalled) return;
      hasBeenCalled = true;
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

// ============================================================================
// User Removal Endpoint (DELETE /bodhi/v1/users/{user_id})
// ============================================================================

/**
 * Mock handler for user removal endpoint
 * Uses generated OpenAPI types directly
 */
export function mockUserRemove(user_id: string) {
  let hasBeenCalled = false;
  return [
    typedHttp.delete(ENDPOINT_USER_ID, async ({ params, response }) => {
      // Only respond if user_id matches
      if (params.user_id !== user_id) {
        return; // Pass through to next handler
      }
      if (hasBeenCalled) return;
      hasBeenCalled = true;
      return response(200).empty();
    }),
  ];
}

/**
 * Mock handler for user removal endpoint error responses
 * Uses generated OpenAPI types directly
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
  let hasBeenCalled = false;
  return [
    typedHttp.delete(ENDPOINT_USER_ID, async ({ params, response }) => {
      // Only respond if user_id matches
      if (params.user_id !== user_id) {
        return; // Pass through to next handler
      }
      if (hasBeenCalled) return;
      hasBeenCalled = true;
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
