'use client';

import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { QueryClient, QueryClientProvider } from 'react-query';
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
import Setup from './page';
import { Toaster } from '@/components/ui/toaster';

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
      <Toaster />
    </QueryClientProvider>
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
  // eslint-disable-next-line @next/next/no-img-element
  default: () => <img alt="mocked image" />,
}));

// Setup MSW server
const server = setupServer(
  rest.get('*/app/info', (req, res, ctx) => {
    return res(ctx.json({ status: 'setup' }));
  }),
  rest.post('*/app/setup', (req, res, ctx) => {
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

  it('renders the setup page when status is setup', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    render(<Setup />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Bodhi App Setup')).toBeInTheDocument();
    });
  });

  it('redirects to /ui/home when status is ready', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    render(<Setup />, { wrapper: createWrapper() });

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

    render(<Setup />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/resource-admin');
    });
  });

  it('sets up authenticated instance and redirects to /ui/home', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      }),
      rest.post('*/app/setup', (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    render(<Setup />, { wrapper: createWrapper() });

    const authButton = await screen.findByText(
      'Setup Authenticated Instance →'
    );
    fireEvent.click(authButton);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/home');
    });
  });

  it('sets up unauthenticated instance and redirects to /ui/setup/resource-admin', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      }),
      rest.post('*/app/setup', (req, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );

    render(<Setup />, { wrapper: createWrapper() });

    const unauthButton = await screen.findByText(
      'Setup Unauthenticated Instance →'
    );
    fireEvent.click(unauthButton);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/resource-admin');
    });
  });

  it('displays error message when setup fails', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      }),
      rest.post('*/app/setup', (req, res, ctx) => {
        return res(ctx.status(500), ctx.json({ message: 'Setup failed' }));
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
