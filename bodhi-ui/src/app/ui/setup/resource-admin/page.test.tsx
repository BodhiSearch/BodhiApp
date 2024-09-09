'use client';

import { render, screen, waitFor } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { QueryClient, QueryClientProvider } from 'react-query';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import ResourceAdminPage from './page';

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};

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
  rest.get('*/app/info', (req, res, ctx) => {
    return res(ctx.json({ status: 'resource-admin' }));
  })
);

beforeAll(() => {
  // Suppress console errors
  const originalConsoleError = console.error;
  console.error = (...args) => {
    if (args[0]?.includes?.('Failed to load resource')) {
      return;
    }
    originalConsoleError(...args);
  };

  server.listen({ onUnhandledRequest: 'bypass' });
});

afterAll(() => server.close());
afterEach(() => server.resetHandlers());

describe('ResourceAdminPage', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
  });

  it('renders the resource admin page when status is resource-admin', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Resource Admin Setup')).toBeInTheDocument();
      expect(screen.getByText('Log In')).toBeInTheDocument();
    });
  });

  it('redirects to /ui/setup when status is setup', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

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

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/home');
    });
  });
});
