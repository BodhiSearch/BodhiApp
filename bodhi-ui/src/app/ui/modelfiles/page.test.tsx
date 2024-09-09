import { render, screen, waitFor } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { QueryClient, QueryClientProvider } from 'react-query';
import {
  afterAll,
  afterEach,
  beforeAll,
  describe,
  expect,
  it,
  vi,
} from 'vitest';
import ModelFilesPage from './page';

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

const server = setupServer(
  rest.get('*/api/ui/modelfiles', (req, res, ctx) => {
    return res(ctx.json(mockModelFilesResponse));
  })
);

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());

describe('ModelFilesPage', () => {
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
