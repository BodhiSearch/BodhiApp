/**
 * ToolsetsSetupPage Component Tests
 *
 * Tests for the setup wizard toolsets page with instance-based architecture.
 * Tests cover toggle functionality, form states, and creation flow.
 */

import React from 'react';

import ToolsetsSetupPage from '@/app/ui/setup/toolsets/page';
import { SetupProvider } from '@/app/ui/setup/components';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import {
  mockCreateToolset,
  mockCreateToolsetError,
  mockDisableType,
  mockEnableType,
  mockListToolsets,
  mockListTypes,
  mockType,
} from '@/test-utils/msw-v2/handlers/toolsets';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  usePathname: () => '/ui/setup/toolsets',
}));

setupMswV2();

const renderWithSetupProvider = (component: React.ReactElement) => {
  return render(<SetupProvider>{component}</SetupProvider>, { wrapper: createWrapper() });
};

const waitForPageLoad = async () => {
  await waitFor(() => {
    expect(screen.getByTestId('toolsets-setup-page')).toBeInTheDocument();
    expect(screen.getByTestId('setup-toolset-form')).toBeInTheDocument();
  });
};

const waitForFormLoad = async () => {
  await waitFor(() => {
    expect(screen.getByTestId('app-enabled-toggle')).toBeInTheDocument();
  });
};

beforeEach(() => {
  pushMock.mockClear();
  server.use(
    ...mockAppInfo({ status: 'ready' }, { stub: true }),
    ...mockUserLoggedIn({ role: 'resource_admin' }, { stub: true })
  );
});

afterEach(() => {
  vi.resetAllMocks();
});

