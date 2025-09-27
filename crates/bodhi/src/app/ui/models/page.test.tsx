import ModelsPage from '@/app/ui/models/page';
import { ENDPOINT_APP_INFO, ENDPOINT_MODELS, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { createMockLoggedInUser, createMockLoggedOutUser } from '@/test-utils/mock-user';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, fireEvent } from '@testing-library/react';
import { setupServer } from 'msw2/node';
import { http, HttpResponse } from '@/test-utils/msw-v2/setup';
import { mockAppInfoReady, mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import {
  mockModelsDefault,
  mockModelsWithApiModel,
  mockModelsWithSourceModel,
  mockModelsError,
} from '@/test-utils/msw-v2/handlers/models';
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
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn(), ...mockModelsDefault());
  });

  it('renders responsive layouts correctly', async () => {
    // Test mobile view (< sm)
    mockMatchMedia(false);

    const { unmount } = render(<ModelsPage />, { wrapper: createWrapper() });

    // Wait for data to load
    await screen.findByTestId('combined-cell-test-model');

    // Mobile view should show combined cell
    expect(screen.getByTestId('combined-cell-test-model')).toBeVisible();

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
    await screen.findByTestId('name-source-cell-test-model');

    // Tablet view should show combined name+source and repo+filename columns
    expect(screen.getByTestId('name-source-cell-test-model')).toBeVisible();
    expect(screen.getByTestId('repo-filename-cell-test-model')).toBeVisible();

    unmount();

    // Test desktop view (>= lg)
    mockMatchMedia(true);

    render(<ModelsPage />, { wrapper: createWrapper() });
    await screen.findByTestId('alias-cell-test-model');

    // Desktop view should show separate columns
    expect(screen.getByTestId('alias-cell-test-model')).toBeVisible();
    expect(screen.getByTestId('repo-cell-test-model')).toBeVisible();
    expect(screen.getByTestId('filename-cell-test-model')).toBeVisible();
    expect(screen.getByTestId('source-cell-test-model')).toBeVisible();
  });

  it('handles API error', async () => {
    server.use(...mockModelsError({ status: 500, message: 'Internal Server Error' }));
    await act(async () => {
      render(<ModelsPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByText('Internal Server Error')).toBeInTheDocument();
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

      expect(pushMock).toHaveBeenCalledWith('/ui/models/new?repo=test-repo&filename=test-file.bin&snapshot=abc123');
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

      expect(pushMock).toHaveBeenCalledWith('/ui/models/edit?alias=test-model');
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

      expect(pushMock).toHaveBeenCalledWith('/ui/chat?model=test-model');
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

      expect(pushMock).toHaveBeenCalledWith('/ui/api-models/new');
    });
  });

  describe('API model display and actions', () => {
    it('displays API models with correct information', async () => {
      server.use(...mockModelsWithApiModel());

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      // Check that API model is displayed with its ID (using getAllByText to handle multiple responsive layouts)
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

      expect(pushMock).toHaveBeenCalledWith('/ui/api-models/edit?id=test-api-model');
    });

    it('shows chat button for API models with model identifier', async () => {
      server.use(...mockModelsWithApiModel());

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      // Find the API model row and then look for the chat button within it
      const apiModelRow = screen.getByTestId('actions-cell-test-api-model');
      const chatButton = apiModelRow.querySelector('[data-testid="model-chat-button-gpt-4"]');
      expect(chatButton).toBeInTheDocument();
      expect(chatButton).toHaveAttribute('title', 'Chat with gpt-4');

      await act(async () => {
        fireEvent.click(chatButton!);
      });

      expect(pushMock).toHaveBeenCalledWith('/ui/chat?model=gpt-4');
    });

    it('does not show HuggingFace button for API models', async () => {
      server.use(...mockModelsWithApiModel());

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      // Should not have HuggingFace button for API models
      const hfButtons = screen.queryAllByTitle('Open in HuggingFace');
      expect(hfButtons).toHaveLength(0);
    });

    it('navigates to edit page when edit button is clicked for API model', async () => {
      server.use(...mockModelsWithApiModel());

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      // Wait for data to load and verify API model is displayed (use getAllByText for responsive layouts)
      await screen.findAllByText('test-api-model');
      expect(screen.getAllByText('openai')[0]).toBeInTheDocument();
      expect(screen.getAllByText('https://api.openai.com/v1')[0]).toBeInTheDocument();

      // Models might be displayed differently in responsive layouts
      const modelsText = screen.getAllByText((content, element) => {
        return content.includes('gpt-4') && content.includes('gpt-3.5-turbo');
      });
      expect(modelsText.length).toBeGreaterThan(0);

      // Find and click the edit button
      const editButton = screen.getAllByTitle('Edit API model test-api-model')[0];
      expect(editButton).toBeInTheDocument();

      await act(async () => {
        editButton.click();
      });

      // Verify navigation to edit page with correct ID
      expect(pushMock).toHaveBeenCalledWith('/ui/api-models/edit?id=test-api-model');
    });

    it('navigates to chat page when clicking on API model for chat', async () => {
      server.use(...mockModelsWithApiModel());

      await act(async () => {
        render(<ModelsPage />, { wrapper: createWrapper() });
      });

      // Wait for data to load (use findAllByText for responsive layouts)
      await screen.findAllByText('test-api-model');

      // Find the API model row and then look for the chat button within it
      const apiModelRow = screen.getByTestId('actions-cell-test-api-model');
      const chatButton = apiModelRow.querySelector('[data-testid="chat-button-test-api-model"]');
      expect(chatButton).not.toBeInTheDocument();
      const modelChatButton = apiModelRow.querySelector('[data-testid="model-chat-button-gpt-4"]');
      expect(modelChatButton).toBeInTheDocument();
      expect(modelChatButton).toHaveAttribute('title', 'Chat with gpt-4');

      await act(async () => {
        fireEvent.click(modelChatButton!);
      });

      // Verify navigation to chat page with the API model ID
      expect(pushMock).toHaveBeenCalledWith('/ui/chat?model=gpt-4');
    });
  });

  it('displays error message when API call fails', async () => {
    server.use(...mockModelsError({ status: 500, message: 'Internal Server Error' }));

    await act(async () => {
      render(<ModelsPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByRole('alert')).toHaveTextContent('Internal Server Error');
  });
});

describe('ModelsPage access control', () => {
  it('should redirect to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfo({ status: 'setup' }), ...mockUserLoggedIn());

    await act(async () => {
      render(<ModelsPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/setup');
  });

  it('should redirect to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedOut());
    await act(async () => {
      render(<ModelsPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/login');
  });
});
