import CreateAliasPage from '@/components/models/new/NewModelPage';
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
vi.mock('@/lib/navigation', () => ({
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

const mockChatTemplatesResponse = ['llama2', 'llama3'];

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
      rest.get(`*${ENDPOINT_CHAT_TEMPLATES}`, (_, res, ctx) => {
        return res(ctx.json(mockChatTemplatesResponse));
      }),
      rest.get(`*${ENDPOINT_MODEL_FILES}`, (_, res, ctx) => {
        return res(ctx.json({ data: [{ repo: 'owner1/repo1', filename: 'file1.gguf', snapshot: 'main' }, { repo: 'owner1/repo1', filename: 'file2.gguf', snapshot: 'main' }] }));
      }),
      rest.post(`*${ENDPOINT_MODELS}`, (_, res, ctx) => {
        return res(
          ctx.status(200),
          ctx.json({ alias: 'test-alias' })
        );
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
    expect(screen.getByLabelText(/chat template/i)).toBeInTheDocument();

    expect(screen.getByRole('combobox', { name: /repo/i })).toBeInTheDocument();
    expect(
      screen.getByRole('combobox', { name: /filename/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('combobox', { name: /chat template/i })
    ).toBeInTheDocument();

    expect(
      screen.getByRole('button', { name: /create model alias/i })
    ).toBeInTheDocument();
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

    await user.type(
      screen.getByRole('combobox', { name: /filename/i }),
      'file1.gguf'
    );

    // Open chat template combobox
    await user.click(screen.getByRole('combobox', { name: /chat template/i }));
    const chatTemplatePopover = screen.getByRole('dialog');
    const chatTemplateItems = within(chatTemplatePopover).getAllByRole('option');
    const llama2Option = chatTemplateItems.find(item =>
      item.textContent?.includes('llama2')
    );
    if (!llama2Option) {
      throw new Error('Could not find llama2 option');
    }
    await user.click(llama2Option);

    await user.click(
      screen.getByRole('button', { name: /create model alias/i })
    );

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Alias test-alias successfully created'));
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
        return res(ctx.json({ status: 'ready', authz: true }));
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
