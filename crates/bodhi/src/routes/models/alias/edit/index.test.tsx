import EditAliasPage from '@/routes/models/alias/edit/index';
import { ENDPOINT_APP_INFO } from '@/hooks/info';
import { ENDPOINT_MODEL_FILES, ENDPOINT_MODELS } from '@/hooks/models';
import { ENDPOINT_USER_INFO } from '@/hooks/users';
import { showSuccessParams } from '@/lib/utils.test';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { mockAppInfo, mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import {
  mockModels,
  mockGetModel,
  mockGetModelInternalError,
  mockUpdateModel,
} from '@/test-utils/msw-v2/handlers/models';
import { mockModelFiles, mockModelPullDownloadsEmpty } from '@/test-utils/msw-v2/handlers/modelfiles';
import { mockDiscoverModelDetail } from '@/test-utils/msw-v2/handlers/reference-models';
import { createDetailModel, createQuant } from '@/test-fixtures/discover-models';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/hooks/use-media-query', () => ({
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
    useSearch: () => ({ id: 'test-uuid-1' }),
  };
});

vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

vi.mock('@/components/ui/toaster', () => ({
  Toaster: () => null,
}));

setupMswV2();
afterEach(() => {
  vi.clearAllMocks();
});
beforeEach(() => {
  navigateMock.mockClear();
});

describe('EditAliasPage', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockGetModel('test-uuid-1', {
        alias: 'test-alias',
        repo: 'owner1/repo1',
        filename: 'file1.gguf',
        snapshot: 'main',
        source: 'user',
        model_params: {},
        request_params: {
          temperature: 0.7,
          max_tokens: 1000,
        },
        context_params: ['--ctx-size 2048', '--parallel 4'],
      }),
      ...mockModels({
        data: [
          {
            repo: 'owner1/repo1',
            filename: 'file1.gguf',
            alias: 'owner1/repo1:file1.gguf',
            snapshot: 'main',
            source: 'model',
          },
          {
            repo: 'owner1/repo1',
            filename: 'file2.gguf',
            alias: 'owner1/repo1:file2.gguf',
            snapshot: 'main',
            source: 'model',
          },
          {
            repo: 'owner2/repo2',
            filename: 'file3.gguf',
            alias: 'owner2/repo2:file3.gguf',
            snapshot: 'main',
            source: 'model',
          },
        ],
      }),
      ...mockModelFiles({
        data: [
          { repo: 'owner1/repo1', filename: 'file1.gguf', snapshot: 'main', size: 1000000, model_params: {} },
          { repo: 'owner1/repo1', filename: 'file2.gguf', snapshot: 'main', size: 1000000, model_params: {} },
          { repo: 'owner2/repo2', filename: 'file3.gguf', snapshot: 'main', size: 1000000, model_params: {} },
        ],
      }),
      ...mockModelPullDownloadsEmpty(),
      // Reference catalog detail for the edited repo: its quants preselect the current filename.
      ...mockDiscoverModelDetail({
        model: createDetailModel({
          namespace: 'owner1',
          repo: 'repo1',
          quants: [
            createQuant({ name: 'file1', filename: 'file1.gguf', size: 1000000, recommended: false }),
            createQuant({ name: 'file2', filename: 'file2.gguf', size: 1000000, recommended: false }),
          ],
        }),
      }),
      ...mockUpdateModel('test-uuid-1', {
        alias: 'test-alias',
        repo: 'owner1/repo1',
        filename: 'file1.gguf',
        snapshot: 'main',
        source: 'user',
        model_params: {},
        request_params: {},
        context_params: [],
      })
    );
  });

  it('renders the page with all form elements pre-filled with model data', async () => {
    await act(async () => {
      render(<EditAliasPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByTestId('alias-input')).toBeInTheDocument();
    expect(screen.getByTestId('repo-input')).toBeInTheDocument();
    expect(screen.getByLabelText(/context parameters/i)).toBeInTheDocument();

    expect(screen.getByRole('button', { name: /update model alias/i })).toBeInTheDocument();

    // Check pre-filled values: alias locked, repo populated, current quant preselected.
    expect(screen.getByTestId('alias-input')).toHaveValue('test-alias');
    expect(screen.getByTestId('alias-input')).toBeDisabled();
    expect(screen.getByTestId('repo-input')).toHaveValue('owner1/repo1');

    await waitFor(() => {
      expect(screen.getByTestId('quant-row-file1')).toHaveAttribute('data-test-state', 'selected');
    });

    // Check context parameters are pre-filled
    const contextParamsTextarea = screen.getByRole('textbox', { name: /context parameters/i });
    expect(contextParamsTextarea).toHaveValue('--ctx-size 2048\n--parallel 4');

    // Request parameters should be expanded since there are existing values
    expect(screen.getByText('Request Parameters')).toBeInTheDocument();
  });

  it('submits the form after changing the selected quant', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<EditAliasPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByTestId('alias-input')).toBeInTheDocument();

    // Switch from the current quant to a different one in the same repo.
    const file2Row = await screen.findByTestId('quant-row-file2');
    await user.click(file2Row);
    await waitFor(() => expect(file2Row).toHaveAttribute('data-test-state', 'selected'));

    await user.click(screen.getByRole('button', { name: /update model alias/i }));

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Alias test-alias successfully updated'));
    });
  });

  it('updates context parameters correctly', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<EditAliasPage />, { wrapper: createWrapper() });
    });

    const contextParamsTextarea = screen.getByRole('textbox', { name: /context parameters/i });

    // Clear existing content and add new context parameters
    await user.clear(contextParamsTextarea);
    await user.type(contextParamsTextarea, '--ctx-size 4096\n--threads 16\n--batch-size 512');

    expect(contextParamsTextarea).toHaveValue('--ctx-size 4096\n--threads 16\n--batch-size 512');

    await user.click(screen.getByRole('button', { name: /update model alias/i }));

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Alias test-alias successfully updated'));
    });
  });

  it('handles empty context parameters correctly', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<EditAliasPage />, { wrapper: createWrapper() });
    });

    const contextParamsTextarea = screen.getByRole('textbox', { name: /context parameters/i });

    // Clear all context parameters
    await user.clear(contextParamsTextarea);
    expect(contextParamsTextarea).toHaveValue('');

    await user.click(screen.getByRole('button', { name: /update model alias/i }));

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Alias test-alias successfully updated'));
    });
  });

  it('prefills the request-params textarea from existing values', async () => {
    await act(async () => {
      render(<EditAliasPage />, { wrapper: createWrapper() });
    });

    // request_params { temperature: 0.7, max_tokens: 1000 } render as key=value lines.
    await waitFor(() => {
      const rp = screen.getByTestId('request-params') as HTMLTextAreaElement;
      expect(rp.value).toContain('temperature=0.7');
      expect(rp.value).toContain('max_tokens=1000');
    });
  });

  it('displays error message when model data fails to load', async () => {
    server.use(...mockGetModelInternalError('test-uuid-1'));

    await act(async () => {
      render(<EditAliasPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByText('Error loading model data')).toBeInTheDocument();
  });
});

describe('EditAliasPage access control', () => {
  it('should redirect to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfoSetup(), ...mockUserLoggedIn({ role: 'resource_user' }));
    await act(async () => {
      render(<EditAliasPage />, { wrapper: createWrapper() });
    });
    expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/' });
  });

  it('should redirect to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfo({ status: 'ready' }), ...mockUserLoggedOut());
    await act(async () => {
      render(<EditAliasPage />, { wrapper: createWrapper() });
    });
    expect(navigateMock).toHaveBeenCalledWith({ to: '/login/' });
  });
});
