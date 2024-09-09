'use client';

import { render, screen, waitFor } from '@testing-library/react';
import {
  beforeAll,
  afterAll,
  beforeEach,
  describe,
  it,
  expect,
  vi,
} from 'vitest';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import AppInitializer from './AppInitializer';
import { QueryClient, QueryClientProvider } from 'react-query';
import { Toaster } from '@/components/ui/toaster';
import { ReactNode } from 'react';
import { waitForElementToBeRemoved } from '@testing-library/react';

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

// Modify the createWrapper function
const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        // Add this to prevent initial automatic refetching
        refetchOnMount: false,
      },
    },
  });
  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
};

// Add this helper function
const renderWithSetup = async (ui: React.ReactElement) => {
  const wrapper = createWrapper();
  const rendered = render(ui, { wrapper });
  // Wait for the loading state to disappear
  await waitForElementToBeRemoved(() => rendered.getByText('Initializing app...'));
  return rendered;
};

describe('AppInitializer', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
  });

  it('displays error message when API call fails', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.status(500), ctx.json({ message: 'API Error' }));
      })
    );

    await renderWithSetup(<AppInitializer />);

    const alert = screen.getByRole('alert');
    expect(alert).toBeInTheDocument();
    expect(alert).toHaveTextContent('Error');
    expect(alert).toHaveTextContent('API Error');
  });

  it('redirects to /ui/setup when status is setup and no allowedStatus is provided', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    await renderWithSetup(<AppInitializer />);

    expect(pushMock).toHaveBeenCalledWith('/ui/setup');
  });

  it('redirects to /ui/home when status is ready and no allowedStatus is provided', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    await renderWithSetup(<AppInitializer />);

    expect(pushMock).toHaveBeenCalledWith('/ui/home');
  });

  it('redirects to /ui/setup/resource-admin when status is resource-admin and no allowedStatus is provided', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );

    await renderWithSetup(<AppInitializer />);

    expect(pushMock).toHaveBeenCalledWith('/ui/setup/resource-admin');
  });

  it('displays error message for unexpected status when no allowedStatus is provided', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'unexpected' }));
      })
    );
    await renderWithSetup(<AppInitializer />);
    await waitFor(() => {
      const alert = screen.getByRole('alert');
      expect(alert).toBeInTheDocument();
      expect(alert).toHaveTextContent('Error');
      expect(alert).toHaveTextContent("unexpected status from /app/info endpoint - 'unexpected'");
    });
  });

  it('redirects to /ui/setup when status is setup and allowedStatus is ready', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    await renderWithSetup(<AppInitializer allowedStatus="ready" />);

    expect(pushMock).toHaveBeenCalledWith('/ui/setup');
  });

  it('redirects to /ui/home when status is ready and allowedStatus is setup', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    await renderWithSetup(<AppInitializer allowedStatus="setup" />);

    expect(pushMock).toHaveBeenCalledWith('/ui/home');
  });

  it('does not redirect when status matches allowedStatus', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    await renderWithSetup(<AppInitializer allowedStatus="ready" />);

    expect(pushMock).not.toHaveBeenCalled();
  });

  it('displays children content if app status matches allowedStatus', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    await renderWithSetup(
      <AppInitializer allowedStatus="ready">
        <div>Child content</div>
      </AppInitializer>
    );

    expect(screen.getByText('Child content')).toBeInTheDocument();
  });

  it('does not display children content if app status does not match allowedStatus', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    await renderWithSetup(
      <AppInitializer allowedStatus="ready">
        <div>Child content</div>
      </AppInitializer>
    );

    await waitFor(() => {
      expect(screen.queryByText('Child content')).not.toBeInTheDocument();
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('displays loading state before resolving app status', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.delay(100), ctx.json({ status: 'ready' }));
      })
    );
    const wrapper = createWrapper();
    const rendered = render(<AppInitializer allowedStatus="ready">
      <div>Child content</div>
    </AppInitializer>, { wrapper });

    expect(screen.getByText('Initializing app...')).toBeInTheDocument();
    await waitForElementToBeRemoved(() => rendered.getByText('Initializing app...'));
    expect(screen.getByText('Child content')).toBeInTheDocument();
  });

  it('displays error message and not children when API call fails', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.status(500), ctx.json({ message: 'API Error' }));
      })
    );

    await renderWithSetup(
      <AppInitializer allowedStatus="ready">
        <div>Child content</div>
      </AppInitializer>
    );

    await waitFor(() => {
      const alert = screen.getByRole('alert');
      expect(alert).toBeInTheDocument();
      expect(alert).toHaveTextContent('Error');
      expect(alert).toHaveTextContent('API Error');
      expect(screen.queryByText('Child content')).not.toBeInTheDocument();
    });
  });

  it('displays error message for unexpected status even with children', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'unexpected' }));
      })
    );

    await renderWithSetup(
      <AppInitializer allowedStatus="ready">
        <div>Child content</div>
      </AppInitializer>
    );

    await waitFor(() => {
      const alert = screen.getByRole('alert');
      expect(alert).toBeInTheDocument();
      expect(alert).toHaveTextContent('Error');
      expect(alert).toHaveTextContent("unexpected status from /app/info endpoint - 'unexpected'");
      expect(screen.queryByText('Child content')).not.toBeInTheDocument();
    });
  });
});
