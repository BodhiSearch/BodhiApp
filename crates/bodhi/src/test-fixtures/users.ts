import { UserInfo } from '@bodhiapp/ts-client';

export const mockUser1: UserInfo = {
  email: 'user1@example.com',
  name: 'User One',
  role: 'resource_user',
  logged_in: true,
};

export const mockUser2: UserInfo = {
  email: 'user2@example.com',
  name: 'User Two',
  role: 'resource_power_user',
  logged_in: true,
};

export const mockManager: UserInfo = {
  email: 'manager@example.com',
  name: 'Manager User',
  role: 'resource_manager',
  logged_in: true,
};

export const mockAdmin: UserInfo = {
  email: 'admin@example.com',
  name: 'Admin User',
  role: 'resource_admin',
  logged_in: true,
};

export const mockMultiRoleUser: UserInfo = {
  email: 'multi@example.com',
  name: 'Multi Role User',
  role: 'resource_manager',
  logged_in: true,
};

// Mock paginated users response
export const mockUsersResponse = {
  users: [mockUser1, mockUser2, mockManager, mockAdmin],
  total: 4,
  page: 1,
  page_size: 10,
};

export const mockEmptyUsersResponse = {
  users: [],
  total: 0,
  page: 1,
  page_size: 10,
};

// Helper function to create users with specific roles
export const createMockUsersWithRoles = (roles: string[]) => {
  return roles.map((role, index) => ({
    email: `${role}${index}@example.com`,
    name: `${role} User ${index}`,
    role: role,
    logged_in: true,
  }));
};
