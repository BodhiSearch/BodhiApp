import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
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
import ModelFilesPage from '@/app/ui/modelfiles/page';

// Mock components
vi.mock('@/components/AppHeader', () => ({
  default: () => <div data-testid="app-header">Mocked AppHeader</div>,
}));

vi.mock('@/components/DataTable', () => ({
  DataTable: ({ data, renderRow }: any) => (
    <table>
      <tbody>
        {data.map((item: any, index: number) => (
          <tr key={index}>{renderRow(item)}</tr>
        ))}
      </tbody>
    </table>
  ),
  Pagination: () => <div data-testid="pagination">Mocked Pagination</div>,
}));

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

const mockModelFilesResponse = {
  data: [
    {
      repo: 'test-repo',
      filename: 'test-file.txt',
      size: 1073741824, // 1 GB
      updated_at: null,
      snapshot: 'abc123',
    },
  ],
  total: 1,
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

describe('ModelFilesPage', () => {
  beforeEach(() => {
    server.use(
      rest.get('*/app/info', (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      }),
      rest.get('*/api/ui/user', (_, res, ctx) => {
        return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
      }),
      rest.get('*/api/ui/modelfiles', (_, res, ctx) => {
        return res(ctx.json(mockModelFilesResponse));
      })
    );
  });

  it('renders model files data successfully', async () => {
    render(<ModelFilesPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByTestId('app-header')).toBeInTheDocument();
      expect(screen.getByText('test-repo')).toBeInTheDocument();
      expect(screen.getByText('test-file.txt')).toBeInTheDocument();
      expect(screen.getByText('1.00 GB')).toBeInTheDocument();
      expect(screen.getByText('abc123')).toBeInTheDocument();
      expect(screen.getByTestId('pagination')).toBeInTheDocument();
      expect(screen.getByText('Displaying 1 items of 1')).toBeInTheDocument();
    });
  });

  it('handles API error', async () => {
    server.use(
      rest.get('*/api/ui/modelfiles', (req, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({ message: 'Internal Server Error' })
        );
      })
    );

    render(<ModelFilesPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(
        screen.getByText('An error occurred: Internal Server Error')
      ).toBeInTheDocument();
    });
  });
});

describe('ModelFilesPage access control', () => {
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
    render(<ModelFilesPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
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
    render(<ModelFilesPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });
});
