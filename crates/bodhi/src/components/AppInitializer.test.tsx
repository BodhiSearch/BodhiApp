
import AppInitializer from '@/components/AppInitializer';
import { ENDPOINT_APP_INFO, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, ROUTE_DEFAULT, ROUTE_SETUP_DOWNLOAD_MODELS } from '@/lib/constants';
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
vi.mock('@/lib/navigation', () => ({
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
  await waitForElementToBeRemoved(() =>
    rendered.getByText('Initializing app...')
  );
  return rendered;
};

describe('AppInitializer loading and error handling', () => {
  // Test loading states
  it('shows loading state when endpoint is loading', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_req, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.json({ status: 'ready' })
        );
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_req, res, ctx) => {
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
    { scenario: 'app/info success, user error', setup: [{ endpoint: `*${ENDPOINT_APP_INFO}`, response: { status: 'ready' }, status: 200 }, { endpoint: `*${ENDPOINT_USER_INFO}`, response: { error: { message: 'API Error' } }, status: 500 }] },
  ])('handles error $scenario', async ({ scenario: _scenario, setup }) => {
    server.use(
      ...setup.map(({ endpoint, response, status }) =>
        rest.get(endpoint, (_req, res, ctx) => {
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
  beforeEach(() => {
    localStorageMock.clear();
  });

  // Update this test to use the constant
  it('redirects to download models page when status is ready and models page not shown', async () => {
    localStorageMock.setItem(FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, 'false');

    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    await renderWithSetup(<AppInitializer />);
    expect(pushMock).toHaveBeenCalledWith(ROUTE_SETUP_DOWNLOAD_MODELS);
  });

  // Update this test to use the constant
  it(`redirects to ${ROUTE_DEFAULT} when status is ready and models page was shown`, async () => {
    localStorageMock.setItem(FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, 'true');

    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    await renderWithSetup(<AppInitializer />);
    expect(pushMock).toHaveBeenCalledWith(ROUTE_DEFAULT);
  });

  // Update the test cases to use the constant
  it.each([
    { status: 'setup', expectedPath: '/ui/setup', localStorage: {} },
    { status: 'ready', expectedPath: ROUTE_DEFAULT, localStorage: { [FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED]: 'true' } },
    { status: 'resource-admin', expectedPath: '/ui/setup/resource-admin', localStorage: {} },
    { status: 'ready', expectedPath: ROUTE_DEFAULT, localStorage: { [FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED]: 'true' } },
    { status: 'ready', expectedPath: ROUTE_SETUP_DOWNLOAD_MODELS, localStorage: { [FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED]: 'false' } },
  ])('redirects to $expectedPath when status is $status and localStorage is $localStorage', async ({ status, expectedPath, localStorage }) => {
    Object.entries(localStorage).forEach(([key, value]) => {
      localStorageMock.setItem(key, value);
    });

    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status }));
      })
    );

    await renderWithSetup(<AppInitializer />);
    expect(pushMock).toHaveBeenCalledWith(expectedPath);
  });

  // Update the status mismatch test cases
  it.each([
    { currentStatus: 'setup', allowedStatus: 'resource-admin', expectedPath: '/ui/setup', localStorage: {} },
    { currentStatus: 'setup', allowedStatus: 'ready', expectedPath: '/ui/setup', localStorage: {} },
    { currentStatus: 'setup', allowedStatus: undefined, expectedPath: '/ui/setup', localStorage: {} },
    { currentStatus: 'resource-admin', allowedStatus: 'setup', expectedPath: '/ui/setup/resource-admin', localStorage: {} },
    { currentStatus: 'resource-admin', allowedStatus: 'ready', expectedPath: '/ui/setup/resource-admin', localStorage: {} },
    { currentStatus: 'resource-admin', allowedStatus: undefined, expectedPath: '/ui/setup/resource-admin', localStorage: {} },
    { currentStatus: 'ready', allowedStatus: 'setup', expectedPath: ROUTE_DEFAULT, localStorage: { [FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED]: 'true' } },
    { currentStatus: 'ready', allowedStatus: 'resource-admin', expectedPath: ROUTE_DEFAULT, localStorage: { [FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED]: 'true' } },
    { currentStatus: 'ready', allowedStatus: undefined, expectedPath: ROUTE_DEFAULT, localStorage: { [FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED]: 'true' } },
  ])('redirects to $expectedPath when currentStatus=$currentStatus does not match allowedStatus=$allowedStatus',
    async ({ currentStatus, allowedStatus, expectedPath, localStorage }) => {
      Object.entries(localStorage).forEach(([key, value]) => {
        localStorageMock.setItem(key, value as string);
      });

      server.use(
        rest.get(`*${ENDPOINT_APP_INFO}`, (_req, res, ctx) => {
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
        rest.get(`*${ENDPOINT_APP_INFO}`, (_req, res, ctx) => {
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
  it('redirects to login when authenticated=true and status allowed', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_req, res, ctx) => {
        return res(ctx.json({ logged_in: false }));
      })
    );

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={true}>
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
    ${true}      | ${true}
    ${false}     | ${false}
    ${false}     | ${true}
  `('displays content when authenticated=$authenticated loggedIn=$loggedIn',
    async ({ authenticated, loggedIn }) => {
      server.use(
        rest.get(`*${ENDPOINT_APP_INFO}`, (_req, res, ctx) => {
          return res(ctx.json({ status: 'ready' }));
        }),
        rest.get(`*${ENDPOINT_USER_INFO}`, (_req, res, ctx) => {
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
  it('user endpoint not called when authenticated=false', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={false}>
        <div>Child content</div>
      </AppInitializer>
    );
    expect(screen.getByText('Child content')).toBeInTheDocument();
    expect(pushMock).not.toHaveBeenCalled();
  });
});
