'use client';

import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
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
import ResourceAdminPage from '@/app/ui/setup/resource-admin/page';
import { ENDPOINT_APP_INFO } from '@/hooks/useQuery';

// Mock the router
const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: () => null,
  }),
}));

// Mock the Image component
vi.mock('next/image', () => ({
  default: () => <img alt="mocked image" />,
}));

// Setup MSW server
const server = setupServer(
  rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
    return res(ctx.json({ status: 'resource-admin' }));
  })
);

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());

describe('ResourceAdminPage', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
  });

  test.skip('renders the resource admin page when status is resource-admin', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Resource Admin Setup')).toBeInTheDocument();
      expect(screen.getByText('Log In')).toBeInTheDocument();
    });
  });

  test.skip('redirects to /ui/setup when status is setup', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  test.skip('redirects to /ui/home when status is ready', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/home');
    });
  });
});
