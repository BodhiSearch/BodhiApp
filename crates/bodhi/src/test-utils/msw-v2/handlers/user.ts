import { delay } from 'msw';

import { ENDPOINT_USER_INFO, ENDPOINT_USERS, ENDPOINT_USER_ROLE, ENDPOINT_USER_ID } from '@/hooks/users';
import {
  mockSimpleUsersResponse,
  mockMultipleAdminsResponse,
  mockMultipleManagersResponse,
  mockEmptyUsersResponse,
} from '@/test-fixtures/users';

import { typedHttp, type components, INTERNAL_SERVER_ERROR } from '../setup';

export function mockUserLoggedOut({
  stub,
  ...rest
}: { stub?: boolean; dashboard?: components['schemas']['DashboardUser'] | null } = {}) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_USER_INFO, ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      return response(200 as const).json({
        auth_status: 'logged_out',
        ...rest,
      });
    }),
  ];
}

export function mockUserLoggedIn(
  {
    user_id = '550e8400-e29b-41d4-a716-446655440000',
    username = 'test@example.com',
    first_name = null,
    last_name = null,
    role = null,
    dashboard,
    ...rest
  }: Partial<components['schemas']['UserInfo']> & { dashboard?: components['schemas']['DashboardUser'] | null } = {},
  { delayMs, stub }: { delayMs?: number; stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_USER_INFO, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

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
        ...(dashboard ? { dashboard } : {}),
        ...rest,
      };
      return response(200 as const).json(responseData);
    }),
  ];
}

export function mockUserInfoError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_USER_INFO, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
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

export function mockUsers(
  {
    client_id = 'resource-test-client',
    users = mockSimpleUsersResponse.users,
    page = 1,
    page_size = 10,
    total_pages = 1,
    total_users = mockSimpleUsersResponse.total,
    has_next = false,
    has_previous = false,
    ...rest
  }: Partial<components['schemas']['UserListResponse']> = {},
  { delayMs, stub }: { delayMs?: number; stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_USERS, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      if (delayMs) {
        await delay(delayMs);
      }
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

export function mockUsersError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = 'Failed to fetch users',
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_USERS, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
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

export function mockUserRoleChange(user_id: string, { stub }: { stub?: boolean } = {}) {
  let hasBeenCalled = false;
  return [
    typedHttp.put(ENDPOINT_USER_ROLE, async ({ params, response }) => {
      if (params.user_id !== user_id) {
        return; // undefined falls through to the next handler
      }
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      return response(200).empty();
    }),
  ];
}

export function mockUserRoleChangeError(
  user_id: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.put(ENDPOINT_USER_ROLE, async ({ params, response }) => {
      if (params.user_id !== user_id) {
        return; // undefined falls through to the next handler
      }
      if (hasBeenCalled && !stub) return;
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

export function mockUserRemove(user_id: string, { stub }: { stub?: boolean } = {}) {
  let hasBeenCalled = false;
  return [
    typedHttp.delete(ENDPOINT_USER_ID, async ({ params, response }) => {
      if (params.user_id !== user_id) {
        return; // undefined falls through to the next handler
      }
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      return response(200).empty();
    }),
  ];
}

export function mockUserRemoveError(
  user_id: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.delete(ENDPOINT_USER_ID, async ({ params, response }) => {
      if (params.user_id !== user_id) {
        return; // undefined falls through to the next handler
      }
      if (hasBeenCalled && !stub) return;
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
