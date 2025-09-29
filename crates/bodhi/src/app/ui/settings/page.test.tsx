import { act, render, screen } from '@testing-library/react';
import { server } from '@/test-utils/msw-v2/setup';
import {
  mockSettings,
  mockSettingsDefault,
  mockSettingsEmpty,
  mockSettingsInternalError,
} from '@/test-utils/msw-v2/handlers/settings';
import { SettingsPageContent } from '@/app/ui/settings/page';
import { ENDPOINT_SETTINGS } from '@/hooks/useSettings';
import { SettingInfo } from '@bodhiapp/ts-client';
import { createWrapper } from '@/tests/wrapper';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import userEvent from '@testing-library/user-event';
import { FileText, Settings, Terminal } from 'lucide-react';

// Mock EditSettingDialog component
vi.mock('@/app/ui/settings/EditSettingDialog', () => ({
  EditSettingDialog: ({ setting, open, onOpenChange }: any) =>
    open ? (
      <div role="dialog" data-testid="mock-edit-dialog">
        <span>Editing: {setting.key}</span>
        <button onClick={() => onOpenChange(false)}>Close</button>
      </div>
    ) : null,
}));

const mockSettingInfos: SettingInfo[] = [
  {
    key: 'BODHI_HOME',
    current_value: '/home/user/.bodhi',
    default_value: '/home/user/.cache/bodhi',
    source: 'default',
    metadata: {
      type: 'string',
    },
  },
  {
    key: 'BODHI_LOG_LEVEL',
    current_value: 'info',
    default_value: 'warn',
    source: 'settings_file',
    metadata: {
      type: 'option',
      options: ['error', 'warn', 'info', 'debug', 'trace'],
    },
  },
  {
    key: 'BODHI_PORT',
    current_value: 1135,
    default_value: 1135,
    source: 'default',
    metadata: {
      type: 'number',
      min: 1025,
      max: 65535,
    },
  },
  {
    key: 'BODHI_EXEC_VARIANT',
    current_value: 'cpu',
    default_value: 'metal',
    source: 'settings_file',
    metadata: {
      type: 'string',
    },
  },
];

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());

const TEST_CONFIG = {
  app: {
    title: 'Test App Config',
    description: 'Test app settings description',
    icon: Settings,
    settings: [
      {
        key: 'BODHI_HOME',
        editable: false,
        description: 'Test home directory',
      },
      {
        key: 'BODHI_PORT',
        editable: false,
        description: 'Server Port',
      },
    ],
  },
  logging: {
    title: 'Test Logging Config',
    description: 'Test logging settings description',
    icon: FileText,
    settings: [
      {
        key: 'BODHI_LOG_LEVEL',
        editable: false,
        description: 'Test log level',
      },
    ],
  },
  execution: {
    title: 'Test Execution Config',
    description: 'Test execution settings description',
    icon: Terminal,
    settings: [
      {
        key: 'BODHI_EXEC_VARIANT',
        editable: true,
        description: 'Test execution path',
      },
    ],
  },
};

describe('SettingsPageContent', () => {
  it('shows loading skeleton with correct groups', () => {
    server.use(...mockSettingsEmpty());

    render(<SettingsPageContent config={TEST_CONFIG} />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('settings-skeleton')).toHaveLength(3);
  });

  it('shows error message', async () => {
    server.use(...mockSettingsInternalError());

    render(<SettingsPageContent config={TEST_CONFIG} />, { wrapper: createWrapper() });

    // Wait for the error message to appear
    const errorMessage = await screen.findByText('Test Error');
    expect(errorMessage).toBeInTheDocument();
  });

  it('renders settings with test configuration', async () => {
    server.use(...mockSettingsDefault());

    render(<SettingsPageContent config={TEST_CONFIG} />, { wrapper: createWrapper() });

    await screen.findByText('Test App Config');
    expect(screen.getByText('Test Logging Config')).toBeInTheDocument();
    expect(screen.getByText('Test Execution Config')).toBeInTheDocument();
  });
});

describe('SettingsPage', () => {
  it('shows loading skeleton initially', () => {
    server.use(...mockSettingsEmpty());

    render(<SettingsPageContent config={TEST_CONFIG} />, {
      wrapper: createWrapper(),
    });
    expect(screen.getAllByTestId('settings-skeleton')).toHaveLength(3); // 3 setting groups
  });

  it('shows error when api fails', async () => {
    server.use(...mockSettingsInternalError());

    render(<SettingsPageContent config={TEST_CONFIG} />, {
      wrapper: createWrapper(),
    });
    expect(await screen.findByText(/Test Error/)).toBeInTheDocument();
  });

  it('displays settings grouped by category', async () => {
    server.use(...mockSettingsDefault());

    await act(async () => {
      render(<SettingsPageContent config={TEST_CONFIG} />, {
        wrapper: createWrapper(),
      });
    });

    // Check group titles
    expect(screen.getByText('Test App Config')).toBeInTheDocument();
    expect(screen.getByText('Test Logging Config')).toBeInTheDocument();
    expect(screen.getByText('Test Execution Config')).toBeInTheDocument();

    // Check setting values
    expect(screen.getByText('BODHI_HOME')).toBeInTheDocument();
    expect(screen.getByText(/\/home\/user\/.bodhi/)).toBeInTheDocument();
  });

  it('shows setting source badges', async () => {
    server.use(...mockSettingsDefault());

    await act(async () => {
      render(<SettingsPageContent config={TEST_CONFIG} />, {
        wrapper: createWrapper(),
      });
    });

    // Use getAllByText for badges since there might be multiple
    const defaultBadges = screen.getAllByText('default');
    const settingsFileBadge = screen.getAllByText('settings_file');

    // Verify we have the correct number of default badges
    expect(defaultBadges).toHaveLength(2); // Based on our mockSettingInfos
    expect(settingsFileBadge).toHaveLength(2); // Based on our mockSettingInfos
  });

  it('shows edit button only for BODHI_EXEC_VARIANT', async () => {
    server.use(...mockSettingsDefault());

    render(<SettingsPageContent config={TEST_CONFIG} />, {
      wrapper: createWrapper(),
    });
    await screen.findByText('BODHI_EXEC_VARIANT');

    const editButtons = screen.getAllByRole('button', {
      name: /edit setting/i,
    });
    expect(editButtons).toHaveLength(1);
  });

  it('opens and closes edit dialog', async () => {
    server.use(...mockSettingsDefault());

    const user = userEvent.setup();
    render(<SettingsPageContent config={TEST_CONFIG} />, {
      wrapper: createWrapper(),
    });

    // Wait for content and click edit
    await screen.findByText('BODHI_EXEC_VARIANT');
    await user.click(screen.getByRole('button', { name: /edit setting/i }));

    // Verify dialog opens with correct setting
    const dialog = screen.getByTestId('mock-edit-dialog');
    expect(dialog).toBeInTheDocument();
    expect(screen.getByText('Editing: BODHI_EXEC_VARIANT')).toBeInTheDocument();

    // Close dialog
    await user.click(screen.getByRole('button', { name: /close/i }));
    expect(dialog).not.toBeInTheDocument();
  });
});
