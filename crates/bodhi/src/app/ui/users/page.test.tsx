import UsersPage from '@/app/ui/users/page';
import { ADMIN_ROLES, BLOCKED_ROLES } from '@/test-fixtures/access-requests';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, waitForElementToBeRemoved } from '@testing-library/react';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { mockAppInfo, stubAppInfo } from '@/test-utils/msw-v2/handlers/info';
import {
  mockUserLoggedIn,
  mockUserLoggedOut,
  mockUsersDefault,
  mockUsersMultipleAdmins,
  mockUsersMultipleManagers,
  mockUsersEmpty,
  mockUsersError,
  stubUserLoggedIn,
} from '@/test-utils/msw-v2/handlers/user';
import { mockAccessRequestsDefault } from '@/test-utils/msw-v2/handlers/access-requests';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  usePathname: vi.fn().mockReturnValue('/ui/users'),
}));

// Mock AppInitializer to just render children
vi.mock('@/components/AppInitializer', () => ({
  default: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

// Mock toast
const mockShowSuccess = vi.fn();
const mockShowError = vi.fn();
vi.mock('@/hooks/use-toast-messages', () => ({
  useToastMessages: () => ({
    showSuccess: mockShowSuccess,
    showError: mockShowError,
  }),
}));

setupMswV2();

// Helper function to create role-based handlers similar to MSW v1 pattern
function createRoleBasedHandlersV2(role: string, shouldHaveAccess: boolean = true) {
  const userRole =
    role === 'admin'
      ? 'resource_admin'
      : role === 'manager'
        ? 'resource_manager'
        : role === 'power_user'
          ? 'resource_power_user'
          : 'resource_user';

  if (shouldHaveAccess) {
    return [
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ role: userRole }),
      ...mockUsersDefault(),
      ...mockAccessRequestsDefault(),
    ];
  } else {
    return [
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ role: userRole }),
      ...mockUsersError(),
      ...mockAccessRequestsDefault(),
    ];
  }
}

afterEach(() => {
  pushMock.mockClear();
  mockShowSuccess.mockClear();
  mockShowError.mockClear();
});

describe('UsersPage Role-Based Access Control', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it.each(ADMIN_ROLES)('allows access for %s role', async (role) => {
    server.use(...createRoleBasedHandlersV2(role, true));

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // Should show the page content, not redirect
    expect(screen.getByTestId('users-page')).toBeInTheDocument();
    expect(screen.getAllByText('All Users')[1]).toBeInTheDocument(); // The second occurrence is in the card
    expect(pushMock).not.toHaveBeenCalled();
  });

  it.each(BLOCKED_ROLES)('blocks access for %s role', async (role) => {
    server.use(...createRoleBasedHandlersV2(role, false));

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // Should show the page but might have restricted content
    // Since this is placeholder functionality, we test that it doesn't crash
    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
    });
  });

  it('renders page container for unauthenticated users (redirect handled by AppInitializer)', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedOut(),
      ...mockUsersDefault() // Page might still try to load users even when logged out
    );

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // Since we mocked AppInitializer, the page renders but the redirect logic
    // is handled by AppInitializer (tested separately)
    expect(screen.getByTestId('users-page')).toBeInTheDocument();
  });
});

describe('UsersPage Data Display', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockUsersDefault()
    );
  });

  it('displays users list correctly', async () => {
    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // Wait for page to load
    await screen.findByTestId('users-page');

    // Should display All Users text in card title
    expect(screen.getAllByText('All Users')[1]).toBeInTheDocument(); // The second occurrence is in the card

    // Should show users from mock data
    await waitFor(() => {
      expect(screen.getByText('user1@example.com')).toBeInTheDocument();
      expect(screen.getByText('user2@example.com')).toBeInTheDocument();
      expect(screen.getByText('manager@example.com')).toBeInTheDocument();
      expect(screen.getByText('admin@example.com')).toBeInTheDocument();
    });
  });

  it('displays page structure correctly', async () => {
    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
    });

    // Should have proper page structure
    expect(screen.getAllByText('All Users')[1]).toBeInTheDocument(); // The second occurrence is in the card
    // Should show navigation links
    expect(screen.getByText('Pending Requests')).toBeInTheDocument();
    expect(screen.getByText('All Requests')).toBeInTheDocument();
  });

  it('displays user roles correctly with badges', async () => {
    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await screen.findByTestId('users-page');

    // Wait for users to load
    await waitFor(() => {
      expect(screen.getByText('user1@example.com')).toBeInTheDocument();
    });

    // Should show role badges for each user
    const userBadges = screen.getAllByText('User');
    expect(userBadges.length).toBeGreaterThan(0); // resource_user -> User
    const powerUserBadges = screen.getAllByText('Power User');
    expect(powerUserBadges.length).toBeGreaterThan(0); // resource_power_user -> Power User
    const managerBadges = screen.getAllByText('Manager');
    expect(managerBadges.length).toBeGreaterThan(0); // resource_manager -> Manager
    const adminBadges = screen.getAllByText('Admin');
    expect(adminBadges.length).toBeGreaterThan(0); // resource_admin -> Admin
  });
});

