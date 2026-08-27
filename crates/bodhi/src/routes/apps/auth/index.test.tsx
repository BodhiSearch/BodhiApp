import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

import { ROUTE_REQUEST_ACCESS } from '@/lib/constants';
import AppsAuthPage from '@/routes/apps/auth/index';
import {
  MOCK_ERROR_REDIRECT_URL,
  MOCK_REDIRECT_URI,
  mockConsentBlocked,
  mockConsentErrorInApp,
  mockConsentErrorRedirect,
  mockConsentOk,
  mockConsentOkPowerUser,
  mockConsentPriorGrantExplicit,
  mockConsentPriorGrantLatest,
  mockConsentRoleOnly,
} from '@/test-fixtures/apps';
import { mockConsentContext, mockSubmitConsent, mockSubmitConsentError } from '@/test-utils/msw-v2/handlers/apps';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockListMcps } from '@/test-utils/msw-v2/handlers/mcps';
import { mockModelsDefault } from '@/test-utils/msw-v2/handlers/models';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, type components } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

const navigateMock = vi.fn();

// The raw authorize query as the app would send it (pre-encoded).
const AUTHORIZE_SEARCH =
  '?client_id=test-app-client' +
  `&redirect_uri=${encodeURIComponent(MOCK_REDIRECT_URI)}` +
  '&response_type=code&state=xyz789&code_challenge=abc123&code_challenge_method=S256' +
  `&scope=${encodeURIComponent('openid scope_user_user')}`;

const AUTH_REDIRECT_URL =
  'https://id.example.com/realms/bodhi/protocol/openid-connect/auth?client_id=test-app-client&scope=openid%20scope_access_request%3Aresource-x.req-1';

vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: any) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
    useNavigate: () => navigateMock,
    useLocation: () => ({ pathname: '/apps/auth' }),
  };
});

vi.mock('@/hooks/useToastMessages', () => ({
  useToastMessages: () => ({
    showSuccess: vi.fn(),
    showError: vi.fn(),
  }),
}));

let originalLocationDescriptor: PropertyDescriptor | undefined;

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  navigateMock.mockClear();
  if (originalLocationDescriptor) {
    Object.defineProperty(window, 'location', originalLocationDescriptor);
    originalLocationDescriptor = undefined;
  }
  vi.restoreAllMocks();
});

// The page reads the raw query from window.location.search, so every test installs it up front.
const setupWindowLocation = (search: string = AUTHORIZE_SEARCH) => {
  originalLocationDescriptor = Object.getOwnPropertyDescriptor(window, 'location');
  const loc = window.location;
  Object.defineProperty(window, 'location', {
    value: {
      href: `${loc.origin}/ui/apps/auth/${search}`,
      origin: loc.origin,
      protocol: loc.protocol,
      host: loc.host,
      hostname: loc.hostname,
      port: loc.port,
      pathname: '/ui/apps/auth/',
      search,
      hash: '',
      assign: vi.fn(),
      replace: vi.fn(),
      reload: vi.fn(),
      toString: () => `${loc.origin}/ui/apps/auth/${search}`,
    },
    writable: true,
    configurable: true,
  });
};

type SessionRole = components['schemas']['UserInfo']['role'];

const setupHandlers = (
  consent?: Parameters<typeof mockConsentContext>[0],
  { role = 'resource_user' }: { role?: SessionRole } = {}
) => {
  const handlers = [
    ...mockAppInfoReady(),
    ...mockUserLoggedIn({ role }),
    // The consent screen fetches candidate models + MCPs for the access pickers.
    ...mockModelsDefault(),
    mockListMcps(),
  ];
  if (consent) {
    handlers.push(...mockConsentContext(consent));
  }
  server.use(...handlers);
};

