import NewMcpPage from '@/app/ui/mcps/new/page';
import { useMcpFormStore } from '@/stores/mcpFormStore';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import {
  mockAuthHeader,
  mockCreateAuthHeader,
  mockCreateMcp,
  mockCreateOAuthConfig,
  mockCreateOAuthConfigError,
  mockDeleteOAuthToken,
  mockDeleteOAuthTokenError,
  mockFetchMcpTools,
  mockFetchMcpToolsError,
  mockGetAuthHeader,
  mockGetMcp,
  mockGetOAuthConfig,
  mockListMcpServers,
  mockListOAuthConfigs,
  mockMcp,
  mockMcpServerResponse,
  mockMcpTool,
  mockMcpWithHeaderAuth,
  mockMcpWithOAuth,
  mockOAuthConfig,
  mockOAuthDiscover,
  mockOAuthLogin,
  mockOAuthToken,
  mockOAuthTokenExchange,
  mockUpdateAuthHeader,
  mockUpdateMcp,
} from '@/test-utils/msw-v2/handlers/mcps';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
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

async function selectServerAndOAuth(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByTestId('mcp-server-combobox'));
  await waitFor(() => {
    expect(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`)).toBeInTheDocument();
  });
  await user.click(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`));

  await user.click(screen.getByTestId('mcp-auth-type-select'));
  await waitFor(() => {
    expect(screen.getByTestId('mcp-auth-type-oauth')).toBeInTheDocument();
  });
  await user.click(screen.getByTestId('mcp-auth-type-oauth'));

  await waitFor(() => {
    expect(screen.getByTestId('oauth-fields-section')).toBeInTheDocument();
  });
}

async function fillOAuthForm(user: ReturnType<typeof userEvent.setup>) {
  await user.type(screen.getByTestId('oauth-client-id'), 'my-client-id');
  await user.type(screen.getByTestId('oauth-client-secret'), 'my-client-secret');
  await user.clear(screen.getByTestId('oauth-authorization-endpoint'));
  await user.type(screen.getByTestId('oauth-authorization-endpoint'), 'https://auth.example.com/authorize');
  await user.clear(screen.getByTestId('oauth-token-endpoint'));
  await user.type(screen.getByTestId('oauth-token-endpoint'), 'https://auth.example.com/token');
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
      mockFetchMcpTools([mockMcpTool]),
      mockCreateAuthHeader(mockAuthHeader),
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

    await user.click(screen.getByTestId('mcp-server-combobox'));
    await waitFor(() => {
      expect(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`)).toBeInTheDocument();
    });
    await user.click(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`));

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

    await user.click(screen.getByTestId('mcp-server-combobox'));
    await waitFor(() => {
      expect(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`)).toBeInTheDocument();
    });
    await user.click(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`));
    await user.click(screen.getByTestId('mcp-fetch-tools-button'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-tool-read_wiki_structure')).toBeInTheDocument();
    });

    const toolCheckbox = screen.getByTestId('mcp-tool-checkbox-read_wiki_structure');
    expect(toolCheckbox).toBeInTheDocument();
  });

  it('creates MCP with tools data in single POST', async () => {
    const user = userEvent.setup();

    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockFetchMcpTools([mockMcpTool]),
      mockCreateMcp(mockMcp)
    );

    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-server-combobox'));
    await waitFor(() => {
      expect(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`)).toBeInTheDocument();
    });
    await user.click(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`));
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
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockFetchMcpToolsError({ message: 'Connection refused' })
    );

    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-server-combobox'));
    await waitFor(() => {
      expect(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`)).toBeInTheDocument();
    });
    await user.click(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`));
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
      mockGetMcp(mockMcp),
      mockFetchMcpTools([mockMcpTool])
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

describe('NewMcpPage - Auth type selector', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockFetchMcpTools([mockMcpTool]),
      mockCreateAuthHeader(mockAuthHeader),
      mockCreateMcp(mockMcp)
    );
  });

  it('renders auth section with default public type', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-section')).toBeInTheDocument();
    });

    expect(screen.getByTestId('mcp-auth-type-select')).toBeInTheDocument();
    expect(screen.queryByTestId('mcp-auth-header-key')).not.toBeInTheDocument();
    expect(screen.queryByTestId('mcp-auth-header-value')).not.toBeInTheDocument();
  });

  it('shows header fields when header auth type selected', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-select')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-header')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-header'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-key')).toBeInTheDocument();
    });
    expect(screen.getByTestId('mcp-auth-header-value')).toBeInTheDocument();
  });

  it('hides header fields when switching back to public', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-select')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-header')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-header'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-key')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-public')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-public'));

    await waitFor(() => {
      expect(screen.queryByTestId('mcp-auth-header-key')).not.toBeInTheDocument();
    });
    expect(screen.queryByTestId('mcp-auth-header-value')).not.toBeInTheDocument();
  });
});

