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
import { Toaster } from '@/components/ui/toaster';
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
    expect(screen.getByRole('combobox', { name: /repo/i })).toBeInTheDocument();
    expect(
      screen.getByRole('combobox', { name: /filename/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('combobox', { name: /chat template/i })
    ).toBeInTheDocument();

    // Check for submit button
    expect(
      screen.getByRole('button', { name: /create model alias/i })
    ).toBeInTheDocument();
  });

  it('populates repo options correctly', async () => {
    render(<CreateAliasPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByLabelText(/repo/i)).toBeInTheDocument();
    });

    const repoSelect = screen.getByRole('combobox', { name: /repo/i });
    await userEvent.click(repoSelect);

    // Check for unique repo options
    expect(
      screen.getByRole('option', { name: 'owner1/repo1' })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('option', { name: 'owner2/repo2' })
    ).toBeInTheDocument();
    expect(screen.getAllByRole('option').length).toBe(2); // Ensure no duplicates
  });

  it('populates filename options based on selected repo', async () => {
    render(<CreateAliasPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByLabelText(/repo/i)).toBeInTheDocument();
    });

    // Select a repo
    const repoSelect = screen.getByRole('combobox', { name: /repo/i });
    await userEvent.click(repoSelect);
    await userEvent.click(screen.getByRole('option', { name: 'owner1/repo1' }));

    // Check filename options
    const filenameSelect = screen.getByRole('combobox', { name: /filename/i });
    await userEvent.click(filenameSelect);

    expect(
      screen.getByRole('option', { name: 'file1.gguf' })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('option', { name: 'file2.gguf' })
    ).toBeInTheDocument();
    expect(
      screen.queryByRole('option', { name: 'file3.gguf' })
    ).not.toBeInTheDocument();
  });

  it('populates chat template options correctly', async () => {
    render(<CreateAliasPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByLabelText(/chat template/i)).toBeInTheDocument();
    });

    const chatTemplateSelect = screen.getByRole('combobox', {
      name: /chat template/i,
    });
    await userEvent.click(chatTemplateSelect);

    expect(screen.getByRole('option', { name: 'llama2' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'llama3' })).toBeInTheDocument();
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

    const repoSelect = screen.getByRole('combobox', { name: /repo/i });
    await user.click(repoSelect);
    await user.click(screen.getByRole('option', { name: 'owner1/repo1' }));

    const filenameSelect = screen.getByRole('combobox', { name: /filename/i });
    await user.click(filenameSelect);
    await user.click(screen.getByRole('option', { name: 'file1.gguf' }));

    const chatTemplateSelect = screen.getByRole('combobox', {
      name: /chat template/i,
    });
    await user.click(chatTemplateSelect);
    await user.click(screen.getByRole('option', { name: 'llama2' }));

    // Submit the form
    await user.click(
      screen.getByRole('button', { name: /create model alias/i })
    );

    // After form submission
    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith({
        title: 'Success',
        description: 'Alias test-alias successfully created',
        duration: 5000,
      });
    });
  });
});
