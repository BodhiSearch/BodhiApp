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
import CreateAliasPage from './page';
import { ToastProvider } from '@/components/ui/toast';

// Mock components and modules
vi.mock('@/components/AppHeader', () => ({
  default: () => <div data-testid="app-header">Mocked AppHeader</div>,
}));

vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: vi.fn(),
  }),
}));

const mockToast = vi.fn();
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
    <>
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    </>
  );
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
  rest.get('*/api/ui/models', (req, res, ctx) => {
    return res(ctx.json(mockModelsResponse));
  }),
  rest.get('*/api/ui/chat_templates', (req, res, ctx) => {
    return res(ctx.json(mockChatTemplatesResponse));
  }),
  rest.post('*/api/ui/models', (req, res, ctx) => {
    return res(
      ctx.status(200),
      ctx.json({ message: 'Model created successfully' })
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

describe('CreateAliasPage', () => {
  it('renders the page with all form elements', async () => {
    render(<CreateAliasPage />, { wrapper: createWrapper() });

    // Wait for the API calls to resolve
    await waitFor(() => {
      expect(screen.getByTestId('app-header')).toBeInTheDocument();
    });

    // Check for form elements
    expect(screen.getByLabelText(/alias/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/repo/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/filename/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/chat template/i)).toBeInTheDocument();

    // Check for shadcn Select components
    expect(screen.getByRole('textbox', { name: /repo/i })).toBeInTheDocument();
    expect(
      screen.getByRole('textbox', { name: /filename/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('textbox', { name: /chat template/i })
    ).toBeInTheDocument();

    // Check for submit button
    expect(
      screen.getByRole('button', { name: /create model alias/i })
    ).toBeInTheDocument();
  });

  it('submits the form with correct data', async () => {
    const user = userEvent.setup();

    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <ToastProvider>
        <QueryClientProvider client={queryClient}>
          {children}
        </QueryClientProvider>
      </ToastProvider>
    );

    render(<CreateAliasPage />, { wrapper });

    await waitFor(() => {
      expect(screen.getByLabelText(/alias/i)).toBeInTheDocument();
    });

    // Fill out the form
    await user.type(screen.getByLabelText(/alias/i), 'test-alias');

    await user.type(
      screen.getByRole('textbox', { name: /repo/i }),
      'owner1/repo1'
    );
    await user.type(
      screen.getByRole('textbox', { name: /filename/i }),
      'file1.gguf'
    );
    await user.type(
      screen.getByRole('textbox', { name: /chat template/i }),
      'llama2'
    );
    await user.click(
      screen.getByRole('button', { name: /create model alias/i })
    );

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith({
        title: 'Success',
        description: 'Alias test-alias successfully created',
        duration: 5000,
      });
    });
  });
});
