import ModelDownloadPage, { ModelDownloadContent } from '@/components/setup/download-models/page';
import { ENDPOINT_APP_INFO, ENDPOINT_MODEL_FILES_PULL, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { showErrorParams } from '@/lib/utils.test';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, within } from '@testing-library/react';
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

// Mock framer-motion
vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, className }: any) => <div className={className}>{children}</div>,
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

const pushMock = vi.fn();
vi.mock('@/lib/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

const server = setupServer();
beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());
beforeEach(() => {
  vi.resetAllMocks();
  pushMock.mockClear();
});

// Add ModelCard mock after existing mocks
vi.mock('@/components/setup/download-models/ModelCard', () => ({
  ModelCard: ({ model }: any) => (
    <div data-testid={`model-card-${model.id}`}>
      <div>Name: {model.name}</div>
      <div>Status: {model.downloadState.status}</div>
      {model.downloadState.status === 'pending' && (
        <div>Progress: {model.downloadState.progress}%</div>
      )}
    </div>
  ),
}));

// Add toast mock after existing mocks
const mockToast = vi.fn();
vi.mock('@/components/ui/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

// Add mock models data after server setup
const mockModels = [
  {
    id: 'meta-llama-3.1-8b-instruct',
    repo: 'bartowski/Meta-Llama-3.1-8B-Instruct-GGUF',
    filename: 'Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf',
    status: 'pending',
  },
  {
    id: 'phi-3.5-mini-128k-instruct',
    repo: 'bartowski/Phi-3.5-mini-instruct-GGUF',
    filename: 'Phi-3.5-mini-instruct-Q8_0.gguf',
    status: 'completed',
  },
];

describe('ModelDownloadPage access control', () => {
  it('should render the page when app is ready without auth', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            version: '0.1.0',
            authz: false,
            status: 'ready',
          })
        );
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            logged_in: false,
          })
        );
      }),
      rest.get(`*${ENDPOINT_MODEL_FILES_PULL}`, (_, res, ctx) => {
        return res(ctx.json({ data: [], page: 1, page_size: 100 }));
      }),
    );

    await act(async () => {
      render(<ModelDownloadPage />, { wrapper: createWrapper() });
    });
    expect(screen.getByText('Recommended Models')).toBeInTheDocument();
    expect(pushMock).not.toHaveBeenCalled();
  });

  it('should redirect to /ui/setup if app status is setup', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            version: '0.1.0',
            status: 'setup',
          })
        );
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            logged_in: false,
          })
        );
      }),
      rest.get(`*${ENDPOINT_MODEL_FILES_PULL}`, (_, res, ctx) => {
        return res(ctx.json({ data: [], page: 1, page_size: 100 }));
      }),
    );

    await act(async () => {
      render(<ModelDownloadPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/setup');
  });

  it('should render the page when app is ready with auth and user is logged in', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            version: '0.1.0',
            authz: true,
            status: 'ready',
          })
        );
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            logged_in: true,
            email: 'user@email.com',
            roles: [],
          })
        );
      }),
      rest.get(`*${ENDPOINT_MODEL_FILES_PULL}`, (_, res, ctx) => {
        return res(ctx.json({ data: [], page: 1, page_size: 100 }));
      }),
    );

    await act(async () => {
      render(<ModelDownloadPage />, { wrapper: createWrapper() });
    });
    expect(screen.getByText('Recommended Models')).toBeInTheDocument();
    expect(pushMock).not.toHaveBeenCalled();
  });

  it('should redirect to /ui/login when app is ready with auth but user is not logged in', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            version: '0.1.0',
            authz: true,
            status: 'ready',
          })
        );
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            logged_in: false,
            email: null,
            roles: [],
          })
        );
      }),
      rest.get(`*${ENDPOINT_MODEL_FILES_PULL}`, (_, res, ctx) => {
        return res(ctx.json({ data: [], page: 1, page_size: 100 }));
      }),
    );

    await act(async () => {
      render(<ModelDownloadPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/login');
  });
});

describe('ModelDownloadPage render', () => {
  beforeEach(() => {
    mockToast.mockClear(); // Clear toast mock before each test
    server.use(
      rest.get(`*${ENDPOINT_MODEL_FILES_PULL}`, (_, res, ctx) => {
        return res(ctx.json({ data: mockModels, page: 1, page_size: 100 }));
      })
    );
  });

  it('should render models with different download states', async () => {
    await act(async () => {
      render(<ModelDownloadContent />, { wrapper: createWrapper() });
    });

    // Wait for models to load
    const idleModel = await screen.findByTestId('model-card-deepseek-r1-distill-llama-8b');
    const pendingModel = screen.getByTestId('model-card-meta-llama-3.1-8b-instruct');
    const completedModel = screen.getByTestId('model-card-phi-3.5-mini-128k-instruct');

    // Check idle model
    expect(idleModel).toBeInTheDocument();
    expect(within(idleModel).getByText('Status: idle')).toBeInTheDocument();

    // Check pending model
    expect(pendingModel).toBeInTheDocument();
    expect(within(pendingModel).getByText('Status: pending')).toBeInTheDocument();

    // Check completed model
    expect(completedModel).toBeInTheDocument();
    expect(within(completedModel).getByText('Status: completed')).toBeInTheDocument();
  });
});