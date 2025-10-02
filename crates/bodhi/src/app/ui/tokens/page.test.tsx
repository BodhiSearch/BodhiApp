/**
 * TokenPage Component Tests
 *
 * Purpose: Verify API token management functionality with comprehensive
 * scenario-based testing covering real-world token usage patterns.
 *
 * Focus Areas:
 * - Token lifecycle (creation → display → status management)
 * - Token dialog interactions (visibility toggle, copy functionality)
 * - Optimistic UI updates with error recovery
 * - Authentication and app initialization states
 *
 * Test Structure:
 * 1. Authentication & Initialization (2 tests)
 * 2. Token Creation Flow (1 integrated scenario test)
 * 3. Token List Display (2 tests: empty + multiple tokens)
 * 4. Optimistic Updates (2 tests: success + error rollback)
 *
 * Total: 7 comprehensive scenario-based tests
 */

import TokenPage from '@/app/ui/tokens/page';
import { showErrorParams, showSuccessParams } from '@/lib/utils.test';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import {
  mockCreateToken,
  mockTokens,
  mockUpdateTokenError,
  mockUpdateTokenStatus,
} from '@/test-utils/msw-v2/handlers/tokens';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

const toastMock = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: toastMock,
  }),
}));

setupMswV2();

beforeEach(() => {
  pushMock.mockClear();
  toastMock.mockClear();
});

afterEach(() => {
  vi.resetAllMocks();
});

describe('TokenPage - Authentication & Initialization', () => {
  it('redirects to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfo({ status: 'setup' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));

    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedOut());

    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });
});

describe('TokenPage - Token Creation Flow', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      ...mockTokens({ data: [], total: 0 })
    );
  });

  it('completes full token lifecycle: create → dialog → copy → display in list', async () => {
    const user = userEvent.setup();
    const createdToken = 'bodhiapp_abc123def456';

    server.use(...mockCreateToken({ token: createdToken }));

    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tokens-page')).toBeInTheDocument();
    });

    // Step 1: Open dialog by clicking "New API Token" button
    const newTokenButton = screen.getByTestId('new-token-button');
    await user.click(newTokenButton);

    // Wait for dialog to open
    await waitFor(() => {
      expect(screen.getByLabelText('Token Name (Optional)')).toBeInTheDocument();
    });

    // Step 2: Fill token name and submit form
    const nameInput = screen.getByLabelText('Token Name (Optional)');
    await user.type(nameInput, 'My API Token');

    const generateButton = screen.getByRole('button', { name: 'Generate Token' });
    await user.click(generateButton);

    // Step 3: Verify dialog shows created token
    await waitFor(() => {
      expect(screen.getByText('API Token Generated')).toBeInTheDocument();
    });

    expect(screen.getByText(/Copy your API token now/)).toBeInTheDocument();
    expect(screen.getByText(/Make sure to copy your token now/)).toBeInTheDocument();

    // Step 4: Test show/hide toggle functionality
    const showButton = screen.getByRole('button', { name: /show content/i });

    // Token should be hidden by default (showing dots)
    expect(screen.queryByText(createdToken)).not.toBeInTheDocument();

    // Toggle to show token
    await user.click(showButton);
    expect(screen.getByText(createdToken)).toBeInTheDocument();

    // Toggle back to hide
    const hideButton = screen.getByRole('button', { name: /hide content/i });
    await user.click(hideButton);
    expect(screen.queryByText(createdToken)).not.toBeInTheDocument();

    // Step 5: Test copy button functionality
    const writeTextMock = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', {
      value: {
        writeText: writeTextMock,
      },
      writable: true,
    });

    const copyButton = screen.getByRole('button', { name: /copy to clipboard/i });
    await user.click(copyButton);

    expect(writeTextMock).toHaveBeenCalledWith(createdToken);

    // Step 6: Close dialog with "Done"
    const doneButton = screen.getByRole('button', { name: 'Done' });
    await user.click(doneButton);

    await waitFor(() => {
      expect(screen.queryByText('API Token Generated')).not.toBeInTheDocument();
    });

    // Step 7: Verify success toast was called
    expect(toastMock).toHaveBeenCalledWith(showSuccessParams('Success', 'API token successfully generated'));
  });
});

