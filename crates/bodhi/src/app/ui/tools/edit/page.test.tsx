/**
 * EditToolPage Component Tests
 *
 * Purpose: Verify tool configuration page functionality with comprehensive
 * scenario-based testing covering tool configuration and admin controls.
 *
 * Focus Areas:
 * - Tool configuration form display
 * - API key management
 * - Admin enable/disable controls
 * - Authentication and app initialization states
 */

import EditToolPage from '@/app/ui/tools/edit/page';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import {
  mockToolConfig,
  mockUpdateToolConfig,
  mockDeleteToolConfig,
  mockSetAppToolEnabled,
  mockSetAppToolDisabled,
} from '@/test-utils/msw-v2/handlers/tools';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
let mockSearchParams: URLSearchParams;

vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => mockSearchParams,
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
  mockSearchParams = new URLSearchParams('toolid=builtin-exa-web-search');
});

afterEach(() => {
  vi.resetAllMocks();
});

describe('EditToolPage - Authentication & Initialization', () => {
  it('redirects to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfo({ status: 'setup' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));

    await act(async () => {
      render(<EditToolPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedOut());

    await act(async () => {
      render(<EditToolPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });
});

describe('EditToolPage - Error States', () => {
  beforeEach(() => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
  });

  it('shows error when toolid is missing', async () => {
    mockSearchParams = new URLSearchParams('');

    await act(async () => {
      render(<EditToolPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByText('Tool ID is required')).toBeInTheDocument();
    });
  });
});

describe('EditToolPage - Tool Configuration Display', () => {
  beforeEach(() => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
  });

  it('displays tool configuration form', async () => {
    server.use(
      ...mockToolConfig('builtin-exa-web-search', {
        app_enabled: true,
        config: {
          tool_id: 'builtin-exa-web-search',
          enabled: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      })
    );

    await act(async () => {
      render(<EditToolPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tool-edit-page')).toBeInTheDocument();
    });

    expect(screen.getByTestId('tool-config-form')).toBeInTheDocument();
    expect(screen.getByTestId('tool-api-key-input')).toBeInTheDocument();
    expect(screen.getByTestId('tool-enabled-toggle')).toBeInTheDocument();
    expect(screen.getByTestId('save-tool-config')).toBeInTheDocument();
  });

  it('displays app disabled message when tool is disabled by admin', async () => {
    server.use(
      ...mockToolConfig('builtin-exa-web-search', {
        app_enabled: false,
        config: {
          tool_id: 'builtin-exa-web-search',
          enabled: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      })
    );

    await act(async () => {
      render(<EditToolPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tool-config-form')).toBeInTheDocument();
    });

    expect(screen.getByText(/This tool is disabled by administrator/)).toBeInTheDocument();
  });
});

describe('EditToolPage - Tool Configuration', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      ...mockToolConfig('builtin-exa-web-search', {
        app_enabled: true,
        config: {
          tool_id: 'builtin-exa-web-search',
          enabled: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      }),
      ...mockUpdateToolConfig('builtin-exa-web-search', {
        app_enabled: true,
        config: {
          tool_id: 'builtin-exa-web-search',
          enabled: true,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: new Date().toISOString(),
        },
      })
    );
  });

  it('saves tool configuration when save button is clicked', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<EditToolPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tool-config-form')).toBeInTheDocument();
    });

    // Enter API key
    const apiKeyInput = screen.getByTestId('tool-api-key-input');
    await user.type(apiKeyInput, 'test-api-key');

    // Enable the tool
    const enableToggle = screen.getByTestId('tool-enabled-toggle');
    await user.click(enableToggle);

    // Save
    const saveButton = screen.getByTestId('save-tool-config');
    await user.click(saveButton);

    await waitFor(() => {
      expect(toastMock).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Success',
          description: 'Tool configuration saved',
        })
      );
    });
  });
});

describe('EditToolPage - Admin Controls', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_admin' }, { stub: true }),
      ...mockToolConfig('builtin-exa-web-search', {
        app_enabled: true,
        config: {
          tool_id: 'builtin-exa-web-search',
          enabled: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      }),
      ...mockSetAppToolEnabled('builtin-exa-web-search'),
      ...mockSetAppToolDisabled('builtin-exa-web-search')
    );
  });

  it('shows admin toggle for resource_admin users', async () => {
    await act(async () => {
      render(<EditToolPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tool-config-form')).toBeInTheDocument();
    });

    expect(screen.getByTestId('app-enabled-toggle')).toBeInTheDocument();
    expect(screen.getByText('Enable for Server')).toBeInTheDocument();
  });

  it('opens confirmation dialog when disabling tool for server', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<EditToolPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tool-config-form')).toBeInTheDocument();
    });

    // Click the app enabled toggle to disable
    const appToggle = screen.getByTestId('app-enabled-toggle');
    await user.click(appToggle);

    // Should show confirmation dialog
    await waitFor(() => {
      expect(screen.getByText('Disable Tool for Server')).toBeInTheDocument();
    });
  });
});

describe('EditToolPage - Clear API Key', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      // Tool is configured with API key
      ...mockToolConfig('builtin-exa-web-search', {
        app_enabled: true,
        config: {
          tool_id: 'builtin-exa-web-search',
          enabled: true,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      }),
      ...mockDeleteToolConfig('builtin-exa-web-search')
    );
  });

  it('shows clear API key button when API key is configured', async () => {
    await act(async () => {
      render(<EditToolPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tool-config-form')).toBeInTheDocument();
    });

    expect(screen.getByTestId('clear-api-key-button')).toBeInTheDocument();
  });

  it('shows confirmation dialog when clearing API key', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<EditToolPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tool-config-form')).toBeInTheDocument();
    });

    const clearButton = screen.getByTestId('clear-api-key-button');
    await user.click(clearButton);

    await waitFor(() => {
      // The dialog title should appear
      expect(screen.getByRole('alertdialog')).toBeInTheDocument();
      expect(screen.getByText(/This will remove your API key/)).toBeInTheDocument();
    });
  });
});
