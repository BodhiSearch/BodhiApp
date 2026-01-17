/**
 * ToolsetsSetupPage Component Tests
 *
 * Purpose: Verify toolsets setup page functionality in the onboarding flow.
 *
 * Focus Areas:
 * - Toolset configuration form renders immediately (optimistic rendering)
 * - Enable for Server toggle defaults to OFF
 * - Form is disabled when Enable for Server is OFF
 * - Auto-enable toolset when API key is entered
 * - Skip functionality
 * - Save configuration with confirmation dialog
 */

import React from 'react';

import ToolsetsSetupPage from '@/app/ui/setup/toolsets/page';
import { SetupProvider } from '@/app/ui/setup/components';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import {
  mockSetAppToolsetEnabled,
  mockToolsetConfig,
  mockUpdateToolsetConfig,
  mockUpdateToolsetConfigError,
} from '@/test-utils/msw-v2/handlers/toolsets';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  usePathname: () => '/ui/setup/toolsets',
}));

const toastMock = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: toastMock,
  }),
}));

setupMswV2();

// Helper to render with SetupProvider
const renderWithSetupProvider = (component: React.ReactElement) => {
  return render(<SetupProvider>{component}</SetupProvider>, { wrapper: createWrapper() });
};

// Helper to wait for page to load (AppInitializer completes)
const waitForPageLoad = async () => {
  await waitFor(() => {
    expect(screen.getByTestId('toolsets-setup-page')).toBeInTheDocument();
  });
};

beforeEach(() => {
  pushMock.mockClear();
  toastMock.mockClear();
});

afterEach(() => {
  vi.resetAllMocks();
});

describe('ToolsetsSetupPage - Display', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_admin' }, { stub: true }),
      ...mockToolsetConfig('builtin-exa-web-search', {
        app_enabled: false,
        config: {
          toolset_id: 'builtin-exa-web-search',
          enabled: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      })
    );
  });

  it('renders form with toolset config elements', async () => {
    renderWithSetupProvider(<ToolsetsSetupPage />);
    await waitForPageLoad();

    expect(screen.getByTestId('toolset-config-form')).toBeInTheDocument();
    expect(screen.getByText('Configure Toolsets')).toBeInTheDocument();
    expect(screen.getByTestId('app-enabled-toggle')).toBeInTheDocument();
    expect(screen.getByTestId('toolset-api-key-input')).toBeInTheDocument();
  });

  it('shows Enable for Server toggle defaulting to OFF', async () => {
    renderWithSetupProvider(<ToolsetsSetupPage />);
    await waitForPageLoad();

    // Toggle should be present and OFF by default
    const toggle = screen.getByTestId('app-enabled-toggle');
    expect(toggle).toBeInTheDocument();
    expect(toggle).not.toBeChecked();

    // Badge should show "Disabled"
    expect(screen.getByText('Disabled')).toBeInTheDocument();
  });

  it('shows disabled message when Enable for Server is OFF', async () => {
    renderWithSetupProvider(<ToolsetsSetupPage />);
    await waitForPageLoad();

    // Should show the disabled message
    expect(screen.getByTestId('app-disabled-message')).toBeInTheDocument();
    expect(screen.getByText('Enable the toolset for this server to configure it.')).toBeInTheDocument();
  });

  it('has form controls disabled when Enable for Server is OFF', async () => {
    renderWithSetupProvider(<ToolsetsSetupPage />);
    await waitForPageLoad();

    // API key input should be disabled (via pointer-events-none on parent)
    const apiKeyInput = screen.getByTestId('toolset-api-key-input');
    expect(apiKeyInput).toBeDisabled();

    // Save button should be disabled
    const saveButton = screen.getByTestId('save-toolset-config');
    expect(saveButton).toBeDisabled();
  });

  it('does NOT show enable for all users checkbox (removed)', async () => {
    renderWithSetupProvider(<ToolsetsSetupPage />);
    await waitForPageLoad();

    // The checkbox should NOT exist anymore
    expect(screen.queryByTestId('app-enable-checkbox')).not.toBeInTheDocument();
    expect(screen.queryByText('Enable this toolset for all users')).not.toBeInTheDocument();
  });

  it('shows skip/continue button in footer', async () => {
    renderWithSetupProvider(<ToolsetsSetupPage />);
    await waitForPageLoad();

    expect(screen.getByTestId('skip-toolsets-setup')).toBeInTheDocument();
  });
});

describe('ToolsetsSetupPage - Enable for Server Toggle', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_admin' }, { stub: true }),
      ...mockToolsetConfig('builtin-exa-web-search', {
        app_enabled: false,
        config: {
          toolset_id: 'builtin-exa-web-search',
          enabled: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      }),
      ...mockSetAppToolsetEnabled('builtin-exa-web-search')
    );
  });

  it('shows confirmation dialog when enabling for server', async () => {
    const user = userEvent.setup();
    renderWithSetupProvider(<ToolsetsSetupPage />);
    await waitForPageLoad();

    // Click the toggle to enable
    const toggle = screen.getByTestId('app-enabled-toggle');
    await user.click(toggle);

    // Confirmation dialog should appear
    await waitFor(() => {
      expect(screen.getByText('Enable Toolset for Server')).toBeInTheDocument();
    });
    expect(screen.getByText(/This will enable Exa Web Search for all users/)).toBeInTheDocument();
  });

  it('enables form when Enable for Server is toggled ON via dialog', async () => {
    const user = userEvent.setup();
    renderWithSetupProvider(<ToolsetsSetupPage />);
    await waitForPageLoad();

    // Click the toggle to enable
    const toggle = screen.getByTestId('app-enabled-toggle');
    await user.click(toggle);

    // Wait for dialog and click Enable
    await waitFor(() => {
      expect(screen.getByText('Enable Toolset for Server')).toBeInTheDocument();
    });
    const enableButton = screen.getByRole('button', { name: 'Enable' });
    await user.click(enableButton);

    // Wait for the API call to complete and UI to update
    await waitFor(() => {
      expect(toastMock).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Success',
          description: 'Toolset enabled for server',
        })
      );
    });
  });
});

