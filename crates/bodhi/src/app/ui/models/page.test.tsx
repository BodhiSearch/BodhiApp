import ModelsPage from '@/app/ui/models/page';
import { ENDPOINT_APP_INFO, ENDPOINT_MODELS, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

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

// Mock window.matchMedia for responsive testing
function mockMatchMedia(matches: boolean) {
  vi.stubGlobal('matchMedia', (query: string) => ({
    matches,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  }));
}

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

  it('renders responsive layouts correctly', async () => {
    // Test mobile view (< sm)
    mockMatchMedia(false);

    const { unmount } = render(<ModelsPage />, { wrapper: createWrapper() });

    // Wait for data to load
    await screen.findByTestId('combined-cell');

    // Mobile view should show combined cell
    expect(screen.getAllByTestId('combined-cell')[0]).toBeVisible();

    unmount();

    // Test tablet view (sm to lg)
    vi.stubGlobal('matchMedia', (query: string) => ({
      matches: query.includes('sm') && !query.includes('lg'),
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }));

    render(<ModelsPage />, { wrapper: createWrapper() });
    await screen.findByTestId('name-source-cell');

    // Tablet view should show combined name+source and repo+filename columns
    expect(screen.getAllByTestId('name-source-cell')[0]).toBeVisible();
    expect(screen.getAllByTestId('repo-filename-cell')[0]).toBeVisible();

    unmount();

    // Test desktop view (>= lg)
    mockMatchMedia(true);

    render(<ModelsPage />, { wrapper: createWrapper() });
    await screen.findByTestId('alias-cell');

    // Desktop view should show separate columns
    expect(screen.getAllByTestId('alias-cell')[0]).toBeVisible();
    expect(screen.getAllByTestId('repo-cell')[0]).toBeVisible();
    expect(screen.getAllByTestId('filename-cell')[0]).toBeVisible();
    expect(screen.getAllByTestId('source-cell')[0]).toBeVisible();
  });

  it('handles API error', async () => {
    server.use(
      rest.get(`*${ENDPOINT_MODELS}`, (req, res, ctx) => {
        return res(ctx.status(500), ctx.json({ error: { message: 'Internal Server Error' } }));
      })
    );
    await act(async () => {
      render(<ModelsPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByText('Internal Server Error')).toBeInTheDocument();
  });

  describe('action buttons', () => {
    it('shows FilePlus2 button for model source type', async () => {
      const modelData = {
        ...mockModelsResponse,
        data: [
          {
            alias: 'test-model',
            source: 'model',
            repo: 'test-repo',
            filename: 'test-file.bin',
            snapshot: 'abc123',
          },
        ],
      };

      server.use(
        rest.get(`*${ENDPOINT_MODELS}`, (_, res, ctx) => {
          return res(ctx.json(modelData));
        })
      );

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      const newButton = screen.getAllByTitle('Create new model alias using this modelfile')[0];
      expect(newButton).toBeInTheDocument();

      await act(async () => {
        newButton.click();
      });

      expect(pushMock).toHaveBeenCalledWith('/ui/models/new?repo=test-repo&filename=test-file.bin&snapshot=abc123');
    });

    it('shows edit button for non-model source type', async () => {
      const modelData = {
        ...mockModelsResponse,
        data: [
          {
            alias: 'test-alias',
            source: 'alias',
            repo: 'test-repo',
            filename: 'test-file.bin',
          },
        ],
      };

      server.use(
        rest.get(`*${ENDPOINT_MODELS}`, (_, res, ctx) => {
          return res(ctx.json(modelData));
        })
      );

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      const editButton = screen.getAllByTitle('Edit test-alias')[0];
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

      const chatButton = screen.getAllByTitle('Chat with the model in playground')[0];
      expect(chatButton).toBeInTheDocument();

      const hfButton = screen.getAllByTitle('Open in HuggingFace')[0];
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

      const hfButton = screen.getAllByTitle('Open in HuggingFace')[0];
      await act(async () => {
        hfButton.click();
      });

      expect(windowOpenSpy).toHaveBeenCalledWith('https://huggingface.co/test-repo/blob/main/test-file.bin', '_blank');

      windowOpenSpy.mockRestore();
    });
  });

  it('displays error message when API call fails', async () => {
    server.use(
      rest.get(`*${ENDPOINT_MODELS}`, (_, res, ctx) => {
        return res(ctx.status(500), ctx.json({ error: { message: 'Internal Server Error' } }));
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
        return res(ctx.json({ status: 'ready' }));
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
