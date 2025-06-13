import EditAliasPage from '@/app/ui/models/edit/page';
import { ENDPOINT_APP_INFO, ENDPOINT_CHAT_TEMPLATES, ENDPOINT_MODEL_FILES, ENDPOINT_MODELS, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { showSuccessParams } from '@/lib/utils.test';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
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

// Mock useMediaQuery hook
vi.mock("@/hooks/use-media-query", () => ({
  useMediaQuery: (query: string) => {
    return true;
  }
}))

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

const mockModelData = {
  alias: 'test-alias',
  repo: 'owner1/repo1',
  filename: 'file1.gguf',
  chat_template: 'llama2',
};

const mockModelsResponse = {
  data: [
    { repo: 'owner1/repo1', filename: 'file1.gguf' },
    { repo: 'owner1/repo1', filename: 'file2.gguf' },
    { repo: 'owner2/repo2', filename: 'file3.gguf' },
  ],
};

const mockChatTemplatesResponse = ['llama2', 'llama3'];

const server = setupServer();

beforeAll(() => {
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

describe('EditAliasPage', () => {
  beforeEach(() => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
      }),
      rest.get(`*${ENDPOINT_MODELS}/:alias`, (_, res, ctx) => {
        return res(ctx.json(mockModelData));
      }),
      rest.get(`*${ENDPOINT_MODELS}`, (_, res, ctx) => {
        return res(ctx.json(mockModelsResponse));
      }),
      rest.get(`*${ENDPOINT_CHAT_TEMPLATES}`, (_, res, ctx) => {
        return res(ctx.json(mockChatTemplatesResponse));
      }),
      rest.get(`*${ENDPOINT_MODEL_FILES}`, (_, res, ctx) => {
        return res(ctx.json({
          data: [
            { repo: 'owner1/repo1', filename: 'file1.gguf', snapshot: 'main' },
            { repo: 'owner1/repo1', filename: 'file2.gguf', snapshot: 'main' },
            { repo: 'owner2/repo2', filename: 'file3.gguf', snapshot: 'main' }
          ]
        }));
      }),
      rest.put(`*${ENDPOINT_MODELS}/test-alias`, (_, res, ctx) => {
        return res(
          ctx.status(200),
          ctx.json({ alias: 'test-alias' })
        );
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
    expect(screen.getByLabelText(/chat template/i)).toBeInTheDocument();

    expect(screen.getByRole('combobox', { name: /repo/i })).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: /filename/i })).toBeInTheDocument();
    expect(
      screen.getByRole('combobox', { name: /chat template/i })
    ).toBeInTheDocument();

    expect(
      screen.getByRole('button', { name: /update model alias/i })
    ).toBeInTheDocument();

    expect(screen.getByLabelText(/alias/i)).toHaveValue('test-alias');
    expect(screen.getByRole('combobox', { name: /repo/i })).toHaveTextContent('owner1/repo1');
    expect(screen.getByRole('combobox', { name: /filename/i })).toHaveTextContent('file1.gguf');
    expect(screen.getByRole('combobox', { name: /chat template/i })).toHaveTextContent('llama2');
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
    const owner2Repo2Option = repoItems.find(item =>
      item.textContent?.includes('owner2/repo2')
    );
    if (!owner2Repo2Option) {
      throw new Error('Could not find owner2/repo2 option');
    }
    await user.click(owner2Repo2Option);

    // Open filename combobox
    await user.click(screen.getByRole('combobox', { name: /filename/i }));
    const filenamePopover = screen.getByRole('dialog');
    const filenameItems = within(filenamePopover).getAllByRole('option');
    const file3Option = filenameItems.find(item =>
      item.textContent?.includes('file3.gguf')
    );
    if (!file3Option) {
      throw new Error('Could not find file3.gguf option');
    }
    await user.click(file3Option);

    // Open chat template combobox
    await user.click(screen.getByRole('combobox', { name: /chat template/i }));
    const chatTemplatePopover = screen.getByRole('dialog');
    const chatTemplateItems = within(chatTemplatePopover).getAllByRole('option');
    const llama3Option = chatTemplateItems.find(item =>
      item.textContent?.includes('llama3')
    );
    if (!llama3Option) {
      throw new Error('Could not find llama3 option');
    }
    await user.click(llama3Option);

    await user.click(
      screen.getByRole('button', { name: /update model alias/i })
    );

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Alias test-alias successfully updated'));
    });
  });

  it('displays error message when model data fails to load', async () => {
    server.use(
      rest.get(`*${ENDPOINT_MODELS}/:alias`, (req, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({ error: { message: 'Internal Server Error' } }));
      })
    );

    await act(async () => {
      render(<EditAliasPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByText('Error loading model data')).toBeInTheDocument();
  });
});

describe('EditAliasPage access control', () => {
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
      render(<EditAliasPage />, { wrapper: createWrapper() });
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
      render(<EditAliasPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/login');
  });
});