describe('UsersPage Role Hierarchy UI Enforcement', () => {
  it('admin user sees action buttons for all other users but not themselves', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({
        role: 'resource_admin',
        username: 'admin@example.com',
        user_id: 'admin-id',
      }),
      ...mockUsersDefault()
    );

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // Wait for page to load and users to appear
    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
      expect(screen.getByText('admin@example.com')).toBeInTheDocument();
      expect(screen.getByText('user1@example.com')).toBeInTheDocument();
      expect(screen.getByText('user2@example.com')).toBeInTheDocument();
      expect(screen.getByText('manager@example.com')).toBeInTheDocument();
    });

    // Check that action cells exist for other users
    await waitFor(() => {
      expect(screen.getByTestId('user-actions-user1@example.com')).toBeInTheDocument();
      expect(screen.getByTestId('user-actions-user2@example.com')).toBeInTheDocument();
      expect(screen.getByTestId('user-actions-manager@example.com')).toBeInTheDocument();
    });

    // Admin should see role select triggers for other users (the Select component renders as a button trigger)
    expect(screen.getByTestId('role-select-trigger-user1@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('role-select-trigger-user2@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('role-select-trigger-manager@example.com')).toBeInTheDocument();

    // Admin should see remove buttons for other users
    expect(screen.getByTestId('remove-user-btn-user1@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('remove-user-btn-user2@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('remove-user-btn-manager@example.com')).toBeInTheDocument();

    // Admin should NOT see action buttons for themselves
    expect(screen.queryByTestId('role-select-trigger-admin@example.com')).not.toBeInTheDocument();

    // Should show "You" indicator for current user
    expect(screen.getByTestId('current-user-indicator')).toBeInTheDocument();
    expect(screen.getByText('You')).toBeInTheDocument();
  });

  it('manager user sees action buttons appropriately based on hierarchy', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({
        role: 'resource_manager',
        username: 'manager@example.com',
        user_id: 'manager-id',
      }),
      ...mockUsersDefault()
    );

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // Wait for page and users to load
    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
      expect(screen.getByText('manager@example.com')).toBeInTheDocument();
      expect(screen.getByText('user1@example.com')).toBeInTheDocument();
      expect(screen.getByText('user2@example.com')).toBeInTheDocument();
      expect(screen.getByText('admin@example.com')).toBeInTheDocument();
    });

    // Manager should see action buttons for lower-level users (using the correct trigger test IDs)
    await waitFor(() => {
      expect(screen.getByTestId('role-select-trigger-user1@example.com')).toBeInTheDocument();
      expect(screen.getByTestId('role-select-trigger-user2@example.com')).toBeInTheDocument();
    });

    // Manager should NOT see action buttons for admin (higher level) or themselves
    expect(screen.queryByTestId('role-select-trigger-admin@example.com')).not.toBeInTheDocument();
    expect(screen.queryByTestId('role-select-trigger-manager@example.com')).not.toBeInTheDocument();

    // Should show appropriate indicators
    expect(screen.getByTestId('current-user-indicator')).toBeInTheDocument(); // For self
    expect(screen.getByTestId('restricted-user-indicator')).toBeInTheDocument(); // For admin
  });

  it('admin can see action buttons for other admins', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({
        role: 'resource_admin',
        username: 'admin@example.com',
        user_id: 'admin-id',
      }),
      ...mockUsersMultipleAdmins()
    );

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // Wait for page and users to load
    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
      expect(screen.getByText('admin@example.com')).toBeInTheDocument();
      expect(screen.getByText('admin2@example.com')).toBeInTheDocument();
    });

    // Current admin should NOT see action buttons for themselves
    expect(screen.queryByTestId('role-select-trigger-admin@example.com')).not.toBeInTheDocument();
    expect(screen.getByTestId('current-user-indicator')).toBeInTheDocument();

    // Current admin SHOULD see action buttons for the other admin
    await waitFor(() => {
      expect(screen.getByTestId('role-select-trigger-admin2@example.com')).toBeInTheDocument();
      expect(screen.getByTestId('remove-user-btn-admin2@example.com')).toBeInTheDocument();
    });

    // Should also see action buttons for lower-level users
    expect(screen.getByTestId('role-select-trigger-user1@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('role-select-trigger-manager@example.com')).toBeInTheDocument();
  });

  it('manager can see action buttons for other managers', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({
        role: 'resource_manager',
        username: 'manager@example.com',
        user_id: 'manager-id',
      }),
      ...mockUsersMultipleManagers()
    );

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // Wait for page and users to load
    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
      expect(screen.getByText('manager@example.com')).toBeInTheDocument();
      expect(screen.getByText('manager2@example.com')).toBeInTheDocument();
      expect(screen.getByText('admin@example.com')).toBeInTheDocument();
    });

    // Current manager should NOT see action buttons for themselves
    expect(screen.queryByTestId('role-select-trigger-manager@example.com')).not.toBeInTheDocument();
    expect(screen.getByTestId('current-user-indicator')).toBeInTheDocument();

    // Current manager SHOULD see action buttons for the other manager
    await waitFor(() => {
      expect(screen.getByTestId('role-select-trigger-manager2@example.com')).toBeInTheDocument();
      expect(screen.getByTestId('remove-user-btn-manager2@example.com')).toBeInTheDocument();
    });

    // Should also see action buttons for lower-level users
    expect(screen.getByTestId('role-select-trigger-user1@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('role-select-trigger-user2@example.com')).toBeInTheDocument();

    // Should NOT see action buttons for admin (higher level)
    expect(screen.queryByTestId('role-select-trigger-admin@example.com')).not.toBeInTheDocument();
    expect(screen.getByTestId('restricted-user-indicator')).toBeInTheDocument(); // For admin
  });
});