describe('AppsAuthPage - Loading & Error States', () => {
  it('shows the loading skeleton while fetching the consent context', async () => {
    setupWindowLocation();
    // No consent handler so the query stays pending and the skeleton shows
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn({ role: 'resource_user' }));

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-loading')).toBeInTheDocument();
    });
  });

  it('renders an in-app error with the error_description and never navigates', async () => {
    setupWindowLocation();
    setupHandlers(mockConsentErrorInApp);
    const initialHref = window.location.href;

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-error')).toBeInTheDocument();
    });
    expect(screen.getByText('redirect_uri does not match the registered redirect URIs')).toBeInTheDocument();
    expect(window.location.href).toBe(initialHref);
  });

  it('an error with a redirect_url navigates unconditionally', async () => {
    setupWindowLocation();
    setupHandlers(mockConsentErrorRedirect);

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(window.location.href).toBe(MOCK_ERROR_REDIRECT_URL);
    });
    expect(screen.getByTestId('consent-redirecting')).toBeInTheDocument();
  });
});

describe('AppsAuthPage - Consent Form Rendering', () => {
  it('renders both grant sections and a fixed User role line by default', async () => {
    setupWindowLocation();
    setupHandlers(mockConsentOk());

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-page')).toBeInTheDocument();
    });

    expect(screen.getByTestId('consent-app-name')).toHaveTextContent('Test Application');
    expect(screen.getByTestId('consent-app-description')).toHaveTextContent('A test third-party application');
    expect(screen.getByTestId('consent-models-section')).toBeInTheDocument();
    expect(screen.getByTestId('consent-list-models-toggle')).toBeInTheDocument();
    expect(screen.getByTestId('consent-model-access-block')).toBeInTheDocument();
    expect(screen.getByTestId('consent-mcps-section')).toBeInTheDocument();
    expect(screen.getByTestId('consent-list-mcps-toggle')).toBeInTheDocument();
    expect(screen.getByTestId('consent-mcp-access-block')).toBeInTheDocument();
    // scope_user_user → the role is fixed, no selector.
    expect(screen.getByTestId('consent-approved-role-fixed')).toHaveTextContent('User');
    expect(screen.queryByTestId('consent-approved-role-select')).not.toBeInTheDocument();
    expect(screen.getByTestId('consent-approve-button')).toBeInTheDocument();
    expect(screen.getByTestId('consent-deny-button')).toBeInTheDocument();
  });

  it('hides the models section when scope.llms is false', async () => {
    setupWindowLocation();
    setupHandlers(mockConsentOk({ scope: { role: 'scope_user_user', llms: false, mcps: true, passthrough: [] } }));

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-mcps-section')).toBeInTheDocument();
    });
    expect(screen.queryByTestId('consent-models-section')).not.toBeInTheDocument();
    expect(screen.queryByTestId('consent-list-models-toggle')).not.toBeInTheDocument();
  });

  it('hides the tools section when scope.mcps is false', async () => {
    setupWindowLocation();
    setupHandlers(mockConsentOk({ scope: { role: 'scope_user_user', llms: true, mcps: false, passthrough: [] } }));

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-models-section')).toBeInTheDocument();
    });
    expect(screen.queryByTestId('consent-mcps-section')).not.toBeInTheDocument();
    expect(screen.queryByTestId('consent-list-mcps-toggle')).not.toBeInTheDocument();
  });
});

