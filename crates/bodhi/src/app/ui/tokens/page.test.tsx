import TokenPage, { TokenPageContent } from '@/app/ui/tokens/page';
import { API_TOKENS_ENDPOINT } from '@/hooks/useQuery';
import { ENDPOINT_APP_INFO, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { showErrorParams, showSuccessParams } from '@/lib/utils.test';
import { createMockLoggedInUser, createMockLoggedOutUser } from '@/test-utils/mock-user';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw2/node';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  mockListTokens,
  mockCreateToken,
  mockUpdateToken,
  mockCreateTokenWithResponse,
  mockUpdateTokenStatus,
  mockUpdateTokenError,
} from '@/test-utils/msw-v2/handlers/tokens';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

const toast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast,
  }),
}));

const mockTokenResponse = {
  offline_token: 'test-token-123',
  name: 'Test Token',
  status: 'active',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

const mockListResponse = {
  data: [
    {
      id: 'token-1',
      name: 'Test Token 1',
      status: 'active',
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
    },
  ],
  total: 1,
  page: 1,
  page_size: 10,
};

const server = setupServer(...mockAppInfoReady(), ...mockUserLoggedIn(), ...mockListTokens());

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());
beforeEach(() => {
  vi.resetAllMocks();
  pushMock.mockClear();
});

describe('TokenPageContent', () => {
  it('shows loading skeleton initially', async () => {
    server.use(...mockListTokens());

    render(<TokenPageContent />, { wrapper: createWrapper() });

    expect(screen.getByTestId('token-page-loading')).toBeInTheDocument();
  });
});

describe('TokenPageContent', () => {
  beforeEach(() => {
    server.use(
      ...mockCreateToken({ offline_token: 'test-token-123' }),
      ...mockUpdateToken('token-1', {
        id: 'token-1',
        name: 'Test Token 1',
        status: 'inactive',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:01Z',
      })
    );
  });

  it('renders authenticated view with form and security warning', async () => {
    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    // Check title and description
    expect(screen.getByText(/API Tokens/)).toBeInTheDocument();
    expect(screen.getByText(/Generate and manage API tokens/)).toBeInTheDocument();

    // Check security warning
    expect(screen.getByText(/API tokens provide full access to the API/)).toBeInTheDocument();
    expect(screen.getByText(/Keep them secure/)).toBeInTheDocument();
    expect(screen.getByText(/Tokens cannot be viewed again/)).toBeInTheDocument();

    // Check form is rendered
    expect(screen.getByLabelText('Token Name (Optional)')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Generate Token' })).toBeInTheDocument();
  });

  it('handles complete token creation flow', async () => {
    const user = userEvent.setup();
    server.use(...mockCreateTokenWithResponse(mockTokenResponse));

    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    // Fill and submit form
    await user.type(screen.getByLabelText('Token Name (Optional)'), 'Test Token');
    await user.click(screen.getByRole('button', { name: 'Generate Token' }));

    // Check token dialog appears
    expect(await screen.findByText('API Token Generated')).toBeInTheDocument();

    // Close dialog
    await user.click(screen.getByRole('button', { name: 'Done' }));
    expect(screen.queryByText('API Token Generated')).not.toBeInTheDocument();
  });
});

describe('TokenPage', () => {
  it('redirects to login when not authenticated', async () => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedOut());

    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    expect(pushMock).toHaveBeenCalledWith('/ui/login');
  });
});

describe('token status updates', () => {
  beforeEach(() => {
    server.use(...mockUpdateTokenStatus('token-1', 'inactive'));
  });

  it('successfully updates token status', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    const switchElement = screen.getByRole('switch');
    expect(switchElement).toBeChecked();

    await user.click(switchElement);

    await waitFor(() => {
      expect(toast).toHaveBeenCalledWith(showSuccessParams('Token Updated', 'Token status changed to inactive'));
    });
  });
});

describe('token status update', () => {
  it('handles token status update error', async () => {
    server.use(
      ...mockUpdateTokenError('token-1', {
        status: 500,
        message: 'Test Error',
      })
    );

    const user = userEvent.setup();

    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    const switchElement = screen.getByRole('switch');
    await user.click(switchElement);

    await waitFor(() => {
      expect(toast).toHaveBeenCalledWith(showErrorParams('Error', 'Test Error'));
    });
  });
});
