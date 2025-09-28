import CreateAliasPage from '@/app/ui/models/new/page';
import { ENDPOINT_APP_INFO, ENDPOINT_MODEL_FILES, ENDPOINT_MODELS, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { showSuccessParams } from '@/lib/utils.test';
import { createMockLoggedInUser, createMockLoggedOutUser } from '@/test-utils/mock-user';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { server } from '@/test-utils/msw-v2/setup';
import { mockAppInfo, mockAppInfoReady, mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { mockModels, mockCreateModel } from '@/test-utils/msw-v2/handlers/models';
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
    get: vi.fn().mockReturnValue('test-alias'),
  }),
}));

vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

vi.mock('@/components/ui/toaster', () => ({
  Toaster: () => null,
}));

const mockModelsResponse = {
  data: [
    { repo: 'owner1/repo1', filename: 'file1.gguf', snapshot: 'main' },
    { repo: 'owner1/repo1', filename: 'file2.gguf', snapshot: 'main' },
    { repo: 'owner2/repo2', filename: 'file3.gguf', snapshot: 'main' },
  ],
};

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
  pushMock.mockClear();
});

// Helper function to select an option from a ComboBoxResponsive component
const selectFromComboBox = async (
  user: ReturnType<typeof userEvent.setup>,
  comboboxName: RegExp,
  optionText: string
) => {
  // Click to open the combobox
  await user.click(screen.getByRole('combobox', { name: comboboxName }));

  // Find the dialog (mobile view) and select the option
  const dialog = screen.getByRole('dialog');
  const options = within(dialog).getAllByRole('option');
  const targetOption = options.find((option) => option.textContent?.includes(optionText));

  if (!targetOption) {
    throw new Error(`Option "${optionText}" not found in combobox`);
  }

  await user.click(targetOption);
};

describe('CreateAliasPage', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfoReady(),
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

    expect(screen.getByLabelText(/alias/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/repo/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/filename/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/snapshot/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/context parameters/i)).toBeInTheDocument();

    expect(screen.getByRole('combobox', { name: /repo/i })).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: /filename/i })).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: /snapshot/i })).toBeInTheDocument();
    expect(screen.getByRole('textbox', { name: /context parameters/i })).toBeInTheDocument();

    expect(screen.getByRole('button', { name: /create model alias/i })).toBeInTheDocument();
  });

  it('submits the form with correct data and auto-selects first snapshot', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByLabelText(/alias/i)).toBeInTheDocument();

    await user.type(screen.getByLabelText(/alias/i), 'test-alias');

    // Select from comboboxes using helper function
    await selectFromComboBox(user, /repo/i, 'owner1/repo1');
    await selectFromComboBox(user, /filename/i, 'file1.gguf');

    // Verify that snapshot is auto-selected to 'main' (first option)
    await waitFor(() => {
      const snapshotCombobox = screen.getByRole('combobox', { name: /snapshot/i });
      expect(snapshotCombobox).toHaveTextContent('main');
    });

    await user.click(screen.getByRole('button', { name: /create model alias/i }));

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Alias test-alias successfully created'));
    });
  });

  it('handles context parameters input correctly', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });

    const contextParamsTextarea = screen.getByRole('textbox', { name: /context parameters/i });
    expect(contextParamsTextarea).toBeInTheDocument();

    // Test context parameters input
    await user.type(contextParamsTextarea, '--ctx-size 2048\n--parallel 4\n--threads 8');
    expect(contextParamsTextarea).toHaveValue('--ctx-size 2048\n--parallel 4\n--threads 8');

    // Fill required fields
    await user.type(screen.getByLabelText(/alias/i), 'test-context-alias');

    // Select from comboboxes using helper function (snapshot will be auto-selected)
    await selectFromComboBox(user, /repo/i, 'owner1/repo1');
    await selectFromComboBox(user, /filename/i, 'file1.gguf');

    await user.click(screen.getByRole('button', { name: /create model alias/i }));

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Alias test-alias successfully created'));
    });
  });

  it('expands and collapses request parameters section', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });

    const requestParamsHeader = screen.getByText('Request Parameters');
    expect(requestParamsHeader).toBeInTheDocument();

    // Wait for initial render to complete
    await waitFor(() => {
      // Initially collapsed, so request param fields should not be visible
      // Check if the content is hidden via CSS (max-height: 0)
      const cardContent = requestParamsHeader.closest('.rounded-lg')?.querySelector('.overflow-hidden');
      expect(cardContent).toHaveClass('max-h-0');
    });

    // Click to expand
    await user.click(requestParamsHeader);

    // Now request param fields should be visible
    await waitFor(() => {
      const cardContent = requestParamsHeader.closest('.rounded-lg')?.querySelector('.overflow-hidden');
      expect(cardContent).toHaveClass('max-h-[1000px]');
      expect(screen.getByLabelText(/temperature/i)).toBeInTheDocument();
    });

    // Click to collapse
    await user.click(requestParamsHeader);

    // Fields should be hidden again
    await waitFor(() => {
      const cardContent = requestParamsHeader.closest('.rounded-lg')?.querySelector('.overflow-hidden');
      expect(cardContent).toHaveClass('max-h-0');
    });
  });
});

describe('CreateAliasPage access control', () => {
  it('should redirect to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfoSetup(), ...mockUserLoggedIn({ role: 'resource_user' }));
    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/setup');
  });

  it('should redirect to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedOut());
    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/login');
  });
});
