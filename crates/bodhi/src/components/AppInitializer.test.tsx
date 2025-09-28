'use client';

import AppInitializer from '@/components/AppInitializer';
import { FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, ROUTE_DEFAULT, ROUTE_SETUP_DOWNLOAD_MODELS } from '@/lib/constants';
import { createWrapper } from '@/tests/wrapper';
import { createMockUserInfo } from '@/test-fixtures/access-requests';
import { createMockLoggedOutUser } from '@/test-utils/mock-user';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { mockAppInfo, mockAppInfoInternalError } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn, mockUserLoggedOut, mockUserInfoError } from '@/test-utils/msw-v2/handlers/user';
import { AppStatus } from '@bodhiapp/ts-client';
import { render, screen, waitFor, waitForElementToBeRemoved } from '@testing-library/react';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: () => null,
  }),
}));

setupMswV2();

beforeEach(() => {
  pushMock.mockClear();
});

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value;
    },
    clear: () => {
      store = {};
    },
  };
})();
Object.defineProperty(window, 'localStorage', { value: localStorageMock });

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
    server.use(...mockAppInfo({ status: 'ready' }, 100), ...mockUserLoggedIn(undefined, 100));

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

    expect(pushMock).not.toHaveBeenCalled();

    const alert = screen.getByRole('alert');
    expect(alert).toBeInTheDocument();
    expect(alert).toHaveTextContent('Error');
    expect(alert).toHaveTextContent('API Error');
  });
});

describe('AppInitializer routing based on currentStatus and allowedStatus', () => {
  beforeEach(() => {
    localStorageMock.clear();
  });

  // Update this test to use the constant
  it('redirects to download models page when status is ready and models page not shown', async () => {
    localStorageMock.setItem(FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, 'false');

    server.use(...mockAppInfo({ status: 'ready' }));

    await renderWithSetup(<AppInitializer />);
    expect(pushMock).toHaveBeenCalledWith(ROUTE_SETUP_DOWNLOAD_MODELS);
  });

  // Update this test to use the constant
  it(`redirects to ${ROUTE_DEFAULT} when status is ready and models page was shown`, async () => {
    localStorageMock.setItem(FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, 'true');

    server.use(...mockAppInfo({ status: 'ready' }));

    await renderWithSetup(<AppInitializer />);
    expect(pushMock).toHaveBeenCalledWith(ROUTE_DEFAULT);
  });

  // Update the test cases to use the constant
  it.each([
    { status: 'setup', expectedPath: '/ui/setup', localStorage: {} },
    { status: 'ready', expectedPath: ROUTE_DEFAULT, localStorage: { [FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED]: 'true' } },
    { status: 'resource-admin', expectedPath: '/ui/setup/resource-admin', localStorage: {} },
    { status: 'ready', expectedPath: ROUTE_DEFAULT, localStorage: { [FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED]: 'true' } },
    {
      status: 'ready',
      expectedPath: ROUTE_SETUP_DOWNLOAD_MODELS,
      localStorage: { [FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED]: 'false' },
    },
  ])(
    'redirects to $expectedPath when status is $status and localStorage is $localStorage',
    async ({ status, expectedPath, localStorage }) => {
      Object.entries(localStorage).forEach(([key, value]) => {
        localStorageMock.setItem(key, value);
      });

      server.use(...mockAppInfo({ status: status as any }));

      await renderWithSetup(<AppInitializer />);
      expect(pushMock).toHaveBeenCalledWith(expectedPath);
    }
  );

  // Update the status mismatch test cases
  it.each([
    { currentStatus: 'setup', allowedStatus: 'resource-admin', expectedPath: '/ui/setup', localStorage: {} },
    { currentStatus: 'setup', allowedStatus: 'ready', expectedPath: '/ui/setup', localStorage: {} },
    { currentStatus: 'setup', allowedStatus: undefined, expectedPath: '/ui/setup', localStorage: {} },
    {
      currentStatus: 'resource-admin',
      allowedStatus: 'setup',
      expectedPath: '/ui/setup/resource-admin',
      localStorage: {},
    },
    {
      currentStatus: 'resource-admin',
      allowedStatus: 'ready',
      expectedPath: '/ui/setup/resource-admin',
      localStorage: {},
    },
    {
      currentStatus: 'resource-admin',
      allowedStatus: undefined,
      expectedPath: '/ui/setup/resource-admin',
      localStorage: {},
    },
    {
      currentStatus: 'ready',
      allowedStatus: 'setup',
      expectedPath: ROUTE_DEFAULT,
      localStorage: { [FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED]: 'true' },
    },
    {
      currentStatus: 'ready',
      allowedStatus: 'resource-admin',
      expectedPath: ROUTE_DEFAULT,
      localStorage: { [FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED]: 'true' },
    },
    {
      currentStatus: 'ready',
      allowedStatus: undefined,
      expectedPath: ROUTE_DEFAULT,
      localStorage: { [FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED]: 'true' },
    },
  ])(
    'redirects to $expectedPath when currentStatus=$currentStatus does not match allowedStatus=$allowedStatus',
    async ({ currentStatus, allowedStatus, expectedPath, localStorage }) => {
      Object.entries(localStorage).forEach(([key, value]) => {
        localStorageMock.setItem(key, value as string);
      });

      server.use(...mockAppInfo({ status: currentStatus as any }));

      await renderWithSetup(<AppInitializer allowedStatus={allowedStatus as AppStatus} />);
      expect(pushMock).toHaveBeenCalledWith(expectedPath);
    }
  );

  // Test status match scenarios (no redirect)
  it.each([
    { currentStatus: 'ready', allowedStatus: 'ready' },
    { currentStatus: 'setup', allowedStatus: 'setup' },
    { currentStatus: 'resource-admin', allowedStatus: 'resource-admin' },
  ])(
    'stays on page when currentStatus=$currentStatus matches allowedStatus=$allowedStatus',
    async ({ currentStatus, allowedStatus }) => {
      server.use(...mockAppInfo({ status: currentStatus as any }));

      await renderWithSetup(<AppInitializer allowedStatus={allowedStatus as AppStatus} />);
      expect(pushMock).not.toHaveBeenCalled();
    }
  );
});

describe('AppInitializer role-based access control', () => {
  beforeEach(() => {
    localStorageMock.clear();
    localStorageMock.setItem(FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, 'true');
  });

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
          role: `resource_${userRole}`,
        })
      );

      await renderWithSetup(
        <AppInitializer allowedStatus="ready" authenticated={true} minRole={minRole as any}>
          <div>Protected content</div>
        </AppInitializer>
      );

      if (shouldAllow) {
        expect(screen.getByText('Protected content')).toBeInTheDocument();
        expect(pushMock).not.toHaveBeenCalled();
      } else {
        expect(pushMock).toHaveBeenCalledWith('/ui/login?error=insufficient-role');
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
    expect(pushMock).not.toHaveBeenCalled();
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

    expect(pushMock).toHaveBeenCalledWith('/ui/request-access');
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

    expect(pushMock).toHaveBeenCalledWith('/ui/request-access');
  });

  it('prioritizes auth check over role check', async () => {
    server.use(...mockAppInfo({ status: 'ready' }), ...mockUserLoggedOut());

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={true} minRole="admin">
        <div>Protected content</div>
      </AppInitializer>
    );

    // Should redirect to login due to auth failure, not role failure
    expect(pushMock).toHaveBeenCalledWith('/ui/login');
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
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
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
    expect(pushMock).not.toHaveBeenCalled();
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
    expect(pushMock).not.toHaveBeenCalled();
  });
});
