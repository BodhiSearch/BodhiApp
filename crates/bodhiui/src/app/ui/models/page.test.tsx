import { render, screen, waitFor } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import {
  afterAll,
  afterEach,
  beforeAll,
  describe,
  expect,
  it,
  vi,
  beforeEach,
} from 'vitest';
import ModelsPage from './page';
import { createWrapper } from '@/tests/wrapper';

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

const mockModelsResponse = {
  data: [
    {
      alias: 'test-model',
      family: 'test-family',
      repo: 'test-repo',
      filename: 'test-file.bin',
      features: ['feature1', 'feature2'],
      snapshot: 'abc123',
      chat_template: 'test-template',
      request_params: {},
      context_params: {},
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

describe('ModelsPage', () => {
  beforeEach(() => {
    server.use(
      rest.get('*/app/info', (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      }),
      rest.get('*/api/ui/user', (_, res, ctx) => {
        return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
      }),
      rest.get('*/api/ui/models', (_, res, ctx) => {
        return res(ctx.json(mockModelsResponse));
      })
    );
  });

  it('renders models data successfully', async () => {
    render(<ModelsPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByTestId('app-header')).toBeInTheDocument();
      expect(screen.getByText('test-model')).toBeInTheDocument();
    });
    expect(screen.getByText('test-family')).toBeInTheDocument();
    expect(screen.getByText('test-repo')).toBeInTheDocument();
    expect(screen.getByText('test-file.bin')).toBeInTheDocument();
    expect(screen.getByText('feature1, feature2')).toBeInTheDocument();
    expect(screen.getByTestId('pagination')).toBeInTheDocument();
    expect(screen.getByText('Displaying 1 items of 1')).toBeInTheDocument();
  });

  it('handles API error', async () => {
    server.use(
      rest.get('*/api/ui/models', (req, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({ message: 'Internal Server Error' })
        );
      })
    );

    render(<ModelsPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(
        screen.getByText('An error occurred: Internal Server Error')
      ).toBeInTheDocument();
    });
  });
});

describe('ModelsPage access control', () => {
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
    render(<ModelsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('should redirect to /ui/login if user is not logged in', async () => {
    server.use(
      rest.get('*/app/info', (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );
    server.use(
      rest.get('*/api/ui/user', (_, res, ctx) => {
        return res(ctx.json({ logged_in: false }));
      })
    );
    render(<ModelsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });
});
