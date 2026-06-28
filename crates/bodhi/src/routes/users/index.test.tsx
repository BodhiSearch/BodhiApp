import UsersPage from '@/routes/users/index';
import { ShellChromeProvider, useShellSlots } from '@/components/shell';
import { mockAppInfo, mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import {
  mockUserLoggedIn,
  mockUsers,
  mockUsersDefault,
  mockUserRoleChange,
  mockUserRemove,
} from '@/test-utils/msw-v2/handlers/user';
import { mockAccessRequestsDefault } from '@/test-utils/msw-v2/handlers/user-access-requests';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: any) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
    useNavigate: () => vi.fn(),
    useLocation: () => ({ pathname: '/users' }),
  };
});

vi.mock('@/hooks/useToastMessages', () => ({
  useToastMessages: () => ({ showSuccess: vi.fn(), showError: vi.fn() }),
}));

setupMswV2();

function SlotsConsumer() {
  const { headerActions, rail, railHeader } = useShellSlots();
  return (
    <>
      <div data-testid="harness-header-actions">{headerActions}</div>
      <div data-testid="harness-rail-header">{railHeader}</div>
      <div data-testid="harness-rail">{rail}</div>
    </>
  );
}

function seed({ multiTenant = false }: { multiTenant?: boolean } = {}) {
  server.use(
    ...(multiTenant ? mockAppInfo({ deployment: 'multi_tenant' }) : mockAppInfoReady()),
    // current user = admin@example.com (admin) — matches mockSimpleUsersResponse's admin row.
    ...mockUserLoggedIn({ user_id: 'admin-id', username: 'admin@example.com', role: 'resource_admin' }),
    ...mockUsersDefault(),
    ...mockAccessRequestsDefault()
  );
}

afterEach(() => {
  localStorage.clear();
  vi.clearAllMocks();
});

async function renderReady() {
  await act(async () => {
    render(
      <ShellChromeProvider>
        <SlotsConsumer />
        <UsersPage />
      </ShellChromeProvider>,
      { wrapper: createWrapper() }
    );
  });
  await waitFor(() => {
    expect(screen.getByTestId('users-page')).toHaveAttribute('data-pagestatus', 'ready');
  });
}

describe('ManageUsers V2', () => {
  it('shows shimmer filter badges while the users query is pending', async () => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ user_id: 'admin-id', username: 'admin@example.com', role: 'resource_admin' }),
      ...mockUsers({}, { delayMs: 200, stub: true }),
      ...mockAccessRequestsDefault()
    );

    render(
      <ShellChromeProvider>
        <SlotsConsumer />
        <UsersPage />
      </ShellChromeProvider>,
      { wrapper: createWrapper() }
    );

    await waitFor(() => {
      expect(screen.getByTestId('users-filter-all')).toBeInTheDocument();
    });
    expect(screen.getByTestId('users-page')).toHaveAttribute('data-pagestatus', 'loading');
    expect(screen.getAllByLabelText('Loading count').length).toBeGreaterThan(0);

    await waitFor(() => {
      expect(screen.getByTestId('users-page')).toHaveAttribute('data-pagestatus', 'ready');
    });
    expect(screen.queryByLabelText('Loading count')).not.toBeInTheDocument();
  });

  it('renders the user list with role badges', async () => {
    seed();
    await renderReady();
    expect(screen.getByTestId('user-row-user1@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('user-role-user1@example.com')).toHaveTextContent('User');
    expect(screen.getByTestId('user-role-admin@example.com')).toHaveTextContent('Admin');
  });

  it('filters by role', async () => {
    const user = userEvent.setup();
    seed();
    await renderReady();
    await user.click(screen.getByTestId('users-filter-resource_power_user'));
    expect(screen.getByTestId('user-row-user2@example.com')).toBeInTheDocument();
    expect(screen.queryByTestId('user-row-user1@example.com')).not.toBeInTheDocument();
  });

  it('opens an editable rail (role select + remove) for a modifiable user', async () => {
    const user = userEvent.setup();
    seed();
    await renderReady();
    await user.click(screen.getByTestId('user-row-user1@example.com'));
    const rail = within(screen.getByTestId('harness-rail')).getByTestId('user-detail-user1@example.com');
    expect(within(rail).getByTestId('role-select-user1@example.com')).toBeInTheDocument();
    expect(within(rail).getByTestId('remove-user-btn-user1@example.com')).toBeInTheDocument();
  });

  it('renders each row as an accessible link and activating it opens the rail', async () => {
    const user = userEvent.setup();
    seed();
    await renderReady();
    const row = screen.getByTestId('user-row-user1@example.com');
    const link = within(row).getByTestId('row-link');
    expect(link.tagName).toBe('A');
    expect(link).toHaveAccessibleName('Open user user1@example.com');
    await user.click(link);
    expect(within(screen.getByTestId('harness-rail')).getByTestId('user-detail-user1@example.com')).toBeInTheDocument();
  });

  it('shows a read-only "You" rail for the current user (no role select / remove)', async () => {
    const user = userEvent.setup();
    seed();
    await renderReady();
    await user.click(screen.getByTestId('user-row-admin@example.com'));
    const rail = within(screen.getByTestId('harness-rail')).getByTestId('user-detail-admin@example.com');
    expect(within(rail).getByTestId('current-user-indicator')).toBeInTheDocument();
    expect(within(rail).queryByTestId('role-select-admin@example.com')).not.toBeInTheDocument();
    expect(within(rail).queryByTestId('remove-user-btn-admin@example.com')).not.toBeInTheDocument();
  });

  it('removes a user via the two-click confirm', async () => {
    const user = userEvent.setup();
    seed();
    server.use(...mockUserRemove('user1-id', { stub: true }));
    await renderReady();
    await user.click(screen.getByTestId('user-row-user1@example.com'));
    const removeBtn = () => within(screen.getByTestId('harness-rail')).getByTestId('remove-user-btn-user1@example.com');
    expect(removeBtn()).toHaveTextContent('Remove user');
    await user.click(removeBtn());
    expect(removeBtn()).toHaveTextContent('Confirm remove');
    await user.click(removeBtn());
    // mutation fires without error
  });

  it('changes a role via the rail select + save', async () => {
    const user = userEvent.setup();
    seed();
    server.use(...mockUserRoleChange('user1-id', { stub: true }));
    await renderReady();
    await user.click(screen.getByTestId('user-row-user1@example.com'));
    const select = within(screen.getByTestId('harness-rail')).getByTestId('role-select-user1@example.com');
    await user.selectOptions(select, 'resource_power_user');
    const save = within(screen.getByTestId('harness-rail')).getByTestId('save-role-user1@example.com');
    await waitFor(() => expect(save).toBeEnabled());
    await user.click(save);
  });

  it('hides the invite action on a single-tenant deployment', async () => {
    seed({ multiTenant: false });
    await renderReady();
    expect(within(screen.getByTestId('harness-header-actions')).queryByTestId('invite-toggle')).not.toBeInTheDocument();
  });

  it('shows the invite action on a multi-tenant deployment', async () => {
    seed({ multiTenant: true });
    await renderReady();
    expect(within(screen.getByTestId('harness-header-actions')).getByTestId('invite-toggle')).toBeInTheDocument();
  });
});