describe('AppsAuthPage - Role-Only Scope', () => {
  it('renders the plain statement without grant sections and reaches ready without picker fetches', async () => {
    setupWindowLocation();
    // Deliberately no models/mcps handlers: role-only scope must not fetch picker candidates.
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockConsentContext(mockConsentRoleOnly)
    );

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-role-only-summary')).toBeInTheDocument();
    });
    expect(screen.getByTestId('consent-page')).toHaveAttribute('data-test-state', 'ready');
    expect(screen.getByTestId('consent-role-only-summary')).toHaveTextContent(
      'Test Application will be able to access the Bodhi APIs as User — no model or tool access.'
    );
    expect(screen.queryByTestId('consent-models-section')).not.toBeInTheDocument();
    expect(screen.queryByTestId('consent-mcps-section')).not.toBeInTheDocument();
    expect(screen.getByTestId('consent-approve-button')).toBeInTheDocument();
    expect(screen.getByTestId('consent-deny-button')).toBeInTheDocument();
  });

  it('approve posts a role-only envelope with empty grants', async () => {
    const user = userEvent.setup();
    setupWindowLocation();
    let capturedBody: unknown = null;
    setupHandlers(mockConsentRoleOnly);
    server.use(
      ...mockSubmitConsent(
        { id: 'req-1', redirect_url: AUTH_REDIRECT_URL },
        { status: 201, onBody: (body) => (capturedBody = body) }
      )
    );

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-approve-button')).not.toBeDisabled();
    });
    await user.click(screen.getByTestId('consent-approve-button'));

    await waitFor(() => expect(capturedBody).not.toBeNull());
    expect(capturedBody).toEqual({
      query: AUTHORIZE_SEARCH.slice(1),
      decision: 'approve',
      approved_role: 'scope_user_user',
      approved: {
        version: '1',
        models_list: false,
        models_access: { type: 'specific', ids: [] },
        mcps_list: false,
        mcps: [],
        mcps_access: { type: 'specific', ids: [] },
      },
    });
    await waitFor(() => {
      expect(window.location.href).toBe(AUTH_REDIRECT_URL);
    });
  });
});

describe('AppsAuthPage - Role Selection', () => {
  it('scope_user_power_user renders the selector with a downgrade option for a power user session', async () => {
    const user = userEvent.setup();
    setupWindowLocation();
    setupHandlers(mockConsentOkPowerUser, { role: 'resource_power_user' });

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-approved-role-select')).toBeInTheDocument();
    });
    expect(screen.queryByTestId('consent-approved-role-fixed')).not.toBeInTheDocument();

    await user.click(screen.getByTestId('consent-approved-role-select'));
    await screen.findByTestId('consent-approved-role-option-scope_user_power_user');
    expect(screen.getByTestId('consent-approved-role-option-scope_user_user')).toBeInTheDocument();
  });

  it('caps the options at User when a resource_user reviews a power_user request', async () => {
    const user = userEvent.setup();
    setupWindowLocation();
    setupHandlers(mockConsentOkPowerUser, { role: 'resource_user' });

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-approved-role-select')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('consent-approved-role-select'));
    await screen.findByTestId('consent-approved-role-option-scope_user_user');
    expect(screen.queryByTestId('consent-approved-role-option-scope_user_power_user')).not.toBeInTheDocument();
  });

  it('approve sends the downgraded approved_role when the user selects User', async () => {
    const user = userEvent.setup();
    setupWindowLocation();
    let capturedBody: unknown = null;
    setupHandlers(mockConsentOkPowerUser, { role: 'resource_power_user' });
    server.use(
      ...mockSubmitConsent(
        { id: 'req-1', redirect_url: AUTH_REDIRECT_URL },
        { status: 201, onBody: (body) => (capturedBody = body) }
      )
    );

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-approved-role-select')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('consent-approved-role-select'));
    await user.click(await screen.findByTestId('consent-approved-role-option-scope_user_user'));

    await user.click(screen.getByTestId('consent-approve-button'));

    await waitFor(() => expect(capturedBody).not.toBeNull());
    expect(capturedBody).toMatchObject({ decision: 'approve', approved_role: 'scope_user_user' });
  });
});

