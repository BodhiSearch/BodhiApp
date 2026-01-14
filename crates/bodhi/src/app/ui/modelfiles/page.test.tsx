import ModelFilesPage from '@/app/ui/modelfiles/page';
import { server } from '@/test-utils/msw-v2/setup';
import { mockAppInfoReady, mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { mockModelFilesDefault, mockModelFilesError } from '@/test-utils/msw-v2/handlers/modelfiles';
import { mockRefreshSingleMetadata } from '@/test-utils/msw-v2/handlers/models';
import { ENDPOINT_APP_INFO } from '@/hooks/useInfo';
import { ENDPOINT_MODEL_FILES } from '@/hooks/useModels';
import { ENDPOINT_USER_INFO } from '@/hooks/useUsers';
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

    // Add fresh mocks for second render
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn(), ...mockModelFilesDefault());

    render(<ModelFilesPage />, { wrapper: createWrapper() });
    await screen.findByTestId('repo-cell');

    // Desktop view should show separate columns
    expect(screen.getAllByTestId('repo-cell')[0]).toBeVisible();
    expect(screen.getAllByTestId('filename-cell')[0]).toBeVisible();
    expect(screen.getAllByTestId('size-cell')[0]).toBeVisible();
    expect(screen.getAllByTestId('actions-cell')[0]).toBeVisible();
  });

  it('handles API error', async () => {
    server.use(...mockModelFilesError());

    await act(async () => {
      render(<ModelFilesPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByText('Internal server error')).toBeInTheDocument();
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

  describe('Model metadata refresh', () => {
    it('shows refresh button in modal and triggers API call with correct query params', async () => {
      server.use(
        ...mockRefreshSingleMetadata({
          repo: 'test-repo',
          filename: 'test-file.txt',
          snapshot: 'abc123',
          metadata: {
            capabilities: { vision: false, audio: false, thinking: false, tools: {} },
            context: {},
            architecture: { format: 'gguf' },
          },
        })
      );

      await act(async () => {
        render(<ModelFilesPage />, { wrapper: createWrapper() });
      });

      // Open preview modal
      const previewButtons = screen.getAllByTestId('modelfiles-preview-button-test-repo-test-file.txt');
      const previewButton = previewButtons[0];
      expect(previewButton).toBeInTheDocument();

      await act(async () => {
        previewButton.click();
      });

      // Modal should have body refresh button (since no metadata initially)
      const refreshButton = screen.getByTestId('preview-modal-refresh-button-body');
      expect(refreshButton).toBeInTheDocument();
      expect(refreshButton).toBeEnabled();

      await act(async () => {
        refreshButton.click();
      });

      // Refresh completes (button should still be visible)
      expect(refreshButton).toBeInTheDocument();
    });
  });

  describe('Model preview modal', () => {
    it('opens preview modal when preview button clicked', async () => {
      await act(async () => {
        render(<ModelFilesPage />, { wrapper: createWrapper() });
      });

      const previewButtons = screen.getAllByTestId('modelfiles-preview-button-test-repo-test-file.txt');
      const previewButton = previewButtons[0];
      expect(previewButton).toBeInTheDocument();

      await act(async () => {
        previewButton.click();
      });

      // Modal should be visible
      expect(screen.getByTestId('model-preview-modal')).toBeInTheDocument();

      // Basic info should be displayed
      expect(screen.getByTestId('preview-basic-alias')).toHaveTextContent('test-repo/test-file.txt');
      expect(screen.getByTestId('preview-basic-repo')).toHaveTextContent('test-repo');
      expect(screen.getByTestId('preview-basic-filename')).toHaveTextContent('test-file.txt');
    });

    it('shows refresh button in modal when no metadata available', async () => {
      await act(async () => {
        render(<ModelFilesPage />, { wrapper: createWrapper() });
      });

      const previewButtons = screen.getAllByTestId('modelfiles-preview-button-test-repo-test-file.txt');
      const previewButton = previewButtons[0];

      await act(async () => {
        previewButton.click();
      });

      // Modal should be visible with body refresh button (since no metadata)
      expect(screen.getByTestId('model-preview-modal')).toBeInTheDocument();
      expect(screen.getByTestId('preview-modal-refresh-button-body')).toBeInTheDocument();
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
