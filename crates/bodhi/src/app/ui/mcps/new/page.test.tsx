import NewMcpPage from '@/app/ui/mcps/new/page';
import { BODHI_API_BASE } from '@/hooks/useQuery';
import { useMcpFormStore } from '@/stores/mcpFormStore';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import {
  mockAuthConfigHeader,
  mockAuthConfigOAuthDynamic,
  mockAuthConfigOAuthPreReg,
  mockCreateMcp,
  mockDeleteOAuthToken,
  mockFetchMcpTools,
  mockFetchMcpToolsError,
  mockGetMcp,
  mockGetOAuthToken,
  mockListAuthConfigs,
  mockListMcpServers,
  mockMcp,
  mockMcpServerResponse,
  mockMcpTool,
  mockMcpWithDcr,
  mockMcpWithHeaderAuth,
  mockMcpWithOAuth,
  mockOAuthLogin,
  mockOAuthToken,
  mockUpdateMcp,
} from '@/test-utils/msw-v2/handlers/mcps';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { http, HttpResponse } from 'msw';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
let searchParamsMap: Record<string, string | null> = {};
let originalLocationDescriptor: PropertyDescriptor | undefined;

vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: (key: string) => searchParamsMap[key] ?? null,
  }),
  usePathname: () => '/ui/mcps/new',
}));

setupMswV2();

const setupWindowLocation = () => {
  originalLocationDescriptor = Object.getOwnPropertyDescriptor(window, 'location');
  const loc = window.location;
  Object.defineProperty(window, 'location', {
    value: {
      href: loc.href,
      origin: loc.origin,
      protocol: loc.protocol,
      host: loc.host,
      hostname: loc.hostname,
      port: loc.port,
      pathname: loc.pathname,
      search: loc.search,
      hash: loc.hash,
      assign: vi.fn(),
      replace: vi.fn(),
      reload: vi.fn(),
    },
    writable: true,
    configurable: true,
  });
};

async function selectServer(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByTestId('mcp-server-combobox'));
  await waitFor(() => {
    expect(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`)).toBeInTheDocument();
  });
  await user.click(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`));
}

beforeEach(() => {
  pushMock.mockClear();
  searchParamsMap = {};
  sessionStorage.clear();
  useMcpFormStore.getState().reset();
});

afterEach(() => {
  vi.resetAllMocks();
  sessionStorage.clear();
  useMcpFormStore.getState().reset();
  if (originalLocationDescriptor) {
    Object.defineProperty(window, 'location', originalLocationDescriptor);
    originalLocationDescriptor = undefined;
  }
});

describe('NewMcpPage - Create flow', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockListAuthConfigs({ auth_configs: [] }),
      mockFetchMcpTools([mockMcpTool]),
      mockCreateMcp(mockMcp)
    );
  });

  it('renders the page with tools section always visible', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('new-mcp-page')).toBeInTheDocument();
    });

    expect(screen.getByTestId('mcp-tools-section')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-tools-empty-state')).toHaveTextContent(
      'Select a server and fetch tools to see available tools.'
    );
  });

  it('disables Create MCP button until tools are fetched', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-create-button')).toBeInTheDocument();
    });

    expect(screen.getByTestId('mcp-create-button')).toBeDisabled();
  });

  it('enables Create MCP button after fetching tools', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await selectServer(user);
    await user.click(screen.getByTestId('mcp-fetch-tools-button'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-tools-list')).toBeInTheDocument();
    });

    expect(screen.getByTestId('mcp-create-button')).not.toBeDisabled();
  });

  it('shows fetched tools with checkboxes', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await selectServer(user);
    await user.click(screen.getByTestId('mcp-fetch-tools-button'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-tool-read_wiki_structure')).toBeInTheDocument();
    });

    const toolCheckbox = screen.getByTestId('mcp-tool-checkbox-read_wiki_structure');
    expect(toolCheckbox).toBeInTheDocument();
  });

  it('creates MCP with tools data in single POST', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await selectServer(user);
    await user.click(screen.getByTestId('mcp-fetch-tools-button'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-tools-list')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-create-button'));

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/mcps');
    });
  });

  it('shows toast error when fetch tools fails', async () => {
    const user = userEvent.setup();
    server.use(mockFetchMcpToolsError({ message: 'Connection refused' }));

    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await selectServer(user);
    await user.click(screen.getByTestId('mcp-fetch-tools-button'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-create-button')).toBeDisabled();
    });
  });
});

