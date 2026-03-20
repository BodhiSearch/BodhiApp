import ServerViewPage from '@/app/ui/mcps/servers/view/page';
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

const pushMock = vi.fn();
let searchParamsMap: Record<string, string | null> = { id: 'server-uuid-1' };
vi.mock('next/navigation', () => ({
  useRouter: () => ({ push: pushMock }),
  usePathname: () => '/ui/mcps/servers/view',
  useSearchParams: () => ({
    get: (key: string) => searchParamsMap[key] ?? null,
  }),
}));

setupMswV2();

beforeEach(() => {
  pushMock.mockClear();
  searchParamsMap = { id: 'server-uuid-1' };
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
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
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

    expect(screen.getByText('Test Server')).toBeInTheDocument();
    expect(screen.getByText('https://test.example.com/mcp')).toBeInTheDocument();
    expect(screen.getByText('A test server description')).toBeInTheDocument();
    expect(screen.getByText('Enabled')).toBeInTheDocument();
  });

  it('navigates to edit page', async () => {
    server.use(mockGetMcpServer(mockMcpServerResponse), mockListAuthConfigs({ auth_configs: [] }));

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('server-view-page')).toBeInTheDocument();
    });

    const editLink = screen.getByRole('link', { name: /edit/i });
    expect(editLink).toHaveAttribute('href', '/ui/mcps/servers/edit?id=server-uuid-1');
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

    expect(screen.getByText('Disabled')).toBeInTheDocument();
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
    expect(screen.getByTestId(`auth-config-type-badge-${mockAuthConfigHeader.id}`)).toHaveTextContent('Header');
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

  it('shows empty state when no auth configs', async () => {
    server.use(mockGetMcpServer(mockMcpServerResponse), mockListAuthConfigs({ auth_configs: [] }));

    await act(async () => {
      render(<ServerViewPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('auth-configs-section')).toBeInTheDocument();
    });

    expect(screen.getByText('No auth configurations yet.')).toBeInTheDocument();
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
    expect(screen.getByText('Delete Auth Config')).toBeInTheDocument();
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

    // Initially form is hidden
    expect(screen.queryByTestId('auth-config-form')).not.toBeInTheDocument();

    // Click to show the form
    await user.click(screen.getByTestId('add-auth-config-button'));
    expect(screen.getByTestId('auth-config-form')).toBeInTheDocument();
    expect(screen.getByTestId('auth-config-name-input')).toBeInTheDocument();

    // Click cancel to hide the form
    await user.click(screen.getByTestId('auth-config-cancel-button'));
    expect(screen.queryByTestId('auth-config-form')).not.toBeInTheDocument();
    // Add button reappears
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

    // Fill out header form fields - multi-entry form
    await user.type(screen.getByTestId('auth-config-name-input'), 'My API Key');
    await user.type(screen.getByTestId('auth-config-entry-key-0'), 'Authorization');

    await user.click(screen.getByTestId('auth-config-save-button'));

    // Form should close after success
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

    // Form should stay open on error
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

    // Click the delete button in the dialog
    const deleteButton = screen.getByRole('button', { name: /^delete$/i });
    await user.click(deleteButton);

    // Dialog should close after success
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

    // Click cancel
    await user.click(screen.getByTestId('auth-config-cancel-button'));

    expect(screen.queryByTestId('auth-config-form')).not.toBeInTheDocument();

    // Re-open and verify form is reset (name is auto-populated)
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

    // Default is header - should show entry fields
    expect(screen.getByTestId('auth-config-entry-key-0')).toBeInTheDocument();
    expect(screen.queryByTestId('auth-config-client-id-input')).not.toBeInTheDocument();

    // Change to oauth
    await user.click(screen.getByTestId('auth-config-type-select'));
    await user.click(screen.getByText('OAuth'));

    // Auto-DCR fires and silently fails → falls back to pre_registered, showing client ID field
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-client-id-input')).toBeInTheDocument();
    });

    // Should now show OAuth fields
    expect(screen.queryByTestId('auth-config-entry-key-0')).not.toBeInTheDocument();
    expect(screen.getByTestId('auth-config-auth-endpoint-input')).toBeInTheDocument();
    expect(screen.getByTestId('auth-config-token-endpoint-input')).toBeInTheDocument();
    expect(screen.getByTestId('auth-config-scopes-input')).toBeInTheDocument();
    // Registration Type dropdown is now always visible
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

    // Name is auto-populated, so button should be enabled
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

    // Click to show form
    await user.click(screen.getByTestId('add-auth-config-button'));

    // Select OAuth from type dropdown
    const typeSelect = screen.getByTestId('auth-config-type-select');
    await user.click(typeSelect);

    await waitFor(() => {
      expect(screen.getByRole('option', { name: /oauth/i })).toBeInTheDocument();
    });
    await user.click(screen.getByRole('option', { name: /oauth/i }));

    // Wait for auto-discovery to complete and populate fields
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-auth-endpoint-input')).toHaveValue('https://mcp.asana.com/authorize');
    });

    // Verify all endpoints are populated
    expect(screen.getByTestId('auth-config-token-endpoint-input')).toHaveValue('https://mcp.asana.com/token');
    expect(screen.getByTestId('auth-config-registration-endpoint-input')).toHaveValue('https://mcp.asana.com/register');
    expect(screen.getByTestId('auth-config-scopes-input')).toHaveValue('mcp:tools mcp:read');

    // Registration Type dropdown should be visible and set to Dynamic Registration
    expect(screen.getByTestId('oauth-registration-type-select')).toBeInTheDocument();

    // Client ID field should NOT be visible (dynamic_registration mode hides it)
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

    // Click to show form
    await user.click(screen.getByTestId('add-auth-config-button'));

    // Select OAuth
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

    // No error should be shown on first failure (silent fallback)
    expect(screen.queryByTestId('auth-config-discover-error')).not.toBeInTheDocument();

    // Registration Type dropdown should be visible and show Pre-Registered
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

    // Click to show form - default type is header
    await user.click(screen.getByTestId('add-auth-config-button'));

    // Wait for auto-populated name
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-name-input')).toHaveValue('header-default');
    });

    // Switch to OAuth type
    const typeSelect = screen.getByTestId('auth-config-type-select');
    await user.click(typeSelect);

    await waitFor(() => {
      expect(screen.getByRole('option', { name: /oauth/i })).toBeInTheDocument();
    });
    await user.click(screen.getByRole('option', { name: /oauth/i }));

    // Name should update from 'header-default' to 'oauth-default'
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

    // Switch to OAuth type
    await user.click(screen.getByTestId('auth-config-type-select'));
    await user.click(screen.getByText('OAuth'));

    // Auto-DCR fires and silently fails → falls back to pre_registered
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-client-id-input')).toBeInTheDocument();
    });

    // Fill in OAuth fields
    await user.clear(screen.getByTestId('auth-config-name-input'));
    await user.type(screen.getByTestId('auth-config-name-input'), 'My OAuth Config');
    await user.type(screen.getByTestId('auth-config-client-id-input'), 'my-client-id');
    await user.type(screen.getByTestId('auth-config-auth-endpoint-input'), 'https://auth.example.com/authorize');
    await user.type(screen.getByTestId('auth-config-token-endpoint-input'), 'https://auth.example.com/token');

    await user.click(screen.getByTestId('auth-config-save-button'));

    // Form should close after success
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

    // Wait for auto-populated name
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-name-input')).toHaveValue('header-default');
    });

    // Set a custom name
    await user.clear(screen.getByTestId('auth-config-name-input'));
    await user.type(screen.getByTestId('auth-config-name-input'), 'My Custom Name');

    // Switch to OAuth type
    await user.click(screen.getByTestId('auth-config-type-select'));
    await waitFor(() => {
      expect(screen.getByRole('option', { name: /oauth/i })).toBeInTheDocument();
    });
    await user.click(screen.getByRole('option', { name: /oauth/i }));

    // Custom name should be preserved (not overwritten to 'oauth-default')
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-name-input')).toHaveValue('My Custom Name');
    });
  });
});