describe('AppsAuthPage - Approve & Deny', () => {
  it('approve posts the raw query minus "?" and navigates to the returned redirect_url', async () => {
    const user = userEvent.setup();
    setupWindowLocation();
    let capturedBody: unknown = null;
    setupHandlers(mockConsentOk());
    server.use(
      ...mockSubmitConsent(
        { id: 'req-1', redirect_url: AUTH_REDIRECT_URL },
        { status: 201, onBody: (body) => (capturedBody = body) }
      )
    );

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-approve-button')).not.toBeDisabled();
    });

    // Turn on "list all models"; leave the pickers at least-privilege defaults.
    await user.click(screen.getByTestId('consent-list-models-toggle'));
    await user.click(screen.getByTestId('consent-approve-button'));

    await waitFor(() => expect(capturedBody).not.toBeNull());
    expect(capturedBody).toEqual({
      query: AUTHORIZE_SEARCH.slice(1),
      decision: 'approve',
      approved_role: 'scope_user_user',
      approved: {
        version: '1',
        models_list: true,
        models_access: { type: 'specific', ids: [] },
        mcps_list: false,
        mcps: [],
        mcps_access: { type: 'specific', ids: [] },
      },
    });
    await waitFor(() => {
      expect(window.location.href).toBe(AUTH_REDIRECT_URL);
    });
  });

  it('deny posts the decision and navigates to the returned redirect_url', async () => {
    const user = userEvent.setup();
    setupWindowLocation();
    let capturedBody: unknown = null;
    const denyRedirect = `${MOCK_REDIRECT_URI}?error=access_denied&error_source=bodhi&state=xyz789`;
    setupHandlers(mockConsentOk());
    server.use(...mockSubmitConsent({ redirect_url: denyRedirect }, { onBody: (body) => (capturedBody = body) }));

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-deny-button')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('consent-deny-button'));

    await waitFor(() => expect(capturedBody).not.toBeNull());
    expect(capturedBody).toEqual({
      query: AUTHORIZE_SEARCH.slice(1),
      decision: 'deny',
    });
    await waitFor(() => {
      expect(window.location.href).toBe(denyRedirect);
    });
  });

  it('re-enables the form and stays on the page when the submit fails', async () => {
    const user = userEvent.setup();
    setupWindowLocation();
    setupHandlers(mockConsentOk());
    server.use(...mockSubmitConsentError({ message: 'privilege escalation', status: 403 }));
    const initialHref = window.location.href;

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-approve-button')).not.toBeDisabled();
    });
    await user.click(screen.getByTestId('consent-approve-button'));

    await waitFor(() => {
      expect(screen.getByTestId('consent-approve-button')).not.toBeDisabled();
    });
    expect(window.location.href).toBe(initialHref);
  });
});

describe('AppsAuthPage - Blocked State', () => {
  it('renders the blocked notice and the return button denies and navigates', async () => {
    const user = userEvent.setup();
    setupWindowLocation();
    let capturedBody: unknown = null;
    const denyRedirect = `${MOCK_REDIRECT_URI}?error=access_denied&error_source=bodhi&state=xyz789`;
    setupHandlers(mockConsentBlocked, { role: 'resource_guest' });
    server.use(...mockSubmitConsent({ redirect_url: denyRedirect }, { onBody: (body) => (capturedBody = body) }));

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-blocked')).toBeInTheDocument();
    });
    // skipRoleGate: the guest session renders the blocked state instead of bouncing.
    expect(navigateMock).not.toHaveBeenCalledWith(expect.objectContaining({ to: ROUTE_REQUEST_ACCESS }));
    expect(screen.getByTestId('consent-blocked')).toHaveAttribute('data-test-state', 'ready');
    expect(screen.getByTestId('consent-app-name')).toHaveTextContent('Test Application');
    expect(screen.getByText(/You don't have access to this Bodhi instance yet/)).toBeInTheDocument();
    expect(screen.queryByTestId('consent-approve-button')).not.toBeInTheDocument();

    const returnButton = screen.getByTestId('consent-return-button');
    expect(returnButton).toHaveTextContent('Return to Test Application');
    await user.click(returnButton);

    await waitFor(() => expect(capturedBody).not.toBeNull());
    expect(capturedBody).toEqual({ query: AUTHORIZE_SEARCH.slice(1), decision: 'deny' });
    await waitFor(() => {
      expect(window.location.href).toBe(denyRedirect);
    });
  });
});

