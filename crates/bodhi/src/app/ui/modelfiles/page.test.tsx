import ModelFilesPage from '@/app/ui/modelfiles/page';
import { server } from '@/test-utils/msw-v2/setup';
import { mockAppInfoReady, mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { mockModelFilesDefault, mockModelFilesError } from '@/test-utils/msw-v2/handlers/modelfiles';
import { ENDPOINT_APP_INFO, ENDPOINT_MODEL_FILES, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { createMockLoggedInUser, createMockLoggedOutUser } from '@/test-utils/mock-user';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen } from '@testing-library/react';
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
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn(), ...mockModelFilesDefault());
  });

  it('renders responsive layouts correctly', async () => {
    // Test mobile view (< sm)
    mockMatchMedia(false);

    const { unmount } = render(<ModelFilesPage />, {
      wrapper: createWrapper(),
    });

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
    server.use(...mockModelFilesError({ status: 500, message: 'Internal Server Error' }));

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

      expect(windowOpenSpy).toHaveBeenCalledWith('https://huggingface.co/test-repo', '_blank');

      windowOpenSpy.mockRestore();
    });
  });
});

describe('ModelFilesPage access control', () => {
  it('should redirect to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfoSetup(), ...mockUserLoggedIn());

    await act(async () => {
      render(<ModelFilesPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/setup');
  });

  it('should redirect to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedOut());

    await act(async () => {
      render(<ModelFilesPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/login');
  });
});