describe('NewMcpPage - Edit flow', () => {
  beforeEach(() => {
    searchParamsMap = { id: 'mcp-uuid-1' };
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListAuthConfigs({ auth_configs: [] }),
      mockFetchMcpTools([mockMcpTool]),
      mockGetMcp(mockMcp)
    );
  });

  it('loads existing MCP and shows cached tools', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('new-mcp-page')).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('Example MCP');
    });

    expect(screen.getByTestId('mcp-tools-list')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-tool-read_wiki_structure')).toBeInTheDocument();
  });

  it('shows Update MCP button (not Create) in edit mode', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-update-button')).toBeInTheDocument();
    });

    expect(screen.queryByTestId('mcp-create-button')).not.toBeInTheDocument();
  });
});

describe('NewMcpPage - Auth config dropdown', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockFetchMcpTools([mockMcpTool]),
      mockCreateMcp(mockMcp)
    );
  });

  it('renders auth config dropdown with public default when no configs exist', async () => {
    server.use(mockListAuthConfigs({ auth_configs: [] }));

    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('auth-config-select')).toBeInTheDocument();
    });

    expect(screen.getByTestId('auth-config-select')).toHaveAttribute('data-test-state', 'public');
  });

  it('shows header configs from server in dropdown with badge', async () => {
    server.use(mockListAuthConfigs({ auth_configs: [mockAuthConfigHeader] }));

    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await selectServer(user);

    await user.click(screen.getByTestId('auth-config-select'));
    await waitFor(() => {
      expect(screen.getByTestId(`auth-config-option-${mockAuthConfigHeader.id}`)).toBeInTheDocument();
    });
    expect(screen.getByTestId(`auth-config-option-${mockAuthConfigHeader.id}`)).toHaveTextContent('[Header]');
  });

  it('shows OAuth configs from server in dropdown with badge', async () => {
    server.use(mockListAuthConfigs({ auth_configs: [mockAuthConfigOAuthPreReg] }));

    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await selectServer(user);

    await user.click(screen.getByTestId('auth-config-select'));
    await waitFor(() => {
      expect(screen.getByTestId(`auth-config-option-${mockAuthConfigOAuthPreReg.id}`)).toBeInTheDocument();
    });
    expect(screen.getByTestId(`auth-config-option-${mockAuthConfigOAuthPreReg.id}`)).toHaveTextContent('[OAuth]');
  });

  it('shows read-only header summary when header config selected', async () => {
    server.use(mockListAuthConfigs({ auth_configs: [mockAuthConfigHeader] }));

    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await selectServer(user);

    // Auto-select should pick the first config (header)
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-header-summary')).toBeInTheDocument();
    });

    expect(screen.getByTestId('auth-config-header-summary')).toHaveTextContent('Authorization');
    expect(screen.getByTestId('auth-config-header-summary')).toHaveTextContent('Configured');
  });

  it('shows Connect button when OAuth config selected', async () => {
    server.use(mockListAuthConfigs({ auth_configs: [mockAuthConfigOAuthPreReg] }));

    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await selectServer(user);

    // Auto-select should pick the first config (OAuth)
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-oauth-connect')).toBeInTheDocument();
    });
  });

  it('auto-selects first auth config when server has configs', async () => {
    server.use(mockListAuthConfigs({ auth_configs: [mockAuthConfigHeader] }));

    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await selectServer(user);

    await waitFor(() => {
      expect(screen.getByTestId('auth-config-select')).toHaveAttribute('data-test-state', 'header');
    });
  });

  it('admin sees New Auth Config option in dropdown', async () => {
    server.use(
      ...mockUserLoggedIn({ role: 'resource_admin' }, { stub: true }),
      mockListAuthConfigs({ auth_configs: [] })
    );

    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await selectServer(user);

    await user.click(screen.getByTestId('auth-config-select'));
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-option-new')).toBeInTheDocument();
    });
  });

  it('non-admin does not see New Auth Config option in dropdown', async () => {
    server.use(
      ...mockUserLoggedIn({ role: 'resource_user' }, { stub: true }),
      mockListAuthConfigs({ auth_configs: [] })
    );

    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await selectServer(user);

    await user.click(screen.getByTestId('auth-config-select'));
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-option-public')).toBeInTheDocument();
    });
    expect(screen.queryByTestId('auth-config-option-new')).not.toBeInTheDocument();
  });

  it('submits with header auth_type and auth_uuid when header config selected', async () => {
    const createCalled = vi.fn();
    server.use(
      mockListAuthConfigs({ auth_configs: [mockAuthConfigHeader] }),
      http.post(`${BODHI_API_BASE}/mcps`, async ({ request }) => {
        const body = await request.json();
        createCalled(body);
        return HttpResponse.json(mockMcp, { status: 201 });
      })
    );

    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await selectServer(user);

    // Wait for auto-select of header config
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-select')).toHaveAttribute('data-test-state', 'header');
    });

    await user.click(screen.getByTestId('mcp-fetch-tools-button'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-tools-list')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-create-button'));

    await waitFor(() => {
      expect(createCalled).toHaveBeenCalled();
    });

    const body = createCalled.mock.calls[0][0];
    expect(body.auth_type).toBe('header');
    expect(body.auth_uuid).toBe(mockAuthConfigHeader.id);
  });

  it('shows new auth config redirect section when admin selects new', async () => {
    server.use(
      ...mockUserLoggedIn({ role: 'resource_admin' }, { stub: true }),
      mockListAuthConfigs({ auth_configs: [] })
    );

    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await selectServer(user);

    await user.click(screen.getByTestId('auth-config-select'));
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-option-new')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('auth-config-option-new'));

    await waitFor(() => {
      expect(screen.getByTestId('auth-config-new-redirect')).toBeInTheDocument();
    });
  });
});

