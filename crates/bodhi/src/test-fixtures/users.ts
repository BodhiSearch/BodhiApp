import { UserInfo, UserListResponse, AppRole } from '@bodhiapp/ts-client';
import { createMockLoggedInUser } from '@/test-utils/mock-user';

export const mockUser1: UserInfo = {
  user_id: 'user1-id',
  username: 'user1@example.com',
  first_name: 'User',
  last_name: 'One',
  role: 'resource_user',
};

export const mockUser2: UserInfo = {
  user_id: 'user2-id',
  username: 'user2@example.com',
  first_name: 'User',
  last_name: 'Two',
  role: 'resource_power_user',
};

export const mockManager: UserInfo = {
  user_id: 'manager-id',
  username: 'manager@example.com',
  first_name: 'Manager',
  last_name: 'User',
  role: 'resource_manager',
};

export const mockAdmin: UserInfo = {
  user_id: 'admin-id',
  username: 'admin@example.com',
  first_name: 'Admin',
  last_name: 'User',
  role: 'resource_admin',
};

export const mockMultiRoleUser: UserInfo = {
  user_id: 'multi-id',
  username: 'multi@example.com',
  first_name: 'Multi',
  last_name: 'Role',
  role: 'resource_manager',
};

// UserInfo objects for the users list API
export const mockUserInfo1: UserInfo = {
  user_id: 'user1-id',
  username: 'user1@example.com',
  role: 'resource_user',
  first_name: 'User',
  last_name: 'One',
};

export const mockUserInfo2: UserInfo = {
  user_id: 'user2-id',
  username: 'user2@example.com',
  role: 'resource_power_user',
  first_name: 'User',
  last_name: 'Two',
};

export const mockManagerInfoResponse: UserInfo = {
  user_id: 'manager-id',
  username: 'manager@example.com',
  role: 'resource_manager',
  first_name: 'Manager',
  last_name: 'User',
};

export const mockAdminInfoResponse: UserInfo = {
  user_id: 'admin-id',
  username: 'admin@example.com',
  role: 'resource_admin',
  first_name: 'Admin',
  last_name: 'User',
};

export const mockSecondAdminInfoResponse: UserInfo = {
  user_id: 'admin2-id',
  username: 'admin2@example.com',
  role: 'resource_admin',
  first_name: 'Second',
  last_name: 'Admin',
};

export const mockSecondManagerInfoResponse: UserInfo = {
  user_id: 'manager2-id',
  username: 'manager2@example.com',
  role: 'resource_manager',
  first_name: 'Second',
  last_name: 'Manager',
};

// Mock simple paginated response for hook compatibility
export const mockSimpleUsersResponse = {
  users: [mockUserInfo1, mockUserInfo2, mockManagerInfoResponse, mockAdminInfoResponse],
  total: 4,
  page: 1,
  page_size: 10,
};

// Mock response with multiple admins for testing admin-to-admin modifications
export const mockMultipleAdminsResponse = {
  users: [mockUserInfo1, mockUserInfo2, mockManagerInfoResponse, mockAdminInfoResponse, mockSecondAdminInfoResponse],
  total: 5,
  page: 1,
  page_size: 10,
};

// Mock response with multiple managers for testing manager-to-manager modifications
export const mockMultipleManagersResponse = {
  users: [mockUserInfo1, mockUserInfo2, mockManagerInfoResponse, mockSecondManagerInfoResponse, mockAdminInfoResponse],
  total: 5,
  page: 1,
  page_size: 10,
};

// Mock paginated users response
export const mockUsersResponse: UserListResponse = {
  client_id: 'test-client-id',
  users: [mockUserInfo1, mockUserInfo2, mockManagerInfoResponse, mockAdminInfoResponse],
  page: 1,
  page_size: 10,
  total_pages: 1,
  total_users: 4,
  has_next: false,
  has_previous: false,
};

export const mockEmptyUsersResponse = {
  users: [],
  total: 0,
  page: 1,
  page_size: 10,
};

// Helper function to create users with specific roles
export const createMockUsersWithRoles = (roles: string[]): UserInfo[] => {
  return roles.map((role, index) => {
    const mockUser = createMockLoggedInUser({ username: `${role}${index}@example.com`, role });
    if (mockUser.auth_status === 'logged_in') {
      const { auth_status, ...userInfo } = mockUser;
      return userInfo;
    }
    // Fallback for logged_out users (shouldn't happen in this context)
    return {
      user_id: `${role}${index}-id`,
      username: `${role}${index}@example.com`,
      first_name: null,
      last_name: null,
      role: role as AppRole,
    };
  });
};

// Helper function to create UserInfo objects with specific roles
export const createMockUserInfos = (roles: string[]): UserInfo[] => {
  return roles.map((role, index) => ({
    user_id: `${role}${index}-id`,
    username: `${role}${index}@example.com`,
    role: role as AppRole,
    first_name: 'Test',
    last_name: `User${index}`,
  }));
};

// Mock for current admin user
export const createMockCurrentAdminUser = (username = 'current-admin@example.com'): UserInfo => {
  const mockUser = createMockLoggedInUser({ username, role: 'resource_admin' });
  if (mockUser.auth_status === 'logged_in') {
    const { auth_status, ...userInfo } = mockUser;
    return userInfo;
  }
  // Fallback
  return {
    user_id: 'admin-id',
    username,
    first_name: 'Admin',
    last_name: 'User',
    role: 'resource_admin' as AppRole,
  };
};
