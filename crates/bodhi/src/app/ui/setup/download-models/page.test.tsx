/**
 * ModelDownloadPage Component Tests
 *
 * Purpose: Integration testing of model download page workflow with real components.
 * Component-level details tested in ModelCard.test.tsx.
 *
 * Focus Areas:
 * - Access control and authentication
 * - Model catalog integration (hooks + components)
 * - Download workflow (user click → API → state update)
 * - Error handling with toast notifications
 * - Navigation and background download continuation
 *
 * Test Coverage:
 * 1. Access Control: Setup/login redirects (3 tests)
 * 2. Integration: Catalog rendering and download workflow (2 tests)
 * 3. Error Handling: API errors with retry (1 test)
 * 4. Navigation: Continue button with localStorage (1 test)
 *
 * Total: 7 integration tests
 *
 * Note: ModelCard tested comprehensively in ModelCard.test.tsx
 */

import ModelDownloadPage, { ModelDownloadContent } from '@/app/ui/setup/download-models/page';
import { mockAppInfoReady, mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import {
  mockModelPullDownloads,
  mockModelPullDownloadsEmpty,
  mockModelPull,
  mockModelPullError,
} from '@/test-utils/msw-v2/handlers/modelfiles';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, ROUTE_SETUP_API_MODELS } from '@/lib/constants';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

const mockToast = vi.fn();
vi.mock('@/hooks/use-toast-messages', () => ({
  useToastMessages: () => ({
    showSuccess: mockToast,
    showError: mockToast,
  }),
}));

setupMswV2();

beforeEach(() => {
  vi.resetAllMocks();
  pushMock.mockClear();
  mockToast.mockClear();
  localStorage.clear();
});

describe('ModelDownloadPage Access Control', () => {
  it('redirects to /ui/setup if app status is setup', async () => {
    server.use(...mockAppInfoSetup(), ...mockUserLoggedOut(), ...mockModelPullDownloadsEmpty());

    await act(async () => {
      render(<ModelDownloadPage />, { wrapper: createWrapper() });
    });

    expect(pushMock).toHaveBeenCalledWith('/ui/setup');
  });

  it('renders the page when app is ready and user is logged in', async () => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ username: 'user@email.com', role: 'resource_user' }),
      ...mockModelPullDownloadsEmpty()
    );

    await act(async () => {
      render(<ModelDownloadPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByText('Chat Models')).toBeInTheDocument();
    });
    expect(pushMock).not.toHaveBeenCalled();
  });

  it('redirects to /ui/login when app is ready but user is not logged in', async () => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedOut(), ...mockModelPullDownloadsEmpty());

    await act(async () => {
      render(<ModelDownloadPage />, { wrapper: createWrapper() });
    });

    expect(pushMock).toHaveBeenCalledWith('/ui/login');
  });
});

describe('ModelDownloadPage Integration Tests', () => {
  beforeEach(() => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn({ username: 'user@email.com', role: 'resource_user' }));
  });

  it('renders catalog and initiates download on click', async () => {
    const user = userEvent.setup();
    server.use(...mockModelPullDownloadsEmpty());
    server.use(
      ...mockModelPull({ repo: 'bartowski/Qwen2.5-14B-Instruct-GGUF', filename: 'Qwen2.5-14B-Instruct-Q4_K_M.gguf' })
    );

    render(<ModelDownloadContent />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Chat Models')).toBeInTheDocument();
    });

    expect(screen.getByText('Qwen2.5 14B')).toBeInTheDocument();
    expect(screen.getByText('Phi-4 14B')).toBeInTheDocument();
    expect(screen.getByText('GPT-OSS 20B')).toBeInTheDocument();

    expect(screen.getByText('Embedding Models')).toBeInTheDocument();
    expect(screen.getByText('Qwen3 Embedding 4B')).toBeInTheDocument();
    expect(screen.getByText('Nomic Embed v1.5')).toBeInTheDocument();
    expect(screen.getByText('BGE Large EN v1.5')).toBeInTheDocument();

    const downloadButtons = screen.getAllByTestId('download-button');
    expect(downloadButtons.length).toBeGreaterThan(0);

    await user.click(downloadButtons[0]);

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith('Success', 'Model download started');
    });
  });

  it('displays existing downloads with correct states', async () => {
    server.use(
      ...mockModelPullDownloads({
        data: [
          {
            id: 'qwen-pending',
            repo: 'bartowski/Qwen2.5-14B-Instruct-GGUF',
            filename: 'Qwen2.5-14B-Instruct-Q4_K_M.gguf',
            status: 'pending',
            error: null,
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
            total_bytes: 9_000_000_000,
            downloaded_bytes: 4_500_000_000,
            started_at: '2024-01-01T00:00:00Z',
          },
          {
            id: 'phi-completed',
            repo: 'bartowski/phi-4-GGUF',
            filename: 'phi-4-Q4_K_M.gguf',
            status: 'completed',
            error: null,
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
            total_bytes: 9_050_000_000,
            downloaded_bytes: 9_050_000_000,
            started_at: '2024-01-01T00:00:00Z',
          },
        ],
        page: 1,
        page_size: 100,
        total: 2,
      })
    );

    render(<ModelDownloadContent />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Chat Models')).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(screen.getByText('50%')).toBeInTheDocument();
    });
    expect(screen.getByTestId('byte-display')).toHaveTextContent('4.2 GB / 8.4 GB');

    const downloadedButtons = screen.getAllByRole('button', { name: /downloaded/i });
    expect(downloadedButtons.length).toBeGreaterThan(0);
    expect(downloadedButtons[0]).toBeDisabled();
  });
});

describe('ModelDownloadPage Error Handling', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ username: 'user@email.com', role: 'resource_user' }),
      ...mockModelPullDownloadsEmpty()
    );
  });

  it('download error shows toast and allows retry', async () => {
    const user = userEvent.setup();

    server.use(
      ...mockModelPullError({
        code: 'internal_server_error',
        message: 'Download failed',
        status: 500,
      })
    );

    render(<ModelDownloadContent />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Chat Models')).toBeInTheDocument();
    });

    const downloadButtons = screen.getAllByTestId('download-button');
    await user.click(downloadButtons[0]);

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith('Error', 'Download failed');
    });

    server.use(
      ...mockModelPull({
        repo: 'bartowski/Qwen2.5-14B-Instruct-GGUF',
        filename: 'Qwen2.5-14B-Instruct-Q4_K_M.gguf',
      })
    );

    await user.click(downloadButtons[0]);

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith('Success', 'Model download started');
    });
  });
});

describe('ModelDownloadPage Navigation', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ username: 'user@email.com', role: 'resource_user' }),
      ...mockModelPullDownloadsEmpty()
    );
  });

  it('continue button navigates and sets localStorage flag', async () => {
    const user = userEvent.setup();

    render(<ModelDownloadContent />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Chat Models')).toBeInTheDocument();
    });

    const continueButton = screen.getByTestId('continue-button');
    expect(continueButton).toBeInTheDocument();

    await user.click(continueButton);

    expect(localStorage.getItem(FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED)).toBe('true');
    expect(pushMock).toHaveBeenCalledWith(ROUTE_SETUP_API_MODELS);
  });
});
