
import { createWrapper } from '@/tests/wrapper';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import {
  afterAll,
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';
import Setup from '@/components/setup/SetupPage';
import { ENDPOINT_APP_INFO, ENDPOINT_APP_SETUP } from '@/hooks/useQuery';
import { ROUTE_DEFAULT, ROUTE_RESOURCE_ADMIN } from '@/lib/constants';

// Mock the router
const pushMock = vi.fn();
vi.mock('@/lib/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: () => null,
  }),
}));

// Mock the Image component
vi.mock('next/image', () => ({
  // eslint-disable-next-line @next/next/no-img-element
  default: () => <img alt="mocked image" />,
}));

// Setup MSW server
const server = setupServer(
  rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
    return res(ctx.json({ status: 'setup' }));
  }),
  rest.post(`*${ENDPOINT_APP_SETUP}`, (req, res, ctx) => {
    return res(ctx.json({ status: 'ready' }));
  })
);

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());

describe('Setup Page', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
  });

  test.skip('renders the setup page when status is setup', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    render(<Setup />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Bodhi App Setup')).toBeInTheDocument();
    });
  });

  test.skip(`redirects to ${ROUTE_DEFAULT} when status is ready`, async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    render(<Setup />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith(ROUTE_DEFAULT);
    });
  });

  test.skip(`redirects to ${ROUTE_RESOURCE_ADMIN} when status is resource-admin`, async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );

    render(<Setup />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith(ROUTE_RESOURCE_ADMIN);
    });
  });

  test.skip(`sets up authenticated instance and redirects to ${ROUTE_DEFAULT}`, async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      }),
      rest.post(`*${ENDPOINT_APP_SETUP}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    render(<Setup />, { wrapper: createWrapper() });

    const authButton = await screen.findByText(
      'Setup Authenticated Instance →'
    );
    fireEvent.click(authButton);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith(ROUTE_DEFAULT);
    });
  });

  test.skip(`sets up authenticated instance and redirects to ${ROUTE_RESOURCE_ADMIN}`, async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      }),
      rest.post(`*${ENDPOINT_APP_SETUP}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );

    render(<Setup />, { wrapper: createWrapper() });

    const unauthButton = await screen.findByText(
      'Setup Unauthenticated Instance →'
    );
    fireEvent.click(unauthButton);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith(ROUTE_RESOURCE_ADMIN);
    });
  });

  test.skip('displays error message when setup fails', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      }),
      rest.post(`*${ENDPOINT_APP_SETUP}`, (req, res, ctx) => {
        return res(ctx.status(500), ctx.json({ error: { message: 'Setup failed' } }));
      })
    );

    render(<Setup />, { wrapper: createWrapper() });

    const authButton = await screen.findByText(
      'Setup Authenticated Instance →'
    );
    fireEvent.click(authButton);

    await waitFor(() => {
      expect(
        screen.getByText('Error while setting up app: Setup failed')
      ).toBeInTheDocument();
    });
  });
});