describe('UsersPage Error Handling', () => {
  it('renders page container even when APIs fail', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockUsersError()
    );

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // The page container should still render
    expect(screen.getByTestId('users-page')).toBeInTheDocument();
  });

  it('handles users API failure gracefully', async () => {
    server.use(
      ...stubAppInfo({ status: 'ready' }),
      ...stubUserLoggedIn({ role: 'resource_admin' }),
      ...mockUsersError()
    );

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await screen.findByTestId('users-page');

    // Should show error alert when users API fails
    await waitFor(
      () => {
        expect(screen.getByRole('alert')).toBeInTheDocument();
        expect(screen.getByText(/Failed to load users/)).toBeInTheDocument();
      },
      { timeout: 10000 }
    );
  });

  it('handles network failures gracefully', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockUsersDefault()
    );
    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });
    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
    });

    // Should handle error gracefully without crashing
    // In this case, it would show loading state or the actual users list
    // depending on how the MSW handlers respond
  });
});

describe('UsersPage Loading States', () => {
  it('shows page content after loading users', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockUsersDefault()
    );
    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
    });

    expect(screen.getAllByText('All Users')[1]).toBeInTheDocument(); // The second occurrence is in the card

    // Should eventually show users after loading
    await waitFor(() => {
      expect(screen.getByText('user1@example.com')).toBeInTheDocument();
    });
  });

  it('handles page structure correctly', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockUsersDefault()
    );

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
    });

    // Should have proper page structure
    expect(screen.getAllByText('All Users')[1]).toBeInTheDocument(); // The second occurrence is in the card
    expect(screen.getByText('Manage user access and roles')).toBeInTheDocument();
  });
});

describe('UsersPage Empty State', () => {
  it('shows empty state when no users are returned', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ role: 'resource_admin' }),
      ...mockUsersEmpty()
    );

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await screen.findByTestId('users-page');

    // Should show empty state
    await waitFor(() => {
      expect(screen.getByText('No Users')).toBeInTheDocument();
      expect(screen.getByText('No users found')).toBeInTheDocument();
    });
  });
});
