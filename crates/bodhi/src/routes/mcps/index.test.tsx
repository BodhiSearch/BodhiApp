import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { ShellSlotsProvider, useShellSlots } from '@/components/shell';
import { myMcpsSearchSchema } from '@/routes/mcps/index';
import { MyMcpsScreen } from '@/routes/mcps/-components/MyMcpsScreen';
import {
  createMockAuthConfigOAuthPreReg,
  createMockMcp,
  createMockMcpServerInfo,
  createMockMcpServerResponse,
} from '@/test-fixtures/mcps';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import {
  mockDeleteMcp,
  mockListAuthConfigs,
  mockListMcps,
  mockListMcpServers,
} from '@/test-utils/msw-v2/handlers/mcps';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { makeRouteRouter, RouteHarness } from '@/test-utils/router-harness';
import { createWrapper } from '@/tests/wrapper';

vi.mock('@/hooks/useViewTransition', () => ({ useViewTransition: () => (cb: () => void) => cb() }));

setupMswV2();

let Wrapper: ReturnType<typeof createWrapper>;

// Two registered servers; only the first has user instances.
const serverA = createMockMcpServerResponse({
  id: 'server-a',
  name: 'Alpha Server',
  url: 'https://alpha.example.com/mcp',
  enabled_mcp_count: 1,
  disabled_mcp_count: 0,
});
const serverB = createMockMcpServerResponse({
  id: 'server-b',
  name: 'Beta Server',
  url: 'https://beta.example.com/mcp',
  enabled_mcp_count: 0,
  disabled_mcp_count: 0,
});
const instanceA = createMockMcp({
  id: 'inst-a',
  name: 'alpha-instance',
  mcp_server: createMockMcpServerInfo({ id: 'server-a', name: 'Alpha Server', url: 'https://alpha.example.com/mcp' }),
});

beforeEach(() => {
  localStorage.clear();
  Wrapper = createWrapper();
  server.use(
    ...mockAppInfoReady(),
    ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin', id_token: 'test-id-token' }),
    mockListMcpServers([serverA, serverB]),
    mockListMcps([instanceA]),
    mockListAuthConfigs({ auth_configs: [createMockAuthConfigOAuthPreReg()] })
  );
});

function SlotsConsumer() {
  const { sidebar, rail, railHeader } = useShellSlots();
  return (
    <>
      <div data-testid="harness-sidebar">{sidebar}</div>
      <div data-testid="harness-rail-header">{railHeader}</div>
      <div data-testid="harness-rail">{rail}</div>
    </>
  );
}

function buildRouter(initialEntries?: string[]) {
  return makeRouteRouter({
    path: '/mcps/',
    validateSearch: myMcpsSearchSchema as never,
    Screen: () => (
      <ShellSlotsProvider>
        <SlotsConsumer />
        <MyMcpsScreen />
      </ShellSlotsProvider>
    ),
    initialEntries,
  });
}

async function renderScreen(initialEntries?: string[]) {
  const router = buildRouter(initialEntries);
  await act(async () => {
    render(<RouteHarness router={router} />, { wrapper: Wrapper });
  });
  await waitFor(() => expect(screen.getByTestId('my-mcps-content')).toHaveAttribute('data-pagestatus', 'ready'));
  return router;
}

describe('MyMcpsScreen — list', () => {
  it('renders registered servers with instance-count status', async () => {
    await renderScreen();
    const list = screen.getByTestId('my-mcps-list');
    expect(within(list).getAllByRole('option').length).toBe(2);
    expect(screen.getByTestId('my-mcps-row-server-a')).toHaveTextContent('Alpha Server');
    expect(screen.getByTestId('my-mcps-status-server-a')).toHaveTextContent('1 instance');
    expect(screen.getByTestId('my-mcps-status-server-b')).toHaveTextContent('Available');
  });

  it('filters by search over name/url', async () => {
    const user = userEvent.setup();
    await renderScreen();
    await user.type(screen.getByTestId('my-mcps-search').querySelector('input')!, 'beta{Enter}');
    await waitFor(() => expect(screen.queryByTestId('my-mcps-row-server-a')).not.toBeInTheDocument());
    expect(screen.getByTestId('my-mcps-row-server-b')).toBeInTheDocument();
  });

  it('Connected scope filter shows only servers with instances', async () => {
    const user = userEvent.setup();
    await renderScreen();
    await user.click(screen.getByTestId('my-mcps-scope-mine'));
    await waitFor(() => expect(screen.queryByTestId('my-mcps-row-server-b')).not.toBeInTheDocument());
    expect(screen.getByTestId('my-mcps-row-server-a')).toBeInTheDocument();
  });
});