describe('NewMcpPage - Bearer warning', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockFetchMcpTools([mockMcpTool]),
      mockCreateAuthHeader(mockAuthHeader),
      mockCreateMcp(mockMcp)
    );
  });

  it('shows warning when Authorization header value does not start with Bearer', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-select')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-header')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-header'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-key')).toBeInTheDocument();
    });

    await user.clear(screen.getByTestId('mcp-auth-header-key'));
    await user.type(screen.getByTestId('mcp-auth-header-key'), 'Authorization');
    await user.type(screen.getByTestId('mcp-auth-header-value'), 'sk-12345');

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-bearer-warning')).toBeInTheDocument();
    });
  });

  it('does not show warning when value starts with Bearer', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-select')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-header')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-header'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-key')).toBeInTheDocument();
    });

    await user.clear(screen.getByTestId('mcp-auth-header-key'));
    await user.type(screen.getByTestId('mcp-auth-header-key'), 'Authorization');
    await user.type(screen.getByTestId('mcp-auth-header-value'), 'Bearer sk-12345');

    expect(screen.queryByTestId('mcp-auth-bearer-warning')).not.toBeInTheDocument();
  });

  it('does not show warning for non-Authorization headers', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-select')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-header')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-header'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-key')).toBeInTheDocument();
    });

    await user.clear(screen.getByTestId('mcp-auth-header-key'));
    await user.type(screen.getByTestId('mcp-auth-header-key'), 'X-API-Key');
    await user.type(screen.getByTestId('mcp-auth-header-value'), 'sk-12345');

    expect(screen.queryByTestId('mcp-auth-bearer-warning')).not.toBeInTheDocument();
  });
});

describe('NewMcpPage - Edit with header auth', () => {
  beforeEach(() => {
    searchParamsMap = { id: 'mcp-uuid-2' };
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mockMcpWithHeaderAuth),
      mockGetAuthHeader(mockAuthHeader),
      mockUpdateAuthHeader(mockAuthHeader),
      mockFetchMcpTools([mockMcpTool]),
      mockUpdateMcp(mockMcpWithHeaderAuth)
    );
  });

  it('loads existing header auth and shows auth fields with header key from config', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('Header Auth MCP');
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-key')).toHaveValue('Authorization');
    });
    expect(screen.getByTestId('mcp-auth-header-value')).toHaveValue('');
  });

  it('shows auth type select with header state on edit', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('Header Auth MCP');
    });

    const trigger = screen.getByTestId('mcp-auth-type-select');
    expect(trigger).toHaveAttribute('data-test-state', 'header');
  });

  it('shows placeholder hint about existing header value in edit mode', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-value')).toBeInTheDocument();
    });

    const input = screen.getByTestId('mcp-auth-header-value');
    expect(input).toHaveAttribute('placeholder', 'Leave empty to keep existing');
  });

  it('toggles header value visibility with eye button', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-value')).toBeInTheDocument();
    });

    const input = screen.getByTestId('mcp-auth-header-value');
    expect(input).toHaveAttribute('type', 'password');

    await user.click(screen.getByTestId('mcp-auth-header-value-visibility-toggle'));
    expect(input).toHaveAttribute('type', 'text');

    await user.click(screen.getByTestId('mcp-auth-header-value-visibility-toggle'));
    expect(input).toHaveAttribute('type', 'password');
  });
});

