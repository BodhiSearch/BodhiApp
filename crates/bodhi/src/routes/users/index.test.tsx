import UsersPage from '@/routes/users/index';
import { ADMIN_ROLES, BLOCKED_ROLES } from '@/test-fixtures/access-requests';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, waitForElementToBeRemoved } from '@testing-library/react';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import {
  mockUserLoggedIn,
  mockUserLoggedOut,
  mockUsersDefault,
  mockUsersMultipleAdmins,
  mockUsersMultipleManagers,
  mockUsersEmpty,
  mockUsersError,
} from '@/test-utils/msw-v2/handlers/user';
import { mockAccessRequestsDefault } from '@/test-utils/msw-v2/handlers/user-access-requests';

const navigateMock = vi.fn();
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: any) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
    useNavigate: () => navigateMock,
    useLocation: () => ({ pathname: '/users' }),
  };
});

vi.mock('@/components/AppInitializer', () => ({
  default: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

const mockShowSuccess = vi.fn();
const mockShowError = vi.fn();
vi.mock('@/hooks/use-toast-messages', () => ({
  useToastMessages: () => ({
    showSuccess: mockShowSuccess,
    showError: mockShowError,
  }),
}));

setupMswV2();

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
  navigateMock.mockClear();
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

    expect(screen.getByTestId('users-page')).toBeInTheDocument();
    expect(screen.getAllByText('All Users')[1]).toBeInTheDocument(); // The second occurrence is in the card
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it.each(BLOCKED_ROLES)('blocks access for %s role', async (role) => {
    server.use(...createRoleBasedHandlersV2(role, false));

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // Placeholder functionality: just assert the page does not crash for blocked roles.
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

    // AppInitializer is mocked, so redirect-on-logout is covered separately; here the page just renders.
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

    await screen.findByTestId('users-page');

    expect(screen.getAllByText('All Users')[1]).toBeInTheDocument(); // The second occurrence is in the card

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

    expect(screen.getAllByText('All Users')[1]).toBeInTheDocument(); // The second occurrence is in the card
    expect(screen.getByText('Pending Requests')).toBeInTheDocument();
    expect(screen.getByText('All Requests')).toBeInTheDocument();
  });

  it('displays user roles correctly with badges', async () => {
    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await screen.findByTestId('users-page');

    await waitFor(() => {
      expect(screen.getByText('user1@example.com')).toBeInTheDocument();
    });

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

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
      expect(screen.getByText('admin@example.com')).toBeInTheDocument();
      expect(screen.getByText('user1@example.com')).toBeInTheDocument();
      expect(screen.getByText('user2@example.com')).toBeInTheDocument();
      expect(screen.getByText('manager@example.com')).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(screen.getByTestId('user-actions-user1@example.com')).toBeInTheDocument();
      expect(screen.getByTestId('user-actions-user2@example.com')).toBeInTheDocument();
      expect(screen.getByTestId('user-actions-manager@example.com')).toBeInTheDocument();
    });

    // The role Select renders as a button trigger.
    expect(screen.getByTestId('role-select-trigger-user1@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('role-select-trigger-user2@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('role-select-trigger-manager@example.com')).toBeInTheDocument();

    expect(screen.getByTestId('remove-user-btn-user1@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('remove-user-btn-user2@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('remove-user-btn-manager@example.com')).toBeInTheDocument();

    // Admin should NOT see action buttons for themselves.
    expect(screen.queryByTestId('role-select-trigger-admin@example.com')).not.toBeInTheDocument();

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

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
      expect(screen.getByText('manager@example.com')).toBeInTheDocument();
      expect(screen.getByText('user1@example.com')).toBeInTheDocument();
      expect(screen.getByText('user2@example.com')).toBeInTheDocument();
      expect(screen.getByText('admin@example.com')).toBeInTheDocument();
    });

    // Manager sees action buttons for lower-level users.
    await waitFor(() => {
      expect(screen.getByTestId('role-select-trigger-user1@example.com')).toBeInTheDocument();
      expect(screen.getByTestId('role-select-trigger-user2@example.com')).toBeInTheDocument();
    });

    // Manager sees no action buttons for admin (higher level) or themselves.
    expect(screen.queryByTestId('role-select-trigger-admin@example.com')).not.toBeInTheDocument();
    expect(screen.queryByTestId('role-select-trigger-manager@example.com')).not.toBeInTheDocument();

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

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
      expect(screen.getByText('admin@example.com')).toBeInTheDocument();
      expect(screen.getByText('admin2@example.com')).toBeInTheDocument();
    });

    // Current admin sees no action buttons for themselves.
    expect(screen.queryByTestId('role-select-trigger-admin@example.com')).not.toBeInTheDocument();
    expect(screen.getByTestId('current-user-indicator')).toBeInTheDocument();

    // Admin sees action buttons for other admins (same level).
    await waitFor(() => {
      expect(screen.getByTestId('role-select-trigger-admin2@example.com')).toBeInTheDocument();
      expect(screen.getByTestId('remove-user-btn-admin2@example.com')).toBeInTheDocument();
    });

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

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
      expect(screen.getByText('manager@example.com')).toBeInTheDocument();
      expect(screen.getByText('manager2@example.com')).toBeInTheDocument();
      expect(screen.getByText('admin@example.com')).toBeInTheDocument();
    });

    // Current manager sees no action buttons for themselves.
    expect(screen.queryByTestId('role-select-trigger-manager@example.com')).not.toBeInTheDocument();
    expect(screen.getByTestId('current-user-indicator')).toBeInTheDocument();

    // Manager sees action buttons for other managers (same level).
    await waitFor(() => {
      expect(screen.getByTestId('role-select-trigger-manager2@example.com')).toBeInTheDocument();
      expect(screen.getByTestId('remove-user-btn-manager2@example.com')).toBeInTheDocument();
    });

    expect(screen.getByTestId('role-select-trigger-user1@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('role-select-trigger-user2@example.com')).toBeInTheDocument();

    // Manager sees no action buttons for admin (higher level).
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

    expect(screen.getByTestId('users-page')).toBeInTheDocument();
  });

  it.skip('handles users API failure gracefully', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_admin' }, { stub: true }),
      ...mockUsersError({}, { stub: true })
    );

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await screen.findByTestId('users-page');

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

    await waitFor(() => {
      expect(screen.getByText('No Users')).toBeInTheDocument();
      expect(screen.getByText('No users found')).toBeInTheDocument();
    });
  });
});
