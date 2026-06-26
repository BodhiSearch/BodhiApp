import ServerViewPage from '@/routes/mcps/servers/view/index';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import {
  mockAuthConfigHeader,
  mockAuthConfigOAuthDynamic,
  mockAuthConfigOAuthPreReg,
  mockCreateAuthConfig,
  mockCreateAuthConfigError,
  mockDeleteAuthConfig,
  mockDeleteAuthConfigError,
  mockDiscoverMcp,
  mockDiscoverMcpError,
  mockGetMcpServer,
  mockListAuthConfigs,
  mockMcpServerResponse,
} from '@/test-utils/msw-v2/handlers/mcps';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const navigateMock = vi.fn();
let mockSearch: Record<string, string | undefined> = { id: 'server-uuid-1' };
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, search, children, ...rest }: any) => {
      const searchStr = search ? '?' + new URLSearchParams(search).toString() : '';
      return (
        <a href={`${to}${searchStr}`} {...rest}>
          {children}
        </a>
      );
    },
    useNavigate: () => navigateMock,
    useLocation: () => ({ pathname: '/mcps/servers/view' }),
    useSearch: () => mockSearch,
  };
});

setupMswV2();

beforeEach(() => {
  navigateMock.mockClear();
  mockSearch = { id: 'server-uuid-1' };
  server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
});

afterEach(() => {
  vi.resetAllMocks();
});

describe('ServerViewPage - Authentication', () => {
  it('redirects to login if user is not logged in', async () => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedOut());

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/login/' });
    });
  });
});

describe('ServerViewPage - Server Info', () => {
  it('renders server info', async () => {
    server.use(
      mockGetMcpServer({
        ...mockMcpServerResponse,
        name: 'Test Server',
        url: 'https://test.example.com/mcp',
        description: 'A test server description',
        enabled: true,
      }),
      mockListAuthConfigs({ auth_configs: [] })
    );

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('server-view-page')).toBeInTheDocument();
    });

    // Configure-server hub: Basic-information read rows.
    expect(screen.getByTestId('server-name-value')).toHaveTextContent('Test Server');
    expect(screen.getByText('https://test.example.com/mcp')).toBeInTheDocument();
    expect(screen.getByText('A test server description')).toBeInTheDocument();
    expect(screen.getByTestId('server-status')).toHaveTextContent('Enabled');
  });

  it('Edit toggles the inline basic-information form (URL locked)', async () => {
    server.use(mockGetMcpServer(mockMcpServerResponse), mockListAuthConfigs({ auth_configs: [] }));

    const user = userEvent.setup();
    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('server-edit-button')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('server-edit-button'));
    expect(screen.getByTestId('server-edit-form')).toBeInTheDocument();
    // URL is the server identity — locked (disabled) in the inline editor.
    expect(screen.getByTestId('mcp-server-url-input')).toBeDisabled();
    expect(screen.getByTestId('mcp-server-name-input')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-server-save-button')).toBeInTheDocument();
  });

  it('shows server disabled status', async () => {
    server.use(
      mockGetMcpServer({ ...mockMcpServerResponse, enabled: false }),
      mockListAuthConfigs({ auth_configs: [] })
    );

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('server-view-page')).toBeInTheDocument();
    });

    expect(screen.getByTestId('server-status')).toHaveTextContent('Disabled');
  });
});