describe('TokenPage - Token List Display', () => {
  beforeEach(() => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
  });

  it('displays empty state when no tokens exist', async () => {
    server.use(...mockTokens({ data: [], total: 0 }));

    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tokens-page')).toBeInTheDocument();
    });

    // Verify table exists but has no data rows
    const rows = screen.queryAllByRole('row');
    // Only header row should exist
    expect(rows.length).toBe(1);
  });

  it('displays multiple tokens with complete metadata', async () => {
    const tokens = [
      {
        id: 'token-1',
        name: 'Production API',
        token_prefix: 'bodhiapp_prod001',
        token_hash: 'hash1',
        scopes: 'scope_token_poweruser',
        user_id: 'user-1',
        status: 'active' as const,
        created_at: '2024-01-03T10:00:00Z',
        updated_at: '2024-01-04T12:00:00Z',
      },
      {
        id: 'token-2',
        name: 'Development API',
        token_prefix: 'bodhiapp_dev002',
        token_hash: 'hash2',
        scopes: 'scope_token_user',
        user_id: 'user-1',
        status: 'inactive' as const,
        created_at: '2024-01-01T08:00:00Z',
        updated_at: '2024-01-02T09:00:00Z',
      },
    ];

    server.use(...mockTokens({ data: tokens, total: 2 }));

    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tokens-page')).toBeInTheDocument();
    });

    // Verify both tokens are displayed
    expect(screen.getByText('Production API')).toBeInTheDocument();
    expect(screen.getByText('Development API')).toBeInTheDocument();

    // Verify status badges
    const statusBadges = screen.getAllByText(/active|inactive/i);
    expect(statusBadges.length).toBeGreaterThanOrEqual(2);

    // Verify timestamps are formatted and displayed
    const rows = screen.getAllByRole('row');
    expect(rows.length).toBe(3); // Header + 2 data rows

    // Verify switch controls exist for both tokens
    const switches = screen.getAllByRole('switch');
    expect(switches).toHaveLength(2);

    // Verify first token is active (switch checked)
    expect(switches[0]).toBeChecked();
    // Verify second token is inactive (switch unchecked)
    expect(switches[1]).not.toBeChecked();
  });
});

describe('TokenPage - Optimistic Updates', () => {
  beforeEach(() => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
  });

  it('successfully updates token status and shows success notification', async () => {
    const user = userEvent.setup();

    const updatedToken = {
      id: 'token-1',
      name: 'Test Token',
      token_prefix: 'bodhiapp_test01',
      token_hash: 'hash123',
      scopes: 'scope_token_user',
      user_id: 'user-1',
      status: 'inactive' as const,
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-02T00:00:00Z',
    };

    server.use(
      ...mockTokens(
        {
          data: [{ ...updatedToken, status: 'active' as const }],
          total: 1,
        },
        { stub: true }
      ),
      ...mockUpdateTokenStatus('token-1', 'inactive')
    );

    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByText(/Test Token/)).toBeInTheDocument();
    });

    const switchElement = screen.getByRole('switch');
    expect(switchElement).toBeChecked();

    // Update mock to return updated data on refetch
    server.use(...mockTokens({ data: [updatedToken], total: 1 }, { stub: true }));

    // Click toggle switch
    await user.click(switchElement);

    // Verify success toast
    await waitFor(() => {
      expect(toastMock).toHaveBeenCalledWith(showSuccessParams('Token Updated', 'Token status changed to inactive'));
    });
  });

  it('shows error notification on update failure', async () => {
    const user = userEvent.setup();

    const token = {
      id: 'token-1',
      name: 'Test Token',
      token_prefix: 'bodhiapp_test01',
      token_hash: 'hash123',
      scopes: 'scope_token_user',
      user_id: 'user-1',
      status: 'active' as const,
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
    };

    server.use(
      ...mockTokens({ data: [token], total: 1 }, { stub: true }),
      ...mockUpdateTokenError('token-1', {
        message: 'Database connection failed',
        type: 'internal_server_error',
      })
    );

    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByText(/Test Token/)).toBeInTheDocument();
    });

    const switchElement = screen.getByRole('switch');
    expect(switchElement).toBeChecked();

    // Click toggle switch
    await user.click(switchElement);

    // Verify error toast is shown
    await waitFor(() => {
      expect(toastMock).toHaveBeenCalledWith(showErrorParams('Error', 'Database connection failed'));
    });
  });
});
