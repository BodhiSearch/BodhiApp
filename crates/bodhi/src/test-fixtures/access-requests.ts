import { PaginatedUserAccessResponse, UserAccessStatusResponse, UserAccessRequest } from '@bodhiapp/ts-client';

export const mockPendingRequest: UserAccessRequest = {
  id: 1,
  email: 'user@example.com',
  name: 'Test User',
  status: 'pending',
  created_at: '2024-01-01T00:00:00Z',
  processed_at: null,
  processed_by: null,
  role: null,
};

export const mockApprovedRequest: UserAccessRequest = {
  id: 2,
  email: 'approved@example.com',
  name: 'Approved User',
  status: 'approved',
  created_at: '2024-01-01T00:00:00Z',
  processed_at: '2024-01-02T00:00:00Z',
  processed_by: 'admin@example.com',
  role: 'resource_user',
};

export const mockRejectedRequest: UserAccessRequest = {
  id: 3,
  email: 'rejected@example.com',
  name: 'Rejected User',
  status: 'rejected',
  created_at: '2024-01-01T00:00:00Z',
  processed_at: '2024-01-02T00:00:00Z',
  processed_by: 'admin@example.com',
  role: null,
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

// User access status scenarios
export const mockUserAccessStatusNone: UserAccessStatusResponse = {
  status: 'none',
  email: 'user@example.com',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

export const mockUserAccessStatusPending: UserAccessStatusResponse = {
  status: 'pending',
  email: 'user@example.com',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

export const mockUserAccessStatusApproved: UserAccessStatusResponse = {
  status: 'approved',
  email: 'approved@example.com',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-02T00:00:00Z',
};

export const mockUserAccessStatusRejected: UserAccessStatusResponse = {
  status: 'rejected',
  email: 'rejected@example.com',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-02T00:00:00Z',
};

// Role definitions for parameterized tests
export const ADMIN_ROLES = ['manager', 'admin'] as const;
export const BLOCKED_ROLES = ['user', 'power_user'] as const;
export const ALL_ROLES = [...ADMIN_ROLES, ...BLOCKED_ROLES] as const;

// Create mock user info for different roles
export const createMockUserInfo = (role: string, loggedIn: boolean = true) => {
  // Ensure role has resource_ prefix for consistency
  const resourceRole = role.startsWith('resource_') ? role : `resource_${role}`;
  return {
    logged_in: loggedIn,
    email: `${role}@example.com`,
    name: `${role} User`,
    role: loggedIn ? resourceRole : null,
  };
};
