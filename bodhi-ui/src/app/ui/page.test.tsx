'use client';

import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, vi, expect, beforeEach, beforeAll, afterAll, afterEach } from 'vitest';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import UiPage from './page';

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

    render(<UiPage />);

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

    render(<UiPage />);

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

    render(<UiPage />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/resource-admin');
    });
  });

  it('displays error message for unexpected status', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'unexpected' }));
      })
    );

    render(<UiPage />);

    await waitFor(() => {
      expect(
        screen.getByText(/unexpected \/app\/info status from server - unexpected/, { exact: false })
      ).toBeInTheDocument();
    });
  });

  it('displays error message when API call fails', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.status(500), ctx.json({ message: 'API Error' }));
      })
    );

    render(<UiPage />);

    await waitFor(() => {
      expect(
        screen.getByText(/Unable to connect to backend/)
      ).toBeInTheDocument();
    });
  });
});
