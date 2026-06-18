import { Route as SettingsRoute } from '@/routes/settings/index';
import { ShellSlotsProvider, useShellSlots } from '@/components/shell';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { mockSettings, mockSettingsDefault, mockUpdateSetting } from '@/test-utils/msw-v2/handlers/settings';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    useNavigate: () => vi.fn(),
    useLocation: () => ({ pathname: '/settings' }),
  };
});

vi.mock('@/hooks/use-toast-messages', () => ({
  useToastMessages: () => ({ showSuccess: vi.fn(), showError: vi.fn() }),
}));

setupMswV2();

const SettingsPage = SettingsRoute.options.component as React.ComponentType;

// Mirror the root shell: render the published sidebar/header/rail slots so we can assert them.
function SlotsConsumer() {
  const { sidebar, headerActions, rail, railHeader } = useShellSlots();
  return (
    <>
      <div data-testid="harness-sidebar">{sidebar}</div>
      <div data-testid="harness-header-actions">{headerActions}</div>
      <div data-testid="harness-rail-header">{railHeader}</div>
      <div data-testid="harness-rail">{rail}</div>
    </>
  );
}

beforeEach(() => {
  server.use(
    ...mockAppInfoReady(),
    ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin' }),
    ...mockSettingsDefault()
  );
});

afterEach(() => {
  localStorage.clear();
  vi.clearAllMocks();
});

async function renderReady() {
  await act(async () => {
    render(
      <ShellSlotsProvider>
        <SlotsConsumer />
        <SettingsPage />
      </ShellSlotsProvider>,
      { wrapper: createWrapper() }
    );
  });
  await waitFor(() => {
    expect(screen.getByTestId('settings-page')).toHaveAttribute('data-pagestatus', 'ready');
  });
}

describe('SettingsPage V2', () => {
  it('shows shimmer filter badges while the settings query is pending', async () => {
    server.use(...mockSettings([], { delayMs: 200, stub: true }));

    render(
      <ShellSlotsProvider>
        <SlotsConsumer />
        <SettingsPage />
      </ShellSlotsProvider>,
      { wrapper: createWrapper() }
    );

    await waitFor(() => {
      expect(screen.getByTestId('settings-filter-all')).toBeInTheDocument();
    });
    expect(screen.getByTestId('settings-page')).toHaveAttribute('data-pagestatus', 'loading');
    expect(screen.getAllByLabelText('Loading count').length).toBeGreaterThan(0);

    await waitFor(() => {
      expect(screen.getByTestId('settings-page')).toHaveAttribute('data-pagestatus', 'ready');
    });
    expect(screen.queryByLabelText('Loading count')).not.toBeInTheDocument();
  });

  it('publishes the settings-group nav to the shell sidebar slot', async () => {
    await renderReady();
    const nav = within(screen.getByTestId('harness-sidebar')).getByTestId('settings-group-nav');
    expect(nav).toBeInTheDocument();
    // Groups derive from the static config; App Config holds BODHI_HOME.
    expect(within(nav).getByTestId('settings-group-app')).toBeInTheDocument();
  });

  it('renders settings rows with source badges', async () => {
    await renderReady();
    expect(screen.getByTestId('setting-row-BODHI_HOME')).toBeInTheDocument();
    expect(screen.getByTestId('setting-source-BODHI_HOME')).toHaveTextContent('default');
  });

  it('filters by Modified (current ≠ default)', async () => {
    const user = userEvent.setup();
    await renderReady();
    // BODHI_LOG_LEVEL is modified (info ≠ warn); BODHI_HOME is not.
    await user.click(screen.getByTestId('settings-filter-modified'));
    expect(screen.getByTestId('setting-row-BODHI_LOG_LEVEL')).toBeInTheDocument();
    expect(screen.queryByTestId('setting-row-BODHI_HOME')).not.toBeInTheDocument();
  });

  it('opens a read-only rail (no editor) for a non-editable setting', async () => {
    const user = userEvent.setup();
    await renderReady();
    await user.click(screen.getByTestId('setting-row-BODHI_HOME'));
    const rail = within(screen.getByTestId('harness-rail')).getByTestId('setting-detail-BODHI_HOME');
    expect(within(rail).getByTestId('setting-readonly-note')).toBeInTheDocument();
    expect(within(rail).queryByTestId('setting-new-value')).not.toBeInTheDocument();
    expect(within(rail).queryByTestId('setting-save')).not.toBeInTheDocument();
  });

  it('renders each row as an accessible link and activating it opens the rail', async () => {
    const user = userEvent.setup();
    await renderReady();
    const row = screen.getByTestId('setting-row-BODHI_HOME');
    const link = within(row).getByTestId('row-link');
    expect(link.tagName).toBe('A');
    expect(link).toHaveAccessibleName('Open setting BODHI_HOME');
    await user.click(link);
    expect(within(screen.getByTestId('harness-rail')).getByTestId('setting-detail-BODHI_HOME')).toBeInTheDocument();
  });

  it('opens an editable rail with a Save button for an editable setting', async () => {
    const user = userEvent.setup();
    await renderReady();
    await user.click(screen.getByTestId('setting-row-BODHI_EXEC_VARIANT'));
    const rail = within(screen.getByTestId('harness-rail')).getByTestId('setting-detail-BODHI_EXEC_VARIANT');
    expect(within(rail).getByTestId('setting-new-value')).toBeInTheDocument();
    expect(within(rail).getByTestId('setting-save')).toBeInTheDocument();
  });

  it('saves an edited setting via useUpdateSetting', async () => {
    const user = userEvent.setup();
    server.use(...mockUpdateSetting('BODHI_EXEC_VARIANT'));
    await renderReady();

    await user.click(screen.getByTestId('setting-row-BODHI_EXEC_VARIANT'));
    const input = within(screen.getByTestId('harness-rail')).getByTestId('setting-new-value');
    await user.clear(input);
    await user.type(input, 'metal');
    const save = within(screen.getByTestId('harness-rail')).getByTestId('setting-save');
    await waitFor(() => expect(save).toBeEnabled());
    await user.click(save);
    // mutation fires without error; rail stays mounted
    await waitFor(() => expect(screen.getByTestId('setting-detail-BODHI_EXEC_VARIANT')).toBeInTheDocument());
  });
});
