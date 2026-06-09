import { PaginatedUserAccessResponse, UserAccessStatusResponse, UserAccessRequest } from '@bodhiapp/ts-client';

export const mockPendingRequest: UserAccessRequest = {
  id: '01HQXYZ0000000000000000001',
  user_id: '550e8400-e29b-41d4-a716-446655440001',
  username: 'user@example.com',
  status: 'pending',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  reviewer: null,
};

export const mockApprovedRequest: UserAccessRequest = {
  id: '01HQXYZ0000000000000000002',
  user_id: '550e8400-e29b-41d4-a716-446655440002',
  username: 'approved@example.com',
  status: 'approved',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-02T00:00:00Z',
  reviewer: 'admin@example.com',
};

export const mockRejectedRequest: UserAccessRequest = {
  id: '01HQXYZ0000000000000000003',
  user_id: '550e8400-e29b-41d4-a716-446655440003',
  username: 'rejected@example.com',
  status: 'rejected',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-02T00:00:00Z',
  reviewer: 'admin@example.com',
};

export const mockPendingRequests: PaginatedUserAccessResponse = {
  requests: [mockPendingRequest],
  total: 1,
  page: 1,
  page_size: 10,
};

export const mockAllRequests: PaginatedUserAccessResponse = {
  requests: [mockPendingRequest, mockApprovedRequest, mockRejectedRequest],
  total: 3,
  page: 1,
  page_size: 10,
};

export const mockEmptyRequests: PaginatedUserAccessResponse = {
  requests: [],
  total: 0,
  page: 1,
  page_size: 10,
};

export const mockUserAccessStatusPending: UserAccessStatusResponse = {
  status: 'pending',
  username: 'user@example.com',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

export const mockUserAccessStatusApproved: UserAccessStatusResponse = {
  status: 'approved',
  username: 'approved@example.com',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-02T00:00:00Z',
};

export const mockUserAccessStatusRejected: UserAccessStatusResponse = {
  status: 'rejected',
  username: 'rejected@example.com',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-02T00:00:00Z',
};

export const ADMIN_ROLES = ['manager', 'admin'] as const;
export const BLOCKED_ROLES = ['user', 'power_user'] as const;
export const ALL_ROLES = [...ADMIN_ROLES, ...BLOCKED_ROLES] as const;

export const createMockUserInfo = (role?: string | null, usernameOrLoggedIn: string | boolean = true) => {
  if (!usernameOrLoggedIn) {
    return {
      auth_status: 'logged_out' as const,
    };
  }

  // user_id values must match test-fixtures/users.ts
  const getUserId = (username: string): string => {
    const userIdMap: Record<string, string> = {
      'admin@example.com': 'admin-id',
      'manager@example.com': 'manager-id',
      'user1@example.com': 'user1-id',
      'user2@example.com': 'user2-id',
      'user@example.com': 'user-general-id',
      'power_user@example.com': 'power-user-id',
    };
    return userIdMap[username] || '550e8400-e29b-41d4-a716-446655440000';
  };

  if (typeof usernameOrLoggedIn === 'string') {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const result: any = {
      auth_status: 'logged_in' as const,
      user_id: getUserId(usernameOrLoggedIn),
      username: usernameOrLoggedIn,
      first_name: null,
      last_name: null,
    };

    // Only add role if provided and not null/undefined (backend doesn't send role field for users without roles)
    if (role !== null && role !== undefined) {
      const resourceRole = role.startsWith('resource_') ? role : `resource_${role}`;
      result.role = resourceRole;
    }

    return result;
  }

  const username = `${role}@example.com`;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const result: any = {
    auth_status: 'logged_in' as const,
    user_id: getUserId(username),
    username: username,
    first_name: null,
    last_name: null,
  };

  // Only add role if provided and not null/undefined (backend doesn't send role field for users without roles)
  if (role !== null && role !== undefined) {
    const resourceRole = role.startsWith('resource_') ? role : `resource_${role}`;
    result.role = resourceRole;
  }

  return result;
};