describe('NewMcpPage - Edit with public auth', () => {
  beforeEach(() => {
    searchParamsMap = { id: 'mcp-uuid-1' };
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mockMcp),
      mockFetchMcpTools([mockMcpTool]),
      mockUpdateMcp(mockMcp)
    );
  });

  it('shows auth type select with public state on edit', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('Example MCP');
    });

    const trigger = screen.getByTestId('mcp-auth-type-select');
    expect(trigger).toHaveAttribute('data-test-state', 'public');
    expect(screen.queryByTestId('mcp-auth-header-key')).not.toBeInTheDocument();
    expect(screen.queryByTestId('mcp-auth-header-value')).not.toBeInTheDocument();
  });
});

describe('NewMcpPage - OAuth auth type', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockListOAuthConfigs({ oauth_configs: [] }),
      mockFetchMcpTools([mockMcpTool]),
      mockCreateMcp(mockMcp),
      mockCreateOAuthConfig(mockOAuthConfig),
      mockOAuthLogin(),
      mockOAuthDiscover(),
      mockOAuthTokenExchange()
    );
  });

  it('shows OAuth fields when oauth-pre-registered auth type selected', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-select')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-oauth')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-oauth'));

    await waitFor(() => {
      expect(screen.getByTestId('oauth-fields-section')).toBeInTheDocument();
    });
    expect(screen.getByTestId('oauth-server-url')).toBeInTheDocument();
    expect(screen.getByTestId('oauth-client-id')).toBeInTheDocument();
    expect(screen.getByTestId('oauth-client-secret')).toBeInTheDocument();
    expect(screen.getByTestId('oauth-authorization-endpoint')).toBeInTheDocument();
    expect(screen.getByTestId('oauth-token-endpoint')).toBeInTheDocument();
    expect(screen.getByTestId('oauth-scopes')).toBeInTheDocument();
    expect(screen.getByTestId('oauth-auto-detect')).toBeInTheDocument();
    expect(screen.getByTestId('oauth-authorize')).toBeInTheDocument();
  });

  it('hides OAuth fields when switching back to public', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-select')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-oauth')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-oauth'));

    await waitFor(() => {
      expect(screen.getByTestId('oauth-fields-section')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-public')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-public'));

    await waitFor(() => {
      expect(screen.queryByTestId('oauth-fields-section')).not.toBeInTheDocument();
    });
  });

  it('auto-detects OAuth endpoints from server URL', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-server-combobox'));
    await waitFor(() => {
      expect(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`)).toBeInTheDocument();
    });
    await user.click(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`));

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-oauth')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-oauth'));

    await waitFor(() => {
      expect(screen.getByTestId('oauth-server-url')).toBeInTheDocument();
    });
    await waitFor(() => {
      expect(screen.getByTestId('oauth-server-url')).toHaveValue('https://mcp.example.com');
    });
    await user.click(screen.getByTestId('oauth-auto-detect'));

    await waitFor(() => {
      expect(screen.getByTestId('oauth-authorization-endpoint')).toHaveValue('https://auth.example.com/authorize');
    });
    expect(screen.getByTestId('oauth-token-endpoint')).toHaveValue('https://auth.example.com/token');
    expect(screen.getByTestId('oauth-scopes')).toHaveValue('mcp:tools mcp:read');
  });
});

describe('NewMcpPage - Edit with OAuth auth', () => {
  beforeEach(() => {
    searchParamsMap = { id: 'mcp-uuid-3' };
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mockMcpWithOAuth),
      mockGetOAuthConfig(mockOAuthConfig),
      mockFetchMcpTools([mockMcpTool]),
      mockUpdateMcp(mockMcpWithOAuth),
      mockOAuthDiscover(),
      mockOAuthLogin(),
      mockCreateOAuthConfig(mockOAuthConfig),
      mockOAuthTokenExchange()
    );
  });

  it('loads existing OAuth MCP and shows OAuth auth type', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('OAuth MCP');
    });

    const trigger = screen.getByTestId('mcp-auth-type-select');
    expect(trigger).toHaveAttribute('data-test-state', 'oauth-pre-registered');
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

