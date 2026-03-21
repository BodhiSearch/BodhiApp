import AppInitializer from '@/components/AppInitializer';
import { ROUTE_DEFAULT } from '@/lib/constants';
import { createWrapper } from '@/tests/wrapper';
import { createMockUserInfo } from '@/test-fixtures/access-requests';
import { createMockLoggedOutUser } from '@/test-utils/mock-user';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { mockAppInfo, mockAppInfoInternalError } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn, mockUserLoggedOut, mockUserInfoError } from '@/test-utils/msw-v2/handlers/user';
import { AppStatus } from '@bodhiapp/ts-client';
import { render, screen, waitFor, waitForElementToBeRemoved } from '@testing-library/react';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

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
    useSearch: () => ({}),
  };
});

setupMswV2();

beforeEach(() => {
  navigateMock.mockClear();
});

const renderWithSetup = async (ui: React.ReactElement) => {
  const wrapper = createWrapper();
  const rendered = render(ui, { wrapper });
  await waitForElementToBeRemoved(() => rendered.getByText('Initializing app...'));
  return rendered;
};

describe('AppInitializer loading and error handling', () => {
  // Test loading states
  it('shows loading state when endpoint is loading', async () => {
    // Use mock handlers with delay
    server.use(...mockAppInfo({ status: 'ready' }, { delayMs: 100 }), ...mockUserLoggedIn(undefined, { delayMs: 100 }));

    const wrapper = createWrapper();
    render(
      <AppInitializer allowedStatus="ready" authenticated={false}>
        <div>Child content</div>
      </AppInitializer>,
      { wrapper }
    );

    expect(screen.getByText('Initializing app...')).toBeInTheDocument();
    await waitForElementToBeRemoved(() => screen.getByText('Initializing app...'));
    expect(screen.getByText('Child content')).toBeInTheDocument();
  });

  // Test error handling
  it.each([
    {
      scenario: 'app/info error',
      appInfoHandlers: () => mockAppInfoInternalError(),
      userHandlers: () => mockUserLoggedIn(),
    },
    {
      scenario: 'app/info success, user error',
      appInfoHandlers: () => mockAppInfo({ status: 'ready' }),
      userHandlers: () => mockUserInfoError({ message: 'API Error', type: 'internal_server_error' }),
    },
  ])('handles error $scenario', async ({ scenario, appInfoHandlers, userHandlers }) => {
    server.use(...appInfoHandlers(), ...userHandlers());

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={true}>
        <div>Child content</div>
      </AppInitializer>
    );

    expect(navigateMock).not.toHaveBeenCalled();

    const alert = screen.getByRole('alert');
    expect(alert).toBeInTheDocument();
    expect(alert).toHaveTextContent('Error');
    expect(alert).toHaveTextContent('API Error');
  });
});

