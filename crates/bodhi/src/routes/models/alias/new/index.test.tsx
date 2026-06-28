import CreateAliasPage from '@/routes/models/alias/new/index';
import { ENDPOINT_APP_INFO } from '@/hooks/info';
import { ENDPOINT_MODEL_FILES, ENDPOINT_MODELS } from '@/hooks/models';
import { ENDPOINT_USER_INFO } from '@/hooks/users';
import { showSuccessParams } from '@/lib/utils.test';
import { createMockLoggedInUser, createMockLoggedOutUser } from '@/test-utils/mock-user';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { server } from '@/test-utils/msw-v2/setup';
import { mockAppInfo, mockAppInfoReady, mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { mockModels, mockCreateModel } from '@/test-utils/msw-v2/handlers/models';
import { mockModelFiles, mockModelPullDownloadsEmpty } from '@/test-utils/msw-v2/handlers/modelfiles';
import {
  mockDiscoverModelDetail,
  mockDiscoverModelsError,
  mockSearchRepos,
} from '@/test-utils/msw-v2/handlers/reference-models';
import { createDetailModel } from '@/test-fixtures/discover-models';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/hooks/useMediaQuery', () => ({
  useMediaQuery: (query: string) => {
    return true;
  },
}));

// Mock required HTMLElement methods and styles for Radix UI and Vaul components
Object.assign(window.HTMLElement.prototype, {
  scrollIntoView: vi.fn(),
  releasePointerCapture: vi.fn(),
  hasPointerCapture: vi.fn(),
  setPointerCapture: vi.fn(),
  getBoundingClientRect: vi.fn().mockReturnValue({
    x: 0,
    y: 0,
    width: 0,
    height: 0,
    top: 0,
    right: 0,
    bottom: 0,
    left: 0,
  }),
});

const mockToast = vi.fn();

const navigateMock = vi.fn();
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: any) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
    useNavigate: () => navigateMock,
    useSearch: () => ({ alias: 'test-alias' }),
  };
});

vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

vi.mock('@/components/ui/toaster', () => ({
  Toaster: () => null,
}));

beforeAll(() => {
  Element.prototype.hasPointerCapture = vi.fn(() => false);
  Element.prototype.setPointerCapture = vi.fn();
  Element.prototype.releasePointerCapture = vi.fn();
  server.listen();
});
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  vi.clearAllMocks();
});
beforeEach(() => {
  navigateMock.mockClear();
});

/**
 * Drive the repo combobox: open it, type the repo, then commit. When `repo` matches a `/api/v1/repos`
 * suggestion it picks that option; otherwise it commits via the free-text "Use this" row.
 */
async function selectRepo(user: ReturnType<typeof userEvent.setup>, repo: string) {
  await user.click(screen.getByTestId('repo-input'));
  const search = await screen.findByPlaceholderText(/search huggingface repos/i);
  await user.type(search, repo);
  const option = await screen.findByRole('option', { name: repo });
  await user.click(option);
  await waitFor(() => expect(screen.queryByPlaceholderText(/search huggingface repos/i)).not.toBeInTheDocument());
}

