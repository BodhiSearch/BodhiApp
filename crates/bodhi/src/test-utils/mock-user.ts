import { UserResponse, AppRole } from '@bodhiapp/ts-client';

/**
 * Create a type-safe mock UserResponse for logged-out state
 */
export function createMockLoggedOutUser(): UserResponse {
  return {
    auth_status: 'logged_out',
  };
}

/**
 * Create a type-safe mock UserResponse for logged-in state
 */
export function createMockLoggedInUser(
  options: {
    username?: string;
    role?: AppRole | string;
    user_id?: string;
    first_name?: string | null;
    last_name?: string | null;
  } = {}
): UserResponse {
  const {
    username = 'test@example.com',
    role,
    user_id = '550e8400-e29b-41d4-a716-446655440000',
    first_name = null,
    last_name = null,
  } = options;

  return {
    auth_status: 'logged_in',
    user_id,
    username,
    first_name,
    last_name,
    ...(role && { role: role as AppRole }),
  };
}

/**
 * Create a mock UserResponse for admin user
 */
export function createMockAdminUser(username = 'admin@example.com'): UserResponse {
  return createMockLoggedInUser({
    username,
    role: 'resource_admin',
    user_id: 'admin-id',
    first_name: 'Admin',
    last_name: 'User',
  });
}

/**
 * Create a mock UserResponse for manager user
 */
export function createMockManagerUser(username = 'manager@example.com'): UserResponse {
  return createMockLoggedInUser({
    username,
    role: 'resource_manager',
    user_id: 'manager-id',
    first_name: 'Manager',
    last_name: 'User',
  });
}

/**
 * Create a mock UserResponse for regular user
 */
export function createMockRegularUser(username = 'user@example.com'): UserResponse {
  return createMockLoggedInUser({
    username,
    role: 'resource_user',
    user_id: 'user-id',
    first_name: 'Regular',
    last_name: 'User',
  });
}

/**
 * Backward compatibility: Create a mock user with just logged_in boolean
 * @deprecated Use createMockLoggedInUser or createMockLoggedOutUser instead
 */
export function createMockUser(
  loggedIn: boolean,
  username = 'test@example.com',
  role?: AppRole | string
): UserResponse {
  if (!loggedIn) {
    return createMockLoggedOutUser();
  }
  return createMockLoggedInUser({ username, role });
}

/**
 * Legacy compatibility for inline object creation
 * @deprecated Use specific mock functions instead
 */
export const mockUserResponses = {
  loggedOut: createMockLoggedOutUser(),
  loggedIn: createMockLoggedInUser(),
  admin: createMockAdminUser(),
  manager: createMockManagerUser(),
  user: createMockRegularUser(),
};