describe('AppInitializer routing based on currentStatus and allowedStatus', () => {
  it(`redirects to ${ROUTE_DEFAULT} when status is ready`, async () => {
    server.use(...mockAppInfo({ status: 'ready' }));

    await renderWithSetup(<AppInitializer />);
    expect(navigateMock).toHaveBeenCalledWith({ to: ROUTE_DEFAULT });
  });

  it.each([
    { status: 'setup', expectedPath: '/setup' },
    { status: 'ready', expectedPath: ROUTE_DEFAULT },
    { status: 'resource_admin', expectedPath: '/setup/resource-admin' },
  ])('redirects to $expectedPath when status is $status', async ({ status, expectedPath }) => {
    server.use(...mockAppInfo({ status: status as any }));

    await renderWithSetup(<AppInitializer />);
    expect(navigateMock).toHaveBeenCalledWith({ to: expectedPath });
  });

  // Update the status mismatch test cases
  it.each([
    { currentStatus: 'setup', allowedStatus: 'resource_admin', expectedPath: '/setup' },
    { currentStatus: 'setup', allowedStatus: 'ready', expectedPath: '/setup' },
    { currentStatus: 'setup', allowedStatus: undefined, expectedPath: '/setup' },
    {
      currentStatus: 'resource_admin',
      allowedStatus: 'setup',
      expectedPath: '/setup/resource-admin',
    },
    {
      currentStatus: 'resource_admin',
      allowedStatus: 'ready',
      expectedPath: '/setup/resource-admin',
    },
    {
      currentStatus: 'resource_admin',
      allowedStatus: undefined,
      expectedPath: '/setup/resource-admin',
    },
    {
      currentStatus: 'ready',
      allowedStatus: 'setup',
      expectedPath: ROUTE_DEFAULT,
    },
    {
      currentStatus: 'ready',
      allowedStatus: 'resource_admin',
      expectedPath: ROUTE_DEFAULT,
    },
    {
      currentStatus: 'ready',
      allowedStatus: undefined,
      expectedPath: ROUTE_DEFAULT,
    },
  ])(
    'redirects to $expectedPath when currentStatus=$currentStatus does not match allowedStatus=$allowedStatus',
    async ({ currentStatus, allowedStatus, expectedPath }) => {
      server.use(...mockAppInfo({ status: currentStatus as any }));

      await renderWithSetup(<AppInitializer allowedStatus={allowedStatus as AppStatus} />);
      expect(navigateMock).toHaveBeenCalledWith({ to: expectedPath });
    }
  );

  // Test status match scenarios (no redirect)
  it.each([
    { currentStatus: 'ready', allowedStatus: 'ready' },
    { currentStatus: 'setup', allowedStatus: 'setup' },
    { currentStatus: 'resource_admin', allowedStatus: 'resource_admin' },
  ])(
    'stays on page when currentStatus=$currentStatus matches allowedStatus=$allowedStatus',
    async ({ currentStatus, allowedStatus }) => {
      server.use(...mockAppInfo({ status: currentStatus as any }));

      await renderWithSetup(<AppInitializer allowedStatus={allowedStatus as AppStatus} />);
      expect(navigateMock).not.toHaveBeenCalled();
    }
  );

  // Multi-tenant routing tests
  it('redirects to /login when ready + multi_tenant + no client_id', async () => {
    server.use(...mockAppInfo({ status: 'ready', deployment: 'multi_tenant' }));

    await renderWithSetup(<AppInitializer />);
    expect(navigateMock).toHaveBeenCalledWith({ to: '/login' });
  });

  it('redirects to chat when ready + multi_tenant + client_id', async () => {
    server.use(...mockAppInfo({ status: 'ready', deployment: 'multi_tenant', client_id: 'test-client' }));

    await renderWithSetup(<AppInitializer />);
    expect(navigateMock).toHaveBeenCalledWith({ to: ROUTE_DEFAULT });
  });

  it('redirects to /setup/tenants when setup + multi_tenant', async () => {
    server.use(...mockAppInfo({ status: 'setup', deployment: 'multi_tenant' }));

    await renderWithSetup(<AppInitializer />);
    expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/tenants' });
  });

  it('stays on page when ready + multi_tenant + no client_id and allowedStatus=ready', async () => {
    server.use(...mockAppInfo({ status: 'ready', deployment: 'multi_tenant' }));

    await renderWithSetup(<AppInitializer allowedStatus="ready" />);
    expect(navigateMock).not.toHaveBeenCalled();
  });
});