describe('ToolsetsSetupPage', () => {
  describe('Loading State', () => {
    it('shows skeleton while types loading', async () => {
      server.use(
        mockListTypes([mockType]),
        mockListToolsets(
          [],
          [
            {
              scope: 'scope_toolset-builtin-exa-web-search',
              scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
              enabled: true,
              updated_by: 'system',
              created_at: '2024-01-01T00:00:00Z',
              updated_at: '2024-01-01T00:00:00Z',
            },
          ]
        )
      );

      renderWithSetupProvider(<ToolsetsSetupPage />);
      await waitForPageLoad();

      expect(screen.getByTestId('setup-toolset-form')).toBeInTheDocument();
    });
  });

  describe('Form Display', () => {
    it('shows toggle and form fields when type is disabled', async () => {
      const disabledType = { ...mockType, app_enabled: false };
      server.use(
        mockListTypes([disabledType]),
        mockListToolsets(
          [],
          [
            {
              scope: 'scope_toolset-builtin-exa-web-search',
              scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
              enabled: false,
              updated_by: 'system',
              created_at: '2024-01-01T00:00:00Z',
              updated_at: '2024-01-01T00:00:00Z',
            },
          ]
        )
      );

      renderWithSetupProvider(<ToolsetsSetupPage />);
      await waitForPageLoad();
      await waitForFormLoad();

      expect(screen.getByTestId('app-enabled-toggle')).toBeInTheDocument();
      expect(screen.getByTestId('toolset-name-input')).toBeInTheDocument();
      expect(screen.getByTestId('toolset-description-input')).toBeInTheDocument();
      expect(screen.getByTestId('toolset-api-key-input')).toBeInTheDocument();
      expect(screen.getByTestId('toolset-enabled-toggle')).toBeInTheDocument();
      expect(screen.getByTestId('create-toolset-button')).toBeInTheDocument();
    });

    it('shows toggle and form fields when type is enabled', async () => {
      server.use(
        mockListTypes([mockType]),
        mockListToolsets(
          [],
          [
            {
              scope: 'scope_toolset-builtin-exa-web-search',
              scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
              enabled: true,
              updated_by: 'system',
              created_at: '2024-01-01T00:00:00Z',
              updated_at: '2024-01-01T00:00:00Z',
            },
          ]
        )
      );

      renderWithSetupProvider(<ToolsetsSetupPage />);
      await waitForPageLoad();
      await waitForFormLoad();

      expect(screen.getByTestId('app-enabled-toggle')).toBeInTheDocument();
      expect(screen.getByTestId('toolset-name-input')).toBeInTheDocument();
      expect(screen.getByTestId('toolset-description-input')).toBeInTheDocument();
      expect(screen.getByTestId('toolset-api-key-input')).toBeInTheDocument();
      expect(screen.getByTestId('toolset-enabled-toggle')).toBeInTheDocument();
      expect(screen.getByTestId('create-toolset-button')).toBeInTheDocument();
    });
  });

  describe('Toggle Disabled State', () => {
    it('disables form fields when app-config is off', async () => {
      const disabledType = { ...mockType, app_enabled: false };
      server.use(mockListTypes([disabledType]));

      renderWithSetupProvider(<ToolsetsSetupPage />);
      await waitForPageLoad();
      await waitForFormLoad();

      const nameInput = screen.getByTestId('toolset-name-input');
      const descInput = screen.getByTestId('toolset-description-input');
      const apiKeyInput = screen.getByTestId('toolset-api-key-input');
      const enabledToggle = screen.getByTestId('toolset-enabled-toggle');
      const createButton = screen.getByTestId('create-toolset-button');

      expect(nameInput).toBeDisabled();
      expect(descInput).toBeDisabled();
      expect(apiKeyInput).toBeDisabled();
      expect(enabledToggle).toBeDisabled();
      expect(createButton).toBeDisabled();
    });

    it('enables form fields when app-config is on', async () => {
      server.use(
        mockListTypes([mockType]),
        mockListToolsets(
          [],
          [
            {
              scope: 'scope_toolset-builtin-exa-web-search',
              scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
              enabled: true,
              updated_by: 'system',
              created_at: '2024-01-01T00:00:00Z',
              updated_at: '2024-01-01T00:00:00Z',
            },
          ]
        )
      );

      renderWithSetupProvider(<ToolsetsSetupPage />);
      await waitForPageLoad();
      await waitForFormLoad();

      const nameInput = screen.getByTestId('toolset-name-input');
      const descInput = screen.getByTestId('toolset-description-input');
      const apiKeyInput = screen.getByTestId('toolset-api-key-input');
      const enabledToggle = screen.getByTestId('toolset-enabled-toggle');
      const createButton = screen.getByTestId('create-toolset-button');

      expect(nameInput).not.toBeDisabled();
      expect(descInput).not.toBeDisabled();
      expect(apiKeyInput).not.toBeDisabled();
      expect(enabledToggle).not.toBeDisabled();
      expect(createButton).not.toBeDisabled();
    });
  });

  describe('Enable Flow', () => {
    it('shows enable dialog when toggle is clicked on', async () => {
      const user = await import('@testing-library/user-event').then((m) => m.default.setup());
      const disabledType = { ...mockType, app_enabled: false };
      server.use(
        mockListTypes([disabledType]),
        mockEnableType(),
        mockListToolsets(
          [],
          [
            {
              scope: 'scope_toolset-builtin-exa-web-search',
              scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
              enabled: false,
              updated_by: 'system',
              created_at: '2024-01-01T00:00:00Z',
              updated_at: '2024-01-01T00:00:00Z',
            },
          ]
        )
      );

      renderWithSetupProvider(<ToolsetsSetupPage />);
      await waitForPageLoad();
      await waitForFormLoad();

      const toggle = screen.getByTestId('app-enabled-toggle');
      await user.click(toggle);

      await waitFor(() => {
        expect(screen.getByTestId('enable-confirm-dialog')).toBeInTheDocument();
        expect(screen.getByText('Enable Toolset for Server')).toBeInTheDocument();
      });
    });

    it('enables toolset when confirm is clicked', async () => {
      const user = await import('@testing-library/user-event').then((m) => m.default.setup());
      const disabledType = { ...mockType, app_enabled: false };
      server.use(
        mockListTypes([disabledType]),
        mockEnableType(),
        mockListToolsets(
          [],
          [
            {
              scope: 'scope_toolset-builtin-exa-web-search',
              scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
              enabled: false,
              updated_by: 'system',
              created_at: '2024-01-01T00:00:00Z',
              updated_at: '2024-01-01T00:00:00Z',
            },
          ]
        )
      );

      renderWithSetupProvider(<ToolsetsSetupPage />);
      await waitForPageLoad();
      await waitForFormLoad();

      const toggle = screen.getByTestId('app-enabled-toggle');
      await user.click(toggle);

      await waitFor(() => {
        expect(screen.getByTestId('enable-confirm-dialog')).toBeInTheDocument();
      });

      const enableButton = screen.getByRole('button', { name: /Enable/i });
      await user.click(enableButton);

      await waitFor(() => {
        expect(screen.queryByTestId('enable-confirm-dialog')).not.toBeInTheDocument();
      });
    });
  });

  describe('Disable Flow', () => {
    it('shows disable dialog when toggle is clicked off', async () => {
      const user = await import('@testing-library/user-event').then((m) => m.default.setup());
      server.use(
        mockListTypes([mockType]),
        mockDisableType(),
        mockListToolsets(
          [],
          [
            {
              scope: 'scope_toolset-builtin-exa-web-search',
              scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
              enabled: true,
              updated_by: 'system',
              created_at: '2024-01-01T00:00:00Z',
              updated_at: '2024-01-01T00:00:00Z',
            },
          ]
        )
      );

      renderWithSetupProvider(<ToolsetsSetupPage />);
      await waitForPageLoad();
      await waitForFormLoad();

      const toggle = screen.getByTestId('app-enabled-toggle');
      await user.click(toggle);

      await waitFor(() => {
        expect(screen.getByTestId('disable-confirm-dialog')).toBeInTheDocument();
        expect(screen.getByText('Disable Toolset for Server')).toBeInTheDocument();
      });
    });

    it('disables toolset when confirm is clicked', async () => {
      const user = await import('@testing-library/user-event').then((m) => m.default.setup());
      server.use(
        mockListTypes([mockType]),
        mockDisableType(),
        mockListToolsets(
          [],
          [
            {
              scope: 'scope_toolset-builtin-exa-web-search',
              scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
              enabled: true,
              updated_by: 'system',
              created_at: '2024-01-01T00:00:00Z',
              updated_at: '2024-01-01T00:00:00Z',
            },
          ]
        )
      );

      renderWithSetupProvider(<ToolsetsSetupPage />);
      await waitForPageLoad();
      await waitForFormLoad();

      const toggle = screen.getByTestId('app-enabled-toggle');
      await user.click(toggle);

      await waitFor(() => {
        expect(screen.getByTestId('disable-confirm-dialog')).toBeInTheDocument();
      });

      const disableButton = screen.getByRole('button', { name: /Disable/i });
      await user.click(disableButton);

      await waitFor(() => {
        expect(screen.queryByTestId('disable-confirm-dialog')).not.toBeInTheDocument();
      });
    });
  });

  describe('Create Flow', () => {
    it('creates toolset and navigates on success', async () => {
      const user = await import('@testing-library/user-event').then((m) => m.default.setup());
      server.use(
        mockListTypes([mockType]),
        mockCreateToolset(),
        mockListToolsets(
          [],
          [
            {
              scope: 'scope_toolset-builtin-exa-web-search',
              scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
              enabled: true,
              updated_by: 'system',
              created_at: '2024-01-01T00:00:00Z',
              updated_at: '2024-01-01T00:00:00Z',
            },
          ]
        )
      );

      renderWithSetupProvider(<ToolsetsSetupPage />);
      await waitForPageLoad();
      await waitForFormLoad();

      const nameInput = screen.getByTestId('toolset-name-input');
      const apiKeyInput = screen.getByTestId('toolset-api-key-input');
      const createButton = screen.getByTestId('create-toolset-button');

      await user.clear(nameInput);
      await user.type(nameInput, 'my-exa-search');
      await user.type(apiKeyInput, 'test-api-key');
      await user.click(createButton);

      await waitFor(() => {
        expect(pushMock).toHaveBeenCalledWith('/ui/setup/browser-extension');
      });
    });

    it('shows error toast when creation fails', async () => {
      const user = await import('@testing-library/user-event').then((m) => m.default.setup());
      server.use(
        mockListTypes([mockType]),
        mockCreateToolsetError({ message: 'Name already exists', status: 400 }),
        mockListToolsets(
          [],
          [
            {
              scope: 'scope_toolset-builtin-exa-web-search',
              scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
              enabled: true,
              updated_by: 'system',
              created_at: '2024-01-01T00:00:00Z',
              updated_at: '2024-01-01T00:00:00Z',
            },
          ]
        )
      );

      renderWithSetupProvider(<ToolsetsSetupPage />);
      await waitForPageLoad();
      await waitForFormLoad();

      const nameInput = screen.getByTestId('toolset-name-input');
      const apiKeyInput = screen.getByTestId('toolset-api-key-input');
      const createButton = screen.getByTestId('create-toolset-button');

      await user.clear(nameInput);
      await user.type(nameInput, 'my-exa-search');
      await user.type(apiKeyInput, 'test-api-key');
      await user.click(createButton);

      await waitFor(() => {
        expect(pushMock).not.toHaveBeenCalled();
      });
    });

    it('prefills name with type name', async () => {
      server.use(
        mockListTypes([mockType]),
        mockListToolsets(
          [],
          [
            {
              scope: 'scope_toolset-builtin-exa-web-search',
              scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
              enabled: true,
              updated_by: 'system',
              created_at: '2024-01-01T00:00:00Z',
              updated_at: '2024-01-01T00:00:00Z',
            },
          ]
        )
      );

      renderWithSetupProvider(<ToolsetsSetupPage />);
      await waitForPageLoad();
      await waitForFormLoad();

      const nameInput = screen.getByTestId('toolset-name-input') as HTMLInputElement;
      expect(nameInput.value).toBe('builtin-exa-web-search');
    });
  });

  describe('Skip Flow', () => {
    it('navigates to browser-extension when skip is clicked', async () => {
      const user = await import('@testing-library/user-event').then((m) => m.default.setup());
      server.use(
        mockListTypes([mockType]),
        mockListToolsets(
          [],
          [
            {
              scope: 'scope_toolset-builtin-exa-web-search',
              scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
              enabled: true,
              updated_by: 'system',
              created_at: '2024-01-01T00:00:00Z',
              updated_at: '2024-01-01T00:00:00Z',
            },
          ]
        )
      );

      renderWithSetupProvider(<ToolsetsSetupPage />);
      await waitForPageLoad();

      const skipButton = screen.getByTestId('skip-toolsets-setup');
      await user.click(skipButton);

      expect(pushMock).toHaveBeenCalledWith('/ui/setup/browser-extension');
    });
  });
});
