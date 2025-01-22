import { render, screen, waitFor } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import SettingsPage, { SettingsPageContent } from './page';
import { ENDPOINT_SETTINGS } from '@/hooks/useQuery';
import { Setting } from '@/types/models';
import { createWrapper } from '@/tests/wrapper';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import userEvent from '@testing-library/user-event';
import { FileText, Settings, Terminal } from 'lucide-react';

// Mock EditSettingDialog component
vi.mock('./components/EditSettingDialog', () => ({
  EditSettingDialog: ({ setting, open, onOpenChange }: any) => (
    open ? (
      <div role="dialog" data-testid="mock-edit-dialog">
        <span>Editing: {setting.key}</span>
        <button onClick={() => onOpenChange(false)}>Close</button>
      </div>
    ) : null
  )
}));

const mockSettings: Setting[] = [
  {
    key: 'BODHI_HOME',
    current_value: '/home/user/.bodhi',
    default_value: '/home/user/.bodhi',
    source: 'default',
    metadata: {
      type: 'string'
    }
  },
  {
    key: 'BODHI_LOG_LEVEL',
    current_value: 'info',
    default_value: 'warn',
    source: 'settings_file',
    metadata: {
      type: 'option',
      options: ['error', 'warn', 'info', 'debug', 'trace']
    }
  },
  {
    key: 'BODHI_PORT',
    current_value: 1135,
    default_value: 1135,
    source: 'default',
    metadata: {
      type: 'number',
      range: {
        min: 1025,
        max: 65535
      }
    }
  },
  {
    key: 'BODHI_EXEC_PATH',
    current_value: '/usr/local/bin/llama',
    default_value: '/usr/bin/llama',
    source: 'settings_file',
    metadata: {
      type: 'string'
    }
  }
];

const server = setupServer(
  rest.get(`*${ENDPOINT_SETTINGS}`, (_, res, ctx) => {
    return res(ctx.status(200), ctx.json(mockSettings));
  })
);

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());

const TEST_CONFIG = {
  app: {
    title: 'Test App Config',
    description: 'Test app settings',
    icon: Settings,
    settings: [
      {
        key: 'BODHI_HOME',
        editable: false,
      }
    ]
  },
  logging: {
    title: 'Test Logging Config',
    description: 'Test logging settings',
    icon: FileText,
    settings: [
      {
        key: 'BODHI_LOG_LEVEL',
        editable: false,
      }
    ]
  },
  execution: {
    title: 'Test Execution Config',
    description: 'Test execution settings',
    icon: Terminal,
    settings: [
      {
        key: 'BODHI_EXEC_PATH',
        editable: true,
      }
    ]
  }
};

describe('SettingsPageContent', () => {
  it('shows loading skeleton with correct groups', () => {
    server.use(
      rest.get(`*${ENDPOINT_SETTINGS}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json([]));
      })
    );
    
    render(
      <SettingsPageContent config={TEST_CONFIG} />,
      { wrapper: createWrapper() }
    );
    expect(screen.getAllByTestId('settings-skeleton')).toHaveLength(3);
  });

  it('shows error message', async () => {
    server.use(
      rest.get(`*${ENDPOINT_SETTINGS}`, (_, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: {
              message: 'Test error'
            }
          })
        );
      })
    );

    render(
      <SettingsPageContent config={TEST_CONFIG} />,
      { wrapper: createWrapper() }
    );
    
    // Wait for the error message to appear
    const errorMessage = await screen.findByText('Failed to load settings: Test error');
    expect(errorMessage).toBeInTheDocument();
  });

  it('renders settings with test configuration', async () => {
    server.use(
      rest.get(`*${ENDPOINT_SETTINGS}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json(mockSettings));
      })
    );

    render(
      <SettingsPageContent config={TEST_CONFIG} />,
      { wrapper: createWrapper() }
    );

    await screen.findByText('Test App Config');
    expect(screen.getByText('Test Logging Config')).toBeInTheDocument();
    expect(screen.getByText('Test Execution Config')).toBeInTheDocument();
  });
});

describe('SettingsPage', () => {
  it('shows loading skeleton initially', () => {
    render(<SettingsPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('settings-skeleton')).toHaveLength(5); // 5 setting groups
  });

  it('shows error when api fails', async () => {
    server.use(
      rest.get(`*${ENDPOINT_SETTINGS}`, (_, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: { message: 'Failed to fetch settings' }
          })
        );
      })
    );

    render(<SettingsPage />, { wrapper: createWrapper() });
    expect(await screen.findByText(/Failed to load settings/)).toBeInTheDocument();
  });

  it('displays settings grouped by category', async () => {
    render(<SettingsPage />, { wrapper: createWrapper() });

    // Wait for data to load
    await screen.findByText('Application Settings');

    // Check group titles
    expect(screen.getByText('App Configuration')).toBeInTheDocument();
    expect(screen.getByText('Logging Configuration')).toBeInTheDocument();
    expect(screen.getByText('Server Configuration')).toBeInTheDocument();

    // Check setting values
    expect(screen.getByText('BODHI_HOME')).toBeInTheDocument();
    expect(screen.getByText(/Current: \/home\/user\/.bodhi/)).toBeInTheDocument();
  });

  it('shows setting source badges', async () => {
    render(<SettingsPage />, { wrapper: createWrapper() });
    await screen.findByText('Application Settings');

    // Use getAllByText for badges since there might be multiple
    const defaultBadges = screen.getAllByText('default');
    const settingsFileBadge = screen.getAllByText('settings_file');

    // Verify we have the correct number of default badges
    expect(defaultBadges).toHaveLength(2); // Based on our mockSettings
    expect(settingsFileBadge).toHaveLength(2); // Based on our mockSettings
  });

  it('shows edit button only for BODHI_EXEC_PATH', async () => {
    render(<SettingsPage />, { wrapper: createWrapper() });
    await screen.findByText('BODHI_EXEC_PATH');

    const editButtons = screen.getAllByRole('button', { name: /edit setting/i });
    expect(editButtons).toHaveLength(1);
  });

  it('opens and closes edit dialog', async () => {
    const user = userEvent.setup();
    render(<SettingsPage />, { wrapper: createWrapper() });

    // Wait for content and click edit
    await screen.findByText('BODHI_EXEC_PATH');
    await user.click(screen.getByRole('button', { name: /edit setting/i }));

    // Verify dialog opens with correct setting
    const dialog = screen.getByTestId('mock-edit-dialog');
    expect(dialog).toBeInTheDocument();
    expect(screen.getByText('Editing: BODHI_EXEC_PATH')).toBeInTheDocument();

    // Close dialog
    await user.click(screen.getByRole('button', { name: /close/i }));
    expect(dialog).not.toBeInTheDocument();
  });
}); 