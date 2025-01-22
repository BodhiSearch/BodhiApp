import ModelsPage from '@/app/ui/models/page';
import { ENDPOINT_APP_INFO, ENDPOINT_MODELS, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
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
      repo: 'test-repo',
      filename: 'test-file.bin',
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
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
      }),
      rest.get(`*${ENDPOINT_MODELS}`, (_, res, ctx) => {
        return res(ctx.json(mockModelsResponse));
      })
    );
  });

  it('renders models data successfully', async () => {
    await act(async () => {
      render(<ModelsPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByText('test-model')).toBeInTheDocument();
    expect(screen.getByText('test-repo')).toBeInTheDocument();
    expect(screen.getByText('test-file.bin')).toBeInTheDocument();
    expect(screen.getByTestId('pagination')).toBeInTheDocument();
    expect(screen.getByText('Displaying 1 items of 1')).toBeInTheDocument();
  });

  it('handles API error', async () => {
    server.use(
      rest.get(`*${ENDPOINT_MODELS}`, (req, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({ error: { message: 'Internal Server Error' } })
        );
      })
    );
    await act(async () => {
      render(<ModelsPage />, { wrapper: createWrapper() });
    });

    expect(
      screen.getByText('An error occurred: Internal Server Error')
    ).toBeInTheDocument();
  });
});

describe('ModelsPage access control', () => {
  it('should redirect to /ui/setup if status is setup', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
      })
    );

    await act(async () => {
      render(<ModelsPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/setup');
  });

  it('should redirect to /ui/login if user is not logged in', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready', authz: true }));
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ logged_in: false }));
      })
    );
    await act(async () => {
      render(<ModelsPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/login');
  });
});