describe('NewMcpPage - OAuth authorize - no existing configs', () => {
  beforeEach(() => {
    setupWindowLocation();
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockListOAuthConfigs({ oauth_configs: [] }),
      mockFetchMcpTools([mockMcpTool]),
      mockCreateOAuthConfig(mockOAuthConfig),
      mockOAuthLogin(),
      mockOAuthDiscover()
    );
  });

  it('clicking Authorize with filled OAuth form creates new config then redirects', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await selectServerAndOAuth(user);

    expect(screen.queryByTestId('oauth-config-dropdown')).not.toBeInTheDocument();

    await fillOAuthForm(user);
    await user.click(screen.getByTestId('oauth-authorize'));

    await waitFor(() => {
      expect(window.location.href).toBe('https://auth.example.com/authorize?client_id=test&state=abc123');
    });
  });

  it('clicking Authorize with missing required fields does not redirect', async () => {
    const user = userEvent.setup();
    const originalHref = window.location.href;
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await selectServerAndOAuth(user);

    await user.click(screen.getByTestId('oauth-authorize'));

    await waitFor(() => {
      expect(screen.getByTestId('oauth-authorize')).not.toBeDisabled();
    });
    expect(window.location.href).toBe(originalHref);
  });

  it('create OAuth config API error does not redirect', async () => {
    server.use(mockCreateOAuthConfigError());

    const user = userEvent.setup();
    const originalHref = window.location.href;
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await selectServerAndOAuth(user);
    await fillOAuthForm(user);
    await user.click(screen.getByTestId('oauth-authorize'));

    await waitFor(() => {
      expect(screen.getByTestId('oauth-authorize')).not.toBeDisabled();
    });
    expect(window.location.href).toBe(originalHref);
  });
});

