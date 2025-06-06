'use client';

import { render, screen, waitFor } from '@testing-library/react';
import {
  describe,
  it,
  vi,
  expect,
  beforeEach,
  beforeAll,
  afterAll,
  afterEach,
} from 'vitest';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import UiPage from '@/app/ui/page';
import { createWrapper } from '@/tests/wrapper';
import { ENDPOINT_APP_INFO } from '@/hooks/useQuery';
import { FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, ROUTE_DEFAULT, ROUTE_RESOURCE_ADMIN, ROUTE_SETUP_DOWNLOAD_MODELS } from '@/lib/constants';

const pushMock = vi.fn();
vi.mock('@/lib/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: () => null,
  }),
}));

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

// Setup MSW server
const server = setupServer(
  rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
    return res(ctx.json({ status: 'setup' }));
  })
);

// Add this configuration before starting the server
beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());

describe('UiPage', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    localStorageMock.clear();
  });

  it('redirects to /ui/setup when status is setup', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    render(<UiPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it(`redirects to ${ROUTE_DEFAULT} when status is ready`, async () => {
    // Set the localStorage flag
    localStorageMock.setItem(FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, 'true');

    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    render(<UiPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith(ROUTE_DEFAULT);
    });
  });

  it(`redirects to ${ROUTE_RESOURCE_ADMIN} when status is resource-admin`, async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );

    render(<UiPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith(ROUTE_RESOURCE_ADMIN);
    });
  });

  it(`redirects to ${ROUTE_SETUP_DOWNLOAD_MODELS} when status is ready and models page not shown`, async () => {
    // Set the localStorage flag to false
    localStorageMock.setItem(FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, 'false');

    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    render(<UiPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith(ROUTE_SETUP_DOWNLOAD_MODELS);
    });
  });
});