describe('ToolsetsSetupPage - Auto-enable on API Key Entry', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_admin' }, { stub: true }),
      ...mockToolsetConfig('builtin-exa-web-search', {
        app_enabled: true, // App is enabled
        config: {
          toolset_id: 'builtin-exa-web-search',
          enabled: false, // User toolset is disabled
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      })
    );
  });

  it('auto-enables toolset toggle when API key is entered', async () => {
    const user = userEvent.setup();
    renderWithSetupProvider(<ToolsetsSetupPage />);
    await waitForPageLoad();

    // Wait for backend state to apply (app_enabled: true)
    await waitFor(() => {
      expect(screen.getByText('Enabled')).toBeInTheDocument();
    });

    // Toolset toggle should be OFF initially
    const toolsetToggle = screen.getByTestId('toolset-enabled-toggle');
    expect(toolsetToggle).not.toBeChecked();

    // Enter API key
    const apiKeyInput = screen.getByTestId('toolset-api-key-input');
    await user.type(apiKeyInput, 'test-api-key');

    // Toolset toggle should now be ON (auto-enabled)
    await waitFor(() => {
      expect(toolsetToggle).toBeChecked();
    });
  });
});

describe('ToolsetsSetupPage - Navigation', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_admin' }, { stub: true }),
      ...mockToolsetConfig('builtin-exa-web-search', {
        app_enabled: false,
        config: {
          toolset_id: 'builtin-exa-web-search',
          enabled: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      })
    );
  });

  it('navigates to browser extension step when skip is clicked', async () => {
    const user = userEvent.setup();

    renderWithSetupProvider(<ToolsetsSetupPage />);
    await waitForPageLoad();

    const skipButton = screen.getByTestId('skip-toolsets-setup');
    await user.click(skipButton);

    expect(pushMock).toHaveBeenCalledWith('/ui/setup/browser-extension');
  });
});

describe('ToolsetsSetupPage - Save Configuration', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_admin' }, { stub: true }),
      ...mockToolsetConfig('builtin-exa-web-search', {
        app_enabled: true, // App is enabled so form is accessible
        config: {
          toolset_id: 'builtin-exa-web-search',
          enabled: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      }),
      ...mockUpdateToolsetConfig('builtin-exa-web-search', {
        app_enabled: true,
        config: {
          toolset_id: 'builtin-exa-web-search',
          enabled: true,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: new Date().toISOString(),
        },
      })
    );
  });

  it('saves configuration and navigates on success', async () => {
    const user = userEvent.setup();

    renderWithSetupProvider(<ToolsetsSetupPage />);
    await waitForPageLoad();

    // Wait for backend state to apply (app_enabled: true)
    await waitFor(() => {
      expect(screen.getByText('Enabled')).toBeInTheDocument();
    });

    // Enter API key (this also auto-enables the toolset)
    const apiKeyInput = screen.getByTestId('toolset-api-key-input');
    await user.type(apiKeyInput, 'test-api-key');

    // Save
    const saveButton = screen.getByTestId('save-toolset-config');
    await user.click(saveButton);

    await waitFor(() => {
      expect(toastMock).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Success',
        })
      );
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/browser-extension');
    });
  });
});

describe('ToolsetsSetupPage - Error Handling', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_admin' }, { stub: true }),
      ...mockToolsetConfig('builtin-exa-web-search', {
        app_enabled: true,
        config: {
          toolset_id: 'builtin-exa-web-search',
          enabled: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      })
    );
  });

  it('shows error toast when save fails', async () => {
    // Override with error response
    server.use(
      ...mockUpdateToolsetConfigError('builtin-exa-web-search', { status: 500, message: 'Internal server error' })
    );

    const user = userEvent.setup();
    renderWithSetupProvider(<ToolsetsSetupPage />);
    await waitForPageLoad();

    // Wait for backend state to apply
    await waitFor(() => {
      expect(screen.getByText('Enabled')).toBeInTheDocument();
    });

    // Enter API key
    const apiKeyInput = screen.getByTestId('toolset-api-key-input');
    await user.type(apiKeyInput, 'test-api-key');

    // Save
    const saveButton = screen.getByTestId('save-toolset-config');
    await user.click(saveButton);

    await waitFor(() => {
      expect(toastMock).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Error',
          variant: 'destructive',
        })
      );
    });
  });
});

describe('ToolsetsSetupPage - Backend State Application', () => {
  it('applies backend state when fetched, overwriting local state', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_admin' }, { stub: true }),
      ...mockToolsetConfig('builtin-exa-web-search', {
        app_enabled: true, // Backend says enabled
        config: {
          toolset_id: 'builtin-exa-web-search',
          enabled: true,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      })
    );

    renderWithSetupProvider(<ToolsetsSetupPage />);
    await waitForPageLoad();

    // Initially defaults to OFF, but backend state will apply
    // Wait for backend state to be applied
    await waitFor(() => {
      expect(screen.getByTestId('app-enabled-toggle')).toBeChecked();
    });

    // Badge should show "Enabled"
    expect(screen.getByText('Enabled')).toBeInTheDocument();
  });
});
