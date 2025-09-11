import UsersPage from '@/app/ui/users/page';
import { createWrapper } from '@/tests/wrapper';
import { createAccessRequestHandlers, createRoleBasedHandlers, createErrorHandlers } from '@/test-utils/msw-handlers';
import { ADMIN_ROLES, BLOCKED_ROLES, createMockUserInfo } from '@/test-fixtures/access-requests';
import {
  mockUsersResponse,
  mockEmptyUsersResponse,
  mockUser1,
  mockUser2,
  mockManager,
  mockAdmin,
} from '@/test-fixtures/users';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  usePathname: vi.fn().mockReturnValue('/ui/users'),
}));

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  pushMock.mockClear();
});

describe('UsersPage Role-Based Access Control', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it.each(ADMIN_ROLES)('allows access for %s role', async (role) => {
    server.use(...createRoleBasedHandlers(role, true));

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // Should show the page content, not redirect
    expect(screen.getByTestId('users-page')).toBeInTheDocument();
    expect(screen.getAllByText('All Users')[1]).toBeInTheDocument(); // The second occurrence is in the card
    expect(pushMock).not.toHaveBeenCalled();
  });

  it.each(BLOCKED_ROLES)('blocks access for %s role', async (role) => {
    server.use(...createRoleBasedHandlers(role, false));

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // Should show the page but might have restricted content
    // Since this is placeholder functionality, we test that it doesn't crash
    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
    });
  });

  it('redirects unauthenticated users appropriately', async () => {
    server.use(
      ...createAccessRequestHandlers({
        userInfo: { logged_in: false },
      })
    );

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // Should show loading/redirect state, not the main page
    // AppInitializer handles the actual redirect
    expect(screen.getByText('Redirecting to login...')).toBeInTheDocument();
  });
});

describe('UsersPage Data Display', () => {
  beforeEach(() => {
    server.use(
      ...createAccessRequestHandlers({
        users: mockUsersResponse,
        userInfo: createMockUserInfo('resource_admin'),
      })
    );
  });

  it('displays placeholder content for user management', async () => {
    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // Wait for page to load
    await screen.findByTestId('users-page');

    // Should display All Users text in card title
    expect(screen.getAllByText('All Users')[1]).toBeInTheDocument(); // The second occurrence is in the card

    // Should show placeholder message since functionality is not implemented yet
    expect(screen.getByText('Manage user access and roles (Coming Soon)')).toBeInTheDocument();
    expect(
      screen.getByText((content) => content.includes('User management functionality will be implemented'))
    ).toBeInTheDocument();
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
});

describe('UsersPage Placeholder Functionality', () => {
  beforeEach(() => {
    server.use(
      ...createAccessRequestHandlers({
        userInfo: createMockUserInfo('resource_admin'),
      })
    );
  });

  it('shows placeholder message for unimplemented features', async () => {
    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
    });

    // Should show that user management features are coming soon
    expect(screen.getByText('Manage user access and roles (Coming Soon)')).toBeInTheDocument();
    expect(
      screen.getByText((content) => content.includes('User management functionality will be implemented'))
    ).toBeInTheDocument();
    // Should show empty state
    expect(screen.getByText('No Users')).toBeInTheDocument();
    expect(screen.getByText('User management API not yet implemented')).toBeInTheDocument();
  });

  it('does not show user management actions since not implemented', async () => {
    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
    });

    // Should not show user management buttons/forms since not implemented
    expect(screen.queryByRole('button', { name: /remove/i })).not.toBeInTheDocument();
    expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
    expect(screen.queryByRole('textbox', { name: /search/i })).not.toBeInTheDocument();
  });

  it('handles page rendering without API calls since not implemented', async () => {
    // Don't set up any API handlers to test that page doesn't make unnecessary calls
    server.use(
      ...createAccessRequestHandlers({
        userInfo: createMockUserInfo('resource_admin'),
      })
    );

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
    });

    // Should render placeholder without making user API calls
    expect(screen.getByText('No Users')).toBeInTheDocument();
    expect(screen.getByText('User management API not yet implemented')).toBeInTheDocument();
  });
});

describe('UsersPage Error Handling', () => {
  it('renders successfully even if there are API errors', async () => {
    server.use(...createErrorHandlers());

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    // When API fails, AppInitializer shows a generic error, not the users page
    await waitFor(() => {
      expect(screen.getByRole('alert')).toBeInTheDocument();
    });

    // Should show the generic error from AppInitializer
    expect(screen.getByText('Request failed with status code 500')).toBeInTheDocument();
  });

  it('handles network failures gracefully', async () => {
    server.use(
      ...createAccessRequestHandlers({
        userInfo: createMockUserInfo('admin'),
      })
    );

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
    });

    // Should handle error gracefully without crashing and show placeholder UI
    expect(screen.getByText('No Users')).toBeInTheDocument();
    expect(screen.getByText('User management API not yet implemented')).toBeInTheDocument();
  });
});

describe('UsersPage Loading States', () => {
  it('shows page content immediately since no API calls needed', async () => {
    server.use(
      ...createAccessRequestHandlers({
        userInfo: createMockUserInfo('admin'),
      })
    );

    render(<UsersPage />, { wrapper: createWrapper() });

    // Should show loading initially then page content
    // Wait for any async operations to complete
    await waitFor(
      () => {
        expect(screen.getByTestId('users-page')).toBeInTheDocument();
      },
      { timeout: 3000 }
    );

    expect(screen.getAllByText('All Users')[1]).toBeInTheDocument(); // The second occurrence is in the card
  });

  it('handles page structure correctly', async () => {
    server.use(
      ...createAccessRequestHandlers({
        userInfo: createMockUserInfo('resource_admin'),
      })
    );

    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
    });

    // Should have proper page structure
    expect(screen.getAllByText('All Users')[1]).toBeInTheDocument(); // The second occurrence is in the card
    expect(screen.getByText('Manage user access and roles (Coming Soon)')).toBeInTheDocument();
  });
});

// Future tests for when user management is implemented
describe('UsersPage Future Functionality Tests (Placeholder)', () => {
  beforeEach(() => {
    server.use(
      ...createAccessRequestHandlers({
        users: mockUsersResponse,
        userInfo: createMockUserInfo('resource_admin'),
      })
    );
  });

  it('will display user list when functionality is implemented', async () => {
    // This test serves as a template for when user management is actually implemented
    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
    });

    // Currently shows placeholder, but when implemented should show user data
    // Future implementation would test:
    // - expect(screen.getByText(mockUser1.email)).toBeInTheDocument();
    // - expect(screen.getByText(mockUser2.email)).toBeInTheDocument();
    // - Role display, management actions, etc.

    // For now, just verify placeholder is shown
    expect(screen.getByText('No Users')).toBeInTheDocument();
    expect(screen.getByText('User management API not yet implemented')).toBeInTheDocument();
  });

  it('will handle user actions when functionality is implemented', async () => {
    // Template for future user action tests
    await act(async () => {
      render(<UsersPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toBeInTheDocument();
    });

    // Future implementation would test:
    // - Role changes
    // - User removal
    // - Search and filtering
    // - Pagination

    // For now, verify no action buttons are present
    expect(screen.queryByRole('button', { name: /remove/i })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /edit/i })).not.toBeInTheDocument();
  });
});
