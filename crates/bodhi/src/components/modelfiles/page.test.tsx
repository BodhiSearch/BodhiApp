import ModelFilesPage from '@/components/modelfiles/page';
import { ENDPOINT_APP_INFO, ENDPOINT_MODEL_FILES, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
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
vi.mock('@/lib/navigation', () => ({
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

describe('ModelFilesPage', () => {
  beforeEach(() => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
      }),
      rest.get(`*${ENDPOINT_MODEL_FILES}`, (_, res, ctx) => {
        return res(ctx.json(mockModelFilesResponse));
      })
    );
  });

  it('renders responsive layouts correctly', async () => {
    // Test mobile view (< sm)
    mockMatchMedia(false);

    const { unmount } = render(<ModelFilesPage />, { wrapper: createWrapper() });

    // Wait for data to load
    await screen.findByTestId('combined-cell');

    // Mobile view should show combined cell
    expect(screen.getAllByTestId('combined-cell')[0]).toBeVisible();

    unmount();

    // Test desktop view (>= sm)
    mockMatchMedia(true);

    render(<ModelFilesPage />, { wrapper: createWrapper() });
    await screen.findByTestId('repo-cell');

    // Desktop view should show separate columns
    expect(screen.getAllByTestId('repo-cell')[0]).toBeVisible();
    expect(screen.getAllByTestId('filename-cell')[0]).toBeVisible();
    expect(screen.getAllByTestId('size-cell')[0]).toBeVisible();
    expect(screen.getAllByTestId('actions-cell')[0]).toBeVisible();
  });

  it('handles API error', async () => {
    server.use(
      rest.get(`*${ENDPOINT_MODEL_FILES}`, (req, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({ error: { message: 'Internal Server Error' } })
        );
      })
    );
    await act(async () => {
      render(<ModelFilesPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByText('Internal Server Error')).toBeInTheDocument();
  });

  describe('action buttons', () => {
    it('shows delete and huggingface buttons', async () => {
      await act(async () => {
        render(<ModelFilesPage />, { wrapper: createWrapper() });
      });

      const deleteButton = screen.getAllByTitle('Delete modelfile')[0];
      expect(deleteButton).toBeInTheDocument();

      const hfButton = screen.getAllByTitle('Open in HuggingFace')[0];
      expect(hfButton).toBeInTheDocument();
    });

    it('opens huggingface link in new tab', async () => {
      const windowOpenSpy = vi.spyOn(window, 'open').mockImplementation(() => null);

      await act(async () => {
        render(<ModelFilesPage />, { wrapper: createWrapper() });
      });

      const hfButton = screen.getAllByTitle('Open in HuggingFace')[0];
      await act(async () => {
        hfButton.click();
      });

      expect(windowOpenSpy).toHaveBeenCalledWith(
        'https://huggingface.co/test-repo',
        '_blank'
      );

      windowOpenSpy.mockRestore();
    });
  });
});

describe('ModelFilesPage access control', () => {
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
      render(<ModelFilesPage />, { wrapper: createWrapper() });
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
      render(<ModelFilesPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/login');
  });
});