describe('AppsAuthPage - Prior Grant', () => {
  it('an explicit prior grant prefills immediately and shows the reauth banner', async () => {
    setupWindowLocation();
    let capturedBody: unknown = null;
    setupHandlers(mockConsentPriorGrantExplicit);
    server.use(
      ...mockSubmitConsent(
        { id: 'req-1', redirect_url: AUTH_REDIRECT_URL },
        { status: 201, onBody: (body) => (capturedBody = body) }
      )
    );

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-reauth-banner')).toBeInTheDocument();
    });
    expect(screen.queryByTestId('consent-restore-banner')).not.toBeInTheDocument();
    // Prior grant had both list toggles on — they load checked.
    expect(screen.getByTestId('consent-list-models-toggle')).toBeChecked();
    expect(screen.getByTestId('consent-list-mcps-toggle')).toBeChecked();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('consent-approve-button'));
    await waitFor(() => expect(capturedBody).not.toBeNull());
    const body = capturedBody as { approved: { models_access: unknown; mcps_access: unknown } };
    expect(body.approved.models_access).toEqual({ type: 'specific', ids: ['model-a'] });
    expect(body.approved.mcps_access).toEqual({ type: 'specific', ids: ['mcp-x'] });
  });

  it('a latest prior grant is not prefilled until Restore is clicked', async () => {
    const user = userEvent.setup();
    setupWindowLocation();
    setupHandlers(mockConsentPriorGrantLatest);

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-restore-banner')).toBeInTheDocument();
    });
    expect(screen.queryByTestId('consent-reauth-banner')).not.toBeInTheDocument();
    // Not applied yet — least-privilege defaults hold.
    expect(screen.getByTestId('consent-list-models-toggle')).not.toBeChecked();
    expect(screen.getByTestId('consent-list-mcps-toggle')).not.toBeChecked();

    await user.click(screen.getByTestId('consent-restore-button'));

    await waitFor(() => {
      expect(screen.getByTestId('consent-list-models-toggle')).toBeChecked();
    });
    expect(screen.getByTestId('consent-list-mcps-toggle')).toBeChecked();
    expect(screen.queryByTestId('consent-restore-button')).not.toBeInTheDocument();
  });

  it('scope-suppressed prior selections are neither shown nor carried forward', async () => {
    const user = userEvent.setup();
    setupWindowLocation();
    let capturedBody: unknown = null;
    // Prior grant carries MCP selections, but the new scope no longer requests MCPs.
    setupHandlers(
      mockConsentOk({
        scope: { role: 'scope_user_user', llms: true, mcps: false, passthrough: [] },
        prior_grant: {
          id: 'prior-grant-1',
          approved_role: 'scope_user_user',
          approved: {
            version: '1' as const,
            models_list: true,
            models_access: { type: 'specific', ids: ['model-a'] },
            mcps_list: true,
            mcps: [],
            mcps_access: { type: 'all' },
          },
          source: 'explicit',
        },
      })
    );
    server.use(
      ...mockSubmitConsent(
        { id: 'req-1', redirect_url: AUTH_REDIRECT_URL },
        { status: 201, onBody: (body) => (capturedBody = body) }
      )
    );

    await act(async () => {
      render(<AppsAuthPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('consent-list-models-toggle')).toBeChecked();
    });
    expect(screen.queryByTestId('consent-mcps-section')).not.toBeInTheDocument();

    await user.click(screen.getByTestId('consent-approve-button'));

    await waitFor(() => expect(capturedBody).not.toBeNull());
    const body = capturedBody as {
      approved: { models_access: unknown; mcps_list: boolean; mcps: unknown[]; mcps_access: unknown };
    };
    // The prior MCP grants must not leak into the suppressed section.
    expect(body.approved.mcps_list).toBe(false);
    expect(body.approved.mcps).toEqual([]);
    expect(body.approved.mcps_access).toEqual({ type: 'specific', ids: [] });
    expect(body.approved.models_access).toEqual({ type: 'specific', ids: ['model-a'] });
  });
});