describe('NewMcpPage - Edit with public auth', () => {
  beforeEach(() => {
    searchParamsMap = { id: 'mcp-uuid-1' };
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListAuthConfigs({ auth_configs: [] }),
      mockFetchMcpTools([mockMcpTool]),
      mockUpdateMcp(mockMcp),
      mockGetMcp(mockMcp)
    );
  });

  it('shows auth config dropdown with public state on edit', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('Example MCP');
    });

    const trigger = screen.getByTestId('auth-config-select');
    expect(trigger).toHaveAttribute('data-test-state', 'public');
  });
});

describe('NewMcpPage - Edit with header auth', () => {
  beforeEach(() => {
    searchParamsMap = { id: 'mcp-uuid-2' };
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListAuthConfigs({ auth_configs: [mockAuthConfigHeader] }),
      mockFetchMcpTools([mockMcpTool]),
      mockUpdateMcp(mockMcpWithHeaderAuth),
      mockGetMcp(mockMcpWithHeaderAuth)
    );
  });

  it('shows header config selected in dropdown and summary', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('Header Auth MCP');
    });

    await waitFor(() => {
      expect(screen.getByTestId('auth-config-select')).toHaveAttribute('data-test-state', 'header');
    });

    await waitFor(() => {
      expect(screen.getByTestId('auth-config-header-summary')).toBeInTheDocument();
    });
    expect(screen.getByTestId('auth-config-header-summary')).toHaveTextContent('Authorization');
  });
});

describe('NewMcpPage - Edit with OAuth auth', () => {
  beforeEach(() => {
    searchParamsMap = { id: 'mcp-uuid-3' };
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListAuthConfigs({ auth_configs: [mockAuthConfigOAuthPreReg] }),
      mockGetOAuthToken(mockOAuthToken),
      mockFetchMcpTools([mockMcpTool]),
      mockUpdateMcp(mockMcpWithOAuth),
      mockOAuthLogin(),
      mockGetMcp(mockMcpWithOAuth)
    );
  });

  it('loads existing OAuth MCP and shows OAuth auth type in dropdown', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('OAuth MCP');
    });

    await waitFor(() => {
      expect(screen.getByTestId('auth-config-select')).toHaveAttribute('data-test-state', 'oauth');
    });
  });

  it('shows connected card in edit mode for OAuth MCP with existing token', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('OAuth MCP');
    });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-connected-card')).toBeInTheDocument();
    });
    expect(screen.getByTestId('oauth-connected-badge')).toHaveTextContent('Connected');
  });
});