describe('CreateAliasPage', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfoReady(),
      ...mockSearchRepos({ ids: ['Qwen/Qwen3-Coder-32B-GGUF', 'Qwen/Qwen2.5-7B-Instruct-GGUF'] }),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockModels({
        data: [
          {
            source: 'user',
            alias: 'model1',
            repo: 'owner1/repo1',
            filename: 'file1.gguf',
            snapshot: 'main',
            request_params: {},
            context_params: [],
          },
          {
            source: 'user',
            alias: 'model2',
            repo: 'owner1/repo1',
            filename: 'file2.gguf',
            snapshot: 'main',
            request_params: {},
            context_params: [],
          },
          {
            source: 'user',
            alias: 'model3',
            repo: 'owner2/repo2',
            filename: 'file3.gguf',
            snapshot: 'main',
            request_params: {},
            context_params: [],
          },
        ],
        total: 3,
        page: 1,
        page_size: 30,
      }),
      ...mockModelFiles({
        data: [
          { repo: 'owner1/repo1', filename: 'file1.gguf', snapshot: 'main', size: 1000000, model_params: {} },
          { repo: 'owner1/repo1', filename: 'file2.gguf', snapshot: 'main', size: 1000000, model_params: {} },
        ],
        total: 2,
        page: 1,
        page_size: 30,
      }),
      ...mockModelPullDownloadsEmpty(),
      // Reference catalog detail for the repo the tests type into the form (quant table source).
      ...mockDiscoverModelDetail({ model: createDetailModel({ namespace: 'Qwen', repo: 'Qwen3-Coder-32B-GGUF' }) }),
      ...mockCreateModel({
        alias: 'test-alias',
        repo: 'test-repo',
        filename: 'test-file.bin',
        snapshot: 'main',
        request_params: {},
        context_params: [],
        model_params: {},
        source: 'user',
      })
    );
  });

  it('renders the page with all form elements', async () => {
    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByTestId('alias-input')).toBeInTheDocument();
    expect(screen.getByTestId('repo-input')).toBeInTheDocument();
    expect(screen.getByTestId('snapshot-input')).toBeInTheDocument();
    expect(screen.getByTestId('context-params')).toBeInTheDocument();

    expect(screen.getByRole('button', { name: /create alias/i })).toBeInTheDocument();
  });

  it('fetches quants for the typed repo and submits the selected quant', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });

    await user.type(screen.getByTestId('alias-input'), 'test-alias');
    await selectRepo(user, 'Qwen/Qwen3-Coder-32B-GGUF');

    // The quant table loads from the reference catalog; pick a quant to set the filename.
    const quantRow = await screen.findByTestId('quant-row-Q4_K_M');
    await user.click(quantRow);
    await waitFor(() => expect(quantRow).toHaveAttribute('data-test-state', 'selected'));

    await user.click(screen.getByRole('button', { name: /create alias/i }));

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Alias test-alias successfully created'));
    });
  });

  it('shows per-quant download status and a download-on-save note for remote quants', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });

    await selectRepo(user, 'Qwen/Qwen3-Coder-32B-GGUF');

    // None of the catalog quants match a downloaded file → all "Not downloaded".
    const status = await screen.findByTestId('quant-status-Q4_K_M');
    expect(status).toHaveTextContent(/not downloaded/i);

    await user.click(screen.getByTestId('quant-row-Q4_K_M'));
    await waitFor(() => {
      expect(screen.getByTestId('quant-download-note')).toHaveTextContent(/download automatically after save/i);
    });
  });

  it('falls back to a manual filename input when the repo has no catalog quants', async () => {
    const user = userEvent.setup();
    server.use(...mockDiscoverModelsError({ status: 404, error: 'not_found' }, {}));
    // The detail endpoint also 404s for an unknown repo.
    server.use(...mockDiscoverModelDetail({ model: createDetailModel({ quants: [] }) }));

    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });

    await selectRepo(user, 'private/unlisted-GGUF');

    const filenameInput = await screen.findByTestId('filename-input');
    await user.type(filenameInput, 'custom-model.gguf');
    expect(filenameInput).toHaveValue('custom-model.gguf');
  });

  it('handles context parameters input correctly', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });

    const contextParamsTextarea = screen.getByTestId('context-params');
    expect(contextParamsTextarea).toBeInTheDocument();

    await user.type(contextParamsTextarea, '--ctx-size 2048\n--parallel 4\n--threads 8');
    expect(contextParamsTextarea).toHaveValue('--ctx-size 2048\n--parallel 4\n--threads 8');

    // Click-to-add a flag from the catalog appends it to the textarea.
    await user.click(screen.getByTestId('context-flag-add---flash-attn'));
    await waitFor(() => expect((contextParamsTextarea as HTMLTextAreaElement).value).toContain('--flash-attn'));
    // Re-adding the same flag is blocked (the catalog entry is now disabled).
    expect(screen.getByTestId('context-flag-add---flash-attn')).toBeDisabled();

    // The mockCreateModel response fixes the alias name, so the success toast reads 'test-alias'.
    await user.type(screen.getByTestId('alias-input'), 'test-alias');
    await selectRepo(user, 'Qwen/Qwen3-Coder-32B-GGUF');
    await user.click(await screen.findByTestId('quant-row-Q4_K_M'));

    await user.click(screen.getByRole('button', { name: /create alias/i }));

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Alias test-alias successfully created'));
    });
  });

  it('edits request params via the textarea + catalog and a system prompt', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });

    // System prompt has its own textarea.
    const systemPrompt = screen.getByTestId('system-prompt');
    await user.type(systemPrompt, 'You are concise.');
    expect(systemPrompt).toHaveValue('You are concise.');

    // Request params are key=value lines; the catalog appends an entry.
    const requestParams = screen.getByTestId('request-params') as HTMLTextAreaElement;
    await user.type(requestParams, 'temperature=0.7');
    await user.click(screen.getByTestId('request-param-add-top_p'));
    await waitFor(() => expect(requestParams.value).toContain('top_p='));
    // Re-adding a present param is blocked.
    expect(screen.getByTestId('request-param-add-top_p')).toBeDisabled();
  });

  it('submits with a system prompt + request params set', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });

    await user.type(screen.getByTestId('alias-input'), 'test-alias');
    await selectRepo(user, 'Qwen/Qwen3-Coder-32B-GGUF');
    await user.click(await screen.findByTestId('quant-row-Q4_K_M'));
    await user.type(screen.getByTestId('system-prompt'), 'Be terse.');
    await user.type(screen.getByTestId('request-params'), 'temperature=0.5');

    await user.click(screen.getByRole('button', { name: /create alias/i }));

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Alias test-alias successfully created'));
    });
  });
});

describe('CreateAliasPage access control', () => {
  it('should redirect to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfoSetup(), ...mockUserLoggedIn({ role: 'resource_user' }));
    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });
    expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/' });
  });

  it('should redirect to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedOut());
    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });
    expect(navigateMock).toHaveBeenCalledWith({ to: '/login/' });
  });
});
