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
import userEvent from '@testing-library/user-event';
import PullPage from './page';

vi.mock('@/components/layout/MainLayout', () => ({
  MainLayout: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
}));

vi.mock('@/components/PullForm', () => ({
  PullForm: () => <div data-testid="pull-form">Pull Form</div>,
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
      repo: 'test/repo1',
      filename: 'model1.gguf',
      status: 'pending',
      error: null,
      updated_at: '2024-01-01T00:00:00Z',
    },
    {
      id: '2',
      repo: 'test/repo2',
      filename: 'model2.gguf',
      status: 'completed',
      error: null,
      updated_at: '2024-01-01T00:00:00Z',
    },
    {
      id: '3',
      repo: 'test/repo3',
      filename: 'model3.gguf',
      status: 'error',
      error: 'Download failed',
      updated_at: '2024-01-01T00:00:00Z',
    },
  ],
  total: 3,
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

  it('renders pull form and downloads table with error details in expanded row', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<PullPage />, { wrapper: createWrapper() });
    });

    // Check form is rendered
    expect(screen.getByTestId('pull-form')).toBeInTheDocument();

    // Check pending download
    expect(screen.getByText('test/repo1')).toBeInTheDocument();
    expect(screen.getByText('model1.gguf')).toBeInTheDocument();
    expect(screen.getByText('pending')).toBeInTheDocument();

    // Check completed download
    expect(screen.getByText('test/repo2')).toBeInTheDocument();
    expect(screen.getByText('model2.gguf')).toBeInTheDocument();
    expect(screen.getByText('completed')).toBeInTheDocument();

    // Check error download
    expect(screen.getByText('test/repo3')).toBeInTheDocument();
    expect(screen.getByText('model3.gguf')).toBeInTheDocument();
    expect(screen.getByText('error')).toBeInTheDocument();

    // Error message should not be visible initially
    expect(screen.queryByText('Error:')).not.toBeInTheDocument();
    expect(screen.queryByText('Download failed')).not.toBeInTheDocument();

    // Find and click the expand button for the error row
    const rows = screen.getAllByRole('row');
    const errorRow = rows.find(row => row.textContent?.includes('test/repo3'));
    const expandButton = errorRow?.querySelector('button');
    expect(expandButton).toBeInTheDocument();
    await user.click(expandButton!);

    // Now error message should be visible
    expect(screen.getByText('Error:')).toBeInTheDocument();
    expect(screen.getByText('Download failed')).toBeInTheDocument();
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