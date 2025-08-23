import CreateAliasPage from '@/app/ui/models/new/page';
import { ENDPOINT_APP_INFO, ENDPOINT_MODEL_FILES, ENDPOINT_MODELS, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { showSuccessParams } from '@/lib/utils.test';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
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
    { repo: 'owner1/repo1', filename: 'file1.gguf' },
    { repo: 'owner1/repo1', filename: 'file2.gguf' },
    { repo: 'owner2/repo2', filename: 'file3.gguf' },
  ],
};

const server = setupServer();

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

describe('CreateAliasPage', () => {
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
      }),

      rest.get(`*${ENDPOINT_MODEL_FILES}`, (_, res, ctx) => {
        return res(
          ctx.json({
            data: [
              { repo: 'owner1/repo1', filename: 'file1.gguf', snapshot: 'main' },
              { repo: 'owner1/repo1', filename: 'file2.gguf', snapshot: 'main' },
            ],
          })
        );
      }),
      rest.post(`*${ENDPOINT_MODELS}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ alias: 'test-alias' }));
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
    expect(screen.getByLabelText(/context parameters/i)).toBeInTheDocument();

    expect(screen.getByRole('combobox', { name: /repo/i })).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: /filename/i })).toBeInTheDocument();
    expect(screen.getByRole('textbox', { name: /context parameters/i })).toBeInTheDocument();

    expect(screen.getByRole('button', { name: /create model alias/i })).toBeInTheDocument();
  });

  it('submits the form with correct data', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByLabelText(/alias/i)).toBeInTheDocument();

    await user.type(screen.getByLabelText(/alias/i), 'test-alias');

    // Open combobox
    await user.click(screen.getByRole('combobox', { name: /repo/i }));

    // Wait for and find the dialog
    const dialog = screen.getByRole('dialog');

    // Find and click option within dialog
    const options = within(dialog).getAllByRole('option');
    await user.click(options[0]); // owner1/repo1 should be the first option

    await user.type(screen.getByRole('combobox', { name: /filename/i }), 'file1.gguf');

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

    // Open repo combobox
    await user.click(screen.getByRole('combobox', { name: /repo/i }));
    const dialog = screen.getByRole('dialog');
    const options = within(dialog).getAllByRole('option');
    await user.click(options[0]);

    await user.type(screen.getByRole('combobox', { name: /filename/i }), 'file1.gguf');

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
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/setup');
  });

  it('should redirect to /ui/login if user is not logged in', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ logged_in: false }));
      })
    );
    await act(async () => {
      render(<CreateAliasPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/login');
  });
});
