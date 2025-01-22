'use client';

import AppInitializer from '@/components/AppInitializer';
import { ENDPOINT_APP_INFO, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { AppStatus } from '@/types/models';
import {
  render,
  screen,
  waitFor,
  waitForElementToBeRemoved,
} from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import {
  afterAll,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: () => null,
  }),
}));

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
beforeEach(() => {
  server.resetHandlers();
  pushMock.mockClear();
});

const renderWithSetup = async (ui: React.ReactElement) => {
  const wrapper = createWrapper();
  const rendered = render(ui, { wrapper });
  await waitForElementToBeRemoved(() =>
    rendered.getByText('Initializing app...')
  );
  return rendered;
};

describe('AppInitializer loading and error handling', () => {
  // Test loading states
  it('shows loading state when endpoint is loading', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.json({ status: 'ready', authz: true })
        );
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (req, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.json({ logged_in: true })
        );
      })
    );

    const wrapper = createWrapper();
    render(
      <AppInitializer allowedStatus="ready" authenticated={false}>
        <div>Child content</div>
      </AppInitializer>,
      { wrapper }
    );

    expect(screen.getByText('Initializing app...')).toBeInTheDocument();
    await waitForElementToBeRemoved(() =>
      screen.getByText('Initializing app...')
    );
    expect(screen.getByText('Child content')).toBeInTheDocument();
  });

  // Test error handling
  it.each([
    { scenario: 'app/info error', setup: [{ endpoint: `*${ENDPOINT_APP_INFO}`, response: { error: { message: 'API Error' } }, status: 500 }, { endpoint: `*${ENDPOINT_USER_INFO}`, response: { logged_in: true }, status: 200 }] },
    { scenario: 'app/info success, user error', setup: [{ endpoint: `*${ENDPOINT_APP_INFO}`, response: { status: 'ready', authz: true }, status: 200 }, { endpoint: `*${ENDPOINT_USER_INFO}`, response: { error: { message: 'API Error' } }, status: 500 }] },
  ])('handles error $scenario', async ({ scenario, setup }) => {
    server.use(
      ...setup.map(({ endpoint, response, status }) =>
        rest.get(endpoint, (req, res, ctx) => {
          return res(ctx.status(status), ctx.json(response));
        })
      )
    );

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
  // Test basic routing scenarios
  it.each([
    { status: 'setup', expectedPath: '/ui/setup' },
    { status: 'ready', expectedPath: '/ui/home' },
    { status: 'resource-admin', expectedPath: '/ui/setup/resource-admin' },
  ])('redirects to $expectedPath when status is $status', async ({ status, expectedPath }) => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.json({ status, authz: false }));
      })
    );

    await renderWithSetup(<AppInitializer />);
    expect(pushMock).toHaveBeenCalledWith(expectedPath);
  });

  // Test status mismatch scenarios (redirects)
  it.each([
    { currentStatus: 'setup', allowedStatus: 'resource-admin', expectedPath: '/ui/setup' },
    { currentStatus: 'setup', allowedStatus: 'ready', expectedPath: '/ui/setup' },
    { currentStatus: 'setup', allowedStatus: undefined, expectedPath: '/ui/setup' },
    { currentStatus: 'resource-admin', allowedStatus: 'setup', expectedPath: '/ui/setup/resource-admin' },
    { currentStatus: 'resource-admin', allowedStatus: 'ready', expectedPath: '/ui/setup/resource-admin' },
    { currentStatus: 'resource-admin', allowedStatus: undefined, expectedPath: '/ui/setup/resource-admin' },
    { currentStatus: 'ready', allowedStatus: 'setup', expectedPath: '/ui/home' },
    { currentStatus: 'ready', allowedStatus: 'resource-admin', expectedPath: '/ui/home' },
    { currentStatus: 'ready', allowedStatus: undefined, expectedPath: '/ui/home' },
  ])('redirects to $expectedPath when currentStatus=$currentStatus does not match allowedStatus=$allowedStatus',
    async ({ currentStatus, allowedStatus, expectedPath }) => {
      server.use(
        rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
          return res(ctx.json({ status: currentStatus }));
        })
      );

      await renderWithSetup(<AppInitializer allowedStatus={allowedStatus as AppStatus} />);
      expect(pushMock).toHaveBeenCalledWith(expectedPath);
    }
  );

  // Test status match scenarios (no redirect)
  it.each([
    { currentStatus: 'ready', allowedStatus: 'ready' },
    { currentStatus: 'setup', allowedStatus: 'setup' },
    { currentStatus: 'resource-admin', allowedStatus: 'resource-admin' },
  ])('stays on page when currentStatus=$currentStatus matches allowedStatus=$allowedStatus',
    async ({ currentStatus, allowedStatus }) => {
      server.use(
        rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
          return res(ctx.json({ status: currentStatus }));
        })
      );

      await renderWithSetup(<AppInitializer allowedStatus={allowedStatus as AppStatus} />);
      expect(pushMock).not.toHaveBeenCalled();
    }
  );
});

describe('AppInitializer authentication behavior', () => {
  // Test redirect scenarios
  it.each`
    authz    | authenticated | loggedIn
    ${true}  | ${true}      | ${false}
  `('redirects to login when authz=$authz authenticated=$authenticated loggedIn=$loggedIn',
    async ({ authz, authenticated, loggedIn }) => {
      server.use(
        rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
          return res(ctx.json({ status: 'ready', authz }));
        }),
        rest.get(`*${ENDPOINT_USER_INFO}`, (req, res, ctx) => {
          return res(ctx.json({ logged_in: loggedIn }));
        })
      );

      await renderWithSetup(
        <AppInitializer allowedStatus="ready" authenticated={authenticated}>
          <div>Child content</div>
        </AppInitializer>
      );

      await waitFor(() => {
        expect(pushMock).toHaveBeenCalledWith('/ui/login');
      });
    }
  );

  // Test content display scenarios
  it.each`
    authz    | authenticated | loggedIn
    ${true}  | ${true}      | ${true}
    ${true}  | ${false}     | ${false}
    ${true}  | ${false}     | ${true}
    ${false} | ${true}      | ${false}
    ${false} | ${true}      | ${true}
    ${false} | ${false}     | ${false}
    ${false} | ${false}     | ${true}
  `('displays content when authz=$authz authenticated=$authenticated loggedIn=$loggedIn',
    async ({ authz, authenticated, loggedIn }) => {
      server.use(
        rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
          return res(ctx.json({ status: 'ready', authz }));
        }),
        rest.get(`*${ENDPOINT_USER_INFO}`, (req, res, ctx) => {
          return res(ctx.json({ logged_in: loggedIn }));
        })
      );
      await renderWithSetup(
        <AppInitializer allowedStatus="ready" authenticated={authenticated}>
          <div>Child content</div>
        </AppInitializer>
      );
      expect(screen.getByText('Child content')).toBeInTheDocument();
      expect(pushMock).not.toHaveBeenCalled();
    }
  );

  // Add new test for user endpoint call conditions
  it.each`
     authenticated | authz
     ${false}      | ${false}
  `('user endpoint not called when authz=$authz authenticated=$authenticated',
    async ({ authenticated, authz }) => {
      server.use(
        rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
          return res(ctx.json({ status: 'ready', authz }));
        })
      );

      await renderWithSetup(
        <AppInitializer allowedStatus="ready" authenticated={authenticated}>
          <div>Child content</div>
        </AppInitializer>
      );
      expect(screen.getByText('Child content')).toBeInTheDocument();
      expect(pushMock).not.toHaveBeenCalled();
    }
  );
});
