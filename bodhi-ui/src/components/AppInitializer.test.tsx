'use client';

import { createWrapper } from '@/tests/wrapper';
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
import AppInitializer from './AppInitializer';

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

// Add this helper function
const renderWithSetup = async (ui: React.ReactElement) => {
  const wrapper = createWrapper();
  const rendered = render(ui, { wrapper });
  // Wait for the loading state to disappear
  await waitForElementToBeRemoved(() =>
    rendered.getByText('Initializing app...')
  );
  return rendered;
};

describe('AppInitializer with no authentication', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    server.use(
      rest.get('*/api/ui/user', (req, res, ctx) => {
        return res(ctx.status(200), ctx.json({ logged_in: false }));
      })
    );
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
      expect(alert).toHaveTextContent(
        "unexpected status from /app/info endpoint - 'unexpected'"
      );
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
    const rendered = render(
      <AppInitializer allowedStatus="ready">
        <div>Child content</div>
      </AppInitializer>,
      { wrapper }
    );

    expect(screen.getByText('Initializing app...')).toBeInTheDocument();
    await waitForElementToBeRemoved(() =>
      rendered.getByText('Initializing app...')
    );
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
      expect(alert).toHaveTextContent(
        "unexpected status from /app/info endpoint - 'unexpected'"
      );
      expect(screen.queryByText('Child content')).not.toBeInTheDocument();
    });
  });
});

describe('AppInitializer with status ready and authentication required', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );
  });

  it('redirects to /ui/login when authenticated is true and user is not logged in', async () => {
    server.use(
      rest.get('*/api/ui/user', (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ logged_in: false }));
      })
    );

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={true} />
    );

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });

  it('displays children when authenticated is true and user is logged in', async () => {
    server.use(
      rest.get('*/api/ui/user', (req, res, ctx) => {
        return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
      })
    );

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={true}>
        <div>Child content</div>
      </AppInitializer>
    );

    expect(screen.getByText('Child content')).toBeInTheDocument();
  });

  it('displays loading state while checking user authentication', async () => {
    server.use(
      rest.get('*/api/ui/user', (req, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.json({ logged_in: true, email: 'test@example.com' })
        );
      })
    );
    const wrapper = createWrapper();
    render(
      <AppInitializer allowedStatus="ready" authenticated={true}>
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

  it('does not check user authentication when authenticated is false', async () => {
    const apiCallSpy = vi.fn();
    server.use(
      rest.get('*/api/ui/user', (req, res, ctx) => {
        apiCallSpy();
        return res(ctx.status(200), ctx.json({ logged_in: false }));
      })
    );

    await renderWithSetup(
      <AppInitializer allowedStatus="ready" authenticated={false}>
        <div>Child content</div>
      </AppInitializer>
    );
    expect(apiCallSpy).not.toHaveBeenCalled();
    expect(screen.getByText('Child content')).toBeInTheDocument();
  });
});
