import { UserInfo, UserInfoResponse, UserListResponse } from '@bodhiapp/ts-client';

export const mockUser1: UserInfo = {
  username: 'user1@example.com',
  role: 'resource_user',
  logged_in: true,
};

export const mockUser2: UserInfo = {
  username: 'user2@example.com',
  role: 'resource_power_user',
  logged_in: true,
};

export const mockManager: UserInfo = {
  username: 'manager@example.com',
  role: 'resource_manager',
  logged_in: true,
};

export const mockAdmin: UserInfo = {
  username: 'admin@example.com',
  role: 'resource_admin',
  logged_in: true,
};

export const mockMultiRoleUser: UserInfo = {
  username: 'multi@example.com',
  role: 'resource_manager',
  logged_in: true,
};

// UserInfoResponse objects for the users list API
export const mockUserInfoResponse1: UserInfoResponse = {
  user_id: 'user1-id',
  username: 'user1@example.com',
  role: 'resource_user',
  first_name: 'User',
  last_name: 'One',
};

export const mockUserInfoResponse2: UserInfoResponse = {
  user_id: 'user2-id',
  username: 'user2@example.com',
  role: 'resource_power_user',
  first_name: 'User',
  last_name: 'Two',
};

export const mockManagerInfoResponse: UserInfoResponse = {
  user_id: 'manager-id',
  username: 'manager@example.com',
  role: 'resource_manager',
  first_name: 'Manager',
  last_name: 'User',
};

export const mockAdminInfoResponse: UserInfoResponse = {
  user_id: 'admin-id',
  username: 'admin@example.com',
  role: 'resource_admin',
  first_name: 'Admin',
  last_name: 'User',
};

// Mock simple paginated response for hook compatibility
export const mockSimpleUsersResponse = {
  users: [mockUserInfoResponse1, mockUserInfoResponse2, mockManagerInfoResponse, mockAdminInfoResponse],
  total: 4,
  page: 1,
  page_size: 10,
};

// Mock paginated users response
export const mockUsersResponse: UserListResponse = {
  client_id: 'test-client-id',
  users: [mockUserInfoResponse1, mockUserInfoResponse2, mockManagerInfoResponse, mockAdminInfoResponse],
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
  return roles.map((role, index) => ({
    username: `${role}${index}@example.com`,
    role: role as any,
    logged_in: true,
  }));
};

// Helper function to create UserInfoResponse objects with specific roles
export const createMockUserInfoResponses = (roles: string[]): UserInfoResponse[] => {
  return roles.map((role, index) => ({
    user_id: `${role}${index}-id`,
    username: `${role}${index}@example.com`,
    role: role,
    first_name: 'Test',
    last_name: `User${index}`,
  }));
};

// Mock for current admin user
export const createMockCurrentAdminUser = (username = 'current-admin@example.com'): UserInfo => ({
  username,
  role: 'resource_admin',
  logged_in: true,
});