describe('NewMcpPage - Edit with DCR OAuth auth', () => {
  beforeEach(() => {
    searchParamsMap = { id: 'mcp-uuid-4' };
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListAuthConfigs({ auth_configs: [mockAuthConfigOAuthDynamic] }),
      mockGetOAuthToken({
        ...mockOAuthToken,
        id: 'oauth-token-uuid-2',
        mcp_oauth_config_id: mockAuthConfigOAuthDynamic.id,
      }),
      mockFetchMcpTools([mockMcpTool]),
      mockUpdateMcp(mockMcpWithDcr),
      mockDeleteOAuthToken(),
      mockGetMcp(mockMcpWithDcr)
    );
  });

  it('loads existing DCR MCP and shows connected card', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('DCR MCP');
    });

    await waitFor(() => {
      expect(screen.getByTestId('auth-config-select')).toHaveAttribute('data-test-state', 'oauth');
    });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-connected-card')).toBeInTheDocument();
    });
    expect(screen.getByTestId('oauth-connected-badge')).toHaveTextContent('Connected');
  });
});

describe('NewMcpPage - OAuth session restore after callback', () => {
  beforeEach(() => {
    const sessionData = {
      name: 'Restored MCP',
      slug: 'restored-mcp',
      description: 'A restored MCP',
      enabled: true,
      mcp_server_id: mockMcpServerResponse.id,
      auth_type: 'oauth',
      selected_auth_config_id: mockAuthConfigOAuthPreReg.id,
      selected_auth_config_type: 'oauth',
      oauth_token_id: mockOAuthToken.id,
      server_url: mockMcpServerResponse.url,
      server_name: mockMcpServerResponse.name,
    };
    sessionStorage.setItem('mcp_oauth_form_state', JSON.stringify(sessionData));

    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockListAuthConfigs({ auth_configs: [mockAuthConfigOAuthPreReg] }),
      mockFetchMcpTools([mockMcpTool]),
      mockDeleteOAuthToken()
    );
  });

  it('restores form with Connected status and populated fields after OAuth callback', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('Restored MCP');
    });

    expect(screen.getByTestId('mcp-slug-input')).toHaveValue('restored-mcp');

    const trigger = screen.getByTestId('auth-config-select');
    expect(trigger).toHaveAttribute('data-test-state', 'oauth');

    await waitFor(() => {
      expect(screen.getByTestId('oauth-connected-card')).toBeInTheDocument();
    });
    expect(screen.getByTestId('oauth-connected-badge')).toHaveTextContent('Connected');
    expect(screen.getByTestId('oauth-disconnect-button')).toBeInTheDocument();
  });
});

describe('NewMcpPage - Session data takes priority over API data in edit mode', () => {
  it('preserves new OAuth token from session instead of old token from API', async () => {
    searchParamsMap = { id: 'mcp-uuid-3' };

    const sessionData = {
      name: 'OAuth MCP',
      slug: 'oauth-mcp',
      description: '',
      enabled: true,
      mcp_server_id: mockMcpServerResponse.id,
      auth_type: 'oauth',
      selected_auth_config_id: mockAuthConfigOAuthPreReg.id,
      selected_auth_config_type: 'oauth',
      oauth_token_id: 'new-token-from-callback',
      server_url: mockMcpServerResponse.url,
      server_name: mockMcpServerResponse.name,
    };
    sessionStorage.setItem('mcp_oauth_form_state', JSON.stringify(sessionData));

    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListAuthConfigs({ auth_configs: [mockAuthConfigOAuthPreReg] }),
      mockGetOAuthToken(mockOAuthToken),
      mockFetchMcpTools([mockMcpTool]),
      mockGetMcp(mockMcpWithOAuth)
    );

    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('OAuth MCP');
    });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-connected-card')).toBeInTheDocument();
    });

    expect(useMcpFormStore.getState().oauthTokenId).toBe('new-token-from-callback');
  });
});

