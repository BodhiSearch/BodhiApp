import { createWrapper } from '@/tests/wrapper';
import { act, render, screen } from '@testing-library/react';
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
import PullPage from './page';

vi.mock('@/components/layout/MainLayout', () => ({
  MainLayout: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
}));

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

const mockDownloadsResponse = {
  data: [
    {
      id: '1',
      repo: 'test/repo',
      filename: 'model.gguf',
      status: 'pending',
      updated_at: '2024-01-01T00:00:00Z',
    },
    {
      id: '2',
      repo: 'test/repo2',
      filename: 'model2.gguf',
      status: 'error',
      error_message: 'Download failed',
      updated_at: '2024-01-01T00:00:00Z',
    },
  ],
  total: 2,
  page: 1,
  page_size: 30,
};

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());
beforeEach(() => {
  vi.resetAllMocks();
  pushMock.mockClear();
});

describe('PullPage', () => {
  beforeEach(() => {
    server.use(
      rest.get('*/app/info', (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      }),
      rest.get('*/api/ui/user', (_, res, ctx) => {
        return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
      }),
      rest.get('*/modelfiles/pull/downloads', (_, res, ctx) => {
        return res(ctx.json(mockDownloadsResponse));
      })
    );
  });

  it('renders downloads table with data', async () => {
    await act(async () => {
      render(<PullPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByText('test/repo')).toBeInTheDocument();
    expect(screen.getByText('model.gguf')).toBeInTheDocument();
    expect(screen.getByText('pending')).toBeInTheDocument();
    expect(screen.getByText('test/repo2')).toBeInTheDocument();
    expect(screen.getByText('model2.gguf')).toBeInTheDocument();
    expect(screen.getByText('error')).toBeInTheDocument();
  });

  it('handles API error', async () => {
    server.use(
      rest.get('*/modelfiles/pull/downloads', (_, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({ message: 'Internal Server Error' })
        );
      })
    );

    await act(async () => {
      render(<PullPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByText('Error loading downloads')).toBeInTheDocument();
  });
});

describe('PullPage access control', () => {
  it('should redirect to /ui/setup if status is setup', async () => {
    server.use(
      rest.get('*/app/info', (_, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );
    server.use(
      rest.get('*/api/ui/user', (_, res, ctx) => {
        return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
      })
    );
    await act(async () => {
      render(<PullPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/setup');
  });

  it('should redirect to /ui/login if user is not logged in', async () => {
    server.use(
      rest.get('*/app/info', (_, res, ctx) => {
        return res(ctx.json({ status: 'ready', authz: true }));
      }),
      rest.get('*/api/ui/user', (_, res, ctx) => {
        return res(ctx.json({ logged_in: false }));
      })
    );
    await act(async () => {
      render(<PullPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/login');
  });
});