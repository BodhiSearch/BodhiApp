import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { QueryClient, QueryClientProvider } from 'react-query';
import {
  afterAll,
  afterEach,
  beforeAll,
  describe,
  expect,
  it,
  vi,
} from 'vitest';
import EditAliasPage from './page';
import { ToastProvider } from '@/components/ui/toast';

const mockToast = vi.fn();

vi.mock('@/components/AppHeader', () => ({
  default: () => <div data-testid="app-header">Mocked AppHeader</div>,
}));

vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: vi.fn(),
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

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <ToastProvider>
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    </ToastProvider>
  );
};

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

const server = setupServer(
  rest.get('*/api/ui/models/:alias', (req, res, ctx) => {
    return res(ctx.json(mockModelData));
  }),
  rest.get('*/api/ui/models', (req, res, ctx) => {
    return res(ctx.json(mockModelsResponse));
  }),
  rest.get('*/api/ui/chat_templates', (req, res, ctx) => {
    return res(ctx.json(mockChatTemplatesResponse));
  }),
  rest.put('*/api/ui/models/:alias', (req, res, ctx) => {
    return res(
      ctx.status(200),
      ctx.json({ message: 'Model updated successfully' })
    );
  })
);

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

describe('EditAliasPage', () => {
  it('renders the page with all form elements pre-filled with model data', async () => {
    render(<EditAliasPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByTestId('app-header')).toBeInTheDocument();
    });

    expect(screen.getByLabelText(/alias/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/repo/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/filename/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/chat template/i)).toBeInTheDocument();

    expect(screen.getByRole('textbox', { name: /repo/i })).toBeInTheDocument();
    expect(
      screen.getByRole('textbox', { name: /filename/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('textbox', { name: /chat template/i })
    ).toBeInTheDocument();

    expect(
      screen.getByRole('button', { name: /update model alias/i })
    ).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByLabelText(/alias/i)).toHaveValue('test-alias');
      expect(screen.getByRole('textbox', { name: /repo/i })).toHaveValue(
        'owner1/repo1'
      );
      expect(screen.getByRole('textbox', { name: /filename/i })).toHaveValue(
        'file1.gguf'
      );
      expect(
        screen.getByRole('textbox', { name: /chat template/i })
      ).toHaveValue('llama2');
    });
  });

  it('submits the form with updated data', async () => {
    const user = userEvent.setup();

    render(<EditAliasPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByLabelText(/alias/i)).toBeInTheDocument();
    });

    await user.clear(screen.getByRole('textbox', { name: /repo/i }));
    await user.type(
      screen.getByRole('textbox', { name: /repo/i }),
      'owner2/repo2'
    );

    await user.clear(screen.getByRole('textbox', { name: /filename/i }));
    await user.type(
      screen.getByRole('textbox', { name: /filename/i }),
      'file3.gguf'
    );

    await user.clear(screen.getByRole('textbox', { name: /chat template/i }));
    await user.type(
      screen.getByRole('textbox', { name: /chat template/i }),
      'llama3'
    );

    await user.click(
      screen.getByRole('button', { name: /update model alias/i })
    );

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith({
        title: 'Success',
        description: 'Alias test-alias successfully updated',
        duration: 5000,
      });
    });
  });

  it('displays error message when model data fails to load', async () => {
    server.use(
      rest.get('*/api/ui/models/:alias', (req, res, ctx) => {
        return res(ctx.status(500));
      })
    );

    render(<EditAliasPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Error loading model data')).toBeInTheDocument();
    });
  });
});