describe('MyMcpsScreen — detail rail', () => {
  it('opens the rail on row select with My Instances + Connect with', async () => {
    const user = userEvent.setup();
    await renderScreen();
    await user.click(screen.getByTestId('my-mcps-row-server-a'));

    await waitFor(() => expect(screen.getByTestId('my-mcps-detail-server-a')).toBeInTheDocument());
    // My Instances lists the user's instance with play/edit/delete.
    expect(screen.getByTestId('my-mcps-instance-inst-a')).toHaveTextContent('alpha-instance');
    expect(screen.getByTestId('my-mcps-instance-play-inst-a')).toBeInTheDocument();
    // Connect with lists the configured oauth mechanism + synthetic public.
    await waitFor(() => expect(screen.getByTestId('my-mcps-detail-mechanisms')).toBeInTheDocument());
    expect(screen.getByTestId('my-mcps-connect-public')).toBeInTheDocument();
    expect(screen.getByTestId(`my-mcps-connect-${createMockAuthConfigOAuthPreReg().id}`)).toBeInTheDocument();
  });

  it('admin sees Configure server; deep-links carry server + auth', async () => {
    const user = userEvent.setup();
    await renderScreen();
    await user.click(screen.getByTestId('my-mcps-row-server-a'));

    await waitFor(() => expect(screen.getByTestId('my-mcps-configure-server')).toBeInTheDocument());
    // The RouteHarness router has no /ui basepath; the app prepends it at runtime via <Link>.
    expect(screen.getByTestId('my-mcps-configure-server')).toHaveAttribute('href', '/mcps/servers/view/?id=server-a');
    expect(screen.getByTestId('my-mcps-connect-public')).toHaveAttribute(
      'href',
      '/mcps/new/?server=server-a&auth=public'
    );
  });

  it('deletes an instance via confirm dialog', async () => {
    let deleted = false;
    server.use(mockDeleteMcp());
    const user = userEvent.setup();
    await renderScreen();
    await user.click(screen.getByTestId('my-mcps-row-server-a'));
    await waitFor(() => expect(screen.getByTestId('my-mcps-instance-delete-inst-a')).toBeInTheDocument());
    await user.click(screen.getByTestId('my-mcps-instance-delete-inst-a'));

    await waitFor(() => expect(screen.getByTestId('my-mcps-delete-dialog')).toBeInTheDocument());
    server.use(mockDeleteMcp(), mockListMcps([]));
    await user.click(screen.getByRole('button', { name: /^Delete$/ }));
    await waitFor(() => expect(screen.queryByTestId('my-mcps-delete-dialog')).not.toBeInTheDocument());
    deleted = true;
    expect(deleted).toBe(true);
  });
});

describe('MyMcpsScreen — role gating', () => {
  it('non-admin does not see Configure server', async () => {
    server.use(...mockUserLoggedIn({ username: 'u@example.com', role: 'resource_user', id_token: 'test-id-token' }));
    const user = userEvent.setup();
    await renderScreen();
    await user.click(screen.getByTestId('my-mcps-row-server-a'));
    await waitFor(() => expect(screen.getByTestId('my-mcps-detail-server-a')).toBeInTheDocument());
    expect(screen.queryByTestId('my-mcps-configure-server')).not.toBeInTheDocument();
  });
});