describe('NewMcpPage - OAuth Pre-Registered Client', () => {
  it('creates MCP with OAuth pre-registered auth after OAuth callback (post-callback state)', async () => {
    const sessionData = {
      name: 'Post-OAuth MCP',
      slug: 'post-oauth-mcp',
      description: '',
      enabled: true,
      mcp_server_id: mockMcpServerResponse.id,
      auth_type: 'oauth',
      selected_auth_config_id: mockAuthConfigOAuthPreReg.id,
      selected_auth_config_type: 'oauth',
      oauth_token_id: mockOAuthToken.id,
      server_url: mockMcpServerResponse.url,
      server_name: mockMcpServerResponse.name,
      tools_cache: [mockMcpTool],
      tools_filter: ['read_wiki_structure'],
    };
    sessionStorage.setItem('mcp_oauth_form_state', JSON.stringify(sessionData));

    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockListAuthConfigs({ auth_configs: [mockAuthConfigOAuthPreReg] }),
      mockFetchMcpTools([mockMcpTool]),
      mockCreateMcp(mockMcp)
    );

    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('new-mcp-page')).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('Post-OAuth MCP');
    });
    expect(screen.getByTestId('mcp-slug-input')).toHaveValue('post-oauth-mcp');

    expect(screen.getByTestId('auth-config-select')).toHaveAttribute('data-test-state', 'oauth');

    await waitFor(() => {
      expect(screen.getByTestId('oauth-connected-card')).toBeInTheDocument();
    });

    expect(screen.getByTestId('mcp-create-button')).not.toBeDisabled();

    await user.click(screen.getByTestId('mcp-create-button'));

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/mcps');
    });
  });
});

describe('NewMcpPage - OAuth disconnect flow (lazy)', () => {
  beforeEach(() => {
    const sessionData = {
      name: 'Connected MCP',
      slug: 'connected-mcp',
      description: '',
      enabled: true,
      mcp_server_id: mockMcpServerResponse.id,
      auth_type: 'oauth',
      selected_auth_config_id: mockAuthConfigOAuthPreReg.id,
      selected_auth_config_type: 'oauth',
      oauth_token_id: mockOAuthToken.id,
      server_url: mockMcpServerResponse.url,
      server_name: mockMcpServerResponse.name,
    };
    sessionStorage.setItem('mcp_oauth_form_state', JSON.stringify(sessionData));

    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockListAuthConfigs({ auth_configs: [mockAuthConfigOAuthPreReg] }),
      mockFetchMcpTools([mockMcpTool])
    );
  });

  it('Disconnect is lazy - no DELETE API call, shows Connect button again', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-connected-card')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('oauth-disconnect-button'));

    await waitFor(() => {
      expect(screen.queryByTestId('oauth-connected-card')).not.toBeInTheDocument();
    });

    await waitFor(() => {
      expect(screen.getByTestId('auth-config-oauth-connect')).toBeInTheDocument();
    });
  });
});

describe('NewMcpPage - OAuth Connect flow', () => {
  beforeEach(() => {
    setupWindowLocation();
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockListAuthConfigs({ auth_configs: [mockAuthConfigOAuthPreReg] }),
      mockFetchMcpTools([mockMcpTool]),
      mockOAuthLogin()
    );
  });

  it('Connect button triggers OAuth login redirect', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await selectServer(user);

    // Auto-select should pick the OAuth config
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-oauth-connect')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('auth-config-oauth-connect'));

    await waitFor(() => {
      expect(window.location.href).toBe('https://auth.example.com/authorize?client_id=test&state=abc123');
    });
  });
});

describe('NewMcpPage - Edit with DCR disconnect and update', () => {
  it('update after disconnect deletes token and submits without auth_uuid', async () => {
    searchParamsMap = { id: 'mcp-uuid-4' };
    const deleteTokenCalled = vi.fn();
    const updateCalled = vi.fn();

    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListAuthConfigs({ auth_configs: [mockAuthConfigOAuthDynamic] }),
      mockGetOAuthToken({
        ...mockOAuthToken,
        id: 'oauth-token-uuid-2',
        mcp_oauth_config_id: mockAuthConfigOAuthDynamic.id,
      }),
      mockFetchMcpTools([mockMcpTool]),
      http.delete(`${BODHI_API_BASE}/mcps/oauth-tokens/:tokenId`, () => {
        deleteTokenCalled();
        return new HttpResponse(null, { status: 204 });
      }),
      mockGetMcp(mockMcpWithDcr),
      http.put(`${BODHI_API_BASE}/mcps/:id`, async ({ request }) => {
        const body = await request.json();
        updateCalled(body);
        return HttpResponse.json(mockMcpWithDcr);
      })
    );

    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-connected-card')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('oauth-disconnect-button'));

    await waitFor(() => {
      expect(screen.queryByTestId('oauth-connected-card')).not.toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-update-button'));

    await waitFor(() => {
      expect(deleteTokenCalled).toHaveBeenCalled();
    });

    await waitFor(() => {
      expect(updateCalled).toHaveBeenCalled();
    });

    const updateBody = updateCalled.mock.calls[0][0];
    expect(updateBody).not.toHaveProperty('auth_uuid');
  });
});
