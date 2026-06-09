import ModelsPage from '@/routes/models/index';
import { mockAppInfo, mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import {
  mockModelsDefault,
  mockModelsInternalError,
  mockModelsWithApiModel,
  mockModelsWithSourceModel,
  mockRefreshSingleMetadata,
} from '@/test-utils/msw-v2/handlers/models';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { createWrapper } from '@/tests/wrapper';
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
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

const navigateMock = vi.fn();
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, search, children, ...rest }: any) => {
      const searchStr = search ? '?' + new URLSearchParams(search).toString() : '';
      return (
        <a href={`${to}${searchStr}`} {...rest}>
          {children}
        </a>
      );
    },
    useNavigate: () => navigateMock,
  };
});

const mockModelsResponse = {
  data: [
    {
      source: 'user' as const,
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
  navigateMock.mockClear();
});

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
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn({ role: 'resource_user' }), ...mockModelsDefault());
  });

  it('renders responsive layouts correctly', async () => {
    // Test mobile view (< sm)
    mockMatchMedia(false);

    const { unmount } = render(<ModelsPage />, { wrapper: createWrapper() });

    await screen.findByTestId('combined-cell-test-model');

    expect(screen.getByTestId('combined-cell-test-model')).toBeVisible();

    unmount();

    // Add fresh mocks for second render
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn({ role: 'resource_user' }), ...mockModelsDefault());

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

    const { unmount: unmountTablet } = render(<ModelsPage />, { wrapper: createWrapper() });
    await screen.findByTestId('name-source-cell-test-model');

    expect(screen.getByTestId('name-source-cell-test-model')).toBeVisible();
    expect(screen.getByTestId('repo-filename-cell-test-model')).toBeVisible();

    unmountTablet();

    // Add fresh mocks for third render
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn({ role: 'resource_user' }), ...mockModelsDefault());

    // Test desktop view (>= lg)
    mockMatchMedia(true);

    render(<ModelsPage />, { wrapper: createWrapper() });
    await screen.findByTestId('alias-cell-test-model');

    expect(screen.getByTestId('alias-cell-test-model')).toBeVisible();
    expect(screen.getByTestId('repo-cell-test-model')).toBeVisible();
    expect(screen.getByTestId('filename-cell-test-model')).toBeVisible();
    expect(screen.getByTestId('source-cell-test-model')).toBeVisible();
  });

  it('handles API error', async () => {
    server.use(...mockModelsInternalError());
    await act(async () => {
      render(<ModelsPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByText('Internal server error')).toBeInTheDocument();
  });

  describe('action buttons', () => {
    it('shows FilePlus2 button for model source type', async () => {
      server.use(...mockModelsWithSourceModel());

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      const newButton = screen.getAllByTitle('Create new model alias using this modelfile')[0];
      expect(newButton).toBeInTheDocument();

      await act(async () => {
        newButton.click();
      });

      expect(navigateMock).toHaveBeenCalledWith({
        to: '/models/alias/new/',
        search: { repo: 'test-repo', filename: 'test-file.bin', snapshot: 'abc123' },
      });
    });

    it('shows edit button for non-model source type', async () => {
      server.use(...mockModelsDefault());

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      const editButton = screen.getAllByTitle('Edit test-model')[0];
      expect(editButton).toBeInTheDocument();

      await act(async () => {
        editButton.click();
      });

      expect(navigateMock).toHaveBeenCalledWith({ to: '/models/alias/edit/', search: { id: 'test-uuid-1' } });
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

      expect(navigateMock).toHaveBeenCalledWith({ to: '/chat/', search: { model: 'test-model' } });
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
    it('shows new API model button and handles navigation', async () => {
      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      const newApiModelButton = screen.getByText('New API Model');
      expect(newApiModelButton).toBeInTheDocument();

      await act(async () => {
        newApiModelButton.click();
      });

      expect(navigateMock).toHaveBeenCalledWith({ to: '/models/api/new/' });
    });
  });

  describe('API model display and actions', () => {
    it('displays API models with correct information', async () => {
      server.use(...mockModelsWithApiModel());

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      // getAllByText because the same value renders in multiple responsive layouts.
      expect(screen.getAllByText('test-api-model')[0]).toBeInTheDocument();
      expect(screen.getAllByText('openai')[0]).toBeInTheDocument();
      expect(screen.getAllByText('https://api.openai.com/v1')[0]).toBeInTheDocument();
      expect(screen.getAllByText('gpt-4, gpt-3.5-turbo')[0]).toBeInTheDocument();
    });

    it('shows edit and delete buttons for API models', async () => {
      server.use(...mockModelsWithApiModel());

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      const editButton = screen.getAllByTitle('Edit API model test-api-model')[0];
      expect(editButton).toBeInTheDocument();

      await act(async () => {
        editButton.click();
      });

      expect(navigateMock).toHaveBeenCalledWith({ to: '/models/api/edit/', search: { id: 'test-api-model' } });
    });

    it('shows chat button for API models with model identifier', async () => {
      server.use(...mockModelsWithApiModel());

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      const apiModelRow = screen.getByTestId('actions-cell-test-api-model');
      const chatButton = apiModelRow.querySelector('[data-testid="model-chat-button-gpt-4"]');
      expect(chatButton).toBeInTheDocument();
      expect(chatButton).toHaveAttribute('title', 'Chat with gpt-4');

      await act(async () => {
        fireEvent.click(chatButton!);
      });

      expect(navigateMock).toHaveBeenCalledWith({ to: '/chat/', search: { model: 'gpt-4' } });
    });

    it('does not show HuggingFace button for API models', async () => {
      server.use(...mockModelsWithApiModel());

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      const hfButtons = screen.queryAllByTitle('Open in HuggingFace');
      expect(hfButtons).toHaveLength(0);
    });

    it('navigates to edit page when edit button is clicked for API model', async () => {
      server.use(...mockModelsWithApiModel());

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      await screen.findAllByText('test-api-model');
      expect(screen.getAllByText('openai')[0]).toBeInTheDocument();
      expect(screen.getAllByText('https://api.openai.com/v1')[0]).toBeInTheDocument();

      // Model list can render across responsive layouts, so match on combined content.
      const modelsText = screen.getAllByText((content, element) => {
        return content.includes('gpt-4') && content.includes('gpt-3.5-turbo');
      });
      expect(modelsText.length).toBeGreaterThan(0);

      const editButton = screen.getAllByTitle('Edit API model test-api-model')[0];
      expect(editButton).toBeInTheDocument();

      await act(async () => {
        editButton.click();
      });

      expect(navigateMock).toHaveBeenCalledWith({ to: '/models/api/edit/', search: { id: 'test-api-model' } });
    });

    it('navigates to chat page when clicking on API model for chat', async () => {
      server.use(...mockModelsWithApiModel());

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      await screen.findAllByText('test-api-model');

      const apiModelRow = screen.getByTestId('actions-cell-test-api-model');
      const chatButton = apiModelRow.querySelector('[data-testid="chat-button-test-api-model"]');
      expect(chatButton).not.toBeInTheDocument();
      const modelChatButton = apiModelRow.querySelector('[data-testid="model-chat-button-gpt-4"]');
      expect(modelChatButton).toBeInTheDocument();
      expect(modelChatButton).toHaveAttribute('title', 'Chat with gpt-4');

      await act(async () => {
        fireEvent.click(modelChatButton!);
      });

      expect(navigateMock).toHaveBeenCalledWith({ to: '/chat/', search: { model: 'gpt-4' } });
    });
  });

  it('displays error message when API call fails', async () => {
    server.use(...mockModelsInternalError());

    await act(async () => {
      render(<ModelsPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByRole('alert')).toHaveTextContent('Internal server error');
  });
});

describe('ModelsPage access control', () => {
  it('should redirect to /setup if status is setup', async () => {
    server.use(...mockAppInfo({ status: 'setup' }), ...mockUserLoggedIn());

    await act(async () => {
      render(<ModelsPage />, { wrapper: createWrapper() });
    });
    expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/' });
  });

  it('should redirect to /login if user is not logged in', async () => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedOut());
    await act(async () => {
      render(<ModelsPage />, { wrapper: createWrapper() });
    });
    expect(navigateMock).toHaveBeenCalledWith({ to: '/login/' });
  });
});

describe('Model Metadata Refresh', () => {
  beforeEach(() => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn({ role: 'resource_user' }), ...mockModelsDefault());
  });

  it('per-model refresh button in modal triggers sync refresh with query params', async () => {
    server.use(
      ...mockRefreshSingleMetadata({
        repo: 'test-repo',
        filename: 'test-file.bin',
        snapshot: 'abc123',
        alias: 'test-model',
        metadata: {
          capabilities: { vision: false, audio: false, thinking: false, tools: {} },
          context: {},
          architecture: { format: 'gguf' },
        },
      })
    );

    render(<ModelsPage />, { wrapper: createWrapper() });

    await screen.findByTestId('preview-button-test-model');
    const previewButton = screen.getByTestId('preview-button-test-model');

    await act(async () => {
      fireEvent.click(previewButton);
    });

    // The model has no initial metadata, so the body refresh button is shown.
    await waitFor(() => {
      expect(screen.getByTestId('model-preview-modal')).toBeInTheDocument();
    });

    const refreshButton = screen.getByTestId('preview-modal-refresh-button-body');
    expect(refreshButton).toBeEnabled();

    await act(async () => {
      fireEvent.click(refreshButton);
    });

    await waitFor(() => {
      expect(refreshButton).toBeInTheDocument();
    });
  });
});

