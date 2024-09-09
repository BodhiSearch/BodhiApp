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

    render(<AppInitializer />);

    await waitFor(() => {
      expect(
        screen.getByText(/Unable to connect to backend/)
      ).toBeInTheDocument();
    });
  });

  it('redirects to /ui/setup when status is setup and no allowedStatus is provided', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    render(<AppInitializer />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/home when status is ready and no allowedStatus is provided', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    render(<AppInitializer />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/home');
    });
  });

  it('redirects to /ui/setup/resource-admin when status is resource-admin and no allowedStatus is provided', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );

    render(<AppInitializer />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/resource-admin');
    });
  });

  it('displays error message for unexpected status when no allowedStatus is provided', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'unexpected' }));
      })
    );

    render(<AppInitializer />);

    await waitFor(() => {
      expect(
        screen.getByText(
          /unexpected \/app\/info status from server - unexpected/,
          { exact: false }
        )
      ).toBeInTheDocument();
    });
  });

  it('redirects to /ui/setup when status is setup and allowedStatus is ready', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    render(<AppInitializer allowedStatus="ready" />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/home when status is ready and allowedStatus is setup', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    render(<AppInitializer allowedStatus="setup" />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/home');
    });
  });

  it('does not redirect when status matches allowedStatus', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    render(<AppInitializer allowedStatus="ready" />);

    await waitFor(() => {
      expect(pushMock).not.toHaveBeenCalled();
    });
  });

  it('displays children content if app status matches allowedStatus', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );

    render(
      <AppInitializer allowedStatus="ready">
        <div>Child content</div>
      </AppInitializer>
    );

    await waitFor(() => {
      expect(screen.getByText('Child content')).toBeInTheDocument();
    });
  });

  it('does not display children content if app status does not match allowedStatus', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    render(
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

    render(
      <AppInitializer allowedStatus="ready">
        <div>Child content</div>
      </AppInitializer>
    );

    expect(screen.getByText('Initializing app...')).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.queryByText('Initializing app...')).not.toBeInTheDocument();
      expect(screen.getByText('Child content')).toBeInTheDocument();
    });
  });

  it('displays error message and not children when API call fails', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.status(500), ctx.json({ message: 'API Error' }));
      })
    );

    render(
      <AppInitializer allowedStatus="ready">
        <div>Child content</div>
      </AppInitializer>
    );

    await waitFor(() => {
      expect(
        screen.getByText(/Unable to connect to backend/)
      ).toBeInTheDocument();
      expect(screen.queryByText('Child content')).not.toBeInTheDocument();
    });
  });

  it('displays children for any status when no allowedStatus is provided', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    render(
      <AppInitializer>
        <div>Child content</div>
      </AppInitializer>
    );

    await waitFor(() => {
      expect(screen.queryByText('Child content')).not.toBeInTheDocument();
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('displays error message for unexpected status even with children', async () => {
    server.use(
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.json({ status: 'unexpected' }));
      })
    );

    render(
      <AppInitializer allowedStatus="ready">
        <div>Child content</div>
      </AppInitializer>
    );

    await waitFor(() => {
      expect(
        screen.getByText(
          /unexpected \/app\/info status from server - unexpected/,
          { exact: false }
        )
      ).toBeInTheDocument();
      expect(screen.queryByText('Child content')).not.toBeInTheDocument();
    });
  });
});