describe('ServerViewPage - Auth Configs', () => {
  it('shows auth header configs', async () => {
    server.use(mockGetMcpServer(mockMcpServerResponse), mockListAuthConfigs({ auth_configs: [mockAuthConfigHeader] }));

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('auth-configs-section')).toBeInTheDocument();
    });

    expect(screen.getByTestId(`auth-config-row-${mockAuthConfigHeader.id}`)).toBeInTheDocument();
    // Header/query configs render under the "API Key" kind label in the V2 hub.
    expect(screen.getByTestId(`auth-config-type-badge-${mockAuthConfigHeader.id}`)).toHaveTextContent('API Key');
    expect(screen.getByText('Keys: header:Authorization')).toBeInTheDocument();
  });

  it('shows OAuth configs', async () => {
    server.use(
      mockGetMcpServer(mockMcpServerResponse),
      mockListAuthConfigs({ auth_configs: [mockAuthConfigOAuthPreReg, mockAuthConfigOAuthDynamic] })
    );

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('auth-configs-section')).toBeInTheDocument();
    });

    expect(screen.getByTestId(`auth-config-row-${mockAuthConfigOAuthPreReg.id}`)).toBeInTheDocument();
    expect(screen.getByTestId(`auth-config-type-badge-${mockAuthConfigOAuthPreReg.id}`)).toHaveTextContent('OAuth');

    expect(screen.getByTestId(`auth-config-row-${mockAuthConfigOAuthDynamic.id}`)).toBeInTheDocument();
    expect(screen.getByTestId(`auth-config-type-badge-${mockAuthConfigOAuthDynamic.id}`)).toHaveTextContent('OAuth');
  });

  it('always shows the built-in Public mechanism even with no configs', async () => {
    server.use(mockGetMcpServer(mockMcpServerResponse), mockListAuthConfigs({ auth_configs: [] }));

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('auth-configs-section')).toBeInTheDocument();
    });

    // Public is always available (synthetic, no DB row) — rendered as a built-in, non-deletable row.
    const publicRow = screen.getByTestId('auth-config-row-public');
    expect(publicRow).toHaveTextContent('Public');
    expect(publicRow).toHaveTextContent('Built-in');
  });

  it('shows delete confirmation dialog', async () => {
    const user = userEvent.setup();
    server.use(mockGetMcpServer(mockMcpServerResponse), mockListAuthConfigs({ auth_configs: [mockAuthConfigHeader] }));

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId(`auth-config-delete-button-${mockAuthConfigHeader.id}`)).toBeInTheDocument();
    });

    await user.click(screen.getByTestId(`auth-config-delete-button-${mockAuthConfigHeader.id}`));

    expect(screen.getByTestId('delete-auth-config-dialog')).toBeInTheDocument();
    expect(screen.getByText('Delete auth mechanism')).toBeInTheDocument();
    expect(screen.getByText(/All associated OAuth tokens will also be deleted/)).toBeInTheDocument();
  });

  it('toggles inline form when add auth config button is clicked', async () => {
    const user = userEvent.setup();
    server.use(mockGetMcpServer(mockMcpServerResponse), mockListAuthConfigs({ auth_configs: [] }));

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('add-auth-config-button')).toBeInTheDocument();
    });

    expect(screen.queryByTestId('auth-config-form')).not.toBeInTheDocument();

    await user.click(screen.getByTestId('add-auth-config-button'));
    expect(screen.getByTestId('auth-config-form')).toBeInTheDocument();
    expect(screen.getByTestId('auth-config-name-input')).toBeInTheDocument();

    await user.click(screen.getByTestId('auth-config-cancel-button'));
    expect(screen.queryByTestId('auth-config-form')).not.toBeInTheDocument();
    expect(screen.getByTestId('add-auth-config-button')).toBeInTheDocument();
  });

  it('creates a header auth config via inline form', async () => {
    const user = userEvent.setup();
    server.use(
      mockGetMcpServer(mockMcpServerResponse),
      mockListAuthConfigs({ auth_configs: [] }),
      mockCreateAuthConfig(mockAuthConfigHeader)
    );

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('add-auth-config-button')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('add-auth-config-button'));

    await user.type(screen.getByTestId('auth-config-name-input'), 'My API Key');
    await user.type(screen.getByTestId('auth-config-entry-key-0'), 'Authorization');

    await user.click(screen.getByTestId('auth-config-save-button'));

    await waitFor(() => {
      expect(screen.queryByTestId('auth-config-form')).not.toBeInTheDocument();
    });
  });

  it('shows error toast when create fails', async () => {
    const user = userEvent.setup();
    server.use(
      mockGetMcpServer(mockMcpServerResponse),
      mockListAuthConfigs({ auth_configs: [] }),
      mockCreateAuthConfigError({ message: 'Name already exists' })
    );

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('add-auth-config-button')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('add-auth-config-button'));
    await user.type(screen.getByTestId('auth-config-name-input'), 'Duplicate');
    await user.type(screen.getByTestId('auth-config-entry-key-0'), 'X-Api-Key');

    await user.click(screen.getByTestId('auth-config-save-button'));

    await waitFor(() => {
      expect(screen.getByTestId('auth-config-form')).toBeInTheDocument();
    });
  });

  it('deletes an auth config via confirmation dialog', async () => {
    const user = userEvent.setup();
    server.use(
      mockGetMcpServer(mockMcpServerResponse),
      mockListAuthConfigs({ auth_configs: [mockAuthConfigHeader] }),
      mockDeleteAuthConfig()
    );

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId(`auth-config-delete-button-${mockAuthConfigHeader.id}`)).toBeInTheDocument();
    });

    await user.click(screen.getByTestId(`auth-config-delete-button-${mockAuthConfigHeader.id}`));

    expect(screen.getByTestId('delete-auth-config-dialog')).toBeInTheDocument();

    const deleteButton = screen.getByRole('button', { name: /^delete$/i });
    await user.click(deleteButton);

    await waitFor(() => {
      expect(screen.queryByTestId('delete-auth-config-dialog')).not.toBeInTheDocument();
    });
  });

  it('cancel button in form resets and hides form', async () => {
    const user = userEvent.setup();
    server.use(mockGetMcpServer(mockMcpServerResponse), mockListAuthConfigs({ auth_configs: [] }));

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('add-auth-config-button')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('add-auth-config-button'));
    await user.clear(screen.getByTestId('auth-config-name-input'));
    await user.type(screen.getByTestId('auth-config-name-input'), 'Test Name');

    await user.click(screen.getByTestId('auth-config-cancel-button'));

    expect(screen.queryByTestId('auth-config-form')).not.toBeInTheDocument();

    // Re-opening resets the form: name auto-populates again
    await user.click(screen.getByTestId('add-auth-config-button'));
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-name-input')).toHaveValue('header-default');
    });
  });

  it('shows OAuth fields when type is changed', async () => {
    const user = userEvent.setup();
    server.use(
      mockGetMcpServer(mockMcpServerResponse),
      mockListAuthConfigs({ auth_configs: [] }),
      mockDiscoverMcpError({ status: 404, message: 'No discovery' })
    );

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('add-auth-config-button')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('add-auth-config-button'));

    expect(screen.getByTestId('auth-config-entry-key-0')).toBeInTheDocument();
    expect(screen.queryByTestId('auth-config-client-id-input')).not.toBeInTheDocument();

    await user.click(screen.getByTestId('auth-config-type-select'));
    await user.click(screen.getByText('OAuth'));

    // Auto-DCR fires and silently fails → falls back to pre_registered, showing client ID field
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-client-id-input')).toBeInTheDocument();
    });

    expect(screen.queryByTestId('auth-config-entry-key-0')).not.toBeInTheDocument();
    expect(screen.getByTestId('auth-config-auth-endpoint-input')).toBeInTheDocument();
    expect(screen.getByTestId('auth-config-token-endpoint-input')).toBeInTheDocument();
    expect(screen.getByTestId('auth-config-scopes-input')).toBeInTheDocument();
    expect(screen.getByTestId('oauth-registration-type-select')).toBeInTheDocument();
  });

  it('save button is enabled with auto-populated name', async () => {
    const user = userEvent.setup();
    server.use(mockGetMcpServer(mockMcpServerResponse), mockListAuthConfigs({ auth_configs: [] }));

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('add-auth-config-button')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('add-auth-config-button'));

    await waitFor(() => {
      expect(screen.getByTestId('auth-config-name-input')).toHaveValue('header-default');
    });
    expect(screen.getByTestId('auth-config-save-button')).not.toBeDisabled();
  });

  it('auto-discovers and populates OAuth fields when OAuth is selected', async () => {
    const user = userEvent.setup();
    server.use(
      mockGetMcpServer(mockMcpServerResponse),
      mockListAuthConfigs({ auth_configs: [] }),
      mockDiscoverMcp({
        authorization_endpoint: 'https://mcp.asana.com/authorize',
        token_endpoint: 'https://mcp.asana.com/token',
        registration_endpoint: 'https://mcp.asana.com/register',
        scopes_supported: ['mcp:tools', 'mcp:read'],
      })
    );

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('add-auth-config-button')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('add-auth-config-button'));

    const typeSelect = screen.getByTestId('auth-config-type-select');
    await user.click(typeSelect);

    await waitFor(() => {
      expect(screen.getByRole('option', { name: /oauth/i })).toBeInTheDocument();
    });
    await user.click(screen.getByRole('option', { name: /oauth/i }));

    await waitFor(() => {
      expect(screen.getByTestId('auth-config-auth-endpoint-input')).toHaveValue('https://mcp.asana.com/authorize');
    });

    expect(screen.getByTestId('auth-config-token-endpoint-input')).toHaveValue('https://mcp.asana.com/token');
    expect(screen.getByTestId('auth-config-registration-endpoint-input')).toHaveValue('https://mcp.asana.com/register');
    expect(screen.getByTestId('auth-config-scopes-input')).toHaveValue('mcp:tools mcp:read');

    expect(screen.getByTestId('oauth-registration-type-select')).toBeInTheDocument();

    // dynamic_registration mode hides the client ID field
    expect(screen.queryByTestId('auth-config-client-id-input')).not.toBeInTheDocument();
  });

  it('silently falls back to pre-registered when discovery fails', async () => {
    const user = userEvent.setup();
    server.use(
      mockGetMcpServer(mockMcpServerResponse),
      mockListAuthConfigs({ auth_configs: [] }),
      mockDiscoverMcpError({ status: 404, message: 'Discovery endpoint not found' })
    );

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('add-auth-config-button')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('add-auth-config-button'));

    const typeSelect = screen.getByTestId('auth-config-type-select');
    await user.click(typeSelect);

    await waitFor(() => {
      expect(screen.getByRole('option', { name: /oauth/i })).toBeInTheDocument();
    });
    await user.click(screen.getByRole('option', { name: /oauth/i }));

    // Auto-DCR fires and silently fails → falls back to pre_registered (no error shown)
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-client-id-input')).toBeInTheDocument();
    });

    // No error on first failure (silent fallback)
    expect(screen.queryByTestId('auth-config-discover-error')).not.toBeInTheDocument();

    expect(screen.getByTestId('oauth-registration-type-select')).toHaveTextContent('Pre-Registered');
  });

  it('updates name field when switching from header to oauth auth type', async () => {
    const user = userEvent.setup();
    server.use(
      mockGetMcpServer(mockMcpServerResponse),
      mockListAuthConfigs({ auth_configs: [] }),
      mockDiscoverMcp({
        authorization_endpoint: 'https://mcp.example.com/authorize',
        token_endpoint: 'https://mcp.example.com/token',
        registration_endpoint: 'https://mcp.example.com/register',
        scopes_supported: ['mcp:tools', 'mcp:read'],
      })
    );

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('add-auth-config-button')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('add-auth-config-button'));

    await waitFor(() => {
      expect(screen.getByTestId('auth-config-name-input')).toHaveValue('header-default');
    });

    const typeSelect = screen.getByTestId('auth-config-type-select');
    await user.click(typeSelect);

    await waitFor(() => {
      expect(screen.getByRole('option', { name: /oauth/i })).toBeInTheDocument();
    });
    await user.click(screen.getByRole('option', { name: /oauth/i }));

    // Name auto-updates 'header-default' → 'oauth-default'
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-name-input')).toHaveValue('oauth-default');
    });
  });

  it('creates OAuth auth config via inline form (pre-registered fallback)', async () => {
    const user = userEvent.setup();
    server.use(
      mockGetMcpServer(mockMcpServerResponse),
      mockListAuthConfigs({ auth_configs: [] }),
      mockDiscoverMcpError({ status: 404, message: 'No discovery' }),
      mockCreateAuthConfig(mockAuthConfigOAuthPreReg)
    );

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('add-auth-config-button')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('add-auth-config-button'));

    await user.click(screen.getByTestId('auth-config-type-select'));
    await user.click(screen.getByText('OAuth'));

    // Auto-DCR fires and silently fails → falls back to pre_registered
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-client-id-input')).toBeInTheDocument();
    });

    await user.clear(screen.getByTestId('auth-config-name-input'));
    await user.type(screen.getByTestId('auth-config-name-input'), 'My OAuth Config');
    await user.type(screen.getByTestId('auth-config-client-id-input'), 'my-client-id');
    await user.type(screen.getByTestId('auth-config-auth-endpoint-input'), 'https://auth.example.com/authorize');
    await user.type(screen.getByTestId('auth-config-token-endpoint-input'), 'https://auth.example.com/token');

    await user.click(screen.getByTestId('auth-config-save-button'));

    await waitFor(() => {
      expect(screen.queryByTestId('auth-config-form')).not.toBeInTheDocument();
    });
  });

  it('preserves custom name when switching from header to OAuth auth type', async () => {
    const user = userEvent.setup();
    server.use(
      mockGetMcpServer(mockMcpServerResponse),
      mockListAuthConfigs({ auth_configs: [] }),
      mockDiscoverMcpError({ status: 404, message: 'No discovery' })
    );

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('add-auth-config-button')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('add-auth-config-button'));

    await waitFor(() => {
      expect(screen.getByTestId('auth-config-name-input')).toHaveValue('header-default');
    });

    await user.clear(screen.getByTestId('auth-config-name-input'));
    await user.type(screen.getByTestId('auth-config-name-input'), 'My Custom Name');

    await user.click(screen.getByTestId('auth-config-type-select'));
    await waitFor(() => {
      expect(screen.getByRole('option', { name: /oauth/i })).toBeInTheDocument();
    });
    await user.click(screen.getByRole('option', { name: /oauth/i }));

    // Custom name is preserved, not overwritten to 'oauth-default'
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-name-input')).toHaveValue('My Custom Name');
    });
  });
});
