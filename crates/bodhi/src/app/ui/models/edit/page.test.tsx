import EditAliasPage from '@/app/ui/models/edit/page';
import { ENDPOINT_APP_INFO, ENDPOINT_CHAT_TEMPLATES, ENDPOINT_MODEL_FILES, ENDPOINT_MODELS, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
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
        return res(ctx.json({ data: [] }));
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

  it('submits the form with updated data', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<EditAliasPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByLabelText(/alias/i)).toBeInTheDocument();

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
        return res(ctx.json({ status: 'ready', authz: true }));
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