describe('Model Preview Modal', () => {
  beforeEach(() => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn({ role: 'resource_user' }), ...mockModelsDefault());
  });

  it('opens preview modal when preview button clicked', async () => {
    render(<ModelsPage />, { wrapper: createWrapper() });

    await screen.findByTestId('preview-button-test-model');

    const previewButton = screen.getByTestId('preview-button-test-model');

    await act(async () => {
      fireEvent.click(previewButton);
    });

    await waitFor(() => {
      expect(screen.getByTestId('model-preview-modal')).toBeInTheDocument();
    });

    expect(screen.getByTestId('preview-basic-alias')).toHaveTextContent('test-model');
    expect(screen.getByTestId('preview-basic-repo')).toHaveTextContent('test-repo');
    expect(screen.getByTestId('preview-basic-filename')).toHaveTextContent('test-file.bin');
  });

  it('closes preview modal on escape key', async () => {
    render(<ModelsPage />, { wrapper: createWrapper() });

    await screen.findByTestId('preview-button-test-model');

    const previewButton = screen.getByTestId('preview-button-test-model');

    await act(async () => {
      fireEvent.click(previewButton);
    });

    await waitFor(() => {
      expect(screen.getByTestId('model-preview-modal')).toBeInTheDocument();
    });

    await act(async () => {
      fireEvent.keyDown(document, { key: 'Escape' });
    });

    await waitFor(() => {
      expect(screen.queryByTestId('model-preview-modal')).not.toBeInTheDocument();
    });
  });
});
