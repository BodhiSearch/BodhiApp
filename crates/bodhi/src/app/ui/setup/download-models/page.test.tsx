import ModelDownloadPage, { ModelDownloadContent } from '@/app/ui/setup/download-models/page';
import { ENDPOINT_APP_INFO, ENDPOINT_MODEL_FILES_PULL, ENDPOINT_USER_INFO } from '@/hooks/useUsers';
import { showErrorParams } from '@/lib/utils.test';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, within } from '@testing-library/react';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { mockAppInfo, mockAppInfoReady, mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { mockModelPullDownloads, mockModelPullDownloadsEmpty } from '@/test-utils/msw-v2/handlers/modelfiles';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

setupMswV2();
beforeEach(() => {
  vi.resetAllMocks();
  pushMock.mockClear();
});

// Add ModelCard mock after existing mocks
vi.mock('@/app/ui/setup/download-models/ModelCard', () => ({
  ModelCard: ({ model }: any) => (
    <div data-testid={`model-card-${model.id}`}>
      <div>Name: {model.name}</div>
      <div>Status: {model.downloadState.status}</div>
      {model.downloadState.status === 'pending' && <div>Progress: {model.downloadState.progress}%</div>}
    </div>
  ),
}));

// Add toast mock after existing mocks
const mockToast = vi.fn();
vi.mock('@/components/ui/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

describe('ModelDownloadPage access control', () => {
  it('should redirect to /ui/setup if app status is setup', async () => {
    server.use(...mockAppInfoSetup(), ...mockUserLoggedOut(), ...mockModelPullDownloadsEmpty());

    await act(async () => {
      render(<ModelDownloadPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/setup');
  });

  it('should render the page when app is ready and user is logged in', async () => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ username: 'user@email.com', role: 'resource_user' }),
      ...mockModelPullDownloadsEmpty()
    );

    await act(async () => {
      render(<ModelDownloadPage />, { wrapper: createWrapper() });
    });
    expect(screen.getByText('Recommended Models')).toBeInTheDocument();
    expect(pushMock).not.toHaveBeenCalled();
  });

  it('should redirect to /ui/login when app is ready but user is not logged in', async () => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedOut(), ...mockModelPullDownloadsEmpty());

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
      ...mockModelPullDownloads({
        data: [
          {
            id: 'deepseek-r1-distill-llama-8b',
            repo: 'deepseek-ai/DeepSeek-R1-Distill-Llama-8B',
            filename: 'DeepSeek-R1-Distill-Llama-8B.gguf',
            status: 'pending',
            error: null,
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
            total_bytes: null,
            downloaded_bytes: null,
            started_at: '2024-01-01T00:00:00Z',
          },
          {
            id: 'meta-llama-3.1-8b-instruct',
            repo: 'bartowski/Meta-Llama-3.1-8B-Instruct-GGUF',
            filename: 'Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf',
            status: 'pending',
            error: null,
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
            total_bytes: 2000000,
            downloaded_bytes: 1000000,
            started_at: '2024-01-01T00:00:00Z',
          },
          {
            id: 'phi-3.5-mini-128k-instruct',
            repo: 'bartowski/Phi-3.5-mini-instruct-GGUF',
            filename: 'Phi-3.5-mini-instruct-Q8_0.gguf',
            status: 'completed',
            error: null,
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
            total_bytes: 1500000,
            downloaded_bytes: 1500000,
            started_at: '2024-01-01T00:00:00Z',
          },
        ],
        page: 1,
        page_size: 100,
        total: 3,
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
