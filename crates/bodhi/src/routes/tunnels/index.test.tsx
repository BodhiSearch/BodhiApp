import { Route as TunnelsRoute } from '@/routes/tunnels/index';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import {
  liveTunnelStatus,
  mockDisableTunnel,
  mockEnableTunnel,
  mockTunnelStatus,
  mockTunnelStatusExact,
  mockTunnelSync,
} from '@/test-utils/msw-v2/handlers/tunnels';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { ShellHarness } from '@/test-utils/shell-harness';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    useNavigate: () => vi.fn(),
    useLocation: () => ({ pathname: '/tunnels' }),
  };
});

vi.mock('@/hooks/useToastMessages', () => ({
  useToastMessages: () => ({ showSuccess: vi.fn(), showError: vi.fn() }),
}));

setupMswV2();

const TunnelsPage = TunnelsRoute.options.component as React.ComponentType;

beforeEach(() => {
  server.use(...mockAppInfoReady(), ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin' }));
});

afterEach(() => {
  localStorage.clear();
  vi.clearAllMocks();
});

async function renderPage() {
  await act(async () => {
    render(
      <ShellHarness>
        <TunnelsPage />
      </ShellHarness>,
      { wrapper: createWrapper() }
    );
  });
  await waitFor(() => {
    expect(screen.getByTestId('tunnels-page')).toHaveAttribute('data-pagestatus', 'ready');
  });
  return screen.getByTestId('tunnels-page');
}

describe('Remote Access page', () => {
  it('shows the unavailable card instead of the ladder', async () => {
    server.use(...mockTunnelStatus({ available: false, unavailable_reason: 'Remote access is disabled here.' }));
    const page = await renderPage();

    expect(page).toHaveAttribute('data-test-state', 'unavailable');
    expect(within(page).getByTestId('tunnel-unavailable')).toHaveTextContent('Remote access is disabled here.');
    expect(within(page).queryByTestId('tunnel-step-1')).not.toBeInTheDocument();
  });

  it('holds steps 2 and 3 in a waiting state until cloudflared is found', async () => {
    server.use(
      ...mockTunnelStatus({
        binary: { state: 'missing', path: null, source: null, version: null, minimum_version: '2025.2.0', error: null },
      })
    );
    const page = await renderPage();

    expect(page).toHaveAttribute('data-test-state', 'binary-missing');
    expect(within(page).getByTestId('tunnel-step-1-pill')).toHaveTextContent('Not found');
    expect(within(page).getByTestId('tunnel-step-2-pill')).toHaveTextContent('Waiting');
    expect(within(page).getByTestId('tunnel-step-3-pill')).toHaveTextContent('Waiting');
    expect(within(page).queryByTestId('tunnel-create')).not.toBeInTheDocument();
  });

  it('gives the terminal command when Cloudflare sign-in is missing', async () => {
    server.use(
      ...mockTunnelStatus({
        login: { state: 'missing', cert_path: null, source: null, zone: null, error: null },
      })
    );
    const page = await renderPage();

    expect(page).toHaveAttribute('data-test-state', 'login-needed');
    expect(within(page).getByTestId('tunnel-step-2-pill')).toHaveTextContent('Signed out');
    expect(within(page).getByTestId('tunnel-login-command')).toHaveTextContent('cloudflared tunnel login');
  });

  it('refuses a dotted subdomain before asking for confirmation', async () => {
    const user = userEvent.setup();
    server.use(...mockTunnelStatus());
    const page = await renderPage();

    await user.type(within(page).getByTestId('tunnel-subdomain'), 'my.bodhi');
    await user.click(within(page).getByTestId('tunnel-create'));

    expect(within(page).getByTestId('tunnel-subdomain-error')).toHaveTextContent(/no dots/i);
    expect(within(page).queryByTestId('tunnel-confirm')).not.toBeInTheDocument();
  });

  it('confirms exposure inline, then creates the tunnel', async () => {
    const user = userEvent.setup();
    const bodies: unknown[] = [];
    server.use(...mockTunnelStatus(), ...mockEnableTunnel(undefined, { onBody: (body) => bodies.push(body) }));
    const page = await renderPage();

    await user.type(within(page).getByTestId('tunnel-subdomain'), 'bodhi');
    await user.click(within(page).getByTestId('tunnel-create'));

    const confirm = within(page).getByTestId('tunnel-confirm');
    expect(confirm).toHaveTextContent(/opens the whole instance to the internet/i);
    expect(confirm).toHaveTextContent('https://bodhi.example.com');

    await user.click(within(page).getByTestId('tunnel-confirm-create'));

    await waitFor(() => expect(bodies).toHaveLength(1));
    expect(bodies[0]).toMatchObject({ subdomain: 'bodhi', replace_dns: false });
  });

  it('can back out of the exposure confirmation', async () => {
    const user = userEvent.setup();
    server.use(...mockTunnelStatus());
    const page = await renderPage();

    await user.type(within(page).getByTestId('tunnel-subdomain'), 'bodhi');
    await user.click(within(page).getByTestId('tunnel-create'));
    await user.click(within(page).getByTestId('tunnel-cancel-create'));

    expect(within(page).queryByTestId('tunnel-confirm')).not.toBeInTheDocument();
    expect(within(page).getByTestId('tunnel-create')).toBeInTheDocument();
  });

  it('shows creation progress and locks the address while connecting', async () => {
    server.use(...mockTunnelStatus({ state: 'connecting', enabled: true, subdomain: 'bodhi' }));
    const page = await renderPage();

    expect(page).toHaveAttribute('data-test-state', 'creating');
    expect(within(page).getByTestId('tunnel-progress')).toBeInTheDocument();
    expect(within(page).getByTestId('tunnel-subdomain')).toBeDisabled();
    expect(within(page).getByTestId('tunnel-step-3-pill')).toHaveTextContent('Creating');
  });

  it('offers an inline DNS replacement rather than a modal', async () => {
    const user = userEvent.setup();
    const bodies: unknown[] = [];
    server.use(
      ...mockTunnelStatus({
        error_code: 'dns_conflict',
        error_message: 'A DNS record already exists for this address.',
        subdomain: 'bodhi',
      }),
      ...mockEnableTunnel(undefined, { onBody: (body) => bodies.push(body) })
    );
    const page = await renderPage();

    expect(page).toHaveAttribute('data-test-state', 'dns-conflict');
    const panel = within(page).getByTestId('tunnel-dns-conflict');
    expect(panel).toHaveTextContent(/already answers at/i);

    await user.click(within(page).getByTestId('tunnel-replace-dns'));
    await waitFor(() => expect(bodies).toHaveLength(1));
    expect(bodies[0]).toMatchObject({ replace_dns: true });
  });

  it('folds and locks the finished checks once the tunnel is live', async () => {
    server.use(...mockTunnelStatusExact(liveTunnelStatus()));
    const page = await renderPage();

    expect(page).toHaveAttribute('data-test-state', 'live');
    expect(within(page).getByTestId('tunnel-step-1')).toHaveAttribute('data-test-folded', 'true');
    expect(within(page).getByTestId('tunnel-step-2')).toHaveAttribute('data-test-folded', 'true');
    expect(within(page).getByTestId('tunnel-step-1-unfold')).toBeDisabled();
    expect(within(page).getAllByTestId('tunnel-step-locked').length).toBe(2);
    expect(within(page).getByTestId('tunnel-subdomain')).toBeDisabled();
    expect(within(page).getByTestId('tunnel-open')).toHaveAttribute('href', 'https://bodhi.example.com');
    expect(within(page).getByTestId('tunnel-turn-off')).toBeInTheDocument();
    expect(within(page).getByTestId('tunnel-overall-pill')).toHaveTextContent('On');
  });

  it('keeps the tunnel marked on when Keycloak was unreachable, and offers a retry', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockTunnelStatusExact(liveTunnelStatus({ auth_sync: { state: 'unreachable', error: 'connection refused' } })),
      ...mockTunnelSync()
    );
    const page = await renderPage();

    expect(page).toHaveAttribute('data-test-state', 'kc-fail-net');
    expect(within(page).getByTestId('tunnel-step-3-pill')).toHaveTextContent('On · sign-in broken');
    expect(within(page).getByTestId('tunnel-kc-note')).toHaveTextContent(/couldn’t reach keycloak/i);
    expect(within(page).getByTestId('tunnel-kc-note')).toHaveTextContent(/API-key requests keep working/i);

    await user.click(within(page).getByTestId('tunnel-retry-sync'));
    await waitFor(() => expect(screen.getByTestId('tunnels-page')).toHaveAttribute('data-test-state', 'live'));
  });

  it('opens the rail on the answer that explains the error being shown', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockTunnelStatusExact(
        liveTunnelStatus({
          state: 'disabled',
          enabled: false,
          error_code: 'dns_conflict',
          auth_sync: { state: 'not_attempted', error: null },
        })
      )
    );
    const page = await renderPage();
    expect(page).toHaveAttribute('data-test-state', 'dns-conflict');

    const rail = screen.getByTestId('harness-rail');
    expect(within(rail).getByTestId('faq-entry-faq-dns')).toHaveAttribute('data-open', 'false');

    await user.click(within(page).getByTestId('faq-link-faq-dns'));

    // The answer for *this* error is open; the rest of the list stays shut.
    expect(within(rail).getByTestId('faq-entry-faq-dns')).toHaveAttribute('data-open', 'true');
    expect(within(rail).getByTestId('faq-entry-faq-install')).toHaveAttribute('data-open', 'false');
  });

  it('publishes the whole FAQ into the shell rail, collapsed', async () => {
    server.use(...mockTunnelStatusExact(liveTunnelStatus()));
    await renderPage();

    const rail = screen.getByTestId('harness-rail');
    expect(within(screen.getByTestId('harness-rail-header')).getByTestId('faq-rail-close')).toBeInTheDocument();
    // A representative id from each group, so a dropped group is caught.
    for (const id of ['faq-install', 'faq-signin', 'faq-kcsync', 'faq-exposure', 'faq-timeout', 'faq-unavailable']) {
      expect(within(rail).getByTestId(`faq-entry-${id}`)).toHaveAttribute('data-open', 'false');
    }
  });

  it('says something different when Keycloak answered and refused', async () => {
    server.use(...mockTunnelStatusExact(liveTunnelStatus({ auth_sync: { state: 'rejected', error: 'forbidden' } })));
    const page = await renderPage();

    expect(page).toHaveAttribute('data-test-state', 'kc-fail-auth');
    const note = within(page).getByTestId('tunnel-kc-note');
    expect(note).toHaveTextContent(/refused the redirect URL change/i);
    expect(note).toHaveTextContent(/isn’t allowed to edit its own Keycloak client/i);
  });

  it('turns a configured-but-off tunnel back on without re-confirming', async () => {
    const user = userEvent.setup();
    const bodies: unknown[] = [];
    server.use(
      ...mockTunnelStatus({ hostname: 'bodhi.example.com', subdomain: 'bodhi' }),
      ...mockEnableTunnel(undefined, { onBody: (body) => bodies.push(body) })
    );
    const page = await renderPage();

    expect(page).toHaveAttribute('data-test-state', 'off');
    expect(within(page).getByTestId('tunnel-step-3-pill')).toHaveTextContent('Off');

    await user.click(within(page).getByTestId('tunnel-turn-on'));
    await waitFor(() => expect(bodies).toHaveLength(1));
    expect(bodies[0]).toMatchObject({ subdomain: 'bodhi' });
  });

  it('stops the connector from the live page', async () => {
    const user = userEvent.setup();
    server.use(...mockTunnelStatusExact(liveTunnelStatus()), ...mockDisableTunnel());
    const page = await renderPage();

    await user.click(within(page).getByTestId('tunnel-turn-off'));
    await waitFor(() => expect(screen.getByTestId('tunnels-page')).toHaveAttribute('data-test-state', 'off'));
  });

  it('leaves the address editable after a failed start so it can be corrected', async () => {
    // enabled is true here: the connector spawned and then died. Locking on it
    // would trap the user in a read-only form with no way out.
    server.use(
      ...mockTunnelStatus({
        enabled: true,
        state: 'failed',
        subdomain: 'bodhi',
        error_code: 'cloudflared_exited',
        error_message: 'The cloudflared connector exited unexpectedly.',
      })
    );
    const page = await renderPage();

    expect(page).toHaveAttribute('data-test-state', 'create-failed');
    expect(within(page).getByTestId('tunnel-subdomain')).toBeEnabled();
    expect(within(page).getByTestId('tunnel-create-failed')).toHaveTextContent(/exited unexpectedly/i);
    expect(within(page).getByTestId('tunnel-retry-create')).toBeInTheDocument();
  });
});