describe('AppInitializer role-based access control', () => {
  it.each([
    { userRole: 'admin', minRole: 'manager', shouldAllow: true },
    { userRole: 'manager', minRole: 'manager', shouldAllow: true },
    { userRole: 'power_user', minRole: 'manager', shouldAllow: false },
    { userRole: 'user', minRole: 'manager', shouldAllow: false },
    { userRole: 'admin', minRole: 'admin', shouldAllow: true },
    { userRole: 'manager', minRole: 'admin', shouldAllow: false },
    { userRole: 'power_user', minRole: 'admin', shouldAllow: false },
    { userRole: 'user', minRole: 'admin', shouldAllow: false },
  ])(
    'handles minRole=$minRole with userRole=$userRole (allow=$shouldAllow)',
    async ({ userRole, minRole, shouldAllow }) => {
      server.use(
        ...mockAppInfo({ status: 'ready' }),
        ...mockUserLoggedIn({
          username: 'test@example.com',
          role: `resource_${userRole}` as any,
        })
      );

      await renderWithSetup(
        <AppInitializer allowedStatus="ready" authenticated={true} minRole={minRole as any}>
          <div>Protected content</div>
        </AppInitializer>
      );

      if (shouldAllow) {
        expect(screen.getByText('Protected content')).toBeInTheDocument();
        expect(navigateMock).not.toHaveBeenCalled();
      } else {
        expect(navigateMock).toHaveBeenCalledWith({ to: '/login', search: { error: 'insufficient-role' } });
      }
    }
  );

  it('allows access when no minRole is specified', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({
        username: 'test@example.com',
        role: 'resource_user',
      })
    );

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={true}>
        <div>Content for all authenticated users</div>
      </AppInitializer>
    );

    expect(screen.getByText('Content for all authenticated users')).toBeInTheDocument();
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it('redirects to login when user has no roles', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({
        username: 'test@example.com',
        role: null,
      })
    );

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={true} minRole="user">
        <div>Protected content</div>
      </AppInitializer>
    );

    expect(navigateMock).toHaveBeenCalledWith({ to: '/request-access' });
  });

  it('redirects to login when user has undefined roles', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({
        username: 'test@example.com',
        role: undefined,
      })
    );

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={true} minRole="user">
        <div>Protected content</div>
      </AppInitializer>
    );

    expect(navigateMock).toHaveBeenCalledWith({ to: '/request-access' });
  });

  it('redirects to request-access when user has guest role', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({
        username: 'test@example.com',
        role: 'resource_guest',
      })
    );

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={true}>
        <div>Protected content</div>
      </AppInitializer>
    );

    expect(navigateMock).toHaveBeenCalledWith({ to: '/request-access' });
  });

  it('redirects to request-access when user has anonymous role', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({
        username: 'test@example.com',
        role: 'resource_anonymous',
      })
    );

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={true}>
        <div>Protected content</div>
      </AppInitializer>
    );

    expect(navigateMock).toHaveBeenCalledWith({ to: '/request-access' });
  });

  it('prioritizes auth check over role check', async () => {
    server.use(...mockAppInfo({ status: 'ready' }), ...mockUserLoggedOut());

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={true} minRole="admin">
        <div>Protected content</div>
      </AppInitializer>
    );

    // Should redirect to login due to auth failure, not role failure
    expect(navigateMock).toHaveBeenCalledWith({ to: '/login' });
  });
});

describe('AppInitializer authentication behavior', () => {
  // Test redirect scenarios
  it.each`
    authenticated | loggedIn
    ${true}       | ${false}
  `('redirects to login when authenticated=$authenticated loggedIn=$loggedIn', async ({ authenticated, loggedIn }) => {
    server.use(...mockAppInfo({ status: 'ready' }), ...(loggedIn ? mockUserLoggedIn() : mockUserLoggedOut()));

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={authenticated}>
        <div>Child content</div>
      </AppInitializer>
    );

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/login' });
    });
  });

  // Test content display scenarios
  it.each`
    authenticated | loggedIn
    ${true}       | ${true}
    ${false}      | ${false}
    ${false}      | ${true}
  `('displays content when authenticated=$authenticated loggedIn=$loggedIn', async ({ authenticated, loggedIn }) => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...(loggedIn
        ? mockUserLoggedIn({
            username: 'test@example.com',
            role: 'resource_user',
          })
        : mockUserLoggedOut())
    );
    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={authenticated}>
        <div>Child content</div>
      </AppInitializer>
    );
    expect(screen.getByText('Child content')).toBeInTheDocument();
    expect(navigateMock).not.toHaveBeenCalled();
  });

  // Add new test for user endpoint call conditions
  it('user endpoint not called when authenticated=false', async () => {
    server.use(...mockAppInfo({ status: 'ready' }));

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={false}>
        <div>Child content</div>
      </AppInitializer>
    );
    expect(screen.getByText('Child content')).toBeInTheDocument();
    expect(navigateMock).not.toHaveBeenCalled();
  });
});
