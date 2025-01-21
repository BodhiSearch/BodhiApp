import { render, screen } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import SettingsPage from './page';
import { ENDPOINT_SETTINGS } from '@/hooks/useQuery';
import { Setting } from '@/types/models';
import { createWrapper } from '@/tests/wrapper';
import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest';

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

describe('SettingsPage', () => {
  it('shows loading skeleton initially', () => {
    const server = setupServer(
      rest.get(`*${ENDPOINT_SETTINGS}`, (_, res, ctx) => {
        return res(ctx.delay(100), ctx.status(200), ctx.json(mockSettings));
      })
    );

    render(<SettingsPage />, { wrapper: createWrapper() });

    // Check for skeletons
    expect(screen.getAllByTestId('settings-skeleton')).toHaveLength(5); // 5 setting groups
  });

  it('shows error when api fails', async () => {
    server.use(
      rest.get(`*${ENDPOINT_SETTINGS}`, (_, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: 'Internal Server Error',
            message: 'Failed to fetch settings'
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

    // Check group titles are present
    expect(screen.getByText('App Configuration')).toBeInTheDocument();
    expect(screen.getByText('Logging Configuration')).toBeInTheDocument();
    expect(screen.getByText('Server Configuration')).toBeInTheDocument();

    // Check setting values are displayed
    expect(screen.getByText('BODHI_HOME')).toBeInTheDocument();
    expect(screen.getByText('Current Value: /home/user/.bodhi')).toBeInTheDocument();

    expect(screen.getByText('BODHI_LOG_LEVEL')).toBeInTheDocument();
    expect(screen.getByText('Current Value: info')).toBeInTheDocument();
    expect(screen.getByText('Default Value: warn')).toBeInTheDocument();

    expect(screen.getByText('BODHI_PORT')).toBeInTheDocument();
    expect(screen.getByText('Current Value: 1135')).toBeInTheDocument();
  });

  it('shows setting source', async () => {
    render(<SettingsPage />, { wrapper: createWrapper() });

    // Wait for data to load
    await screen.findByText('Application Settings');

    // Check sources are displayed
    // expect(screen.getByText('Source: default')).toBeInTheDocument();
    expect(screen.getByText('Source: settings_file')).toBeInTheDocument();
  });
}); 