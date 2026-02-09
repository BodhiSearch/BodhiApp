import EditAliasPage from '@/app/ui/models/edit/page';
import { ENDPOINT_APP_INFO } from '@/hooks/useInfo';
import { ENDPOINT_MODEL_FILES, ENDPOINT_MODELS } from '@/hooks/useModels';
import { ENDPOINT_USER_INFO } from '@/hooks/useUsers';
import { showSuccessParams } from '@/lib/utils.test';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, within } from '@testing-library/react';
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
import { mockModelFiles } from '@/test-utils/msw-v2/handlers/modelfiles';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

// Mock useMediaQuery hook
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
const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: (key: string) => {
      if (key === 'id') return 'test-uuid-1';
      return null;
    },
  }),
}));

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
  pushMock.mockClear();
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

    expect(screen.getByLabelText(/alias/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/repo/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/filename/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/context parameters/i)).toBeInTheDocument();

    expect(screen.getByRole('combobox', { name: /repo/i })).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: /filename/i })).toBeInTheDocument();
    expect(screen.getByRole('textbox', { name: /context parameters/i })).toBeInTheDocument();

    expect(screen.getByRole('button', { name: /update model alias/i })).toBeInTheDocument();

    // Check pre-filled values
    expect(screen.getByLabelText(/alias/i)).toHaveValue('test-alias');
    expect(screen.getByRole('combobox', { name: /repo/i })).toHaveTextContent('owner1/repo1');
    expect(screen.getByRole('combobox', { name: /filename/i })).toHaveTextContent('file1.gguf');

    // Check context parameters are pre-filled
    const contextParamsTextarea = screen.getByRole('textbox', { name: /context parameters/i });
    expect(contextParamsTextarea).toHaveValue('--ctx-size 2048\n--parallel 4');

    // Request parameters should be expanded since there are existing values
    expect(screen.getByText('Request Parameters')).toBeInTheDocument();
  });

  it('submits the form with updated data', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<EditAliasPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByLabelText(/alias/i)).toBeInTheDocument();

    // Open repo combobox
    await user.click(screen.getByRole('combobox', { name: /repo/i }));
    const repoPopover = screen.getByRole('dialog');
    const repoItems = within(repoPopover).getAllByRole('option');
    const owner2Repo2Option = repoItems.find((item) => item.textContent?.includes('owner2/repo2'));
    if (!owner2Repo2Option) {
      throw new Error('Could not find owner2/repo2 option');
    }
    await user.click(owner2Repo2Option);

    // Open filename combobox
    await user.click(screen.getByRole('combobox', { name: /filename/i }));
    const filenamePopover = screen.getByRole('dialog');
    const filenameItems = within(filenamePopover).getAllByRole('option');
    const file3Option = filenameItems.find((item) => item.textContent?.includes('file3.gguf'));
    if (!file3Option) {
      throw new Error('Could not find file3.gguf option');
    }
    await user.click(file3Option);

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

  it('expands request parameters when there are existing values', async () => {
    await act(async () => {
      render(<EditAliasPage />, { wrapper: createWrapper() });
    });

    // Request parameters should be expanded since mockModelData has request_params
    await waitFor(() => {
      expect(screen.getByLabelText(/temperature/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/max_tokens/i)).toBeInTheDocument();
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
    expect(pushMock).toHaveBeenCalledWith('/ui/setup');
  });

  it('should redirect to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfo({ status: 'ready' }), ...mockUserLoggedOut());
    await act(async () => {
      render(<EditAliasPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/login');
  });
});
