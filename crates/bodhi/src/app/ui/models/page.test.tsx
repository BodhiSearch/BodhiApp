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
      screen.getByText('Internal Server Error')
    ).toBeInTheDocument();
  });

  describe('action buttons', () => {
    it('shows FilePlus2 button for model source type', async () => {
      const modelData = {
        ...mockModelsResponse,
        data: [{
          alias: 'test-model',
          source: 'model',
          repo: 'test-repo',
          filename: 'test-file.bin',
          snapshot: 'abc123'
        }]
      };

      server.use(
        rest.get(`*${ENDPOINT_MODELS}`, (_, res, ctx) => {
          return res(ctx.json(modelData));
        })
      );

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      const newButton = screen.getByTitle('Create new model alias using this modelfile');
      expect(newButton).toBeInTheDocument();
      
      await act(async () => {
        newButton.click();
      });
      
      expect(pushMock).toHaveBeenCalledWith(
        '/ui/models/new?repo=test-repo&filename=test-file.bin&snapshot=abc123'
      );
    });

    it('shows edit button for non-model source type', async () => {
      const modelData = {
        ...mockModelsResponse,
        data: [{
          alias: 'test-alias',
          source: 'alias',
          repo: 'test-repo',
          filename: 'test-file.bin'
        }]
      };

      server.use(
        rest.get(`*${ENDPOINT_MODELS}`, (_, res, ctx) => {
          return res(ctx.json(modelData));
        })
      );

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      const editButton = screen.getByTitle('Edit test-alias');
      expect(editButton).toBeInTheDocument();
      
      await act(async () => {
        editButton.click();
      });
      
      expect(pushMock).toHaveBeenCalledWith('/ui/models/edit?alias=test-alias');
    });

    it('shows chat and huggingface buttons for all models', async () => {
      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      const chatButton = screen.getByTitle('Chat with the model in playground');
      expect(chatButton).toBeInTheDocument();
      
      const hfButton = screen.getByTitle('Open in HuggingFace');
      expect(hfButton).toBeInTheDocument();

      await act(async () => {
        chatButton.click();
      });
      
      expect(pushMock).toHaveBeenCalledWith('/ui/chat?alias=test-model');
    });

    it('opens huggingface link in new tab', async () => {
      const windowOpenSpy = vi.spyOn(window, 'open').mockImplementation(() => null);

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      const hfButton = screen.getByTitle('Open in HuggingFace');
      await act(async () => {
        hfButton.click();
      });
      
      expect(windowOpenSpy).toHaveBeenCalledWith(
        'https://huggingface.co/test-repo/blob/main/test-file.bin',
        '_blank'
      );

      windowOpenSpy.mockRestore();
    });
  });

  it('displays error message when API call fails', async () => {
    server.use(
      rest.get(`*${ENDPOINT_MODELS}`, (_, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({ error: { message: 'Internal Server Error' } })
        );
      })
    );

    await act(async () => {
      render(<ModelsPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByRole('alert')).toHaveTextContent('Internal Server Error');
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