describe('NewMcpPage - OAuth authorize - with existing configs', () => {
  beforeEach(() => {
    setupWindowLocation();
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockListOAuthConfigs({ oauth_configs: [mockOAuthConfig] }),
      mockFetchMcpTools([mockMcpTool]),
      mockCreateOAuthConfig(mockOAuthConfig),
      mockOAuthLogin(),
      mockOAuthDiscover()
    );
  });

  it('shows config dropdown when existing configs exist for server', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await selectServerAndOAuth(user);

    await waitFor(() => {
      expect(screen.getByTestId('oauth-config-dropdown')).toBeInTheDocument();
    });
    expect(screen.getByTestId('oauth-config-select')).toBeInTheDocument();
  });

  it('selecting existing config shows summary with Authorize button', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await selectServerAndOAuth(user);

    await waitFor(() => {
      expect(screen.getByTestId('oauth-config-select')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('oauth-config-select'));
    await waitFor(() => {
      expect(screen.getByTestId(`oauth-config-option-${mockOAuthConfig.id}`)).toBeInTheDocument();
    });
    await user.click(screen.getByTestId(`oauth-config-option-${mockOAuthConfig.id}`));

    await waitFor(() => {
      expect(screen.getByTestId('oauth-config-summary')).toBeInTheDocument();
    });
    expect(screen.getByTestId('oauth-authorize-existing')).toBeInTheDocument();
  });

  it('Authorize on existing config calls login directly and redirects', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await selectServerAndOAuth(user);

    await waitFor(() => {
      expect(screen.getByTestId('oauth-config-select')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('oauth-config-select'));
    await waitFor(() => {
      expect(screen.getByTestId(`oauth-config-option-${mockOAuthConfig.id}`)).toBeInTheDocument();
    });
    await user.click(screen.getByTestId(`oauth-config-option-${mockOAuthConfig.id}`));

    await waitFor(() => {
      expect(screen.getByTestId('oauth-authorize-existing')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('oauth-authorize-existing'));

    await waitFor(() => {
      expect(window.location.href).toBe('https://auth.example.com/authorize?client_id=test&state=abc123');
    });
  });

  it('selecting New Configuration from dropdown shows OAuth form fields', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await selectServerAndOAuth(user);

    await waitFor(() => {
      expect(screen.getByTestId('oauth-config-select')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('oauth-config-select'));
    await waitFor(() => {
      expect(screen.getByTestId('oauth-config-option-new')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('oauth-config-option-new'));

    await waitFor(() => {
      expect(screen.getByTestId('oauth-client-id')).toBeInTheDocument();
    });
    expect(screen.getByTestId('oauth-client-secret')).toBeInTheDocument();
    expect(screen.getByTestId('oauth-authorization-endpoint')).toBeInTheDocument();
    expect(screen.getByTestId('oauth-token-endpoint')).toBeInTheDocument();
    expect(screen.getByTestId('oauth-authorize')).toBeInTheDocument();
  });

  it('filling new config from dropdown and clicking Authorize creates config then redirects', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await selectServerAndOAuth(user);

    await waitFor(() => {
      expect(screen.getByTestId('oauth-config-select')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('oauth-config-select'));
    await waitFor(() => {
      expect(screen.getByTestId('oauth-config-option-new')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('oauth-config-option-new'));

    await waitFor(() => {
      expect(screen.getByTestId('oauth-client-id')).toBeInTheDocument();
    });

    await fillOAuthForm(user);
    await user.click(screen.getByTestId('oauth-authorize'));

    await waitFor(() => {
      expect(window.location.href).toBe('https://auth.example.com/authorize?client_id=test&state=abc123');
    });
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
      auth_type: 'oauth-pre-registered',
      oauth_config_id: mockOAuthConfig.id,
      oauth_token_id: mockOAuthToken.id,
      oauth_server_url: 'https://mcp.example.com',
      server_url: mockMcpServerResponse.url,
      server_name: mockMcpServerResponse.name,
    };
    sessionStorage.setItem('mcp_oauth_form_state', JSON.stringify(sessionData));

    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockListOAuthConfigs({ oauth_configs: [mockOAuthConfig] }),
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

    const trigger = screen.getByTestId('mcp-auth-type-select');
    expect(trigger).toHaveAttribute('data-test-state', 'oauth-pre-registered');

    await waitFor(() => {
      expect(screen.getByTestId('oauth-connected-card')).toBeInTheDocument();
    });
    expect(screen.getByTestId('oauth-connected-badge')).toHaveTextContent('Connected');
    expect(screen.getByTestId('oauth-disconnect-button')).toBeInTheDocument();
  });
});

describe('NewMcpPage - OAuth disconnect flow', () => {
  beforeEach(() => {
    const sessionData = {
      name: 'Connected MCP',
      slug: 'connected-mcp',
      description: '',
      enabled: true,
      mcp_server_id: mockMcpServerResponse.id,
      auth_type: 'oauth-pre-registered',
      oauth_config_id: mockOAuthConfig.id,
      oauth_token_id: mockOAuthToken.id,
      oauth_server_url: 'https://mcp.example.com',
      server_url: mockMcpServerResponse.url,
      server_name: mockMcpServerResponse.name,
    };
    sessionStorage.setItem('mcp_oauth_form_state', JSON.stringify(sessionData));

    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockListOAuthConfigs({ oauth_configs: [mockOAuthConfig] }),
      mockFetchMcpTools([mockMcpTool])
    );
  });

  it('Disconnect calls DELETE oauth-token and on success shows config dropdown', async () => {
    server.use(mockDeleteOAuthToken());

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
      expect(screen.getByTestId('oauth-config-dropdown')).toBeInTheDocument();
    });
  });

  it('Disconnect on API failure still clears local connected state', async () => {
    server.use(mockDeleteOAuthTokenError());

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
  });
});

describe('NewMcpPage - OAuth data-test-state attributes', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockListOAuthConfigs({ oauth_configs: [] }),
      mockFetchMcpTools([mockMcpTool]),
      mockOAuthDiscover(),
      mockCreateOAuthConfig(mockOAuthConfig),
      mockOAuthLogin()
    );
  });

  it('auto-detect button has data-test-state reflecting mutation state', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await selectServerAndOAuth(user);

    const autoDetectBtn = screen.getByTestId('oauth-auto-detect');
    expect(autoDetectBtn).toHaveAttribute('data-test-state', 'idle');

    await user.click(autoDetectBtn);

    await waitFor(() => {
      expect(screen.getByTestId('oauth-auto-detect')).toHaveAttribute('data-test-state', 'success');
    });
  });

  it('authorize button has data-test-state reflecting mutation state', async () => {
    setupWindowLocation();
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await selectServerAndOAuth(user);
    await fillOAuthForm(user);

    const authorizeBtn = screen.getByTestId('oauth-authorize');
    expect(authorizeBtn).toHaveAttribute('data-test-state', 'idle');
  });
});
