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

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: () => null,
  }),
}));

// Setup MSW server
const server = setupServer(
  rest.get('*/app/info', (req, res, ctx) => {
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
  });

  it('redirects to /ui/setup when status is setup', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    render(<UiPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/home when status is ready', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    render(<UiPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/home');
    });
  });

  it('redirects to /ui/setup/resource-admin when status is resource-admin', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );

    render(<UiPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/resource-admin');
    });
  });
});
